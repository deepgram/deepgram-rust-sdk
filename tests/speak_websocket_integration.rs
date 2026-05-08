//! Integration tests for the Speak WebSocket against a local mock
//! server. Verifies the streaming-TTS lifecycle:
//!
//!   client connects
//!   ↓
//!   server emits Metadata
//!   client sends Speak("hello"), Flush
//!   ↓
//!   server emits binary audio chunks → Flushed
//!   client closes
//!
//! Run with:
//!
//! ```bash
//! cargo test --test speak_websocket_integration --features speak
//! ```

#![cfg(feature = "speak")]

use std::net::SocketAddr;

use bytes::Bytes;
use futures::stream::StreamExt;
use futures::SinkExt;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{self, protocol::Message};

use deepgram::speak::{
    options::{Encoding, Model},
    response::SpeakResponse,
};
use deepgram::Deepgram;

const FAKE_REQUEST_ID: &str = "550e8400-e29b-41d4-a716-446655440000";
const AUDIO_PAYLOAD: &[u8] = &[0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

/// Spin up a local WS server that scripts a single Flush sequence.
///
/// Returns the bound address and a one-shot receiver yielding the raw
/// text of the first JSON message the client sent (so the test can
/// assert on the `Speak` payload shape).
async fn mock_speak_server() -> (SocketAddr, tokio::sync::oneshot::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (first_msg_tx, first_msg_rx) = tokio::sync::oneshot::channel::<String>();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();

        #[allow(clippy::result_large_err)]
        let callback = |_req: &tungstenite::handshake::server::Request,
                        mut resp: tungstenite::handshake::server::Response| {
            resp.headers_mut()
                .insert("dg-request-id", FAKE_REQUEST_ID.parse().unwrap());
            Ok(resp)
        };

        let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .unwrap();

        // 1. Server emits Metadata.
        ws.send(Message::Text(
            json!({
                "type": "Metadata",
                "request_id": FAKE_REQUEST_ID,
                "model_name": "aura-asteria-en",
                "model_version": "1.0.0",
                "model_uuid": "11111111-2222-3333-4444-555555555555",
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

        // 2. Wait for the client's first JSON message (a Speak payload).
        let speak_text = loop {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => break text.to_string(),
                Some(Ok(_)) => continue,
                Some(Err(err)) => panic!("server recv error: {err}"),
                None => panic!("client closed before sending Speak"),
            }
        };
        let _ = first_msg_tx.send(speak_text);

        // 3. Wait for the Flush message.
        loop {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    let parsed: Value = serde_json::from_str(text.as_str()).unwrap();
                    if parsed["type"] == "Flush" {
                        break;
                    }
                }
                Some(Ok(_)) => continue,
                Some(Err(err)) => panic!("server recv error: {err}"),
                None => panic!("client closed before sending Flush"),
            }
        }

        // 4. Stream a binary audio chunk.
        ws.send(Message::Binary(Bytes::from_static(AUDIO_PAYLOAD)))
            .await
            .unwrap();

        // 5. Send Flushed.
        ws.send(Message::Text(
            json!({"type": "Flushed", "sequence_id": 1})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();

        // Wait for client to close.
        loop {
            match ws.next().await {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => continue,
                Some(Err(_)) => break,
            }
        }
        let _ = ws.close(None).await;
    });

    (addr, first_msg_rx)
}

#[tokio::test]
async fn speak_streaming_round_trip_through_mock_server() {
    let (addr, first_msg_rx) = mock_speak_server().await;

    let dg = Deepgram::new("test-key").unwrap();
    let (mut handle, mut stream) = dg
        .text_to_speech()
        .websocket()
        .url(format!("ws://{addr}/v1/speak"))
        .model(Model::AuraAsteriaEn)
        .encoding(Encoding::Linear16)
        .sample_rate(24_000)
        .start()
        .await
        .expect("failed to connect to mock speak server");

    assert_eq!(
        handle.request_id().map(|u| u.to_string()),
        Some(FAKE_REQUEST_ID.to_string())
    );

    // Metadata arrives first.
    let evt = stream.next().await.expect("expected Metadata").unwrap();
    match evt {
        SpeakResponse::Metadata(m) => {
            assert_eq!(m.model_name, "aura-asteria-en");
            assert_eq!(m.request_id, FAKE_REQUEST_ID);
        }
        other => panic!("expected Metadata, got {other:?}"),
    }

    // Send text, then flush.
    handle.send_text("hello").await.expect("send_text");
    handle.flush().await.expect("flush");

    // Verify the server saw a properly-shaped Speak message.
    let received = first_msg_rx
        .await
        .expect("server should have received a JSON message");
    let parsed: Value = serde_json::from_str(&received).unwrap();
    assert_eq!(parsed["type"], "Speak");
    assert_eq!(parsed["text"], "hello");

    // Audio chunk.
    let evt = stream.next().await.expect("expected Audio").unwrap();
    match evt {
        SpeakResponse::Audio(bytes) => assert_eq!(&bytes[..], AUDIO_PAYLOAD),
        other => panic!("expected Audio, got {other:?}"),
    }

    // Flushed.
    let evt = stream.next().await.expect("expected Flushed").unwrap();
    match evt {
        SpeakResponse::Flushed(f) => assert_eq!(f.sequence_id, 1),
        other => panic!("expected Flushed, got {other:?}"),
    }

    handle.close().await.expect("close");
}

#[tokio::test]
async fn unknown_speak_event_falls_through_to_unknown_variant() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();

        #[allow(clippy::result_large_err)]
        let callback = |_req: &tungstenite::handshake::server::Request,
                        mut resp: tungstenite::handshake::server::Response| {
            resp.headers_mut()
                .insert("dg-request-id", FAKE_REQUEST_ID.parse().unwrap());
            Ok(resp)
        };

        let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .unwrap();

        ws.send(Message::Text(
            json!({"type": "FutureSpeakEvent", "weird": [1, 2]})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();

        ws.close(None).await.ok();
    });

    let dg = Deepgram::new("test-key").unwrap();
    let (_handle, mut stream) = dg
        .text_to_speech()
        .websocket()
        .url(format!("ws://{addr}/v1/speak"))
        .start()
        .await
        .unwrap();

    let evt = stream.next().await.expect("expected an event").unwrap();
    match evt {
        SpeakResponse::Unknown(value) => {
            assert_eq!(value["type"], "FutureSpeakEvent");
            assert_eq!(value["weird"][0], 1);
        }
        other => panic!("expected Unknown, got {other:?}"),
    }
}
