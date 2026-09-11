//! Streaming Text-to-Speech over the `/v1/speak` WebSocket (Aura models).
//!
//! The TTS WebSocket lets you stream text in (for example, as an LLM produces
//! it) and receive synthesized audio out with low latency. Construct a request
//! with [`Speak::speak_stream`], configure it, then open the connection with
//! [`SpeakStreamBuilder::handle`].
//!
//! This is the Aura (`aura-*` models) streaming endpoint. For the turn-based
//! Flux TTS WebSocket (`/v2/speak`, `flux-*` models) use
//! [`Speak::flux_request`](crate::Speak::flux_request) from
//! [`crate::speak::flux`] instead.
//!
//! See the [Deepgram TTS WebSocket API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/text-to-speech/speak-streaming

use anyhow::anyhow;
use bytes::Bytes;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    future::poll_fn,
    pin_mut, select,
    stream::StreamExt,
    FutureExt, SinkExt, Stream,
};
use http::Request;
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio_tungstenite::tungstenite::protocol::Message;
use tungstenite::handshake::client;
use tungstenite::Utf8Bytes;
use url::Url;
use uuid::Uuid;

use super::options::Model;
use crate::speak::options::Encoding;
use crate::{Deepgram, DeepgramError, Result, Speak};

static SPEAK_STREAM_URL_PATH: &str = "v1/speak";

/// A builder for a streaming Text-to-Speech WebSocket request.
///
/// Construct one with [`Speak::speak_stream`].
#[derive(Debug, Clone)]
pub struct SpeakStreamBuilder<'a> {
    deepgram: &'a Deepgram,
    model: Option<Model>,
    encoding: Option<Encoding>,
    sample_rate: Option<u32>,
    speed: Option<f32>,
    mip_opt_out: Option<bool>,
    query_params: Vec<(String, String)>,
    stream_url: Url,
}

/// The inclusive range of values accepted for the streaming TTS `speed`
/// parameter.
const SPEED_RANGE: std::ops::RangeInclusive<f32> = 0.7..=1.5;

/// The output sample rates (Hz) accepted by the streaming TTS endpoint.
const SAMPLE_RATES: [u32; 5] = [8000, 16000, 24000, 32000, 48000];

/// After a `Close` has been sent, how long the worker waits for the *next*
/// server frame before giving up on the server and closing the socket itself.
/// The server streams the remaining audio continuously after a `Close`, so a
/// gap this long means it is not going to finish.
const CLOSE_DRAIN_IDLE_TIMEOUT: Duration = Duration::from_secs(10);

impl<'a> Speak<'a> {
    /// Begin configuring a streaming Text-to-Speech WebSocket request.
    ///
    /// Once configured, open the connection with
    /// [`SpeakStreamBuilder::handle`].
    ///
    /// ```
    /// use deepgram::{speak::options::Encoding, Deepgram};
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap_or_default()).unwrap();
    /// let builder = dg.text_to_speech().speak_stream().encoding(Encoding::Linear16);
    /// ```
    pub fn speak_stream(&self) -> SpeakStreamBuilder<'a> {
        SpeakStreamBuilder {
            deepgram: self.0,
            model: None,
            encoding: None,
            sample_rate: None,
            speed: None,
            mip_opt_out: None,
            query_params: Vec::new(),
            stream_url: self.speak_stream_url(),
        }
    }

    fn speak_stream_url(&self) -> Url {
        let mut url = self
            .0
            .base_url
            .join(SPEAK_STREAM_URL_PATH)
            .expect("base_url is validated on construction");
        match url.scheme() {
            "http" | "ws" => url
                .set_scheme("ws")
                .expect("a valid conversion according to the set_scheme docs"),
            "https" | "wss" => url
                .set_scheme("wss")
                .expect("a valid conversion according to the set_scheme docs"),
            _ => unreachable!("base_url scheme is validated on construction"),
        }
        url
    }
}

