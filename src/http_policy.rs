// Copyright Reacon contributors. Licensed under Apache-2.0.
use crate::apis::{configuration::Configuration, Error, ResponseContent};
use reqwest::{header::HeaderMap, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{error::Error as StdError, fmt, time::Duration};

#[derive(Debug)]
pub enum RequestError {
    InvalidTimeout,
    Timeout {
        timeout: Duration,
        cause: Box<dyn StdError + Send + Sync>,
    },
    Transport(reqwest::Error),
}
impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeout => write!(
                f,
                "Reacon request timeout must be positive and representable"
            ),
            Self::Timeout { timeout, .. } => {
                write!(f, "Reacon network deadline exceeded ({timeout:?})")
            }
            Self::Transport(_) => write!(f, "Reacon network request failed"),
        }
    }
}
impl StdError for RequestError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::InvalidTimeout => None,
            Self::Timeout { cause, .. } => Some(cause.as_ref()),
            Self::Transport(cause) => Some(cause),
        }
    }
}

#[derive(Debug)]
pub struct ResponseDecodeError {
    pub status: StatusCode,
    pub headers: HeaderMap,
    /// Original response bytes, including invalid UTF-8.
    pub body: Vec<u8>,
    pub cause: Box<dyn StdError + Send + Sync>,
}
impl ResponseDecodeError {
    pub fn request_id(&self) -> Option<&str> {
        self.headers
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
    }
}
impl fmt::Display for ResponseDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Reacon response does not match its declared format (HTTP {})",
            self.status
        )
    }
}
impl StdError for ResponseDecodeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.cause.as_ref())
    }
}

impl<T> ResponseContent<T> {
    pub fn request_id(&self) -> Option<&str> {
        self.headers
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
    }
    pub fn parsed_body(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|_| Value::String(self.content.clone()))
    }
    pub fn api_code(&self) -> Option<String> {
        let parsed = self.parsed_body();
        parsed
            .get("code")
            .or_else(|| parsed.get("error").and_then(|error| error.get("code")))
            .and_then(Value::as_str)
            .map(str::to_owned)
    }
}

impl Configuration {
    /// Cheap per-call configuration copy. The underlying reqwest pool is shared.
    pub fn with_request_timeout(&self, timeout: Duration) -> Self {
        let mut configuration = self.clone();
        configuration.request_timeout = timeout;
        configuration
    }
}

/// Preserves supplied TLS/proxy/pool settings while disabling redirects/retries.
/// Built clients injected directly into Configuration remain caller-owned policy.
pub fn build_client(builder: reqwest::ClientBuilder) -> Result<reqwest::Client, reqwest::Error> {
    builder
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .build()
}

pub(crate) struct BufferedResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

pub(crate) async fn execute<E>(
    configuration: &Configuration,
    request: reqwest::Request,
) -> Result<BufferedResponse, Error<E>> {
    let duration = configuration.request_timeout;
    let deadline = tokio::time::Instant::now()
        .checked_add(duration)
        .filter(|_| !duration.is_zero())
        .ok_or(Error::Request(RequestError::InvalidTimeout))?;
    let network = async {
        let response = configuration.client.execute(request).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await?.to_vec();
        Ok::<_, reqwest::Error>(BufferedResponse {
            status,
            headers,
            body,
        })
    };
    match tokio::time::timeout_at(deadline, network).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(cause)) if cause.is_timeout() => Err(Error::Request(RequestError::Timeout {
            timeout: duration,
            cause: Box::new(cause.without_url()),
        })),
        Ok(Err(cause)) => Err(Error::Request(RequestError::Transport(cause.without_url()))),
        Err(cause) => Err(Error::Request(RequestError::Timeout {
            timeout: duration,
            cause: Box::new(cause),
        })),
    }
}

impl BufferedResponse {
    fn ensure_success<E: DeserializeOwned>(self) -> Result<Self, Error<E>> {
        if self.status.is_success() {
            return Ok(self);
        }
        let content = String::from_utf8_lossy(&self.body).into_owned();
        let entity = serde_json::from_slice(&self.body).ok();
        Err(Error::ResponseError(ResponseContent {
            status: self.status,
            headers: self.headers,
            body: self.body,
            content,
            entity,
        }))
    }
    fn decode_error<E>(self, cause: impl StdError + Send + Sync + 'static) -> Error<E> {
        Error::Decode(ResponseDecodeError {
            status: self.status,
            headers: self.headers,
            body: self.body,
            cause: Box::new(cause),
        })
    }
    fn media_type(&self) -> String {
        self.headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase()
    }
    pub(crate) fn json<T: DeserializeOwned, E: DeserializeOwned>(self) -> Result<T, Error<E>> {
        let response = self.ensure_success()?;
        let media = response.media_type();
        if !(media == "application/json"
            || (media.starts_with("application/") && media.ends_with("+json")))
        {
            return Err(response.decode_error(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected a JSON response",
            )));
        }
        let mut decoder = serde_json::Deserializer::from_slice(&response.body);
        let value = match serde_path_to_error::deserialize(&mut decoder) {
            Ok(value) => value,
            Err(cause) => return Err(response.decode_error(cause)),
        };
        // serde_path_to_error decodes one value; end() also rejects trailing JSON.
        if let Err(cause) = decoder.end() {
            return Err(response.decode_error(cause));
        }
        Ok(value)
    }
    pub(crate) fn no_content<E: DeserializeOwned>(self) -> Result<(), Error<E>> {
        self.ensure_success()?;
        Ok(())
    }
    pub(crate) fn csv<E: DeserializeOwned>(self) -> Result<String, Error<E>> {
        let response = self.ensure_success()?;
        if response.media_type() != "text/csv" {
            return Err(response.decode_error(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Expected a CSV response",
            )));
        }
        match std::str::from_utf8(&response.body) {
            Ok(value) => Ok(value.to_owned()),
            Err(cause) => Err(response.decode_error(cause)),
        }
    }
}
