//! Voice Agent WebSocket connection — the live session at
//! `wss://agent.deepgram.com/v1/agent/converse`.
//!
//! This module ties together the typed message surfaces from
//! [`crate::agent::messages`] (client→server) and
//! [`crate::agent::response`] (server→client) into an actual connection.
//! Pattern mirrors [`crate::listen::flux::FluxHandle`]: a `Sender`-side
//! handle for outgoing messages and a `Stream` for incoming events.
//!
//! # Example
//!
//! ```no_run
//! use deepgram::Deepgram;
//! use deepgram::agent::{
//!     audio::{AudioConfig, AudioInput, AudioInputEncoding},
//!     listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
//!     settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
//!     speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
//!     think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
//!     AgentEvent,
//! };
//! use futures::StreamExt;
//!
//! # async fn run() -> Result<(), deepgram::DeepgramError> {
//! let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap_or_default())?;
//! let (mut handle, mut events) = dg.agent().start().await?;
//!
//! handle
//!     .send_settings(SettingsMessage::new(
//!         AudioConfig::new(
//!             Some(AudioInput::new(AudioInputEncoding::Linear16, 16_000)),
//!             None,
//!         ),
//!         AgentConfig::inline(InlineAgentConfig::from_parts(
//!             AgentListenSettings::new(AgentListenProvider::DeepgramV2(
//!                 DeepgramListenV2Provider::new("flux-general-en"),
//!             )),
//!             ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
//!                 OpenAiModel::Gpt4oMini,
//!             ))),
//!             SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
//!                 DeepgramSpeakModel::Aura2ThaliaEn,
//!             ))),
//!         )),
//!     ))
//!     .await?;
//!
//! while let Some(event) = events.next().await {
//!     match event? {
//!         AgentEvent::Audio(_bytes) => { /* play out */ }
//!         AgentEvent::Json(_response) => { /* dispatch on AgentResponse variant */ }
//!         _ => {} // AgentEvent is #[non_exhaustive]
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures::channel::mpsc::{self, Receiver, Sender};
use futures::stream::StreamExt;
use futures::{select_biased, SinkExt, Stream};
use http::Request;
use pin_project::pin_project;
use serde::Serialize;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::handshake::client;
use uuid::Uuid;

use crate::agent::messages::{
    FunctionCallResponseMessage, InjectAgentMessageMessage, InjectUserMessageMessage,
    KeepAliveMessage, UpdatePromptMessage, UpdateSpeakMessage, UpdateThinkMessage,
};
use crate::agent::response::AgentResponse;
use crate::agent::settings::SettingsMessage;
use crate::{Deepgram, DeepgramError, Result};

/// Default Voice Agent WebSocket endpoint (SaaS).
///
/// Self-hosted agent support (a configurable host on the
/// [`Deepgram`] client) is on the `0.10.0` roadmap; for now this is a
/// fixed constant. See `IMPLEMENTATION_PLAN_2026-05-08.md`.
const AGENT_WS_URL: &str = "wss://agent.deepgram.com/v1/agent/converse";

/// Sub-client for the Voice Agent.
///
/// Construct via [`Deepgram::agent`]. Exposes [`Agent::start`] /
/// [`Agent::start_at_url`] for the WebSocket and the
/// [`Agent::configurations`], [`Agent::variables`],
/// [`Agent::think_models`] sub-client accessors for the REST surface.
#[derive(Debug, Clone)]
pub struct Agent<'a>(#[allow(unused)] pub &'a Deepgram);

