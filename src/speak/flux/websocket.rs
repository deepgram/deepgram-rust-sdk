//! Websocket Flux TTS module — streaming, turn-based text-to-speech
//! over the `/v2/speak` WebSocket.
//!
//! See the [Deepgram Flux TTS API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/text-to-speech/speak-flux

use anyhow::anyhow;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    future::poll_fn,
    pin_mut, select, FutureExt, SinkExt, StreamExt,
};
use http::Request;
use serde::Serialize;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::{
    handshake::client,
    protocol::frame::coding::{CloseCode, Data, OpCode},
    Utf8Bytes,
};
use url::Url;
use uuid::Uuid;

use super::{options::Options, response::FluxSpeakResponse};
use crate::{Deepgram, DeepgramError, Result, Speak};

static FLUX_SPEAK_URL_PATH: &str = "v2/speak";

impl Speak<'_> {
    /// Begin to configure a Flux TTS streaming request with the given
    /// [`Options`].
    ///
    /// Once configured, the connection is initiated with
    /// [`FluxSpeakBuilder::handle`].
    ///
    /// The WebSocket transport does not accept the REST-only options
    /// (`container`, `bit_rate`, `callback`, `callback_method`,
    /// `priority`) or the REST-only compressed encodings (`mp3`, `opus`,
    /// `flac`, `aac`); [`FluxSpeakBuilder::handle`] returns
    /// [`DeepgramError::InvalidOptions`] if any of them are set.
    ///
    /// ```
    /// use deepgram::{
    ///     speak::flux::options::{Encoding, Model, Options},
    ///     Deepgram,
    /// };
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap_or_default()).unwrap();
    /// let options = Options::builder(Model::FluxHaleyEn)
    ///     .encoding(Encoding::Linear16)
    ///     .sample_rate(24000)
    ///     .build();
    /// let builder = dg.text_to_speech().flux_request(options);
    /// ```
    pub fn flux_request(&self, options: Options) -> FluxSpeakBuilder<'_> {
        FluxSpeakBuilder {
            deepgram: self.0,
            options,
            stream_url: self.flux_speak_url(),
        }
    }

    fn flux_speak_url(&self) -> Url {
        let mut url =
            self.0.base_url.join(FLUX_SPEAK_URL_PATH).expect(
                "base_url is checked to be a valid base_url when constructing Deepgram client",
            );

        match url.scheme() {
            "http" | "ws" => url
                .set_scheme("ws")
                .expect("a valid conversion according to the .set_scheme docs"),
            "https" | "wss" => url
                .set_scheme("wss")
                .expect("a valid conversion according to the .set_scheme docs"),
            _ => unreachable!(
                "base_url is validated to have a scheme of http, https, ws, or wss when constructing Deepgram client"
            ),
        }
        url
    }
}

/// A Flux TTS streaming request in the process of being built.
/// Created by [`Speak::flux_request`].
#[derive(Clone, Debug)]
pub struct FluxSpeakBuilder<'a> {
    deepgram: &'a Deepgram,
    options: Options,
    stream_url: Url,
}

impl FluxSpeakBuilder<'_> {
    /// Return the options in urlencoded format. If serialization would
    /// fail, this will also return an error.
    ///
    /// This is intended primarily to help with debugging API requests.
    pub fn urlencoded(&self) -> std::result::Result<String, serde_urlencoded::ser::Error> {
        Ok(self.as_url()?.query().unwrap_or_default().to_string())
    }

    fn as_url(&self) -> std::result::Result<Url, serde_urlencoded::ser::Error> {
        let mut url = self.stream_url.clone();
        {
            let mut pairs = url.query_pairs_mut();
            pairs.extend_pairs(
                serde_urlencoded::from_str::<Vec<(String, String)>>(&self.options.urlencoded()?)
                    .expect("constructed query string can be deserialized"),
            );
        }
        Ok(url)
    }

    /// Connect to the `/v2/speak` WebSocket, returning a
    /// [`FluxSpeakHandle`] for sending text and receiving synthesized
    /// audio and events.
    pub async fn handle(self) -> Result<FluxSpeakHandle> {
        if let Some(param) = self.options.rest_only_options_set() {
            return Err(DeepgramError::InvalidOptions(format!(
                "the `{param}` option applies to the REST (batch) transport only and is not accepted by the /v2/speak websocket"
            )));
        }
        if let Some(encoding) = self.options.rest_only_encoding_set() {
            return Err(DeepgramError::InvalidOptions(format!(
                "the `{encoding}` encoding applies to the REST (batch) transport only; the /v2/speak websocket emits raw audio (`linear16`, `mulaw`, or `alaw`)"
            )));
        }
        FluxSpeakHandle::new(self).await
    }
}