impl<'a> SpeakStreamBuilder<'a> {
    /// Set the TTS model / voice.
    pub fn model(mut self, model: Model) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the output audio encoding.
    ///
    /// The streaming endpoint emits raw (non-containerized) audio only, so it
    /// supports just [`Encoding::Linear16`], [`Encoding::Mulaw`], and
    /// [`Encoding::Alaw`]. The compressed / containerized REST encodings
    /// (`mp3`, `opus`, `flac`, `aac`) are rejected locally by
    /// [`handle`](Self::handle) with [`DeepgramError::InvalidOptions`] before
    /// a connection is opened. [`Encoding::CustomEncoding`] is passed through
    /// unchecked as an escape hatch.
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = Some(encoding);
        self
    }

    /// Set the output sample rate in Hz.
    ///
    /// The streaming endpoint accepts `8000`, `16000`, `24000`, `32000`, and
    /// `48000` (the default is `24000`); any other value is rejected locally by
    /// [`handle`](Self::handle) with [`DeepgramError::InvalidOptions`]. The
    /// server further restricts `mulaw` and `alaw` output to `8000` and
    /// `16000`. To send a value outside this list, leave this unset and pass
    /// it through [`query_params`](Self::query_params) instead.
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    /// Set the speaking-rate multiplier (`1.0` is the model's nominal rate).
    ///
    /// The documented range is `0.7..=1.5`; values outside it are rejected
    /// locally by [`handle`](Self::handle) with
    /// [`DeepgramError::InvalidOptions`]. Not yet supported for all models
    /// and languages.
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Opt this request out of the [Deepgram Model Improvement Program][mip].
    ///
    /// Refer to the docs for pricing impacts before setting this to `true`.
    ///
    /// [mip]: https://dpgr.am/deepgram-mip
    pub fn mip_opt_out(mut self, mip_opt_out: bool) -> Self {
        self.mip_opt_out = Some(mip_opt_out);
        self
    }

    /// Append extra query parameters to the end of the WebSocket URL. Prefer
    /// the typed builder methods; this exists as an escape hatch for using
    /// parameters before they have been added to the SDK.
    ///
    /// Calling this twice will add both sets of parameters. The parameters
    /// are not validated locally.
    ///
    /// # Examples
    ///
    /// ```
    /// use deepgram::Deepgram;
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap_or_default()).unwrap();
    /// let builder = dg
    ///     .text_to_speech()
    ///     .speak_stream()
    ///     .query_params([("extra".to_string(), "parameter".to_string())]);
    /// ```
    pub fn query_params(mut self, params: impl IntoIterator<Item = (String, String)>) -> Self {
        self.query_params.extend(params);
        self
    }

    /// Check the configured options against the streaming endpoint's
    /// contract so callers get a clear local error instead of a server-side
    /// rejection after the socket is opened.
    fn validate(&self) -> Result<()> {
        if let Some(encoding) = &self.encoding {
            match encoding {
                Encoding::Linear16 | Encoding::Mulaw | Encoding::Alaw => {}
                Encoding::Mp3 | Encoding::Opus | Encoding::Flac | Encoding::Aac => {
                    return Err(DeepgramError::InvalidOptions(format!(
                        "encoding `{}` is not supported by the streaming TTS endpoint; \
                         use `linear16`, `mulaw`, or `alaw`",
                        encoding.as_str()
                    )));
                }
                // Escape hatch: let the server decide.
                Encoding::CustomEncoding(_) => {}
            }
        }
        if let Some(sample_rate) = self.sample_rate {
            if !SAMPLE_RATES.contains(&sample_rate) {
                return Err(DeepgramError::InvalidOptions(format!(
                    "sample_rate `{sample_rate}` is not supported by the streaming TTS endpoint; \
                     use one of 8000, 16000, 24000, 32000, or 48000"
                )));
            }
        }
        if let Some(speed) = self.speed {
            if !speed.is_finite() || !SPEED_RANGE.contains(&speed) {
                return Err(DeepgramError::InvalidOptions(format!(
                    "speed `{speed}` is out of range; the streaming TTS endpoint accepts {}..={}",
                    SPEED_RANGE.start(),
                    SPEED_RANGE.end()
                )));
            }
        }
        Ok(())
    }

    fn as_url(&self) -> Url {
        let mut url = self.stream_url.clone();
        {
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
            pairs.extend_pairs(&self.query_params);
        }
        url
    }

    /// Open the WebSocket connection and return a [`SpeakStreamHandle`] for
    /// sending text and receiving audio.
    ///
    /// Returns [`DeepgramError::InvalidOptions`] without connecting if the
    /// configured [`encoding`](Self::encoding) is not supported by the
    /// streaming endpoint, [`sample_rate`](Self::sample_rate) is not one of
    /// the accepted rates, or [`speed`](Self::speed) is out of range.
    pub async fn handle(self) -> Result<SpeakStreamHandle> {
        self.validate()?;
        SpeakStreamHandle::new(self).await
    }
}

/// A message sent from the client to the TTS WebSocket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Speak { text: String },
    Flush,
    Clear,
    Close,
}

/// A message received from the TTS WebSocket.
///
/// Audio arrives as [`SpeakResponse::Audio`]; the remaining variants are the
/// control/metadata events emitted by the server.
///
/// See the [Deepgram TTS WebSocket API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/reference/text-to-speech/speak-streaming
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SpeakResponse {
    /// A chunk of synthesized audio.
    Audio(Bytes),

    /// Metadata about the audio generation, emitted once when the connection
    /// opens.
    #[non_exhaustive]
    Metadata {
        /// The unique identifier for the request.
        request_id: String,
        /// The name of the model used.
        model_name: Option<String>,
        /// The version of the model used.
        model_version: Option<String>,
        /// The unique identifier of the model used.
        model_uuid: Option<String>,
        /// Unique identifiers of any additional models used to generate the
        /// audio, if the server reported them.
        additional_model_uuids: Option<Vec<Uuid>>,
    },

    /// Emitted after a `Flush`, once all buffered audio has been sent.
    #[non_exhaustive]
    Flushed {
        /// The sequence identifier of the flushed segment.
        sequence_id: Option<u32>,
    },

    /// Emitted after a `Clear`, confirming the buffer was cleared.
    #[non_exhaustive]
    Cleared {
        /// The sequence identifier of the cleared segment.
        sequence_id: Option<u32>,
    },

    /// A non-fatal warning from the server.
    #[non_exhaustive]
    Warning {
        /// A human-readable description of the warning.
        description: Option<String>,
        /// A machine-readable warning code.
        code: Option<String>,
    },

    /// An unrecognized text message, preserved as raw JSON for
    /// forward-compatibility.
    Unknown(String),
}

