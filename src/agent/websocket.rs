//! Voice Agent WebSocket connection — the live session at
//! `wss://agent.deepgram.com/v1/agent/converse`.
//!
//! This module ties together the typed message surfaces from
//! [`crate::agent::messages`] (client→server) and
//! [`crate::agent::response`] (server→client) into an actual connection.
//! Pattern mirrors `crate::listen::flux::FluxHandle`: a `Sender`-side
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
//! let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap_or_default())?;
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
use core::fmt;
use futures::channel::mpsc::{self, Receiver, Sender};
use futures::stream::StreamExt;
use futures::{select, SinkExt, Stream};
use http::Request;
use pin_project::pin_project;
use serde::Serialize;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::handshake::client;
use uuid::Uuid;

use crate::agent::messages::{
    ForceEndTurnMessage, FunctionCallResponseMessage, InjectAgentMessageMessage,
    InjectUserMessageMessage, KeepAliveMessage, UpdateListenMessage, UpdatePromptMessage,
    UpdateSpeakMessage, UpdateThinkMessage,
};
use crate::agent::response::AgentResponse;
use crate::agent::settings::SettingsMessage;
use crate::{Deepgram, DeepgramError, Result};

/// Default Voice Agent WebSocket endpoint (SaaS).
const AGENT_WS_URL: &str = "wss://agent.deepgram.com/v1/agent/converse";

/// Sub-client for the Voice Agent.
///
/// Construct via [`Deepgram::agent`]. Exposes [`Agent::start`] /
/// [`Agent::start_at_url`] for opening a live agent WebSocket session.
#[derive(Debug, Clone)]
pub struct Agent<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Agent`] sub-client from a [`Deepgram`].
    pub fn agent(&self) -> Agent<'_> {
        self.into()
    }
}

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
    /// # TLS is required
    ///
    /// The Deepgram credential (API key or temporary token) is sent in the
    /// handshake's `Authorization` header, so the URL **must use TLS**
    /// (`wss://`). The single exception is a **loopback host** —
    /// `localhost`, `127.0.0.0/8`, or `[::1]` — where plain `ws://` is
    /// accepted so integration tests can target a local mock server. Any
    /// other `ws://` URL is rejected *before* a connection is attempted
    /// with [`DeepgramError::InsecureAgentUrl`], which carries an
    /// [`InsecureAgentUrl`], so the credential is never transmitted in
    /// cleartext. Schemes other than `ws`/`wss` return
    /// [`DeepgramError::InvalidUrl`].
    ///
    /// Use cases:
    /// - **Self-hosted agent deployments** — point at your own
    ///   `wss://agent.your-domain.example/...` host.
    /// - **Integration tests** — point at a local mock server (e.g.
    ///   `ws://127.0.0.1:NNNN/...`).
    ///
    /// The `Host` header is built from the URL's full authority: host plus
    /// any non-default port, with IPv6 literals bracketed (e.g.
    /// `127.0.0.1:NNNN`, `[::1]:NNNN`, `agent.deepgram.com`). All other
    /// behavior matches [`Agent::start`]: same auth headers, same
    /// handshake, same returned types.
    pub async fn start_at_url(&self, url: &str) -> Result<(AgentHandle, AgentEventStream)> {
        let url: url::Url = url.parse().map_err(|_| DeepgramError::InvalidUrl)?;
        validate_agent_url(&url)?;
        let host = crate::websocket_host_header(&url).ok_or(DeepgramError::InvalidUrl)?;

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

        // The client's explicit rustls connector (see `crate::tls`), shared
        // with every other WebSocket surface, so trust roots cannot differ
        // between surfaces or be changed by downstream feature unification.
        // A plaintext loopback `ws://` URL resolves nothing and never reads
        // the OS certificate store.
        let tls = self.0.tls.resolve_for(&url).await;
        let (ws_stream, upgrade_response) = tokio_tungstenite::connect_async_tls_with_config(
            request,
            None,
            false,
            Some(tls.connector()),
        )
        .await
        .map_err(|err| tls.connect_error(err, url.host_str().unwrap_or_default()))?;

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

/// Error returned by [`Agent::start_at_url`] when asked to open a
/// cleartext (`ws://`) session to a host that is not loopback.
///
/// The Deepgram credential travels in the handshake's `Authorization`
/// header; sending it over `ws://` would expose it to anyone on the
/// network path. Surfaced as [`DeepgramError::InsecureAgentUrl`], so
/// callers can match on that variant directly.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InsecureAgentUrl {
    /// Origin of the rejected URL as `scheme://host[:port]`. The path,
    /// query, and any `user:pass@` userinfo are deliberately omitted so
    /// the error can be logged without echoing a credential.
    pub url: String,
}

