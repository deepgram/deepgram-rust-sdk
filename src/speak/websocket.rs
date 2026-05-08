//! Streaming TTS WebSocket connection at `wss://api.deepgram.com/v1/speak`.
//!
//! Mirrors the AsyncAPI definition in `asyncapi/channels/speak.v1.yml`
//! and `asyncapi/schemas/schemas.speak.v1.yml`. Same pattern as
//! [`crate::agent::websocket`]: a [`SpeakHandle`] for outgoing
//! text/control messages and a [`SpeakStream`] of incoming
//! [`SpeakResponse`] events.
//!
//! # Example
//!
//! ```no_run
//! use deepgram::Deepgram;
//! use deepgram::speak::options::Model;
//! use deepgram::speak::response::SpeakResponse;
//! use futures::StreamExt;
//!
//! # async fn run() -> Result<(), deepgram::DeepgramError> {
//! let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap_or_default())?;
//! let (mut handle, mut stream) = dg
//!     .text_to_speech()
//!     .websocket()
//!     .model(Model::AuraAsteriaEn)
//!     .sample_rate(24_000)
//!     .start()
//!     .await?;
//!
//! handle.send_text("Hello from Deepgram streaming TTS.").await?;
//! handle.flush().await?;
//!
//! while let Some(event) = stream.next().await {
//!     match event? {
//!         SpeakResponse::Audio(_bytes) => { /* play out */ }
//!         SpeakResponse::Metadata(_) => {}
//!         SpeakResponse::Flushed(_) => break,
//!         _ => {}
//!     }
//! }
//! handle.close().await?;
//! # Ok(())
//! # }
//! ```

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::channel::mpsc::{self, Receiver, Sender};
use futures::stream::StreamExt;
use futures::{select_biased, SinkExt, Stream};
use http::Request;
use pin_project::pin_project;
use serde::Serialize;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::handshake::client;
use uuid::Uuid;

use crate::speak::options::{Encoding, Model};
use crate::speak::response::SpeakResponse;
use crate::{Deepgram, DeepgramError, Result, Speak};

const SPEAK_WS_PATH: &str = "v1/speak";

/// Builder for a streaming TTS WebSocket session.
#[derive(Debug, Clone)]
pub struct WebsocketBuilder<'a> {
    deepgram: &'a Deepgram,
    model: Option<Model>,
    encoding: Option<Encoding>,
    sample_rate: Option<u32>,
    speed: Option<f64>,
    mip_opt_out: Option<bool>,
    /// Optional override of the connection URL. When `None`, the URL is
    /// derived from the `Deepgram` client's base URL with the WS scheme.
    url_override: Option<String>,
}

impl<'a> Speak<'a> {
    /// Begin configuring a streaming TTS WebSocket session.
    ///
    /// Once configured, call [`WebsocketBuilder::start`] to open the
    /// connection. Returns a [`SpeakHandle`] for sending text/control
    /// messages and a [`SpeakStream`] of incoming
    /// [`SpeakResponse`](crate::speak::response::SpeakResponse) events.
    pub fn websocket(&self) -> WebsocketBuilder<'a> {
        WebsocketBuilder {
            deepgram: self.0,
            model: None,
            encoding: None,
            sample_rate: None,
            speed: None,
            mip_opt_out: None,
            url_override: None,
        }
    }
}