impl<'a> From<&'a Deepgram> for Agent<'a> {
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Agent<'_> {
    /// Borrow the underlying [`Deepgram`] client.
    pub fn deepgram(&self) -> &Deepgram {
        self.0
    }

    /// Open a new Voice Agent WebSocket session at the SaaS endpoint
    /// (`wss://agent.deepgram.com/v1/agent/converse`).
    ///
    /// Returns a handle for sending messages and a stream of incoming
    /// events. The first message you typically send is a
    /// [`SettingsMessage`] via [`AgentHandle::send_settings`].
    ///
    /// The session terminates when [`AgentHandle::close`] is called, when
    /// the underlying handle is dropped, or when the server closes the
    /// connection.
    pub async fn start(&self) -> Result<(AgentHandle, AgentEventStream)> {
        self.start_at_url(AGENT_WS_URL).await
    }

    /// Open a session at a custom WebSocket URL.
    ///
    /// Use cases:
    /// - **Self-hosted agent deployments** — point at your own
    ///   `wss://agent.your-domain.example/...` host.
    /// - **Integration tests** — point at a local mock server (e.g.
    ///   `ws://127.0.0.1:NNNN/...`).
    ///
    /// All other behavior matches [`Agent::start`]: same auth headers,
    /// same handshake, same returned types.
    pub async fn start_at_url(&self, url: &str) -> Result<(AgentHandle, AgentEventStream)> {
        let url: url::Url = url.parse().map_err(|_| DeepgramError::InvalidUrl)?;
        let host = url.host_str().ok_or(DeepgramError::InvalidUrl)?;

        let request = {
            let http_builder = Request::builder()
                .method("GET")
                .uri(url.to_string())
                .header("sec-websocket-key", client::generate_key())
                .header("host", host)
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13")
                .header("user-agent", crate::USER_AGENT);

            let http_builder = if let Some(auth) = &self.0.auth {
                http_builder.header("authorization", auth.header_value())
            } else {
                http_builder
            };
            http_builder.body(())?
        };

        let (ws_stream, upgrade_response) = tokio_tungstenite::connect_async(request).await?;

        // The agent server may include a `dg-request-id` header on the
        // upgrade response. If it's absent or malformed we still hand
        // back a usable session — the server's `Welcome` event will
        // include `request_id` and the user can rely on that instead.
        let request_id = upgrade_response
            .headers()
            .get("dg-request-id")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok());

        let (message_tx, message_rx) = mpsc::channel(256);
        let (response_tx, response_rx) = mpsc::channel(256);

        tokio::task::spawn(run_agent_worker(ws_stream, message_rx, response_tx));

        let handle = AgentHandle {
            message_tx,
            request_id,
        };
        let stream = AgentEventStream {
            rx: response_rx,
            request_id,
        };

        Ok((handle, stream))
    }
}

/// A single event received from the Voice Agent server.
///
/// JSON events and binary audio frames are interleaved on the same
/// stream — this is the natural ordering of the protocol (the server
/// emits `AgentStartedSpeaking`, then audio chunks, then `AgentAudioDone`).
//
// `Json` carries an `AgentResponse` (~hundreds of bytes for the
// `Settings*` and `History` variants) while `Audio` is just two
// `usize`s + a heap pointer. The disparity is irrelevant in practice:
// stream items are constructed once and immediately consumed by the
// caller's match arm.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum AgentEvent {
    /// A typed JSON event from the server.
    Json(AgentResponse),
    /// A raw binary audio frame.
    Audio(Bytes),
}

/// Handle for sending messages on a live Voice Agent session.
///
/// Methods are split by message type:
/// - JSON message types use the `send_*` prefix (`send_settings`,
///   `send_update_speak`, etc.) — matches the existing
///   [`crate::listen::flux::FluxHandle::send_data`] convention.
/// - [`AgentHandle::send_data`] sends a binary audio frame.
/// - [`AgentHandle::keep_alive`], [`AgentHandle::close`] are
///   control-only (no prefix), matching `FluxHandle::keep_alive` /
///   `FluxHandle::close_stream`.
#[derive(Debug)]
pub struct AgentHandle {
    message_tx: Sender<WsMessage>,
    request_id: Option<Uuid>,
}

impl AgentHandle {
    /// `dg-request-id` from the upgrade response, if the server
    /// included one. Returns `None` if the header was missing — in that
    /// case the server's `Welcome` event carries the request ID instead.
    pub fn request_id(&self) -> Option<Uuid> {
        self.request_id
    }

    /// Send a `Settings` message (typically the first JSON message of a session).
    pub async fn send_settings(&mut self, message: SettingsMessage) -> Result<()> {
        self.send_json(&message).await
    }

    /// Send an `UpdateSpeak` message.
    pub async fn send_update_speak(&mut self, message: UpdateSpeakMessage) -> Result<()> {
        self.send_json(&message).await
    }

    /// Send an `UpdateThink` message.
    pub async fn send_update_think(&mut self, message: UpdateThinkMessage) -> Result<()> {
        self.send_json(&message).await
    }