/// The typed text events emitted by the server (everything except binary
/// audio). Deserialized from the message's `type` field.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum TextEvent {
    Metadata {
        request_id: String,
        model_name: Option<String>,
        model_version: Option<String>,
        model_uuid: Option<String>,
        #[serde(default)]
        additional_model_uuids: Option<Vec<Uuid>>,
    },
    Flushed {
        sequence_id: Option<u32>,
    },
    Cleared {
        sequence_id: Option<u32>,
    },
    Warning {
        description: Option<String>,
        code: Option<String>,
    },
}

impl From<TextEvent> for SpeakResponse {
    fn from(event: TextEvent) -> Self {
        match event {
            TextEvent::Metadata {
                request_id,
                model_name,
                model_version,
                model_uuid,
                additional_model_uuids,
            } => SpeakResponse::Metadata {
                request_id,
                model_name,
                model_version,
                model_uuid,
                additional_model_uuids,
            },
            TextEvent::Flushed { sequence_id } => SpeakResponse::Flushed { sequence_id },
            TextEvent::Cleared { sequence_id } => SpeakResponse::Cleared { sequence_id },
            TextEvent::Warning { description, code } => {
                SpeakResponse::Warning { description, code }
            }
        }
    }
}

fn parse_text_message(text: &str) -> SpeakResponse {
    match serde_json::from_str::<TextEvent>(text) {
        Ok(event) => event.into(),
        // Unknown message type or shape: preserve the raw JSON rather than
        // breaking the stream.
        Err(_) => SpeakResponse::Unknown(text.to_string()),
    }
}

/// A handle to an open Text-to-Speech WebSocket.
///
/// Send text with [`speak`](Self::speak), force generation with
/// [`flush`](Self::flush), and end the session with [`close`](Self::close).
/// Receive audio and events by consuming the handle as a [`futures::Stream`]
/// or by calling [`receive`](Self::receive).
///
/// # Sending and receiving concurrently
///
/// To send text while audio is still arriving — the usual shape when the
/// text comes from an LLM and the audio goes to a player — call
/// [`split`](Self::split) and drive the two halves from separate tasks, or
/// take a [`sender`](Self::sender) clone for the producing task while this
/// handle keeps draining events:
///
/// ```no_run
/// # use deepgram::{speak::SpeakResponse, Deepgram};
/// # use futures::StreamExt;
/// # async fn run() -> Result<(), deepgram::DeepgramError> {
/// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap_or_default())?;
/// let handle = dg.text_to_speech().speak_stream().handle().await?;
/// let (sender, mut events) = handle.split();
///
/// let producer = tokio::spawn(async move {
///     sender.speak("Hello, ").await?;
///     sender.speak("world.").await?;
///     sender.flush().await?;
///     sender.close().await
/// });
///
/// while let Some(event) = events.next().await {
///     if let SpeakResponse::Audio(chunk) = event? {
///         // play or buffer `chunk`
///         let _ = chunk;
///     }
/// }
/// producer.await.expect("producer task")?;
/// # Ok(()) }
/// ```
///
/// Text is written to the socket as soon as it is enqueued, independently of
/// how quickly events are consumed, so an unconsumed backlog of audio never
/// blocks `speak`. Audio is delivered in order, with backpressure: when
/// events are not being consumed the worker stops reading from the socket.
///
/// # Lifecycle
///
/// [`close`](Self::close) sends `Close`; the server then finishes the
/// remaining audio and closes the connection, after which the event stream
/// ends. Dropping the handle (or the last sender and the events half) without
/// calling `close` also sends `Close` and then closes the WebSocket. If the
/// server sends nothing for ten seconds after a `Close`, the worker closes
/// the socket itself and the stream ends with a
/// [`DeepgramError::UnexpectedServerResponse`].
///
/// [`request_id`](SpeakStreamHandle::request_id) is taken from the connection
/// upgrade headers and is [`Uuid::nil`] if the server did not provide one.
#[derive(Debug)]
#[pin_project]
pub struct SpeakStreamHandle {
    sender: SpeakStreamSender,
    #[pin]
    events: SpeakStreamEvents,
}

/// The sending half of a [`SpeakStreamHandle`], obtained from
/// [`SpeakStreamHandle::split`] or [`SpeakStreamHandle::sender`].
///
/// It is `Clone` and every method takes `&self`, so several tasks can feed
/// the same session. Messages from one sender are written in the order they
/// were enqueued. Once the session has ended (after [`close`](Self::close),
/// or once the server closes the connection) every method returns an error.
#[derive(Debug, Clone)]
pub struct SpeakStreamSender {
    message_tx: Sender<ClientMessage>,
}