impl<'a> WebsocketBuilder<'a> {
    /// Set the TTS voice model.
    pub fn model(mut self, model: Model) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the audio encoding (`linear16`, `mulaw`, `alaw` for streaming TTS).
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = Some(encoding);
        self
    }

    /// Set the audio sample rate in Hz.
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    /// Set the speaking-rate multiplier (spec range: 0.7 – 1.5).
    pub fn speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Opt out of the Deepgram Model Improvement Program for this session.
    pub fn mip_opt_out(mut self, mip_opt_out: bool) -> Self {
        self.mip_opt_out = Some(mip_opt_out);
        self
    }

    /// Override the connection URL. Useful for self-hosted deployments
    /// and integration tests against a local mock server.
    ///
    /// When unset, the URL is derived from the [`Deepgram`] client's
    /// base URL with `v1/speak` appended and `https`/`http` rewritten
    /// to `wss`/`ws`.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url_override = Some(url.into());
        self
    }

    /// Open the WebSocket session.
    pub async fn start(self) -> Result<(SpeakHandle, SpeakStream)> {
        let url = self.connect_url()?;
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

            let http_builder = if let Some(auth) = &self.deepgram.auth {
                http_builder.header("authorization", auth.header_value())
            } else {
                http_builder
            };
            http_builder.body(())?
        };

        let (ws_stream, upgrade_response) = tokio_tungstenite::connect_async(request).await?;

        let request_id = upgrade_response
            .headers()
            .get("dg-request-id")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok());

        let (message_tx, message_rx) = mpsc::channel(256);
        let (response_tx, response_rx) = mpsc::channel(256);

        tokio::task::spawn(run_speak_worker(ws_stream, message_rx, response_tx));

        let handle = SpeakHandle {
            message_tx,
            request_id,
        };
        let stream = SpeakStream {
            rx: response_rx,
            request_id,
        };
        Ok((handle, stream))
    }

    fn connect_url(&self) -> Result<url::Url> {
        if let Some(custom) = &self.url_override {
            let mut url: url::Url = custom.parse().map_err(|_| DeepgramError::InvalidUrl)?;
            self.append_query_params(&mut url);
            return Ok(url);
        }

        let mut url = self
            .deepgram
            .base_url
            .join(SPEAK_WS_PATH)
            .map_err(|_| DeepgramError::InvalidUrl)?;

        match url.scheme() {
            "http" | "ws" => url
                .set_scheme("ws")
                .map_err(|_| DeepgramError::InvalidUrl)?,
            "https" | "wss" => url
                .set_scheme("wss")
                .map_err(|_| DeepgramError::InvalidUrl)?,
            _ => return Err(DeepgramError::InvalidUrl),
        }

        self.append_query_params(&mut url);
        Ok(url)
    }

    fn append_query_params(&self, url: &mut url::Url) {
        let mut pairs = url.query_pairs_mut();
        if let Some(model) = &self.model {
            pairs.append_pair("model", model.as_ref());
        }
        if let Some(encoding) = &self.encoding {
            pairs.append_pair("encoding", encoding.as_str());
        }
        if let Some(sample_rate) = self.sample_rate {
            pairs.append_pair("sample_rate", &sample_rate.to_string());
        }
        if let Some(speed) = self.speed {
            pairs.append_pair("speed", &speed.to_string());
        }
        if let Some(mip_opt_out) = self.mip_opt_out {
            pairs.append_pair("mip_opt_out", &mip_opt_out.to_string());
        }
    }
}

/// Handle for sending messages on a live Speak WebSocket session.
#[derive(Debug)]
pub struct SpeakHandle {
    message_tx: Sender<WsMessage>,
    request_id: Option<Uuid>,
}

impl SpeakHandle {
    /// `dg-request-id` from the upgrade response, if present.
    pub fn request_id(&self) -> Option<Uuid> {
        self.request_id
    }

    /// Send text to be synthesized.
    ///
    /// Multiple calls before a `Flush` are concatenated server-side; the
    /// agent emits audio incrementally as it generates it.
    pub async fn send_text(&mut self, text: impl Into<String>) -> Result<()> {
        let msg = SpeakTextMessage {
            message_type: SpeakTextType::Speak,
            text: text.into(),
        };
        self.send_json(&msg).await
    }

    /// Force the server to emit any pending audio for everything sent so far.
    pub async fn flush(&mut self) -> Result<()> {
        self.send_json(&ControlMessage::Flush).await
    }

    /// Discard any text the server has buffered but not yet synthesized.
    pub async fn clear(&mut self) -> Result<()> {
        self.send_json(&ControlMessage::Clear).await
    }

    /// Gracefully close the WebSocket. After this returns, `send_text`
    /// and the control methods will fail.
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

/// Stream of [`SpeakResponse`] events from the server.
#[derive(Debug)]
#[pin_project]
pub struct SpeakStream {
    #[pin]
    rx: Receiver<Result<SpeakResponse>>,
    request_id: Option<Uuid>,
}

impl SpeakStream {
    /// Same `dg-request-id` reported by [`SpeakHandle::request_id`].
    pub fn request_id(&self) -> Option<Uuid> {
        self.request_id
    }
}

impl Stream for SpeakStream {
    type Item = Result<SpeakResponse>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().rx.poll_next(cx)
    }
}

// ---------- client→server messages (private) ----------

