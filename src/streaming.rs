// Copyright Reacon contributors. Licensed under Apache-2.0.
use std::{fmt, pin::Pin, time::Duration};
use futures_util::{Stream, StreamExt};
use eventsource_stream::{Eventsource, EventStreamError};
use reqwest::header::HeaderMap;
use serde_json::Value;
use tokio::time::{Instant, timeout, timeout_at};
use crate::{apis::configuration::{Configuration, ApiKey}, models};

#[derive(Debug)]
pub enum StreamError {
    Api { status: u16, headers: HeaderMap, body: Value, event: Option<models::VerificationStreamError> },
    Protocol(&'static str),
    Timeout(&'static str),
    Transport(reqwest::Error),
}
impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api { status, .. } => write!(f, "Reacon stream/API failure (HTTP {status})"),
            Self::Protocol(message) => write!(f, "Reacon stream protocol error: {message}"),
            Self::Timeout(phase) => write!(f, "Reacon {phase} timeout"),
            Self::Transport(_) => write!(f, "Reacon stream transport failure"),
        }
    }
}
impl std::error::Error for StreamError {}

#[derive(Debug)]
pub enum VerificationEvent {
    Stage { data: models::VerificationStage, raw: Value },
    Progress { data: models::VerificationProgress, raw: Value },
    Final { data: models::VerificationFinal, raw: Value },
    Unknown { raw: Value },
}
pub type VerificationStream = Pin<Box<dyn Stream<Item = Result<VerificationEvent, StreamError>> + Send>>;

#[derive(Clone, Debug)]
pub struct StreamOptions {
    pub cache_max_age: Option<String>,
    pub only_if_free: Option<String>,
    pub idle_timeout: Duration,
    pub total_timeout: Duration,
}
impl Default for StreamOptions {
    fn default() -> Self { Self { cache_max_age: None, only_if_free: None, idle_timeout: Duration::from_secs(30), total_timeout: Duration::from_secs(300) } }
}

/// Per-client credentials and pooled transport. Generated JSON operations accept
/// `configuration()`. Streaming is explicit; drop its stream to cancel and close.
pub struct Reacon { configuration: Configuration }
impl Reacon {
    pub fn new(api_key: impl Into<String>) -> Result<Self, reqwest::Error> {
        let mut configuration = Configuration::with_client_builder(reqwest::Client::builder())?;
        configuration.api_key = Some(ApiKey { key: api_key.into(), prefix: None });
        Ok(Self { configuration })
    }
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.configuration.base_path = base_url.into().trim_end_matches('/').to_string(); self
    }
    /// An injected client must preserve no-retry/no-redirect behavior for billed calls.
    pub fn with_http_client(mut self, client: reqwest::Client) -> Self { self.configuration.client = client; self }
    pub fn configuration(&self) -> &Configuration { &self.configuration }

    pub async fn stream_verification(&self, email: &str, options: StreamOptions) -> Result<VerificationStream, StreamError> {
        if options.total_timeout.is_zero() || options.idle_timeout.is_zero() || email.is_empty() {
            return Err(StreamError::Protocol("Email and positive timeouts are required"));
        }
        if options.only_if_free.as_deref().is_some_and(|value| !["true", "false"].contains(&value)) ||
            options.cache_max_age.as_deref().is_some_and(|value| !["live", "1d", "1w", "1m"].contains(&value)) {
            return Err(StreamError::Protocol("Invalid verification option"));
        }
        let deadline = Instant::now() + options.total_timeout;
        let mut request = self.configuration.client.get(format!("{}/v1/verify", self.configuration.base_path))
            .header("Accept", "text/event-stream").query(&[("email", email)]);
        if let Some(key) = &self.configuration.api_key { request = request.header("X-API-Key", &key.key); }
        if let Some(value) = &options.only_if_free { request = request.query(&[("onlyIfFree", value)]); }
        if let Some(value) = &options.cache_max_age { request = request.query(&[("cacheMaxAge", value)]); }
        let response = timeout_at(deadline, request.send()).await.map_err(|_| StreamError::Timeout("total"))?
            .map_err(|error| StreamError::Transport(error.without_url()))?;
        let status = response.status();
        let headers = response.headers().clone();
        if !status.is_success() {
            let mut bytes = response.bytes_stream(); let mut body = Vec::new();
            while body.len() < 65536 {
                let chunk = timeout_at(deadline, timeout(options.idle_timeout, bytes.next())).await
                    .map_err(|_| StreamError::Timeout("total"))?.map_err(|_| StreamError::Timeout("idle"))?;
                let Some(chunk) = chunk else { break; };
                let chunk = chunk.map_err(|error| StreamError::Transport(error.without_url()))?;
                body.extend_from_slice(&chunk[..chunk.len().min(65536 - body.len())]);
            }
            let body = serde_json::from_slice(&body).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body).into()));
            return Err(StreamError::Api { status: status.as_u16(), headers, body, event: None });
        }
        if headers.get("content-type").and_then(|value| value.to_str().ok()).unwrap_or("").split(';').next().unwrap_or("").trim() != "text/event-stream" {
            return Err(StreamError::Protocol("Expected a text/event-stream response body"));
        }
        let mut bytes = response.bytes_stream();
        let raw = async_stream::stream! {
            loop {
                match timeout(options.idle_timeout, bytes.next()).await {
                    Ok(Some(Ok(bytes))) => yield Ok(bytes),
                    Ok(Some(Err(error))) => { yield Err(StreamError::Transport(error.without_url())); return; },
                    Ok(None) => return,
                    Err(_) => { yield Err(StreamError::Timeout("idle")); return; },
                }
            }
        };
        let mut events = Box::pin(raw.eventsource());
        Ok(Box::pin(async_stream::try_stream! {
            loop {
                let event = timeout_at(deadline, events.next()).await.map_err(|_| StreamError::Timeout("total"))?
                    .ok_or(StreamError::Protocol("Verification stream ended before a terminal event"))?
                    .map_err(|error| match error {
                        EventStreamError::Transport(error) => error,
                        _ => StreamError::Protocol("Malformed SSE framing or UTF-8"),
                    })?;
                let raw: Value = serde_json::from_str(&event.data).map_err(|_| StreamError::Protocol("Malformed SSE JSON payload"))?;
                if !raw.is_object() { Err(StreamError::Protocol("Expected an SSE JSON object"))?; }
                if raw.get("error").is_some() {
                    let event = serde_json::from_value(raw.clone()).map_err(|_| StreamError::Protocol("Malformed verification error event"))?;
                    Err(StreamError::Api { status: status.as_u16(), headers: headers.clone(), body: raw.clone(), event: Some(event) })?;
                }
                if raw.get("result").is_some() {
                    let data = serde_json::from_value(raw.clone()).map_err(|_| StreamError::Protocol("Malformed verification result event"))?;
                    drop(events); // release the HTTP body before delivering final
                    yield VerificationEvent::Final { data, raw };
                    return;
                } else if raw.get("stage").is_some() {
                    let data = serde_json::from_value(raw.clone()).map_err(|_| StreamError::Protocol("Malformed verification stage event"))?;
                    yield VerificationEvent::Stage { data, raw };
                } else if raw.get("state").is_some() {
                    let data = serde_json::from_value(raw.clone()).map_err(|_| StreamError::Protocol("Malformed verification progress event"))?;
                    yield VerificationEvent::Progress { data, raw };
                } else { yield VerificationEvent::Unknown { raw }; }
            }
        }))
    }
}