/// The receiving half of a [`SpeakStreamHandle`], obtained from
/// [`SpeakStreamHandle::split`].
///
/// Yields synthesized audio and server events as a [`futures::Stream`] (or
/// via [`receive`](Self::receive)) until the connection ends.
#[derive(Debug)]
#[pin_project]
pub struct SpeakStreamEvents {
    #[pin]
    response_rx: Receiver<Result<SpeakResponse>>,
    request_id: Uuid,
}

impl SpeakStreamHandle {
    async fn new(builder: SpeakStreamBuilder<'_>) -> Result<SpeakStreamHandle> {
        let url = builder.as_url();
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

            let http_builder = if let Some(auth) = &builder.deepgram.auth {
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
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Uuid::parse_str(value).ok())
            .unwrap_or_default();

        let (message_tx, message_rx) = mpsc::channel(256);
        let (response_tx, response_rx) = mpsc::channel(256);

        tokio::task::spawn(run_worker(ws_stream, message_rx, response_tx));

        Ok(SpeakStreamHandle {
            sender: SpeakStreamSender { message_tx },
            events: SpeakStreamEvents {
                response_rx,
                request_id,
            },
        })
    }

    /// Split the handle into its sending and receiving halves so text can be
    /// sent from one task while audio is consumed in another.
    pub fn split(self) -> (SpeakStreamSender, SpeakStreamEvents) {
        (self.sender, self.events)
    }

    /// A clone of the sending half. Use it to feed text from another task
    /// while this handle keeps receiving events.
    pub fn sender(&self) -> SpeakStreamSender {
        self.sender.clone()
    }

    /// Send text to be synthesized.
    pub async fn speak(&self, text: impl Into<String>) -> Result<()> {
        self.sender.speak(text).await
    }

    /// Flush the server's buffer, forcing it to synthesize and return audio for
    /// all text sent so far.
    pub async fn flush(&self) -> Result<()> {
        self.sender.flush().await
    }

    /// Clear the server's buffer, discarding any text that has not yet been
    /// synthesized.
    pub async fn clear(&self) -> Result<()> {
        self.sender.clear().await
    }

    /// Gracefully close the connection after all pending audio is generated.
    ///
    /// No further messages can be sent after calling this; keep receiving
    /// until the stream ends to get the remaining audio.
    pub async fn close(&self) -> Result<()> {
        self.sender.close().await
    }

    /// Receive the next audio chunk or event, or [`None`] once the stream ends.
    pub async fn receive(&mut self) -> Option<Result<SpeakResponse>> {
        self.events.receive().await
    }

    /// The Deepgram request ID for this TTS session.
    pub fn request_id(&self) -> Uuid {
        self.events.request_id
    }
}

impl SpeakStreamSender {
    /// Send text to be synthesized.
    pub async fn speak(&self, text: impl Into<String>) -> Result<()> {
        self.send(ClientMessage::Speak { text: text.into() }).await
    }

    /// Flush the server's buffer, forcing it to synthesize and return audio for
    /// all text sent so far.
    pub async fn flush(&self) -> Result<()> {
        self.send(ClientMessage::Flush).await
    }

    /// Clear the server's buffer, discarding any text that has not yet been
    /// synthesized.
    pub async fn clear(&self) -> Result<()> {
        self.send(ClientMessage::Clear).await
    }

    /// Gracefully close the connection after all pending audio is generated.
    ///
    /// This ends the session for every clone of the sender: no further
    /// messages can be sent afterwards. Calling it more than once is a no-op.
    pub async fn close(&self) -> Result<()> {
        if self.message_tx.is_closed() {
            return Ok(());
        }
        self.send(ClientMessage::Close).await?;
        // Closing through a clone closes the shared channel for every sender.
        // Messages already enqueued (including the `Close` above) are still
        // delivered to the worker.
        self.message_tx.clone().close_channel();
        Ok(())
    }

    async fn send(&self, message: ClientMessage) -> Result<()> {
        // A bounded `futures` sender needs `&mut self` to send; a clone is
        // cheap and lets every clone of this sender share one queue.
        self.message_tx
            .clone()
            .send(message)
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))
    }
}

impl SpeakStreamEvents {
    /// Receive the next audio chunk or event, or [`None`] once the stream ends.
    pub async fn receive(&mut self) -> Option<Result<SpeakResponse>> {
        self.response_rx.next().await
    }

    /// The Deepgram request ID for this TTS session.
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }
}

impl Stream for SpeakStreamHandle {
    type Item = Result<SpeakResponse, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.events.poll_next(cx)
    }
}

impl Stream for SpeakStreamEvents {
    type Item = Result<SpeakResponse, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.response_rx.poll_next(cx)
    }
}

type WsStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn run_worker(
    ws_stream: WsStream,
    message_rx: Receiver<ClientMessage>,
    response_tx: Sender<Result<SpeakResponse>>,
) {
    run_worker_with_drain_timeout(ws_stream, message_rx, response_tx, CLOSE_DRAIN_IDLE_TIMEOUT)
        .await
}