impl fmt::Display for InsecureAgentUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "refusing to open a Voice Agent session at `{}`: the Deepgram credential is sent in \
             the Authorization header and `ws://` would transmit it in cleartext. Use `wss://`; \
             plain `ws://` is only accepted for loopback hosts (localhost, 127.0.0.0/8, ::1) in \
             local tests",
            self.url
        )
    }
}

impl std::error::Error for InsecureAgentUrl {}

/// Enforce the TLS rule documented on [`Agent::start_at_url`].
fn validate_agent_url(url: &url::Url) -> Result<()> {
    match url.scheme() {
        "wss" => Ok(()),
        "ws" if is_loopback_host(url) => Ok(()),
        "ws" => Err(InsecureAgentUrl {
            url: format!(
                "{}://{}",
                url.scheme(),
                crate::websocket_host_header(url).unwrap_or_default()
            ),
        }
        .into()),
        _ => Err(DeepgramError::InvalidUrl),
    }
}

/// `true` for `localhost`, any `127.0.0.0/8` address, or `::1`.
fn is_loopback_host(url: &url::Url) -> bool {
    match url.host() {
        Some(url::Host::Domain(domain)) => domain.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(addr)) => addr.is_loopback(),
        Some(url::Host::Ipv6(addr)) => addr.is_loopback(),
        None => false,
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
///   `crate::listen::flux::FluxHandle::send_data` convention.
/// - [`AgentHandle::send_data`] sends a binary audio frame.
/// - [`AgentHandle::keep_alive`], [`AgentHandle::close`] are
///   control-only (no prefix), matching `FluxHandle::keep_alive` /
///   `FluxHandle::close_stream`.
///
/// The handle and its paired [`AgentEventStream`] are backed by bounded
/// channels. For a long session, keep draining the event stream while you
/// send (typically from separate tasks) rather than queueing an unbounded
/// amount of outgoing audio without reading incoming events.
///
/// Once the session has ended — the server closed the connection or the
/// transport failed — every `send_*` method on a retained handle returns
/// an error rather than silently accepting a message that can no longer
/// be delivered. If the consumer drops the [`AgentEventStream`] instead,
/// the worker only notices when it next has something to deliver (the
/// next inbound frame or close), so on a silent connection sends keep
/// succeeding until then; from that point they fail the same way.
/// [`AgentHandle::close`] on an already-ended session is a no-op.
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

    /// Send an `UpdateListen` message (change the STT model, language
    /// hints, Flux STT end-of-turn thresholds, or keyterms mid-session). The
    /// server confirms with `ListenUpdated`.
    pub async fn send_update_listen(&mut self, message: UpdateListenMessage) -> Result<()> {
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
    /// Named to match `crate::listen::flux::FluxHandle::send_data` for
    /// consistency with the existing audio-streaming convention in this
    /// SDK.
    pub async fn send_data(&mut self, data: Vec<u8>) -> Result<()> {
        self.message_tx
            .send(WsMessage::Audio(data))
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))
    }

    /// Send a `KeepAlive` message.
    ///
    /// Only needed while the client is not sending audio: the server
    /// closes connections that go silent, so during an idle period send
    /// one `KeepAlive` every 8 seconds. Sessions that stream microphone
    /// audio continuously do not need it. `KeepAlive` does not extend the
    /// 2-hour maximum session length; the server closes every session at
    /// that mark (preceded by a `MAXIMUM_SESSION_LENGTH_APPROACHING`
    /// warning five minutes earlier).
    pub async fn keep_alive(&mut self) -> Result<()> {
        self.send_json(&KeepAliveMessage::default()).await
    }

    /// Send a `ForceEndTurn` message — end the current user turn now,
    /// without waiting for end-of-turn detection.
    ///
    /// Requires a Deepgram V2 (Flux STT) listen provider; with any other
    /// provider the server replies with a `FORCE_END_TURN_UNSUPPORTED`
    /// `Warning` and the turn does not end. Typically paired with
    /// `eot_threshold: 1.0` on the listen provider, which suppresses
    /// natural endpointing so the application controls turn boundaries.
    pub async fn force_end_turn(&mut self) -> Result<()> {
        self.send_json(&ForceEndTurnMessage::default()).await
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

    /// Send an arbitrary JSON value as a client message.
    ///
    /// This is the escape hatch for client messages the SDK has not typed
    /// yet: a think or speak provider the API accepts but this release
    /// does not model (for example a `Settings` message whose
    /// `agent.think.provider` is `{"type":"nvidia", ...}`), a field added
    /// to an existing message after this release, or an entirely new
    /// message `type`. Prefer the typed `send_*` methods whenever one
    /// exists.
    ///
    /// The value is serialized with `serde_json` exactly as given — no
    /// fields are added, renamed, or validated client-side — and travels
    /// through the same bounded channel as the typed messages, so it is
    /// delivered in order relative to them. The server answers a
    /// malformed or unsupported message with an `Error` event.
    pub async fn send_raw_json(&mut self, value: serde_json::Value) -> Result<()> {
        self.send_json(&value).await
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
    message_rx: Receiver<WsMessage>,
    response_tx: Sender<Result<AgentEvent>>,
) -> Result<()> {
    let (ws_send, ws_recv) = ws_stream.split();
    drive_agent_session(ws_send, ws_recv, message_rx, response_tx).await
}

/// The worker loop, generic over the two halves of the split socket.
///
/// Generic so the terminal write path can be unit-tested: a real
/// transport cannot be made to fail a write while keeping its read half
/// healthy, which is exactly the case that used to park the worker.
async fn drive_agent_session<Si, St>(
    mut ws_send: Si,
    ws_recv: St,
    mut message_rx: Receiver<WsMessage>,
    mut response_tx: Sender<Result<AgentEvent>>,
) -> Result<()>
where
    Si: futures::Sink<Message, Error = tungstenite::Error> + Unpin,
    St: Stream<Item = std::result::Result<Message, tungstenite::Error>> + Unpin,
{
    let mut ws_recv = ws_recv.fuse();
    let mut is_open = true;

    loop {
        if is_open {
            // Unbiased `select!`: when both the inbound WebSocket and the
            // outbound command channel are ready, the branch is chosen at
            // random, so a server streaming audio continuously cannot
            // starve outgoing audio, function responses, KeepAlives, or
            // the Close request. (`select_biased!` preferring `ws_recv`
            // did exactly that under sustained inbound load.)
            select! {
                inbound = ws_recv.next() => {
                    if let std::ops::ControlFlow::Break(stop) =
                        handle_agent_inbound(inbound, &mut ws_send, &mut response_tx).await
                    {
                        if matches!(stop, InboundStop::TransportGone) {
                            is_open = false;
                        }
                        break;
                    }
                }
                outbound = message_rx.next() => {
                    match outbound {
                        Some(WsMessage::Json(json)) => {
                            if let Err(err) = ws_send.send(Message::Text(json.into())).await {
                                // A failed write is terminal for the
                                // transport: forward it once and end the
                                // worker, matching the Flux TTS worker.
                                // Continuing here would park on the read
                                // half with the command channel still
                                // open, so a retained handle's send would
                                // return `Ok` into a channel nobody
                                // drains — the silent drop the worker
                                // contract forbids.
                                let _ = response_tx.send(Err(err.into())).await;
                                is_open = false;
                                break;
                            }
                        }
                        Some(WsMessage::Audio(audio)) => {
                            if let Err(err) =
                                ws_send.send(Message::Binary(Bytes::from(audio))).await
                            {
                                // Terminal, as above.
                                let _ = response_tx.send(Err(err.into())).await;
                                is_open = false;
                                break;
                            }
                        }
                        Some(WsMessage::Close) | None => {
                            let _ = ws_send.send(Message::Close(None)).await;
                            is_open = false;
                        }
                    }
                }
            }
        } else {
            // Input is closed: only drain server messages until the connection
            // closes. We must NOT keep selecting on `message_rx` here — it now
            // yields `Ready(None)` synchronously, which would busy-spin the task
            // (and hang a current-thread runtime).
            if let std::ops::ControlFlow::Break(stop) =
                handle_agent_inbound(ws_recv.next().await, &mut ws_send, &mut response_tx).await
            {
                if matches!(stop, InboundStop::TransportGone) {
                    is_open = false;
                }
                break;
            }
        }
    }

    // Terminal cleanup — reached on peer close, transport error, or when the
    // consumer dropped the event stream. Order matters:
    //
    // 1. Close the command channel *first* so every retained `AgentHandle`
    //    send fails promptly instead of being accepted and silently dropped.
    //    (`Receiver::close` rejects new sends while letting us drain what is
    //    already buffered.)
    message_rx.close();
    // 2. Discard anything buffered. Non-blocking: `try_recv` returns `Err`
    //    once the (closed) channel is empty rather than waiting on senders.
    while message_rx.try_recv().is_ok() {}
    // 3. If we stopped while the connection was still open (e.g. the
    //    consumer dropped the event stream), tell the peer we're going away.
    if is_open {
        let _ = ws_send.send(Message::Close(None)).await;
    }
    // 4. Signal end-of-stream to the consumer and return immediately — the
    //    worker must not linger until every handle is dropped.
    response_tx.close_channel();
    Ok(())
}

/// Why [`handle_agent_inbound`] asked the worker to stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InboundStop {
    /// The socket can still be written to, so terminal cleanup should
    /// send a Close frame: either we are answering a peer close or the
    /// consumer dropped the event stream while the session was healthy.
    TransportUsable,
    /// The transport is gone (read error or end of stream); do not try to
    /// write to it again.
    TransportGone,
}