/// `Speak` message — wraps text to synthesize.
#[derive(Debug, Serialize)]
struct SpeakTextMessage {
    #[serde(rename = "type")]
    message_type: SpeakTextType,
    text: String,
}

#[derive(Debug, Serialize)]
enum SpeakTextType {
    Speak,
}

/// `Flush`/`Clear`/`Close` control messages share a shape: just `{type}`.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ControlMessage {
    Flush,
    Clear,
    #[allow(dead_code)] // reserved; we use a WS Close frame instead today
    Close,
}

// ---------- worker plumbing ----------

#[derive(Debug)]
enum WsMessage {
    Json(String),
    Close,
}

async fn run_speak_worker(
    ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    mut message_rx: Receiver<WsMessage>,
    mut response_tx: Sender<Result<SpeakResponse>>,
) -> Result<()> {
    let (mut ws_send, ws_recv) = ws_stream.split();
    let mut ws_recv = ws_recv.fuse();
    let mut is_open = true;

    loop {
        select_biased! {
            inbound = ws_recv.next() => {
                match inbound {
                    Some(Ok(Message::Text(text))) => {
                        let event: std::result::Result<SpeakResponse, _> =
                            serde_json::from_str(&text);
                        let event = event.map_err(DeepgramError::from);
                        if response_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Binary(bytes))) => {
                        let event = Ok(SpeakResponse::Audio(bytes));
                        if response_tx.send(event).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = ws_send.send(Message::Pong(payload)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    Some(Ok(Message::Close(None))) => return Ok(()),
                    Some(Ok(Message::Close(Some(frame)))) => {
                        let err = DeepgramError::WebsocketClose {
                            code: frame.code.into(),
                            reason: frame.reason.to_string(),
                        };
                        let _ = response_tx.send(Err(err)).await;
                        return Ok(());
                    }
                    Some(Ok(Message::Frame(_))) => {}
                    Some(Err(err)) => {
                        if response_tx.send(Err(err.into())).await.is_err() {
                            break;
                        }
                    }
                    None => return Ok(()),
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
    while message_rx.next().await.is_some() {}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speak::options::Model;

    #[test]
    fn url_with_query_params() {
        let dg = Deepgram::new("test-key").unwrap();
        let speak = dg.text_to_speech();
        let builder = speak
            .websocket()
            .model(Model::AuraAsteriaEn)
            .encoding(Encoding::Linear16)
            .sample_rate(24_000)
            .speed(1.0)
            .mip_opt_out(true);
        let url = builder.connect_url().unwrap();
        assert_eq!(url.scheme(), "wss");
        assert_eq!(url.host_str(), Some("api.deepgram.com"));
        assert_eq!(url.path(), "/v1/speak");
        let q: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(q.get("model").map(String::as_str), Some("aura-asteria-en"));
        assert_eq!(q.get("encoding").map(String::as_str), Some("linear16"));
        assert_eq!(q.get("sample_rate").map(String::as_str), Some("24000"));
        assert_eq!(q.get("speed").map(String::as_str), Some("1"));
        assert_eq!(q.get("mip_opt_out").map(String::as_str), Some("true"));
    }

    #[test]
    fn url_override_takes_precedence() {
        let dg = Deepgram::new("test-key").unwrap();
        let speak = dg.text_to_speech();
        let builder = speak
            .websocket()
            .url("ws://127.0.0.1:9999/v1/speak")
            .model(Model::AuraAsteriaEn);
        let url = builder.connect_url().unwrap();
        assert_eq!(url.scheme(), "ws");
        assert_eq!(url.host_str(), Some("127.0.0.1"));
        assert_eq!(url.port(), Some(9999));
        let q: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(q.get("model").map(String::as_str), Some("aura-asteria-en"));
    }

    #[test]
    fn speak_text_message_serializes() {
        let msg = SpeakTextMessage {
            message_type: SpeakTextType::Speak,
            text: "hello".into(),
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json, serde_json::json!({"type": "Speak", "text": "hello"}));
    }

    #[test]
    fn control_messages_serialize() {
        for (msg, expected) in [
            (ControlMessage::Flush, "Flush"),
            (ControlMessage::Clear, "Clear"),
            (ControlMessage::Close, "Close"),
        ] {
            let v = serde_json::to_value(&msg).unwrap();
            assert_eq!(v, serde_json::json!({"type": expected}));
        }
    }
}