/// Client messages for the `/v2/speak` WebSocket, in their wire format.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Speak {
        text: String,
    },
    Flush,
    Interrupt {
        #[serde(skip_serializing_if = "Option::is_none")]
        playback_offset: Option<PlaybackOffset>,
    },
    Configure {
        #[serde(skip_serializing_if = "Option::is_none")]
        speed: Option<f64>,
    },
    Close,
}

/// How much audio the client had played when the user barged in,
/// attached to an `Interrupt` message.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
enum PlaybackOffset {
    /// Milliseconds of session audio the client played before barging
    /// in. Cumulative from the start of the session, not the turn.
    #[serde(rename = "time_ms")]
    TimeMs { value: u64 },
}

/// Handle for a live `/v2/speak` WebSocket connection.
///
/// Stream text into the active turn with [`speak`](Self::speak), end
/// the turn with [`flush`](Self::flush), and read synthesized audio and
/// lifecycle events with [`receive`](Self::receive).
#[derive(Debug)]
pub struct FluxSpeakHandle {
    message_tx: Sender<ClientMessage>,
    response_rx: Receiver<Result<FluxSpeakResponse>>,
    request_id: Uuid,
}

impl FluxSpeakHandle {
    async fn new(builder: FluxSpeakBuilder<'_>) -> Result<FluxSpeakHandle> {
        let url = builder.as_url()?;
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

            let builder = if let Some(auth) = &builder.deepgram.auth {
                http_builder.header("authorization", auth.header_value())
            } else {
                http_builder
            };
            builder.body(())?
        };

        let (ws_stream, upgrade_response) = tokio_tungstenite::connect_async(request).await?;

        let request_id = upgrade_response
            .headers()
            .get("dg-request-id")
            .ok_or(DeepgramError::UnexpectedServerResponse(anyhow!(
                "Websocket upgrade headers missing request ID"
            )))?
            .to_str()
            .ok()
            .and_then(|req_header_str| Uuid::parse_str(req_header_str).ok())
            .ok_or(DeepgramError::UnexpectedServerResponse(anyhow!(
                "Received malformed request ID in websocket upgrade headers"
            )))?;

        let (message_tx, message_rx) = mpsc::channel(256);
        let (response_tx, response_rx) = mpsc::channel(256);

        tokio::task::spawn(run_flux_speak_worker(ws_stream, message_rx, response_tx));