/// The connection worker. Owns the socket; forwards outbound
/// [`ClientMessage`]s from the handle and inbound frames to the handle.
///
/// `drain_idle_timeout` bounds the wait for each server frame once a `Close`
/// has been sent (see [`CLOSE_DRAIN_IDLE_TIMEOUT`]); it is a parameter so
/// tests can shorten it.
async fn run_worker_with_drain_timeout(
    ws_stream: WsStream,
    mut message_rx: Receiver<ClientMessage>,
    mut response_tx: Sender<Result<SpeakResponse>>,
    drain_idle_timeout: Duration,
) {
    let (mut ws_sink, ws_source) = ws_stream.split();
    let mut ws_source = ws_source.fuse();
    // Set once a `Close` text message has been written to the socket, either
    // explicitly or because every sender was dropped. `message_rx` is not
    // polled afterwards: a closed-and-drained channel reports `Ready(None)`
    // on every poll, which would busy-spin while the final audio drains.
    let mut close_sent = false;
    // Cleared once the peer closed the connection or the transport failed, so
    // the cleanup below does not try to write a Close frame to a dead socket.
    let mut socket_open = true;

    /// One scheduling decision per loop iteration.
    enum Step {
        /// An inbound frame arrived, with response-channel capacity already
        /// reserved for forwarding it.
        Inbound(Option<std::result::Result<Message, tungstenite::Error>>),
        /// An outbound message is ready to be written to the socket (`None`
        /// once every sender has been dropped).
        Outbound(Option<ClientMessage>),
        /// The response consumer went away.
        ResponsesClosed,
        /// After `Close`, the server sent nothing for `drain_idle_timeout`.
        DrainTimedOut,
    }

    loop {
        // Reserve response-channel capacity *before* reading an inbound
        // frame: when the consumer is backpressured, inbound reads pause
        // (backpressure propagates to the socket) instead of blocking this
        // loop mid-forward — so outbound text and control messages keep
        // reaching the wire while audio is arriving. The inbound future
        // borrows `response_tx` and `ws_source`, so it is scoped to the
        // selection and dropped before the step is handled.
        let step = {
            let inbound = async {
                match poll_fn(|cx| response_tx.poll_ready(cx)).await {
                    Err(_) => Step::ResponsesClosed,
                    Ok(()) if close_sent => {
                        match tokio::time::timeout(drain_idle_timeout, ws_source.next()).await {
                            Ok(incoming) => Step::Inbound(incoming),
                            Err(_elapsed) => Step::DrainTimedOut,
                        }
                    }
                    Ok(()) => Step::Inbound(ws_source.next().await),
                }
            }
            .fuse();
            pin_mut!(inbound);
            if close_sent {
                inbound.await
            } else {
                // A fair (rather than biased) select, so neither a flood of
                // inbound audio nor a flood of outbound text can starve the
                // other direction.
                select! {
                    step = inbound => step,
                    message = message_rx.next() => Step::Outbound(message),
                }
            }
        };

        match step {
            Step::ResponsesClosed => break,
            Step::DrainTimedOut => {
                // Capacity was reserved above, so this does not block.
                let _ =
                    response_tx.start_send(Err(DeepgramError::UnexpectedServerResponse(anyhow!(
                        "the server sent nothing for {}s after Close and did not close the \
                         connection; closing it",
                        drain_idle_timeout.as_secs_f64()
                    ))));
                break;
            }
            Step::Inbound(incoming) => {
                match handle_incoming(incoming, &mut ws_sink, &mut response_tx).await {
                    Incoming::Continue => {}
                    Incoming::Stop { socket_open: open } => {
                        socket_open = open;
                        break;
                    }
                }
            }
            Step::Outbound(message) => {
                // Every sender dropped without an explicit Close: send one so
                // the server flushes the remaining audio and shuts down.
                let message = message.unwrap_or(ClientMessage::Close);
                let is_close = message == ClientMessage::Close;
                let text = serde_json::to_string(&message).unwrap_or_default();
                if let Err(err) = ws_sink.send(Message::Text(Utf8Bytes::from(text))).await {
                    // A failed write means the transport is broken: forward
                    // the terminal error and end the worker.
                    let _ = response_tx.send(Err(err.into())).await;
                    socket_open = false;
                    break;
                }
                if is_close {
                    close_sent = true;
                }
            }
        }
    }

    if socket_open {
        // The consumer went away, or the server stalled after Close. Tell the
        // server the session is over if we have not already (the consumer may
        // have dropped the whole handle before the `Outbound(None)` branch
        // ran), then complete the WebSocket closing handshake from our side
        // (this writes a Close frame) instead of tearing the TCP connection
        // down silently.
        if !close_sent {
            let text = serde_json::to_string(&ClientMessage::Close).unwrap_or_default();
            let _ = ws_sink.send(Message::Text(Utf8Bytes::from(text))).await;
        }
        let _ = ws_sink.close().await;
    }
    response_tx.close_channel();
}

/// What the worker should do after handling one inbound frame.
enum Incoming {
    Continue,
    /// Stop the worker. `socket_open` is `false` when the peer closed the
    /// connection or the transport failed, so no Close frame should be
    /// written during cleanup.
    Stop {
        socket_open: bool,
    },
}

