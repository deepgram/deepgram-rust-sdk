//! CI-runnable regression test for the Flux TTS worker's transport
//! behavior, using a localhost WebSocket server so no network access or
//! API key is required.

#![cfg(feature = "speak")]

use std::time::Duration;

use deepgram::{
    speak::flux::options::{Model, Options},
    Deepgram,
};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;

const REQUEST_ID: &str = "0193b1c8-6d3f-7a4e-b8f0-1234567890ab";

/// Far more responses than the client's bounded response channel (256) can
/// hold, so the channel is guaranteed to be full while they remain undrained.
const QUEUED_RESPONSES: usize = 600;

/// PR #171 review, B1: `Interrupt` must reach the wire even when the
/// inbound response channel is full of audio and nothing is draining it.
#[tokio::test]
async fn interrupt_reaches_wire_while_audio_responses_are_backpressured() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();
    let (interrupt_tx, interrupt_rx) = tokio::sync::oneshot::channel::<String>();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        // The callback signature (and its large `ErrorResponse` Err variant)
        // is fixed by tungstenite's accept_hdr API.
        #[allow(clippy::result_large_err)]
        let callback = |_request: &Request, mut response: Response| {
            response
                .headers_mut()
                .insert("dg-request-id", REQUEST_ID.parse().unwrap());
            Ok(response)
        };
        let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .expect("upgrade");

        // Flood the client with audio frames without reading anything, so
        // the client's bounded response channel fills long before the
        // control message is sent.
        for _ in 0..QUEUED_RESPONSES {
            ws.send(Message::Binary(vec![0u8; 160].into()))
                .await
                .expect("server send");
        }
        // Only now start reading, waiting for the Interrupt control
        // message. The queued audio stays undrained on the client.
        let mut interrupt_tx = Some(interrupt_tx);
        while let Some(Ok(message)) = ws.next().await {
            if let Message::Text(text) = message {
                if text.contains("Interrupt") {
                    if let Some(tx) = interrupt_tx.take() {
                        let _ = tx.send(text.to_string());
                    }
                    break;
                }
            }
        }
    });

    let dg = Deepgram::with_base_url_and_api_key(
        format!("http://127.0.0.1:{port}").as_str(),
        "fake-key",
    )
    .expect("client");
    let options = Options::builder(Model::FluxHaleyEn).build();
    let speak = dg.text_to_speech();
    let mut handle = speak.flux_request(options).handle().await.expect("connect");

    // Earlier outbound text, then time for the worker to fill its bounded
    // response channel. Nothing consumes responses in this test.
    handle.speak("Hello there!").await.expect("send text");
    tokio::time::sleep(Duration::from_millis(200)).await;

    handle
        .interrupt(Some(1_500))
        .await
        .expect("interrupt enqueues");

    let received = tokio::time::timeout(Duration::from_secs(5), interrupt_rx)
        .await
        .expect("Interrupt must reach the server while responses are undrained")
        .expect("server saw Interrupt");
    assert_eq!(
        received,
        r#"{"type":"Interrupt","playback_offset":{"type":"time_ms","value":1500}}"#
    );
}
