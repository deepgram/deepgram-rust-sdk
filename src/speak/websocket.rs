//! Streaming Text-to-Speech over a WebSocket.
//!
//! The TTS WebSocket lets you stream text in (for example, as an LLM produces
//! it) and receive synthesized audio out with low latency. Construct a request
//! with [`Speak::speak_stream`], configure it, then open the connection with
//! [`SpeakStreamBuilder::handle`].
//!
//! See the [Deepgram TTS WebSocket API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/text-to-speech/speak-streaming

use bytes::Bytes;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    select_biased,
    stream::StreamExt,
    SinkExt, Stream,
};
use http::Request;
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::{
    pin::Pin,
    task::{Context, Poll},
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
    stream_url: Url,
}

impl<'a> Speak<'a> {
    /// Begin configuring a streaming Text-to-Speech WebSocket request.
    ///
    /// Once configured, open the connection with
    /// [`SpeakStreamBuilder::handle`].
    ///
    /// ```
    /// use deepgram::{speak::options::Encoding, Deepgram};
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap_or_default()).unwrap();
    /// let builder = dg.text_to_speech().speak_stream().encoding(Encoding::Linear16);
    /// ```
    pub fn speak_stream(&self) -> SpeakStreamBuilder<'a> {
        SpeakStreamBuilder {
            deepgram: self.0,
            model: None,
            encoding: None,
            sample_rate: None,
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

    /// Set the output audio encoding. The streaming endpoint supports
    /// `linear16`, `mulaw`, and `alaw`.
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = Some(encoding);
        self
    }

    /// Set the output sample rate in Hz.
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
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
        }
        url
    }

    /// Open the WebSocket connection and return a [`SpeakStreamHandle`] for
    /// sending text and receiving audio.
    pub async fn handle(self) -> Result<SpeakStreamHandle> {
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
    Metadata {
        /// The unique identifier for the request.
        request_id: String,
        /// The name of the model used.
        model_name: Option<String>,
        /// The version of the model used.
        model_version: Option<String>,
        /// The unique identifier of the model used.
        model_uuid: Option<String>,
    },

    /// Emitted after a `Flush`, once all buffered audio has been sent.
    Flushed {
        /// The sequence identifier of the flushed segment.
        sequence_id: Option<u32>,
    },

    /// Emitted after a `Clear`, confirming the buffer was cleared.
    Cleared {
        /// The sequence identifier of the cleared segment.
        sequence_id: Option<u32>,
    },

    /// A non-fatal warning from the server.
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
            } => SpeakResponse::Metadata {
                request_id,
                model_name,
                model_version,
                model_uuid,
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
/// Send text with [`SpeakStreamHandle::speak`], force generation with
/// [`SpeakStreamHandle::flush`], and end the session with
/// [`SpeakStreamHandle::close`]. Receive audio and events by consuming the
/// handle as a [`futures::Stream`] or by calling
/// [`SpeakStreamHandle::receive`].
///
/// The send methods and the receive path both take `&mut self`, so a single
/// owner cannot send and receive concurrently. The intended pattern is to
/// enqueue the text you want (`speak` … `flush` … `close`) and then drain the
/// audio and events. The internal channels are bounded, so avoid enqueuing an
/// unbounded amount of text before draining; interleave `flush`/drain for long
/// sessions, or move the handle into a task and communicate over your own
/// channel.
///
/// [`request_id`](SpeakStreamHandle::request_id) is taken from the connection
/// upgrade headers and is [`Uuid::nil`] if the server did not provide one.
#[derive(Debug)]
#[pin_project]
pub struct SpeakStreamHandle {
    message_tx: Sender<ClientMessage>,
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
            message_tx,
            response_rx,
            request_id,
        })
    }

    /// Send text to be synthesized.
    pub async fn speak(&mut self, text: impl Into<String>) -> Result<()> {
        self.send(ClientMessage::Speak { text: text.into() }).await
    }

    /// Flush the server's buffer, forcing it to synthesize and return audio for
    /// all text sent so far.
    pub async fn flush(&mut self) -> Result<()> {
        self.send(ClientMessage::Flush).await
    }

    /// Clear the server's buffer, discarding any text that has not yet been
    /// synthesized.
    pub async fn clear(&mut self) -> Result<()> {
        self.send(ClientMessage::Clear).await
    }

    /// Gracefully close the connection after all pending audio is generated.
    ///
    /// No further messages should be sent after calling this.
    pub async fn close(&mut self) -> Result<()> {
        if !self.message_tx.is_closed() {
            self.send(ClientMessage::Close).await?;
            self.message_tx.close_channel();
        }
        Ok(())
    }

    async fn send(&mut self, message: ClientMessage) -> Result<()> {
        self.message_tx
            .send(message)
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))
    }

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
        this.response_rx.poll_next(cx)
    }
}