        Ok(FluxSpeakHandle {
            message_tx,
            response_rx,
            request_id,
        })
    }

    /// Send text to be synthesized into the active turn. The server
    /// applies light normalization and preprocessing before synthesis,
    /// and starts generating and streaming audio as soon as it has
    /// enough text — there is no need to chunk text or place flush
    /// points yourself.
    pub async fn speak(&mut self, text: impl Into<String>) -> Result<()> {
        self.send_message(ClientMessage::Speak { text: text.into() })
            .await
    }

    /// End the active turn. The server drains the buffer, generates the
    /// remaining audio, echoes a
    /// [`FluxSpeakResponse::Flushed`] immediately, and reports the
    /// completed turn with a [`FluxSpeakResponse::SpeechMetadata`] event
    /// after all of the turn's audio has been sent.
    pub async fn flush(&mut self) -> Result<()> {
        self.send_message(ClientMessage::Flush).await
    }

    /// Report that the user barged in. The server stops active audio
    /// generation, discards any buffered text pending generation, and
    /// replies with [`FluxSpeakResponse::SpeechInterrupted`] once the
    /// cancelled turn has drained.
    ///
    /// `playback_offset_ms` is how much session audio the client had
    /// played when the user barged in, in milliseconds — cumulative from
    /// the start of the session, not the turn, and each `Interrupt` must
    /// advance past the position the previous one established. Pass
    /// `None` if unknown; the server then cannot split the turn's text,
    /// so `SpeechInterrupted` omits `text_spoken` and `text_remaining`.
    pub async fn interrupt(&mut self, playback_offset_ms: Option<u64>) -> Result<()> {
        self.send_message(ClientMessage::Interrupt {
            playback_offset: playback_offset_ms.map(|value| PlaybackOffset::TimeMs { value }),
        })
        .await
    }

    /// Update the speech-rate multiplier mid-session without
    /// reconnecting. An accepted change takes effect at the next segment
    /// boundary, not mid-segment. Answered by exactly one of
    /// [`FluxSpeakResponse::ConfigureSuccess`] or
    /// [`FluxSpeakResponse::ConfigureFailure`]; a rejected `Configure`
    /// leaves the previous configuration in force.
    pub async fn configure_speed(&mut self, speed: f64) -> Result<()> {
        self.send_message(ClientMessage::Configure { speed: Some(speed) })
            .await
    }

    /// Gracefully close the connection. The server drains all remaining
    /// audio (the active turn plus any turns still queued behind it),
    /// emits a final [`FluxSpeakResponse::SessionMetadata`], then closes
    /// the socket. No more messages should be sent after this is called.
    pub async fn close(&mut self) -> Result<()> {
        if !self.message_tx.is_closed() {
            self.message_tx
                .send(ClientMessage::Close)
                .await
                .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
            self.message_tx.close_channel();
        }
        Ok(())
    }

    /// Receive the next message from the server: synthesized audio
    /// ([`FluxSpeakResponse::Audio`]) or a lifecycle event. Returns
    /// `None` once the connection has closed and all messages have been
    /// received.
    pub async fn receive(&mut self) -> Option<Result<FluxSpeakResponse>> {
        self.response_rx.next().await
    }

    /// Returns the Deepgram request ID for the Flux TTS streaming
    /// request.
    ///
    /// A request ID needs to be provided to Deepgram as part of any
    /// support or troubleshooting assistance related to a specific
    /// request.
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }

    async fn send_message(&mut self, message: ClientMessage) -> Result<()> {
        self.message_tx
            .send(message)
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
        Ok(())
    }
}