/// Handle a single inbound WebSocket message. Returns [`ControlFlow::Break`]
/// when the worker should stop (connection closed, a terminal read error, or
/// the consumer dropped the event stream), carrying whether the socket is
/// still writable.
async fn handle_agent_inbound<S>(
    inbound: Option<std::result::Result<Message, tungstenite::Error>>,
    ws_send: &mut S,
    response_tx: &mut Sender<Result<AgentEvent>>,
) -> std::ops::ControlFlow<InboundStop>
where
    S: futures::Sink<Message> + Unpin,
{
    use std::ops::ControlFlow;
    match inbound {
        Some(Ok(Message::Text(text))) => {
            let parsed: std::result::Result<AgentResponse, _> = serde_json::from_str(&text);
            let event = parsed.map(AgentEvent::Json).map_err(DeepgramError::from);
            if response_tx.send(event).await.is_err() {
                return ControlFlow::Break(InboundStop::TransportUsable);
            }
        }
        Some(Ok(Message::Binary(bytes))) => {
            if response_tx
                .send(Ok(AgentEvent::Audio(bytes)))
                .await
                .is_err()
            {
                return ControlFlow::Break(InboundStop::TransportUsable);
            }
        }
        Some(Ok(Message::Ping(payload))) => {
            // Ignore failures: if the WS is gone the next recv will tell us.
            let _ = ws_send.send(Message::Pong(payload)).await;
        }
        Some(Ok(Message::Pong(_))) => {
            // Server-emitted pongs are unexpected; ignore.
        }
        Some(Ok(Message::Close(None))) => return ControlFlow::Break(InboundStop::TransportUsable),
        Some(Ok(Message::Close(Some(frame)))) => {
            // A normal (1000) close is not an error: the server ends every
            // session this way (after a client-initiated close and at the
            // 2-hour session cap). Only abnormal close codes reach the
            // consumer, matching the streaming TTS worker.
            if u16::from(frame.code) != 1000 {
                let _ = response_tx
                    .send(Err(DeepgramError::WebsocketClose {
                        code: frame.code.into(),
                        reason: frame.reason.to_string(),
                    }))
                    .await;
            }
            return ControlFlow::Break(InboundStop::TransportUsable);
        }
        Some(Ok(Message::Frame(_))) => {
            // Unfragmented Frame deliveries; tungstenite normally surfaces
            // these as Text/Binary. Ignore anything raw that slips through.
        }
        Some(Err(err)) => {
            // A read error is terminal for the transport: forward it once
            // and end the worker, rather than keep polling a broken
            // socket (which can surface duplicate errors) or accept
            // further commands doomed to fail. Matches the Flux TTS
            // worker.
            let _ = response_tx.send(Err(err.into())).await;
            return ControlFlow::Break(InboundStop::TransportGone);
        }
        None => return ControlFlow::Break(InboundStop::TransportGone),
    }
    ControlFlow::Continue(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

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

    /// A write half that fails every send, paired with a read half that
    /// never yields — a healthy but silent socket. No real transport can
    /// be put in this state on demand, which is why the worker loop is
    /// generic over its two halves.
    struct FailingSink;

    impl futures::Sink<Message> for FailingSink {
        type Error = tungstenite::Error;

        fn poll_ready(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn start_send(
            self: Pin<&mut Self>,
            _item: Message,
        ) -> std::result::Result<(), Self::Error> {
            Err(tungstenite::Error::AlreadyClosed)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
    }

    struct SilentStream;

    impl Stream for SilentStream {
        type Item = std::result::Result<Message, tungstenite::Error>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    /// A failed write ends the worker after forwarding exactly one error,
    /// and the terminal cleanup closes the command channel so a retained
    /// handle's next send fails.
    ///
    /// Regression: the write-error arms used to forward the error and
    /// mark the session closed but leave the command channel open, then
    /// park on the read half. With a read half that stays silent — a
    /// tungstenite write error that leaves the read half healthy — the
    /// worker never returned, so `send_data` / `keep_alive` on a retained
    /// handle returned `Ok` into a channel nobody drained. The worker
    /// contract requires an error instead.
    #[tokio::test]
    async fn write_failure_ends_worker_after_single_error_and_fails_later_sends() {
        let (message_tx, message_rx) = mpsc::channel::<WsMessage>(8);
        let (response_tx, mut response_rx) = mpsc::channel::<Result<AgentEvent>>(8);

        let worker = tokio::spawn(drive_agent_session(
            FailingSink,
            SilentStream,
            message_rx,
            response_tx,
        ));

        let mut handle = AgentHandle {
            message_tx,
            request_id: None,
        };
        // Accepted into the channel; the worker's write of it fails.
        handle.send_data(vec![0u8; 8]).await.unwrap();

        let first = tokio::time::timeout(Duration::from_secs(5), response_rx.next())
            .await
            .expect("the worker must end promptly after a failed write")
            .expect("the terminal error must be forwarded");
        assert!(first.is_err(), "expected the write error, got {first:?}");
        assert!(
            tokio::time::timeout(Duration::from_secs(5), response_rx.next())
                .await
                .expect("the event stream must end")
                .is_none(),
            "exactly one error may be forwarded for one fault"
        );

        tokio::time::timeout(Duration::from_secs(5), worker)
            .await
            .expect("the worker task must return")
            .expect("the worker task must not panic")
            .expect("the worker returns Ok after terminal cleanup");

        assert!(
            handle.send_data(vec![0u8; 8]).await.is_err(),
            "send_data after a terminal write error must fail, not be dropped"
        );
        assert!(
            handle.keep_alive().await.is_err(),
            "keep_alive after a terminal write error must fail, not be dropped"
        );
        assert!(handle.close().await.is_ok(), "close is a no-op once ended");
    }

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
        serde_json::to_string(&UpdateListenMessage::new(AgentListenSettings::new(
            AgentListenProvider::DeepgramV2(DeepgramListenV2Provider::new("flux-general-en")),
        )))
        .expect("update_listen serializes");
        serde_json::to_string(&ForceEndTurnMessage::default()).expect("force_end_turn serializes");
    }

    fn parse(url: &str) -> url::Url {
        url.parse().expect("valid URL")
    }

    #[test]
    fn tls_rule_allows_wss_anywhere() {
        assert!(validate_agent_url(&parse("wss://agent.deepgram.com/v1/agent/converse")).is_ok());
        assert!(validate_agent_url(&parse("wss://agent.internal.example:8443/agent")).is_ok());
        assert!(validate_agent_url(&parse(AGENT_WS_URL)).is_ok());
    }

    #[test]
    fn tls_rule_allows_cleartext_only_for_loopback() {
        for ok in [
            "ws://127.0.0.1:9000/v1/agent/converse",
            "ws://127.0.0.1/v1/agent/converse",
            "ws://127.42.0.7:1234/",
            "ws://localhost:9000/",
            "ws://LOCALHOST:9000/",
            "ws://[::1]:9000/",
        ] {
            assert!(
                validate_agent_url(&parse(ok)).is_ok(),
                "{ok} should be allowed"
            );
        }
    }

    #[test]
    fn tls_rule_rejects_remote_cleartext_with_clear_error() {
        for bad in [
            "ws://agent.deepgram.com/v1/agent/converse",
            "ws://10.0.0.5:9000/agent",
            "ws://192.168.1.20/agent",
            "ws://[2001:db8::1]:9000/agent",
            "ws://localhost.example.com/agent",
            "ws://evil-localhost/agent",
        ] {
            let err = validate_agent_url(&parse(bad)).expect_err(bad);
            let insecure = match &err {
                DeepgramError::InsecureAgentUrl(insecure) => insecure,
                other => panic!("{bad}: expected InsecureAgentUrl, got {other:?}"),
            };
            let parsed = parse(bad);
            let expected_origin = format!(
                "ws://{}",
                crate::websocket_host_header(&parsed).expect("bad URLs here all have hosts")
            );
            assert_eq!(insecure.url, expected_origin, "{bad}");
            let text = err.to_string();
            assert!(text.contains("cleartext"), "{bad}: {text}");
            assert!(text.contains("wss://"), "{bad}: {text}");
            assert!(text.contains("Authorization"), "{bad}: {text}");
        }
    }

    /// The rejection error is likely to be logged, so it must not echo
    /// userinfo, path, or query from the URL that was passed in.
    #[test]
    fn insecure_url_error_omits_userinfo_path_and_query() {
        let bad = "ws://alice:s3cret@agent.example.com:8080/v1/agent/converse?token=abc";
        let err = validate_agent_url(&parse(bad)).expect_err(bad);
        let DeepgramError::InsecureAgentUrl(insecure) = &err else {
            panic!("expected InsecureAgentUrl, got {err:?}");
        };
        assert_eq!(insecure.url, "ws://agent.example.com:8080");
        let text = err.to_string();
        for secret in ["alice", "s3cret", "token=abc", "/v1/agent/converse"] {
            assert!(
                !text.contains(secret),
                "error text leaked {secret:?}: {text}"
            );
        }
        assert!(text.contains("ws://agent.example.com:8080"), "{text}");
    }

    #[test]
    fn tls_rule_rejects_non_websocket_schemes_as_invalid_url() {
        for bad in [
            "http://127.0.0.1:9000/",
            "https://agent.deepgram.com/",
            "ftp://x/",
        ] {
            assert!(
                matches!(
                    validate_agent_url(&parse(bad)),
                    Err(DeepgramError::InvalidUrl)
                ),
                "{bad}"
            );
        }
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