async fn run_worker(
    ws_stream: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    mut message_rx: Receiver<ClientMessage>,
    mut response_tx: Sender<Result<SpeakResponse>>,
) {
    let (mut ws_sink, ws_source) = ws_stream.split();
    let mut ws_source = ws_source.fuse();
    let mut input_open = true;

    loop {
        if input_open {
            select_biased! {
                incoming = ws_source.next() => {
                    if handle_incoming(incoming, &mut ws_sink, &mut response_tx).await.is_break() {
                        break;
                    }
                }
                outgoing = message_rx.next() => {
                    match outgoing {
                        Some(message) => {
                            let is_close = message == ClientMessage::Close;
                            let text = serde_json::to_string(&message).unwrap_or_default();
                            if let Err(err) = ws_sink.send(Message::Text(Utf8Bytes::from(text))).await {
                                let _ = response_tx.send(Err(err.into())).await;
                                break;
                            }
                            if is_close {
                                input_open = false;
                            }
                        }
                        None => {
                            // The input channel was dropped without an explicit
                            // Close: send one so the server flushes and shuts down.
                            let text = serde_json::to_string(&ClientMessage::Close).unwrap_or_default();
                            let _ = ws_sink.send(Message::Text(Utf8Bytes::from(text))).await;
                            input_open = false;
                        }
                    }
                }
            }
        } else {
            // Input is closed: only drain server messages until the connection
            // closes. We must NOT keep selecting on `message_rx` here — it now
            // yields `Ready(None)` synchronously, which would busy-spin the task
            // (and hang a current-thread runtime).
            if handle_incoming(ws_source.next().await, &mut ws_sink, &mut response_tx)
                .await
                .is_break()
            {
                break;
            }
        }
    }

    response_tx.close_channel();
}

/// Handle a single message received from the server. Returns
/// [`ControlFlow::Break`] when the worker should stop (connection closed or
/// errored, or the consumer dropped the response stream).
async fn handle_incoming<S>(
    incoming: Option<std::result::Result<Message, tungstenite::Error>>,
    ws_sink: &mut S,
    response_tx: &mut Sender<Result<SpeakResponse>>,
) -> std::ops::ControlFlow<()>
where
    S: futures::Sink<Message> + Unpin,
{
    use std::ops::ControlFlow;
    match incoming {
        Some(Ok(Message::Binary(audio))) => {
            if response_tx
                .send(Ok(SpeakResponse::Audio(audio)))
                .await
                .is_err()
            {
                return ControlFlow::Break(());
            }
        }
        Some(Ok(Message::Text(text))) => {
            if response_tx
                .send(Ok(parse_text_message(&text)))
                .await
                .is_err()
            {
                return ControlFlow::Break(());
            }
        }
        Some(Ok(Message::Ping(payload))) => {
            let _ = ws_sink.send(Message::Pong(payload)).await;
        }
        Some(Ok(Message::Close(None))) => return ControlFlow::Break(()),
        Some(Ok(Message::Close(Some(frame)))) => {
            // A normal (1000) close is not an error.
            if u16::from(frame.code) != 1000 {
                let _ = response_tx
                    .send(Err(DeepgramError::WebsocketClose {
                        code: frame.code.into(),
                        reason: frame.reason.to_string(),
                    }))
                    .await;
            }
            return ControlFlow::Break(());
        }
        Some(Ok(Message::Frame(_) | Message::Pong(_))) => {
            // Raw frames and pongs can be ignored.
        }
        Some(Err(err)) => {
            let _ = response_tx.send(Err(err.into())).await;
            return ControlFlow::Break(());
        }
        None => return ControlFlow::Break(()),
    }
    ControlFlow::Continue(())
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
}