async fn run_flux_speak_worker(
    ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    mut message_rx: Receiver<ClientMessage>,
    mut response_tx: Sender<Result<FluxSpeakResponse>>,
) -> Result<()> {
    // Partial frames are accumulated here until the final fragment
    // arrives. Text fragments may split multi-byte characters, so bytes
    // are collected and parsed only once the message is complete.
    let mut partial_frame: Vec<u8> = Vec::new();
    let mut partial_frame_is_text = false;
    let (mut ws_stream_send, ws_stream_recv) = ws_stream.split();
    let mut ws_stream_recv = ws_stream_recv.fuse();
    let mut is_open: bool = true;

    /// One scheduling decision per loop iteration.
    enum Step {
        /// An inbound frame arrived, with response-channel capacity already
        /// reserved for forwarding it.
        Inbound(Option<std::result::Result<Message, tungstenite::Error>>),
        /// An outbound message is ready to be written to the socket.
        Outbound(Option<ClientMessage>),
        /// The response consumer went away.
        ResponsesClosed,
    }

    loop {
        // Reserve response-channel capacity *before* reading an inbound
        // frame: when the consumer is backpressured, inbound reads pause
        // (backpressure propagates to the socket) instead of blocking this
        // loop mid-forward — so outbound control messages (Interrupt,
        // Configure, Close) stay deliverable while audio is arriving. The
        // inbound future borrows `response_tx` and `ws_stream_recv`, so it
        // is scoped to the selection and dropped before the step is handled.
        let step = {
            let inbound = async {
                match poll_fn(|cx| response_tx.poll_ready(cx)).await {
                    Ok(()) => Step::Inbound(ws_stream_recv.next().await),
                    Err(_) => Step::ResponsesClosed,
                }
            }
            .fuse();
            pin_mut!(inbound);
            // A fair (rather than biased) select, so neither a flood of
            // inbound audio nor a flood of outbound text can starve the
            // other direction.
            select! {
                step = inbound => step,
                message = message_rx.next() => Step::Outbound(message),
            }
        };

        match step {
            Step::ResponsesClosed => {
                // Responses are no longer being received; close the stream.
                break;
            }
            Step::Inbound(response) => {
                match response {
                    Some(Ok(Message::Text(response))) => {
                        let response = serde_json::from_str(&response).map_err(DeepgramError::from);
                        // Capacity was reserved above, so this does not block.
                        if response_tx.start_send(response).is_err() {
                            // Responses are no longer being received; close the stream.
                            break;
                        }
                    }
                    Some(Ok(Message::Binary(audio))) => {
                        if response_tx
                            .start_send(Ok(FluxSpeakResponse::Audio(audio)))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Some(Ok(Message::Ping(value))) => {
                        // We don't really care if the server receives the pong.
                        let _ = ws_stream_send.send(Message::Pong(value)).await;
                    }
                    Some(Ok(Message::Close(closeframe))) => {
                        // A normal closure carries no error information;
                        // anything else is surfaced to the consumer before
                        // the response channel closes.
                        if let Some(closeframe) = closeframe {
                            if closeframe.code != CloseCode::Normal {
                                let _ =
                                    response_tx.start_send(Err(DeepgramError::WebsocketClose {
                                        code: closeframe.code.into(),
                                        reason: closeframe.reason.to_string(),
                                    }));
                            }
                        }
                        // The server is closing the connection; don't send
                        // Close during cleanup.
                        is_open = false;
                        break;
                    }
                    Some(Ok(Message::Frame(frame))) => {
                        match frame.header().opcode {
                            OpCode::Data(Data::Text) => {
                                partial_frame_is_text = true;
                                partial_frame.extend(frame.payload());
                            }
                            OpCode::Data(Data::Binary) => {
                                partial_frame_is_text = false;
                                partial_frame.extend(frame.payload());
                            }
                            // We know which message we're continuing because
                            // otherwise partial_frame would be empty.
                            OpCode::Data(Data::Continue) if !partial_frame.is_empty() => {
                                partial_frame.extend(frame.payload())
                            }
                            _ => {
                                // Ignore other partial frames.
                            }
                        }
                        if frame.header().is_final && !partial_frame.is_empty() {
                            let payload = std::mem::take(&mut partial_frame);
                            let response = if partial_frame_is_text {
                                serde_json::from_slice(&payload).map_err(|err| err.into())
                            } else {
                                Ok(FluxSpeakResponse::Audio(payload.into()))
                            };
                            if response_tx.start_send(response).is_err() {
                                // Responses are no longer being received; close the stream.
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // We don't expect pongs from the API. They can be
                        // safely ignored.
                    }
                    Some(Err(err)) => {
                        if response_tx.start_send(Err(err.into())).is_err() {
                            // Responses are no longer being received; close the stream.
                            break;
                        }
                    }
                    None => {
                        // Upstream is closed; there is no socket to send
                        // Close to during cleanup.
                        is_open = false;
                        break;
                    }
                }
            }
            Step::Outbound(message) => {
                if is_open {
                    let message = match message {
                        Some(ClientMessage::Close) | None => {
                            is_open = false;
                            ClientMessage::Close
                        }
                        Some(message) => message,
                    };
                    match serde_json::to_string(&message) {
                        Ok(json) => {
                            if let Err(err) = ws_stream_send
                                .send(Message::Text(Utf8Bytes::from(json)))
                                .await
                            {
                                if response_tx.send(Err(err.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            if response_tx.send(Err(err.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    // Post-loop cleanup: ensure Close is sent if the connection is still open
    if is_open {
        if let Err(err) = ws_stream_send
            .send(Message::Text(Utf8Bytes::from(
                serde_json::to_string(&ClientMessage::Close).unwrap_or_default(),
            )))
            .await
        {
            // If the response channel is closed, there's nothing to be done about it now.
            let _ = response_tx.send(Err(err.into())).await;
        }
    }
    response_tx.close_channel();
    // Waiting for message_tx to be dropped before exiting
    while message_rx.next().await.is_some() {
        // Receiving messages after closing down. Ignore them.
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ClientMessage, PlaybackOffset};
    use crate::speak::flux::options::{Encoding, Model, Options};

    fn wire_json(message: &ClientMessage) -> String {
        serde_json::to_string(message).unwrap()
    }

    #[test]
    fn speak_wire_format() {
        let message = ClientMessage::Speak {
            text: "Hello! How can I help you today?".to_string(),
        };
        assert_eq!(
            wire_json(&message),
            r#"{"type":"Speak","text":"Hello! How can I help you today?"}"#
        );
    }

    #[test]
    fn flush_wire_format() {
        assert_eq!(wire_json(&ClientMessage::Flush), r#"{"type":"Flush"}"#);
    }

    #[test]
    fn interrupt_wire_format_without_offset() {
        let message = ClientMessage::Interrupt {
            playback_offset: None,
        };
        assert_eq!(wire_json(&message), r#"{"type":"Interrupt"}"#);
    }

    #[test]
    fn interrupt_wire_format_with_offset() {
        let message = ClientMessage::Interrupt {
            playback_offset: Some(PlaybackOffset::TimeMs { value: 1500 }),
        };
        assert_eq!(
            wire_json(&message),
            r#"{"type":"Interrupt","playback_offset":{"type":"time_ms","value":1500}}"#
        );
    }

    #[test]
    fn configure_wire_format() {
        let message = ClientMessage::Configure { speed: Some(1.05) };
        assert_eq!(wire_json(&message), r#"{"type":"Configure","speed":1.05}"#);
    }

    #[test]
    fn close_wire_format() {
        assert_eq!(wire_json(&ClientMessage::Close), r#"{"type":"Close"}"#);
    }

    #[test]
    fn test_flux_speak_url() {
        let dg = crate::Deepgram::new("token").unwrap();
        assert_eq!(
            dg.text_to_speech().flux_speak_url().to_string(),
            "wss://api.deepgram.com/v2/speak",
        );
    }

    #[test]
    fn test_flux_speak_url_custom_host() {
        let dg =
            crate::Deepgram::with_base_url_and_api_key("http://localhost:8080", "token").unwrap();
        assert_eq!(
            dg.text_to_speech().flux_speak_url().to_string(),
            "ws://localhost:8080/v2/speak",
        );
    }

    #[test]
    fn builder_url_carries_options() {
        let dg = crate::Deepgram::new("token").unwrap();
        let options = Options::builder(Model::FluxHaleyEn)
            .encoding(Encoding::Linear16)
            .sample_rate(24000)
            .speed(1.05)
            .build();
        let speak = dg.text_to_speech();
        let builder = speak.flux_request(options);
        assert_eq!(
            builder.urlencoded().unwrap(),
            "model=flux-haley-en&encoding=linear16&sample_rate=24000&speed=1.05"
        );
    }

    #[tokio::test]
    async fn handle_rejects_rest_only_encodings() {
        let dg = crate::Deepgram::new("token").unwrap();
        for (encoding, name) in [
            (Encoding::Mp3, "mp3"),
            (Encoding::Opus, "opus"),
            (Encoding::Flac, "flac"),
            (Encoding::Aac, "aac"),
        ] {
            let options = Options::builder(Model::FluxHaleyEn)
                .encoding(encoding)
                .build();
            let speak = dg.text_to_speech();
            let result = speak.flux_request(options).handle().await;
            match result {
                Err(crate::DeepgramError::InvalidOptions(message)) => {
                    assert!(
                        message.contains(name),
                        "error for `{name}` should name the encoding: {message}"
                    );
                }
                _ => panic!("expected InvalidOptions error for `{name}`"),
            }
        }
    }

    #[tokio::test]
    async fn handle_rejects_rest_only_options() {
        let dg = crate::Deepgram::new("token").unwrap();
        let options = Options::builder(Model::FluxHaleyEn)
            .callback("https://example.com/hook")
            .build();
        let speak = dg.text_to_speech();
        let result = speak.flux_request(options).handle().await;
        match result {
            Err(crate::DeepgramError::InvalidOptions(message)) => {
                assert!(message.contains("callback"));
            }
            _ => panic!("expected InvalidOptions error"),
        }
    }
}
