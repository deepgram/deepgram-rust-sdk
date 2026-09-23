//! CI-runnable proof of what the four `/v1/read` methods put on the wire.
//!
//! `analyze_text` and `analyze_url` differ only in a single JSON body key, and
//! the callback variants only add a query parameter, so a swap between them
//! would compile and would only surface as wrong results against the live API.
//! A localhost HTTP server captures the real request each method sends; no
//! network access or API key is required.

#![cfg(feature = "read")]

use std::time::Duration;

use deepgram::read::options::{CallbackMethod, Options};
use deepgram::Deepgram;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// What the server saw: the request line plus the decoded body.
struct Captured {
    target: String,
    body: String,
}

/// A canned `/v1/read` response, enough for the SDK to deserialize.
const READ_RESPONSE: &str = r#"{"metadata":{"request_id":"0193b1c8-6d3f-7a4e-b8f0-1234567890ab","created":"2026-01-01T00:00:00.000Z","language":"en"},"results":{}}"#;

/// A canned callback acknowledgement.
const CALLBACK_RESPONSE: &str = r#"{"request_id":"0193b1c8-6d3f-7a4e-b8f0-1234567890ab"}"#;

/// Accept one HTTP/1.1 request, send the request target and body through
/// `captured_tx`, and answer with `response_body`.
///
/// Hand-rolled rather than pulling in an HTTP server dev-dependency: the SDK
/// sends one small request with an explicit `Content-Length`, so reading the
/// headers and then exactly that many body bytes is sufficient.
async fn spawn_capturing_server(response_body: &'static str) -> (u16, oneshot::Receiver<Captured>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();
    let (captured_tx, captured_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");

        // Read until the end of the headers, then the declared body length.
        let mut buf = Vec::new();
        let header_end = loop {
            if let Some(at) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break at + 4;
            }
            let mut chunk = [0u8; 1024];
            let read = stream.read(&mut chunk).await.expect("read headers");
            assert_ne!(read, 0, "connection closed before the headers ended");
            buf.extend_from_slice(&chunk[..read]);
        };

        let headers = String::from_utf8(buf[..header_end].to_vec()).expect("utf-8 headers");
        let target = headers
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .expect("request target")
            .to_owned();
        assert!(
            headers.starts_with("POST "),
            "expected a POST, got: {headers}"
        );
        let content_length: usize = headers
            .lines()
            .find_map(|line| {
                line.split_once(':').and_then(|(name, value)| {
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse().expect("content-length"))
                })
            })
            .expect("content-length header");

        let mut body = buf[header_end..].to_vec();
        while body.len() < content_length {
            let mut chunk = [0u8; 1024];
            let read = stream.read(&mut chunk).await.expect("read body");
            assert_ne!(read, 0, "connection closed before the body ended");
            body.extend_from_slice(&chunk[..read]);
        }
        body.truncate(content_length);

        let _ = captured_tx.send(Captured {
            target,
            body: String::from_utf8(body).expect("utf-8 body"),
        });

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .await
            .expect("write response");
        stream.flush().await.expect("flush");
    });

    (port, captured_rx)
}

fn client(port: u16) -> Deepgram {
    Deepgram::with_base_url_and_api_key(format!("http://127.0.0.1:{port}").as_str(), "fake-key")
        .expect("client")
}

async fn captured(rx: oneshot::Receiver<Captured>) -> Captured {
    tokio::time::timeout(Duration::from_secs(5), rx)
        .await
        .expect("server captured a request")
        .expect("server did not panic")
}

fn options() -> Options {
    Options::builder().sentiment(true).build()
}

#[tokio::test]
async fn analyze_text_posts_a_text_body() {
    let (port, rx) = spawn_capturing_server(READ_RESPONSE).await;

    client(port)
        .text_intelligence()
        .analyze_text("The weather today is lovely.", &options())
        .await
        .expect("analyze_text");

    let captured = captured(rx).await;
    assert_eq!(captured.body, r#"{"text":"The weather today is lovely."}"#);
    assert!(
        captured.target.starts_with("/v1/read?"),
        "target was: {}",
        captured.target
    );
    assert!(
        captured.target.contains("sentiment=true") && captured.target.contains("language=en"),
        "target was: {}",
        captured.target
    );
}

#[tokio::test]
async fn analyze_url_posts_a_url_body() {
    let (port, rx) = spawn_capturing_server(READ_RESPONSE).await;

    client(port)
        .text_intelligence()
        .analyze_url("https://example.com/doc.txt", &options())
        .await
        .expect("analyze_url");

    let captured = captured(rx).await;
    assert_eq!(captured.body, r#"{"url":"https://example.com/doc.txt"}"#);
}

#[tokio::test]
async fn analyze_text_callback_posts_a_text_body_and_the_callback_query() {
    let (port, rx) = spawn_capturing_server(CALLBACK_RESPONSE).await;

    let options = Options::builder()
        .sentiment(true)
        .callback_method(CallbackMethod::PUT)
        .build();

    client(port)
        .text_intelligence()
        .analyze_text_callback("hello", &options, "https://example.com/hook")
        .await
        .expect("analyze_text_callback");

    let captured = captured(rx).await;
    assert_eq!(captured.body, r#"{"text":"hello"}"#);
    assert!(
        captured
            .target
            .contains("callback=https%3A%2F%2Fexample.com%2Fhook"),
        "target was: {}",
        captured.target
    );
    assert!(
        captured.target.contains("callback_method=put"),
        "target was: {}",
        captured.target
    );
}

#[tokio::test]
async fn analyze_url_callback_posts_a_url_body() {
    let (port, rx) = spawn_capturing_server(CALLBACK_RESPONSE).await;

    client(port)
        .text_intelligence()
        .analyze_url_callback(
            "https://example.com/doc.txt",
            &options(),
            "https://example.com/hook",
        )
        .await
        .expect("analyze_url_callback");

    let captured = captured(rx).await;
    assert_eq!(captured.body, r#"{"url":"https://example.com/doc.txt"}"#);
    assert!(
        captured
            .target
            .contains("callback=https%3A%2F%2Fexample.com%2Fhook"),
        "target was: {}",
        captured.target
    );
}