/// Handle a single message received from the server. Response-channel
/// capacity must already be reserved (see the worker loop), so forwarding
/// uses `start_send` and never blocks.
async fn handle_incoming<S>(
    incoming: Option<std::result::Result<Message, tungstenite::Error>>,
    ws_sink: &mut S,
    response_tx: &mut Sender<Result<SpeakResponse>>,
) -> Incoming
where
    S: futures::Sink<Message> + Unpin,
{
    match incoming {
        Some(Ok(Message::Binary(audio))) => {
            if response_tx
                .start_send(Ok(SpeakResponse::Audio(audio)))
                .is_err()
            {
                return Incoming::Stop { socket_open: true };
            }
        }
        Some(Ok(Message::Text(text))) => {
            if response_tx
                .start_send(Ok(parse_text_message(&text)))
                .is_err()
            {
                return Incoming::Stop { socket_open: true };
            }
        }
        Some(Ok(Message::Ping(payload))) => {
            let _ = ws_sink.send(Message::Pong(payload)).await;
        }
        Some(Ok(Message::Close(frame))) => {
            // A normal (1000) close is not an error.
            if let Some(frame) = frame {
                if u16::from(frame.code) != 1000 {
                    let _ = response_tx.start_send(Err(DeepgramError::WebsocketClose {
                        code: frame.code.into(),
                        reason: frame.reason.to_string(),
                    }));
                }
            }
            // Reading a Close frame queues the acknowledgement on the shared
            // websocket context; flush the write half so the closing
            // handshake completes now rather than at socket teardown.
            let _ = ws_sink.flush().await;
            return Incoming::Stop { socket_open: false };
        }
        Some(Ok(Message::Frame(_) | Message::Pong(_))) => {
            // Raw frames and pongs can be ignored.
        }
        Some(Err(err)) => {
            let _ = response_tx.start_send(Err(err.into()));
            return Incoming::Stop { socket_open: false };
        }
        None => return Incoming::Stop { socket_open: false },
    }
    Incoming::Continue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn speak_stream_url_wss() {
        let dg = Deepgram::new("token").unwrap();
        assert_eq!(
            dg.text_to_speech().speak_stream_url().to_string(),
            "wss://api.deepgram.com/v1/speak"
        );
    }

    #[test]
    fn speak_stream_url_custom_host() {
        let dg = Deepgram::with_base_url_and_api_key("http://localhost:8080", "token").unwrap();
        assert_eq!(
            dg.text_to_speech().speak_stream_url().to_string(),
            "ws://localhost:8080/v1/speak"
        );
    }

    #[test]
    fn query_params_serialize() {
        let dg = Deepgram::new("token").unwrap();
        let builder = dg
            .text_to_speech()
            .speak_stream()
            .model(Model::AuraAsteriaEn)
            .encoding(Encoding::Linear16)
            .sample_rate(24000);
        assert_eq!(
            builder.as_url().to_string(),
            "wss://api.deepgram.com/v1/speak?model=aura-asteria-en&encoding=linear16&sample_rate=24000"
        );
    }

    #[test]
    fn query_params_serialize_speed_and_mip_opt_out() {
        let dg = Deepgram::new("token").unwrap();
        let builder = dg
            .text_to_speech()
            .speak_stream()
            .encoding(Encoding::Mulaw)
            .sample_rate(8000)
            .speed(1.2)
            .mip_opt_out(true);
        assert!(builder.validate().is_ok());
        assert_eq!(
            builder.as_url().to_string(),
            "wss://api.deepgram.com/v1/speak?encoding=mulaw&sample_rate=8000&speed=1.2&mip_opt_out=true"
        );

        let builder = dg
            .text_to_speech()
            .speak_stream()
            .speed(1.0)
            .mip_opt_out(false);
        assert_eq!(
            builder.as_url().to_string(),
            "wss://api.deepgram.com/v1/speak?speed=1&mip_opt_out=false"
        );
    }

    #[test]
    fn validate_accepts_streaming_encodings() {
        let dg = Deepgram::new("token").unwrap();
        for encoding in [
            Encoding::Linear16,
            Encoding::Mulaw,
            Encoding::Alaw,
            // Escape hatch is passed through to the server unchecked.
            Encoding::CustomEncoding("future-codec".to_string()),
        ] {
            let builder = dg
                .text_to_speech()
                .speak_stream()
                .encoding(encoding.clone());
            assert!(
                builder.validate().is_ok(),
                "{} should be accepted",
                encoding.as_str()
            );
        }
        // No encoding at all is fine: the server default (linear16) applies.
        assert!(dg.text_to_speech().speak_stream().validate().is_ok());
    }

    #[test]
    fn validate_rejects_rest_only_encodings() {
        let dg = Deepgram::new("token").unwrap();
        for encoding in [Encoding::Mp3, Encoding::Opus, Encoding::Flac, Encoding::Aac] {
            let name = encoding.as_str().to_string();
            let err = dg
                .text_to_speech()
                .speak_stream()
                .encoding(encoding)
                .validate()
                .unwrap_err();
            match err {
                DeepgramError::InvalidOptions(msg) => {
                    assert!(msg.contains(&name), "message should name `{name}`: {msg}");
                    assert!(
                        msg.contains("linear16"),
                        "message should list alternatives: {msg}"
                    );
                }
                other => panic!("expected InvalidOptions for `{name}`, got {other:?}"),
            }
        }
    }

    #[test]
    fn validate_speed_range() {
        let dg = Deepgram::new("token").unwrap();
        for speed in [0.7, 1.0, 1.5] {
            assert!(
                dg.text_to_speech()
                    .speak_stream()
                    .speed(speed)
                    .validate()
                    .is_ok(),
                "speed {speed} should be accepted"
            );
        }
        for speed in [0.69, 1.51, 0.0, -1.0, f32::NAN, f32::INFINITY] {
            let err = dg
                .text_to_speech()
                .speak_stream()
                .speed(speed)
                .validate()
                .unwrap_err();
            assert!(
                matches!(&err, DeepgramError::InvalidOptions(msg) if msg.contains("speed")),
                "speed {speed} should be rejected with InvalidOptions, got {err:?}"
            );
        }
    }

    #[tokio::test]
    async fn handle_rejects_invalid_options_before_connecting() {
        // An unroutable base URL: if validation did not short-circuit, this
        // would attempt (and fail) a network connection with a different error.
        let dg = Deepgram::with_base_url_and_api_key("http://127.0.0.1:1", "token").unwrap();
        let err = dg
            .text_to_speech()
            .speak_stream()
            .encoding(Encoding::Mp3)
            .handle()
            .await
            .unwrap_err();
        assert!(matches!(err, DeepgramError::InvalidOptions(_)), "{err:?}");

        let err = dg
            .text_to_speech()
            .speak_stream()
            .speed(2.0)
            .handle()
            .await
            .unwrap_err();
        assert!(matches!(err, DeepgramError::InvalidOptions(_)), "{err:?}");
    }

    #[test]
    fn client_message_serialization() {
        assert_eq!(
            serde_json::to_string(&ClientMessage::Speak {
                text: "hi".to_string()
            })
            .unwrap(),
            r#"{"type":"Speak","text":"hi"}"#
        );
        assert_eq!(
            serde_json::to_string(&ClientMessage::Flush).unwrap(),
            r#"{"type":"Flush"}"#
        );
        assert_eq!(
            serde_json::to_string(&ClientMessage::Clear).unwrap(),
            r#"{"type":"Clear"}"#
        );
        assert_eq!(
            serde_json::to_string(&ClientMessage::Close).unwrap(),
            r#"{"type":"Close"}"#
        );
    }

    #[test]
    fn parses_metadata_and_unknown() {
        let metadata = parse_text_message(
            r#"{"type":"Metadata","request_id":"abc","model_name":"aura-asteria-en","model_version":"1","model_uuid":"u"}"#,
        );
        assert!(matches!(
            metadata,
            SpeakResponse::Metadata { request_id, .. } if request_id == "abc"
        ));

        let flushed = parse_text_message(r#"{"type":"Flushed","sequence_id":3}"#);
        assert!(matches!(
            flushed,
            SpeakResponse::Flushed {
                sequence_id: Some(3)
            }
        ));

        let unknown = parse_text_message(r#"{"type":"SomethingNew","foo":1}"#);
        assert!(matches!(unknown, SpeakResponse::Unknown(_)));
    }

    #[test]
    fn parses_metadata_additional_model_uuids() {
        let a = Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap();
        let b = Uuid::parse_str("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
        let metadata = parse_text_message(&format!(
            r#"{{"type":"Metadata","request_id":"abc","model_name":"aura-asteria-en","model_version":"1","model_uuid":"u","additional_model_uuids":["{a}","{b}"]}}"#
        ));
        assert_eq!(
            metadata,
            SpeakResponse::Metadata {
                request_id: "abc".to_string(),
                model_name: Some("aura-asteria-en".to_string()),
                model_version: Some("1".to_string()),
                model_uuid: Some("u".to_string()),
                additional_model_uuids: Some(vec![a, b]),
            }
        );

        // Absent and empty are both fine.
        let without = parse_text_message(r#"{"type":"Metadata","request_id":"abc"}"#);
        assert!(matches!(
            without,
            SpeakResponse::Metadata {
                additional_model_uuids: None,
                ..
            }
        ));
        let empty = parse_text_message(
            r#"{"type":"Metadata","request_id":"abc","additional_model_uuids":[]}"#,
        );
        assert!(matches!(
            empty,
            SpeakResponse::Metadata {
                additional_model_uuids: Some(ref v),
                ..
            } if v.is_empty()
        ));
    }

    #[test]
    fn parses_cleared() {
        let cases = [
            (
                r#"{"type":"Cleared","sequence_id":0}"#,
                SpeakResponse::Cleared {
                    sequence_id: Some(0),
                },
            ),
            (
                r#"{"type":"Cleared","sequence_id":7}"#,
                SpeakResponse::Cleared {
                    sequence_id: Some(7),
                },
            ),
            (
                r#"{"type":"Cleared"}"#,
                SpeakResponse::Cleared { sequence_id: None },
            ),
        ];
        for (json, expected) in cases {
            assert_eq!(parse_text_message(json), expected, "{json}");
        }
    }

    #[test]
    fn parses_warning() {
        let cases = [
            (
                r#"{"type":"Warning","description":"Text input exceeds recommended length for optimal performance","code":"TEXT_LENGTH_WARNING"}"#,
                SpeakResponse::Warning {
                    description: Some(
                        "Text input exceeds recommended length for optimal performance".to_string(),
                    ),
                    code: Some("TEXT_LENGTH_WARNING".to_string()),
                },
            ),
            (
                r#"{"type":"Warning","description":"only a description"}"#,
                SpeakResponse::Warning {
                    description: Some("only a description".to_string()),
                    code: None,
                },
            ),
            (
                r#"{"type":"Warning","code":"ONLY_A_CODE"}"#,
                SpeakResponse::Warning {
                    description: None,
                    code: Some("ONLY_A_CODE".to_string()),
                },
            ),
            (
                r#"{"type":"Warning"}"#,
                SpeakResponse::Warning {
                    description: None,
                    code: None,
                },
            ),
        ];
        for (json, expected) in cases {
            assert_eq!(parse_text_message(json), expected, "{json}");
        }
    }

    #[test]
    fn query_params_escape_hatch_reaches_the_url() {
        // An unmodeled parameter must reach the wire, after the typed ones,
        // and calling `query_params` twice appends both sets.
        let dg = Deepgram::new("token").unwrap();
        let builder = dg
            .text_to_speech()
            .speak_stream()
            .encoding(Encoding::Linear16)
            .query_params([("future_param".to_string(), "on".to_string())])
            .query_params(vec![("another".to_string(), "a b".to_string())]);
        assert_eq!(
            builder.as_url().to_string(),
            "wss://api.deepgram.com/v1/speak?encoding=linear16&future_param=on&another=a+b"
        );
    }

    #[test]
    fn validate_sample_rate() {
        let dg = Deepgram::new("token").unwrap();
        for rate in SAMPLE_RATES {
            assert!(
                dg.text_to_speech()
                    .speak_stream()
                    .sample_rate(rate)
                    .validate()
                    .is_ok(),
                "sample_rate {rate} should be accepted"
            );
        }
        for rate in [0, 11025, 22050, 44100, 96000] {
            let err = dg
                .text_to_speech()
                .speak_stream()
                .sample_rate(rate)
                .validate()
                .unwrap_err();
            assert!(
                matches!(&err, DeepgramError::InvalidOptions(msg) if msg.contains("sample_rate") && msg.contains(&rate.to_string())),
                "sample_rate {rate} should be rejected with InvalidOptions, got {err:?}"
            );
        }
    }

    #[tokio::test]
    async fn handle_rejects_invalid_sample_rate_before_connecting() {
        let dg = Deepgram::with_base_url_and_api_key("http://127.0.0.1:1", "token").unwrap();
        let err = dg
            .text_to_speech()
            .speak_stream()
            .sample_rate(44100)
            .handle()
            .await
            .unwrap_err();
        assert!(matches!(err, DeepgramError::InvalidOptions(_)), "{err:?}");
    }

    /// N3: after `Close` is sent, the wait for the server is bounded. If the
    /// server neither sends nor closes, the worker surfaces one error, writes
    /// a WebSocket Close frame itself, and ends the stream.
    #[tokio::test]
    async fn drain_after_close_is_bounded_and_closes_the_socket() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let (server_saw_close_tx, server_saw_close_rx) = tokio::sync::oneshot::channel::<bool>();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
            let mut saw_close_frame = false;
            // Read the client's `Close` text, then stall: never send anything
            // and never close. The only thing that should arrive afterwards
            // is the client's WebSocket Close frame once it gives up.
            while let Some(Ok(message)) = ws.next().await {
                if matches!(message, Message::Close(_)) {
                    saw_close_frame = true;
                    break;
                }
            }
            let _ = server_saw_close_tx.send(saw_close_frame);
        });

        let (ws_stream, _) = tokio_tungstenite::connect_async(format!("ws://127.0.0.1:{port}"))
            .await
            .unwrap();
        let (mut message_tx, message_rx) = mpsc::channel(8);
        let (response_tx, mut response_rx) = mpsc::channel(8);
        let worker = tokio::spawn(run_worker_with_drain_timeout(
            ws_stream,
            message_rx,
            response_tx,
            Duration::from_millis(200),
        ));

        message_tx
            .send(ClientMessage::Speak { text: "hi".into() })
            .await
            .unwrap();
        message_tx.send(ClientMessage::Close).await.unwrap();

        let first = tokio::time::timeout(Duration::from_secs(5), response_rx.next())
            .await
            .expect("the stream must end within the drain timeout")
            .expect("a timeout error is surfaced before the stream ends");
        assert!(
            matches!(first, Err(DeepgramError::UnexpectedServerResponse(_))),
            "expected the drain-timeout error, got {first:?}"
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(5), response_rx.next())
                .await
                .expect("stream ends")
                .is_none(),
            "the stream must end after the timeout error"
        );
        tokio::time::timeout(Duration::from_secs(5), worker)
            .await
            .expect("worker exits")
            .unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(5), server_saw_close_rx)
                .await
                .expect("server task reports")
                .unwrap(),
            "the client must write a WebSocket Close frame when it gives up"
        );
    }
}