    /// Send an `UpdatePrompt` message.
    pub async fn send_update_prompt(&mut self, message: UpdatePromptMessage) -> Result<()> {
        self.send_json(&message).await
    }

    /// Send an `InjectUserMessage`.
    pub async fn send_inject_user_message(
        &mut self,
        message: InjectUserMessageMessage,
    ) -> Result<()> {
        self.send_json(&message).await
    }

    /// Send an `InjectAgentMessage`.
    pub async fn send_inject_agent_message(
        &mut self,
        message: InjectAgentMessageMessage,
    ) -> Result<()> {
        self.send_json(&message).await
    }

    /// Send a `FunctionCallResponse` (for client-side function execution results).
    pub async fn send_function_call_response(
        &mut self,
        message: FunctionCallResponseMessage,
    ) -> Result<()> {
        self.send_json(&message).await
    }

    /// Send a binary audio frame.
    ///
    /// Named to match [`crate::listen::flux::FluxHandle::send_data`] for
    /// consistency with the existing audio-streaming convention in this
    /// SDK.
    pub async fn send_data(&mut self, data: Vec<u8>) -> Result<()> {
        self.message_tx
            .send(WsMessage::Audio(data))
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))
    }

    /// Send a `KeepAlive` message.
    pub async fn keep_alive(&mut self) -> Result<()> {
        self.send_json(&KeepAliveMessage::default()).await
    }

    /// Close the WebSocket. After this returns, `send_*` methods will fail.
    ///
    /// The server closes the connection on its own when the session ends
    /// naturally; calling `close` is appropriate for client-initiated shutdown.
    pub async fn close(&mut self) -> Result<()> {
        if !self.message_tx.is_closed() {
            self.message_tx
                .send(WsMessage::Close)
                .await
                .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
            self.message_tx.close_channel();
        }
        Ok(())
    }

    async fn send_json<T: Serialize>(&mut self, value: &T) -> Result<()> {
        let serialized = serde_json::to_string(value)?;
        self.message_tx
            .send(WsMessage::Json(serialized))
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))
    }
}

/// Stream of events from a Voice Agent session.
///
/// Yields [`AgentEvent`] values; binary audio frames and JSON events
/// arrive on the same stream in their natural order.
///
/// The stream terminates when the server closes the connection, when
/// the corresponding [`AgentHandle`] is dropped, or when the session
/// hits a fatal error.
#[derive(Debug)]
#[pin_project]
pub struct AgentEventStream {
    #[pin]
    rx: Receiver<Result<AgentEvent>>,
    request_id: Option<Uuid>,
}

impl AgentEventStream {
    /// Same `dg-request-id` reported by [`AgentHandle::request_id`].
    pub fn request_id(&self) -> Option<Uuid> {
        self.request_id
    }
}

impl Stream for AgentEventStream {
    type Item = Result<AgentEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().rx.poll_next(cx)
    }
}

// ---------- internal worker plumbing ----------

#[derive(Debug)]
enum WsMessage {
    /// JSON-serialized client message (Settings, Update*, etc.).
    Json(String),
    /// Binary audio frame.
    Audio(Vec<u8>),
    /// Graceful close (sends a WebSocket Close frame).
    Close,
}

