//! CI-runnable proof that every WebSocket surface sends the URL's full
//! authority in the `Host` header: host plus the port when the port is not
//! the scheme default. A localhost server on a random port captures the
//! header from the upgrade request, so the expected value always carries a
//! port; no network access or API key is required.
//!
//! The same capture reports the upgrade request's target, which gives the
//! streaming text-to-speech escape hatches (`query_params` and
//! `Encoding::CustomEncoding`) a wire-level guard rather than only the
//! `as_url()` unit assertions.

#![cfg(any(feature = "listen", feature = "speak"))]

use deepgram::Deepgram;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

const REQUEST_ID: &str = "0193b1c8-6d3f-7a4e-b8f0-1234567890ab";

/// Accept one upgrade, send the captured `Host` header value and the
/// upgrade request's target (path and query) through `host_tx`, answer with
/// a `dg-request-id`, then close.
async fn spawn_host_capturing_server() -> (u16, oneshot::Receiver<(String, String)>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();
    let (host_tx, host_rx) = oneshot::channel();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let mut host_tx = Some(host_tx);
        // The callback signature (and its large `ErrorResponse` Err variant)
        // is fixed by tungstenite's accept_hdr API.
        #[allow(clippy::result_large_err)]
        let callback = |request: &Request, mut response: Response| {
            let host = request
                .headers()
                .get("host")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_owned();
            let target = request.uri().to_string();
            if let Some(tx) = host_tx.take() {
                let _ = tx.send((host, target));
            }
            response
                .headers_mut()
                .insert("dg-request-id", REQUEST_ID.parse().unwrap());
            Ok(response)
        };
        if let Ok(mut ws) = tokio_tungstenite::accept_hdr_async(stream, callback).await {
            let _ = futures::SinkExt::close(&mut ws).await;
        }
    });

    (port, host_rx)
}

fn client(port: u16) -> Deepgram {
    Deepgram::with_base_url_and_api_key(format!("http://127.0.0.1:{port}").as_str(), "fake-key")
        .expect("client")
}

#[cfg(feature = "listen")]
#[tokio::test]
async fn live_transcription_sends_host_with_port() {
    let (port, host_rx) = spawn_host_capturing_server().await;

    client(port)
        .transcription()
        .stream_request_with_options(deepgram::common::options::Options::default())
        .handle()
        .await
        .expect("connect");

    assert_eq!(
        host_rx.await.expect("host captured").0,
        format!("127.0.0.1:{port}")
    );
}

#[cfg(feature = "listen")]
#[tokio::test]
async fn flux_speech_to_text_sends_host_with_port() {
    let (port, host_rx) = spawn_host_capturing_server().await;

    client(port)
        .transcription()
        .flux_request()
        .handle()
        .await
        .expect("connect");

    assert_eq!(
        host_rx.await.expect("host captured").0,
        format!("127.0.0.1:{port}")
    );
}

#[cfg(feature = "speak")]
#[tokio::test]
async fn streaming_text_to_speech_sends_host_with_port() {
    let (port, host_rx) = spawn_host_capturing_server().await;

    client(port)
        .text_to_speech()
        .speak_stream()
        .handle()
        .await
        .expect("connect");

    assert_eq!(
        host_rx.await.expect("host captured").0,
        format!("127.0.0.1:{port}")
    );
}

#[cfg(feature = "speak")]
#[tokio::test]
async fn flux_text_to_speech_sends_host_with_port() {
    use deepgram::speak::flux::options::{Model, Options};

    let (port, host_rx) = spawn_host_capturing_server().await;

    client(port)
        .text_to_speech()
        .flux_request(Options::builder(Model::FluxHaleyEn).build())
        .handle()
        .await
        .expect("connect");

    assert_eq!(
        host_rx.await.expect("host captured").0,
        format!("127.0.0.1:{port}")
    );
}

/// PR #166 review, N1: the streaming text-to-speech escape hatches have to
/// reach the wire, not just `as_url()`. An unmodeled `query_params` pair and
/// `Encoding::CustomEncoding` must both appear verbatim in the upgrade
/// request's target, unvalidated and unreordered.
#[cfg(feature = "speak")]
#[tokio::test]
async fn streaming_text_to_speech_sends_the_escape_hatches_on_the_wire() {
    use deepgram::speak::options::Encoding;

    let (port, capture_rx) = spawn_host_capturing_server().await;

    client(port)
        .text_to_speech()
        .speak_stream()
        .encoding(Encoding::CustomEncoding("future-codec".to_string()))
        .query_params([("future_param".to_string(), "on".to_string())])
        .handle()
        .await
        .expect("connect");

    let (_host, target) = capture_rx.await.expect("upgrade captured");
    assert_eq!(target, "/v1/speak?encoding=future-codec&future_param=on");
}
