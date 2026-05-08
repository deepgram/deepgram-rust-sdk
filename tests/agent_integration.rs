//! Integration tests for the Voice Agent WebSocket against a local
//! mock server. Verifies the full session lifecycle:
//!
//!   client connects
//!   ↓
//!   server emits Welcome
//!   client sends Settings
//!   ↓
//!   server emits SettingsApplied → ConversationText → audio chunks → AgentAudioDone
//!   client closes
//!
//! Run with:
//!
//! ```bash
//! cargo test --test agent_integration --features agent
//! ```

#![cfg(feature = "agent")]

use std::net::SocketAddr;

use bytes::Bytes;
use futures::stream::StreamExt;
use futures::SinkExt;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{self, protocol::Message};

use deepgram::agent::{
    audio::{AudioConfig, AudioInput, AudioInputEncoding},
    listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
    settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
    speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
    think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
    AgentEvent, AgentResponse,
};
use deepgram::Deepgram;

const FAKE_REQUEST_ID: &str = "550e8400-e29b-41d4-a716-446655440000";
const AUDIO_PAYLOAD: &[u8] = &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

/// Spin up a local WS server that scripts a single full agent session.
///
/// Returns the bound address and a one-shot receiver that yields the
/// raw text of the Settings message the server received from the
/// client (so the test can assert on it).
async fn mock_agent_server() -> (SocketAddr, tokio::sync::oneshot::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (settings_tx, settings_rx) = tokio::sync::oneshot::channel::<String>();

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

        // 1. Server greets with Welcome.
        ws.send(Message::Text(
            json!({
                "type": "Welcome",
                "request_id": FAKE_REQUEST_ID,
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

        // 2. Wait for the client's Settings message.
        let settings_text = loop {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => break text.to_string(),
                Some(Ok(Message::Binary(_))) => continue, // ignore stray binary
                Some(Ok(Message::Ping(payload))) => {
                    let _ = ws.send(Message::Pong(payload)).await;
                    continue;
                }
                Some(Ok(Message::Pong(_) | Message::Frame(_))) => continue,
                Some(Ok(Message::Close(_))) | None => {
                    panic!("client closed before sending Settings")
                }
                Some(Err(err)) => panic!("server recv error: {err}"),
            }
        };
        let _ = settings_tx.send(settings_text);

        // 3. Acknowledge with SettingsApplied.
        ws.send(Message::Text(
            json!({"type": "SettingsApplied"}).to_string().into(),
        ))
        .await
        .unwrap();

        // 4. Send a ConversationText event.
        ws.send(Message::Text(
            json!({
                "type": "ConversationText",
                "role": "assistant",
                "content": "Hello!"
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

        // 5. Stream a binary audio chunk (interleaved with JSON events).
        ws.send(Message::Binary(Bytes::from_static(AUDIO_PAYLOAD)))
            .await
            .unwrap();

        // 6. AgentAudioDone signals the end of the audio response.
        ws.send(Message::Text(
            json!({"type": "AgentAudioDone"}).to_string().into(),
        ))
        .await
        .unwrap();

        // Wait for the client to close (or for it to drop).
        loop {
            match ws.next().await {
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => continue,
                Some(Err(_)) => break,
            }
        }
        let _ = ws.close(None).await;
    });

    (addr, settings_rx)
}

fn sample_settings() -> SettingsMessage {
    SettingsMessage::new(
        AudioConfig::new(
            Some(AudioInput::new(AudioInputEncoding::Linear16, 16_000)),
            None,
        ),
        AgentConfig::inline(InlineAgentConfig::from_parts(
            AgentListenSettings::new(AgentListenProvider::DeepgramV2(
                DeepgramListenV2Provider::new("flux-general-en"),
            )),
            ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
                OpenAiModel::Gpt4oMini,
            ))),
            SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                DeepgramSpeakModel::Aura2ThaliaEn,
            ))),
        )),
    )
}

#[tokio::test]
async fn agent_session_round_trip_through_mock_server() {
    let (addr, settings_rx) = mock_agent_server().await;

    let dg = Deepgram::new("test-key").unwrap();
    let url = format!("ws://{addr}/v1/agent/converse");
    let (mut handle, mut events) = dg
        .agent()
        .start_at_url(&url)
        .await
        .expect("failed to connect to mock agent server");

    // The mock server injects `dg-request-id` on the upgrade response.
    assert_eq!(
        handle.request_id().map(|u| u.to_string()),
        Some(FAKE_REQUEST_ID.to_string()),
        "expected mock-server-injected dg-request-id"
    );

    // Welcome arrives first.
    let evt = events
        .next()
        .await
        .expect("expected Welcome event")
        .unwrap();
    match evt {
        AgentEvent::Json(AgentResponse::Welcome(w)) => {
            assert_eq!(w.request_id, FAKE_REQUEST_ID);
        }
        other => panic!("expected Welcome, got {other:?}"),
    }

    // Send Settings.
    handle
        .send_settings(sample_settings())
        .await
        .expect("send_settings");

    // Verify server received Settings with the right shape.
    let server_received_settings = settings_rx
        .await
        .expect("server should have received Settings");
    let settings_value: Value =
        serde_json::from_str(&server_received_settings).expect("Settings is valid JSON");
    assert_eq!(settings_value["type"], "Settings");
    assert_eq!(settings_value["audio"]["input"]["encoding"], "linear16");
    assert_eq!(
        settings_value["agent"]["listen"]["provider"]["model"],
        "flux-general-en"
    );

    // SettingsApplied arrives next.
    let evt = events
        .next()
        .await
        .expect("expected SettingsApplied")
        .unwrap();
    assert!(matches!(
        evt,
        AgentEvent::Json(AgentResponse::SettingsApplied(_))
    ));

    // ConversationText.
    let evt = events
        .next()
        .await
        .expect("expected ConversationText")
        .unwrap();
    match evt {
        AgentEvent::Json(AgentResponse::ConversationText(c)) => {
            assert_eq!(c.content, "Hello!");
        }
        other => panic!("expected ConversationText, got {other:?}"),
    }

    // Audio frame.
    let evt = events.next().await.expect("expected Audio").unwrap();
    match evt {
        AgentEvent::Audio(bytes) => {
            assert_eq!(&bytes[..], AUDIO_PAYLOAD);
        }
        other => panic!("expected Audio, got {other:?}"),
    }

    // AgentAudioDone closes the response.
    let evt = events
        .next()
        .await
        .expect("expected AgentAudioDone")
        .unwrap();
    assert!(matches!(
        evt,
        AgentEvent::Json(AgentResponse::AgentAudioDone(_))
    ));

    // Client-initiated close.
    handle.close().await.expect("close failed");
}

#[tokio::test]
async fn unknown_server_event_falls_through_to_unknown_variant() {
    // Tiny mock that emits an unknown JSON event then closes.
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
            json!({
                "type": "FutureEvent",
                "some_data": [1, 2, 3]
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

        ws.close(None).await.ok();
    });

    let dg = Deepgram::new("test-key").unwrap();
    let url = format!("ws://{addr}/v1/agent/converse");
    let (_handle, mut events) = dg.agent().start_at_url(&url).await.unwrap();

    let evt = events.next().await.expect("expected an event").unwrap();
    match evt {
        AgentEvent::Json(AgentResponse::Unknown(value)) => {
            assert_eq!(value["type"], "FutureEvent");
            assert_eq!(value["some_data"][2], 3);
        }
        other => panic!("expected Unknown variant, got {other:?}"),
    }
}