async fn run_agent_worker(
    ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    mut message_rx: Receiver<WsMessage>,
    mut response_tx: Sender<Result<AgentEvent>>,
) -> Result<()> {
    let (mut ws_send, ws_recv) = ws_stream.split();
    let mut ws_recv = ws_recv.fuse();
    let mut is_open = true;

    loop {
        select_biased! {
            inbound = ws_recv.next() => {
                match inbound {
                    Some(Ok(Message::Text(text))) => {
                        let parsed: std::result::Result<AgentResponse, _> =
                            serde_json::from_str(&text);
                        let event = parsed
                            .map(AgentEvent::Json)
                            .map_err(DeepgramError::from);
                        if response_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Binary(bytes))) => {
                        let event = Ok(AgentEvent::Audio(bytes));
                        if response_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        // Ignore failures: if the WS is gone the next
                        // recv will tell us anyway.
                        let _ = ws_send.send(Message::Pong(payload)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Server-emitted pongs are unexpected; ignore.
                    }
                    Some(Ok(Message::Close(None))) => {
                        return Ok(());
                    }
                    Some(Ok(Message::Close(Some(frame)))) => {
                        let err = DeepgramError::WebsocketClose {
                            code: frame.code.into(),
                            reason: frame.reason.to_string(),
                        };
                        let _ = response_tx.send(Err(err)).await;
                        return Ok(());
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Unfragmented Frame deliveries; tungstenite
                        // normally surfaces these as Text/Binary. Ignore
                        // anything raw that slips through.
                    }
                    Some(Err(err)) => {
                        if response_tx.send(Err(err.into())).await.is_err() {
                            break;
                        }
                    }
                    None => {
                        return Ok(());
                    }
                }
            }
            outbound = message_rx.next() => {
                if !is_open {
                    continue;
                }
                match outbound {
                    Some(WsMessage::Json(json)) => {
                        if let Err(err) = ws_send.send(Message::Text(json.into())).await {
                            let _ = response_tx.send(Err(err.into())).await;
                            is_open = false;
                        }
                    }
                    Some(WsMessage::Audio(audio)) => {
                        if let Err(err) =
                            ws_send.send(Message::Binary(Bytes::from(audio))).await
                        {
                            let _ = response_tx.send(Err(err.into())).await;
                            is_open = false;
                        }
                    }
                    Some(WsMessage::Close) | None => {
                        let _ = ws_send.send(Message::Close(None)).await;
                        is_open = false;
                    }
                }
            }
        }
    }

    if is_open {
        let _ = ws_send.send(Message::Close(None)).await;
    }
    response_tx.close_channel();
    // Drain any remaining outbound messages so the sender side closes cleanly.
    while message_rx.next().await.is_some() {
        // Discard.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::messages::{InjectAgentBehavior, KeepAliveMessage};
    use crate::agent::settings::{AgentConfig, SettingsMessage};
    use crate::agent::{
        audio::{AudioConfig, AudioInput, AudioInputEncoding},
        listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
        speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
        think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
        InlineAgentConfig,
    };

    /// The agent URL is a constant; this test exists so a future change
    /// that introduces a configurable host doesn't accidentally break
    /// the SaaS default.
    #[test]
    fn agent_url_is_well_known_saas_endpoint() {
        let url: url::Url = AGENT_WS_URL.parse().expect("valid URL");
        assert_eq!(url.scheme(), "wss");
        assert_eq!(url.host_str(), Some("agent.deepgram.com"));
        assert_eq!(url.path(), "/v1/agent/converse");
    }

    /// Sanity that every send_* method's payload round-trips through
    /// the same `send_json` serializer used by the real handle. This is
    /// not a wire test — it just guarantees that none of the message
    /// types we ship have a Serialize impl that errors.
    #[test]
    fn all_client_messages_serialize_cleanly() {
        let settings = SettingsMessage::new(
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
        );

        // Each of these serializes as the body of a corresponding send_*.
        serde_json::to_string(&settings).expect("settings serializes");
        serde_json::to_string(&UpdateSpeakMessage::one(SpeakSettings::new(
            SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                DeepgramSpeakModel::AuraAsteriaEn,
            )),
        )))
        .expect("update_speak serializes");
        serde_json::to_string(&UpdateThinkMessage::one(ThinkSettings::new(
            ThinkProvider::OpenAi(OpenAiThinkProvider::new(OpenAiModel::Gpt4o)),
        )))
        .expect("update_think serializes");
        serde_json::to_string(&UpdatePromptMessage::new("hi")).expect("update_prompt serializes");
        serde_json::to_string(&InjectUserMessageMessage::new("hello"))
            .expect("inject_user serializes");
        serde_json::to_string(
            &InjectAgentMessageMessage::new("hi").with_behavior(InjectAgentBehavior::Queue),
        )
        .expect("inject_agent serializes");
        serde_json::to_string(&FunctionCallResponseMessage::with_id("f1", "fn", "{}"))
            .expect("function_call_response serializes");
        serde_json::to_string(&KeepAliveMessage::default()).expect("keep_alive serializes");
    }

    #[test]
    fn agent_event_audio_variant_round_trip_via_bytes() {
        let bytes = Bytes::from_static(&[0x10, 0x20, 0x30]);
        let event = AgentEvent::Audio(bytes.clone());
        match event {
            AgentEvent::Audio(b) => assert_eq!(b, bytes),
            AgentEvent::Json(_) => panic!("expected Audio"),
        }
    }
}
