use std::time::Duration;
use reacon_sdk::{Reacon, StreamOptions, StreamError, VerificationEvent};
use futures_util::StreamExt;

fn options() -> StreamOptions { StreamOptions { only_if_free: Some("true".into()), ..Default::default() } }
async fn collect(client: &Reacon, scenario: &str, options: StreamOptions) -> Result<Vec<VerificationEvent>, StreamError> {
    let mut stream = client.stream_verification(&format!("{scenario}@example.test"), options).await?;
    let mut values = Vec::new(); while let Some(value) = stream.next().await { values.push(value?); } Ok(values)
}
#[tokio::main]
async fn main() {
    let url = std::env::var("REACON_TEST_URL").unwrap();
    let client = Reacon::new("synthetic-rust").unwrap().with_http_client(reacon_sdk::apis::configuration::Configuration::with_client_builder(fixture_builder(&url, reqwest::Client::builder())).unwrap().client);
    let isolated = Reacon::new("isolated-rust").unwrap().with_http_client(reacon_sdk::apis::configuration::Configuration::with_client_builder(fixture_builder(&url, reqwest::Client::builder())).unwrap().client);
    drop(client.stream_verification("never@example.test", options())); // not polled, no request
    let (values, isolated_values) = tokio::join!(collect(&client, "success", options()), collect(&isolated, "isolated", options()));
    for values in [values.unwrap(), isolated_values.unwrap()] {
        assert_eq!(values.len(), 4);
        assert!(matches!(&values[0], VerificationEvent::Stage { raw, .. } if raw["label"] == "hé🚀"));
        assert!(matches!(&values[1], VerificationEvent::Unknown { raw } if raw["future"]["value"] == "hé🚀"));
        assert!(matches!(&values[2], VerificationEvent::Progress { .. }));
        assert!(matches!(&values[3], VerificationEvent::Final { data, .. } if data.result.accepts_all.is_none() && data.result.status == "future-status"));
    }
    match collect(&client, "error", options()).await.unwrap_err() {
        StreamError::Api { status, headers, event: Some(event), .. } => {
            assert_eq!(status, 200); assert_eq!(headers["x-request-id"], "req-stream");
            assert_eq!(event.code, "INSUFFICIENT_CREDITS"); assert_eq!(event.remaining_credits, Some(0.0));
        }, _ => panic!("Missing terminal API error"),
    }
    for (scenario, expected) in [("pre402", 402), ("pre429", 429), ("proxy", 502), ("redirect", 307)] {
        match collect(&client, scenario, options()).await.unwrap_err() {
            StreamError::Api { status, headers, body, .. } => {
                assert_eq!(status, expected);
                if [402, 429].contains(&expected) { assert_eq!(body["code"], "FIXTURE_ERROR"); assert_eq!(headers["x-request-id"], "req-stream"); }
                if expected == 502 { assert!(!headers.contains_key("x-request-id") && body.is_string()); }
            }, _ => panic!("Missing HTTP error"),
        }
    }
    for scenario in ["wrongtype", "malformed", "invalidresult", "eof"] {
        assert!(matches!(collect(&client, scenario, options()).await.unwrap_err(), StreamError::Protocol(_)));
    }
    assert!(matches!(collect(&client, "disconnect", options()).await.unwrap_err(), StreamError::Transport(_)));
    for phase in ["idle", "total"] {
        let limited = StreamOptions { idle_timeout: Duration::from_millis(80), total_timeout: Duration::from_millis(200), ..options() };
        assert!(matches!(collect(&client, phase, limited).await.unwrap_err(), StreamError::Timeout(actual) if actual == phase));
    }
    let limited = StreamOptions { total_timeout: Duration::from_millis(200), ..options() };
    assert!(matches!(collect(&client, "headers", limited).await.unwrap_err(), StreamError::Timeout(_)));
    let mut cancel = client.stream_verification("cancel@example.test", options()).await.unwrap();
    assert!(matches!(cancel.next().await.unwrap().unwrap(), VerificationEvent::Stage { .. }));
    // Dropping a pending next future then the stream is Rust's native cancellation.
    assert!(tokio::time::timeout(Duration::from_millis(20), cancel.next()).await.is_err());
    drop(cancel);
    let mut early = client.stream_verification("early@example.test", options()).await.unwrap();
    assert!(matches!(early.next().await.unwrap().unwrap(), VerificationEvent::Stage { .. }));
    drop(early);
    assert!(reqwest::get(format!("{url}/_assert_closed")).await.unwrap().status().is_success());
    println!("Rust streaming: framing, terminal/error, isolation, timeout, cancellation and early-close assertions passed");
}

// Test transport: SDK requests retain https://api.reacon.io and normal TLS checks.
fn fixture_builder(target: &str, builder: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    let url = reqwest::Url::parse(target).unwrap();
    assert!(matches!(url.host_str(), Some("127.0.0.1" | "localhost")));
    let mut proxy = reqwest::Url::parse(&std::env::var("REACON_FIXTURE_PROXY_ENDPOINT").expect("Fixture proxy required")).unwrap();
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut encoded = String::new();
    for chunk in target.as_bytes().chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        for i in 0..chunk.len()+1 { encoded.push(alphabet[((n >> (18-i*6)) & 63) as usize] as char); }
    }
    proxy.set_username(&encoded).unwrap();
    let builder = builder.proxy(reqwest::Proxy::https(proxy).unwrap());
    if url.scheme() == "http" { builder.add_root_certificate(reqwest::Certificate::from_pem(std::env::var("REACON_FIXTURE_CA_PEM").unwrap().as_bytes()).unwrap()) } else { builder }
}
