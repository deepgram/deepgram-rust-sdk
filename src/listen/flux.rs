// TODO: Remove this lint
// Currently not documented because interface of this module is still changing
#![allow(missing_docs)]

//! Types used for Flux turn-based conversational streaming.
//!
//! See the [Deepgram Flux API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/speech-to-text/listen-flux

use std::{
    error::Error,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use anyhow::anyhow;
use bytes::Bytes;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    future::poll_fn,
    pin_mut, select, select_biased,
    stream::StreamExt,
    FutureExt, SinkExt, Stream,
};
use http::Request;
use pin_project::pin_project;
use serde_urlencoded;
use tokio::fs::File;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::{
    handshake::client,
    protocol::frame::coding::{CloseCode, Data, OpCode},
    Utf8Bytes,
};
use url::Url;
use uuid::Uuid;

use self::file_chunker::FileChunker;
use crate::{
    common::{
        flux_response::{ConfigureThresholds, FluxResponse},
        options::{Encoding, Options},
    },
    Deepgram, DeepgramError, Result, Transcription,
};

use serde::Serialize;

static FLUX_URL_PATH: &str = "v2/listen";

#[derive(Clone, Debug)]
pub struct FluxBuilder<'a> {
    deepgram: &'a Deepgram,
    options: Options,
    encoding: Option<Encoding>,
    sample_rate: Option<u32>,
    stream_url: Url,
}

impl Transcription<'_> {
    /// Begin to configure a Flux streaming request with common options
    /// set to their default values.
    ///
    /// Once configured, the connection can be initiated with any of
    /// [`FluxBuilder::file`], [`FluxBuilder::stream`], or
    /// [`FluxBuilder::handle`].
    ///
    /// ```
    /// use deepgram::{
    ///     Deepgram,
    ///     common::options::{Encoding, Model, Options},
    /// };
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap_or_default()).unwrap();
    /// let transcription = dg.transcription();
    /// let builder = transcription
    ///     .flux_request()
    ///     .encoding(Encoding::Linear16);
    /// ```
    pub fn flux_request(&self) -> FluxBuilder<'_> {
        let options = Options::builder()
            .model(crate::common::options::Model::FluxGeneralEn)
            .build();
        self.flux_request_with_options(options)
    }

    /// Construct a Flux streaming request with common options
    /// specified in [`Options`].
    ///
    /// Once configured, the connection can be initiated with any of
    /// [`FluxBuilder::file`], [`FluxBuilder::stream`], or
    /// [`FluxBuilder::handle`].
    ///
    /// ```
    /// use deepgram::{
    ///     Deepgram,
    ///     common::options::{Encoding, Model, Options},
    /// };
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap_or_default()).unwrap();
    /// let transcription = dg.transcription();
    /// let options = Options::builder()
    ///     .model(Model::FluxGeneralEn)
    ///     .eager_eot_threshold(0.8)
    ///     .build();
    /// let builder = transcription
    ///     .flux_request_with_options(options)
    ///     .encoding(Encoding::Linear16);
    /// ```
    pub fn flux_request_with_options(&self, options: Options) -> FluxBuilder<'_> {
        FluxBuilder {
            deepgram: self.0,
            options,
            encoding: None,
            sample_rate: None,
            stream_url: self.flux_url(),
        }
    }

    fn flux_url(&self) -> Url {
        let mut url =
            self.0.base_url.join(FLUX_URL_PATH).expect(
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

impl FluxBuilder<'_> {
    /// Return the options in urlencoded format. If serialization would
    /// fail, this will also return an error.
    ///
    /// This is intended primarily to help with debugging API requests.
    pub fn urlencoded(&self) -> std::result::Result<String, serde_urlencoded::ser::Error> {
        Ok(self.as_url()?.query().unwrap_or_default().to_string())
    }

    fn as_url(&self) -> std::result::Result<Url, serde_urlencoded::ser::Error> {
        let Self {
            deepgram: _,
            options,
            encoding,
            sample_rate,
            stream_url,
        } = self;

        let mut url = stream_url.clone();
        {
            let mut pairs = url.query_pairs_mut();

            // Add standard options.
            pairs.extend_pairs(
                serde_urlencoded::from_str::<Vec<(String, String)>>(&options.urlencoded()?)
                    .expect("constructed query string can be deserialized"),
            );

            // Add streaming-specific options
            if let Some(encoding) = encoding {
                pairs.append_pair("encoding", encoding.as_str());
            }
            if let Some(sample_rate) = sample_rate {
                pairs.append_pair("sample_rate", &sample_rate.to_string());
            }
        }

        Ok(url)
    }

    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = Some(encoding);
        self
    }

    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }
}

impl FluxBuilder<'_> {
    pub async fn file(
        self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
    ) -> Result<FluxStream, DeepgramError> {
        let file = File::open(filename).await?;
        let mut chunker = FileChunker::new(file, frame_size);
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let rx_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        let task = async move {
            while let Some(frame) = chunker.next().await {
                tokio::time::sleep(frame_delay).await;
                // This unwrap() is safe because application logic dictates that the Receiver won't
                // be dropped before the Sender.
                if tx.send(frame).await.is_err() {
                    break;
                }
            }
        };
        tokio::spawn(task);
        self.stream(rx_stream).await
    }

    pub async fn stream<S, E>(self, stream: S) -> Result<FluxStream>
    where
        S: Stream<Item = Result<Bytes, E>> + Send + Unpin + 'static,
        E: Error + Send + Sync + 'static,
    {
        let handle = self.handle().await?;

        let (tx, rx) = mpsc::channel(1);
        let request_id = handle.request_id();
        tokio::task::spawn(async move {
            let mut handle = handle;
            let mut tx = tx;
            let mut stream = stream.fuse();

            loop {
                select_biased! {
                    // Receiving messages from FluxHandle
                    response = handle.response_rx.next() => {
                        match response {
                            Some(response) => {
                                if tx.send(response).await.is_err() {
                                    // Receiver has been dropped.
                                    break;
                                }
                            }
                            None => {
                                tx.close_channel();
                                // No more responses
                                break;
                            }
                        }
                    }
                    // Receiving audio data from stream.
                    chunk = stream.next() => {
                        match chunk {
                            Some(Ok(audio)) => {
                                if let Err(err) = handle.send_data(audio.to_vec()).await {
                                    if tx.send(Err(err)).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Some(Err(err)) => {
                                if tx.send(Err(DeepgramError::from(Box::new(err) as Box<dyn Error + Send + Sync + 'static>))).await.is_err() {
                                    break;
                                }
                            }
                            None => {
                                if let Err(err) = handle.close_stream().await {
                                    if tx.send(Err(err)).await.is_err() {
                                        break;
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok(FluxStream { rx, request_id })
    }

    /// A low level interface to the Deepgram Flux websocket API.
    pub async fn handle(self) -> Result<FluxHandle> {
        FluxHandle::new(self).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "type")]
enum ControlMessage {
    CloseStream,
    ForceEndTurn,
}

#[derive(Debug, Clone, PartialEq)]
enum WsMessage {
    Audio(Vec<u8>),
    /// Pre-serialized JSON for a `Configure` message.
    Configure(String),
    ForceEndTurn,
    CloseStream,
}

/// Settings to update mid-session via [`FluxHandle::configure`].
///
/// Mirrors the wire-level `Configure` message defined in
/// `asyncapi/schemas/schemas.listen.v2.yml`. All fields are optional —
/// any field set to `None` is left unchanged on the server.
///
/// Construct via [`ConfigureRequest::new`] + the `with_*` builder
/// methods rather than struct-literal syntax — `#[non_exhaustive]`
/// keeps future field additions backwards-compatible.
#[derive(Debug, Clone, Default, PartialEq)]
#[non_exhaustive]
pub struct ConfigureRequest {
    /// New end-of-turn threshold values. Omit to leave thresholds untouched.
    pub thresholds: Option<ConfigureThresholds>,

    /// New keyterm list. Omit (`None`) to leave keyterms untouched;
    /// pass `Some(vec![])` to clear them.
    pub keyterms: Option<Vec<String>>,

    /// New language hints. Only meaningful with `flux-general-multi`.
    /// Omit (`None`) to leave hints untouched; pass `Some(vec![])` to clear them.
    pub language_hints: Option<Vec<String>>,
}

impl ConfigureRequest {
    /// Construct an empty request — no fields will be sent unless
    /// populated via the `with_*` builders.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the threshold update.
    pub fn with_thresholds(mut self, thresholds: ConfigureThresholds) -> Self {
        self.thresholds = Some(thresholds);
        self
    }

    /// Replace the keyterms list.
    pub fn with_keyterms<I, S>(mut self, keyterms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keyterms = Some(keyterms.into_iter().map(Into::into).collect());
        self
    }

    /// Replace the language-hint list.
    pub fn with_language_hints<I, S>(mut self, hints: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.language_hints = Some(hints.into_iter().map(Into::into).collect());
        self
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ConfigureWireMessage<'a> {
    Configure {
        #[serde(skip_serializing_if = "Option::is_none")]
        thresholds: Option<&'a ConfigureThresholds>,
        #[serde(skip_serializing_if = "Option::is_none")]
        keyterms: Option<&'a [String]>,
        #[serde(skip_serializing_if = "Option::is_none")]
        language_hints: Option<&'a [String]>,
    },
}

#[derive(Debug)]
pub struct FluxHandle {
    message_tx: Sender<WsMessage>,
    pub(crate) response_rx: Receiver<Result<FluxResponse>>,
    request_id: Uuid,
}

impl FluxHandle {
    async fn new(builder: FluxBuilder<'_>) -> Result<FluxHandle> {
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

        tokio::task::spawn(run_flux_worker(ws_stream, message_rx, response_tx));

        Ok(FluxHandle {
            message_tx,
            response_rx,
            request_id,
        })
    }

    pub async fn send_data(&mut self, data: Vec<u8>) -> Result<()> {
        self.message_tx
            .send(WsMessage::Audio(data))
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
        Ok(())
    }

    /// Send a `Configure` message to update thresholds, keyterms, or
    /// language hints mid-session.
    ///
    /// The server will respond with either a
    /// [`crate::common::flux_response::FluxResponse::ConfigureSuccess`]
    /// echoing the post-update settings, or a
    /// [`crate::common::flux_response::FluxResponse::ConfigureFailure`]
    /// indicating the request was rejected.
    pub async fn configure(&mut self, request: ConfigureRequest) -> Result<()> {
        let wire = ConfigureWireMessage::Configure {
            thresholds: request.thresholds.as_ref(),
            keyterms: request.keyterms.as_deref(),
            language_hints: request.language_hints.as_deref(),
        };
        let json = serde_json::to_string(&wire)?;
        self.message_tx
            .send(WsMessage::Configure(json))
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
        Ok(())
    }

    /// Send a `ForceEndTurn` message to end the current turn immediately.
    ///
    /// Flux ends the turn on the audio transcribed so far — the transcript
    /// matches the most recent `Update` — and emits a standard `EndOfTurn`
    /// [`crate::common::flux_response::FluxResponse::TurnInfo`] event with
    /// [`trigger`](crate::common::flux_response::TurnTrigger) set to
    /// [`Manual`](crate::common::flux_response::TurnTrigger::Manual).
    ///
    /// # Confirming the turn actually ended
    ///
    /// `Ok(())` means only that the message was queued locally for the
    /// connection worker — it is **not** confirmation that the server ended
    /// the turn. Server acceptance arrives asynchronously through
    /// [`receive`](Self::receive):
    ///
    /// - **Confirmed**: an `EndOfTurn` `TurnInfo` event with
    ///   `trigger: Manual`.
    /// - **No active turn**: sent while no turn is in progress (e.g. during
    ///   leading silence), the message is silently ignored — no response is
    ///   produced.
    ///
    /// Callers that need to know the turn ended manually must observe the
    /// `EndOfTurn` event rather than rely on this method's return value.
    pub async fn force_end_turn(&mut self) -> Result<()> {
        self.message_tx
            .send(WsMessage::ForceEndTurn)
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
        Ok(())
    }

    /// Close the websocket stream. No more data should be sent after this is called.
    pub async fn close_stream(&mut self) -> Result<()> {
        if !self.message_tx.is_closed() {
            self.message_tx
                .send(WsMessage::CloseStream)
                .await
                .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
            self.message_tx.close_channel();
        }
        Ok(())
    }

    #[allow(clippy::let_and_return)]
    pub async fn receive(&mut self) -> Option<Result<FluxResponse>> {
        let resp = self.response_rx.next().await;
        resp
    }

    pub fn request_id(&self) -> Uuid {
        self.request_id
    }
}

async fn run_flux_worker(
    ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    mut message_rx: Receiver<WsMessage>,
    mut response_tx: Sender<Result<FluxResponse>>,
) -> Result<()> {
    // We use Vec<u8> for partial frames because we don't know if a fragment of a string is valid utf-8.
    let mut partial_frame: Vec<u8> = Vec::new();
    let (mut ws_stream_send, ws_stream_recv) = ws_stream.split();
    let mut ws_stream_recv = ws_stream_recv.fuse();
    let mut is_open: bool = true;
    // False once `message_rx` is exhausted (all senders dropped or the
    // channel closed): a closed-and-drained channel reports `Ready(None)`
    // on every poll, so keeping it in the select would busy-spin while
    // draining the final inbound responses.
    let mut commands_open: bool = true;

    /// One scheduling decision per loop iteration.
    enum Step {
        /// An inbound frame arrived, with response-channel capacity already
        /// reserved for forwarding it.
        Inbound(Option<std::result::Result<Message, tungstenite::Error>>),
        /// An outbound message is ready to be written to the socket.
        Outbound(Option<WsMessage>),
        /// The response consumer went away.
        ResponsesClosed,
    }

    loop {
        // Reserve response-channel capacity *before* reading an inbound
        // frame: when the consumer is backpressured, inbound reads pause
        // (backpressure propagates to the socket) instead of blocking this
        // loop mid-forward — so outbound control messages (ForceEndTurn,
        // Configure, CloseStream) stay deliverable throughout. The inbound
        // future borrows `response_tx` and `ws_stream_recv`, so it is scoped
        // to the selection and dropped before the step is handled.
        let step = {
            let inbound = async {
                match poll_fn(|cx| response_tx.poll_ready(cx)).await {
                    Ok(()) => Step::Inbound(ws_stream_recv.next().await),
                    Err(_) => Step::ResponsesClosed,
                }
            }
            .fuse();
            pin_mut!(inbound);
            if commands_open {
                // A fair (rather than biased) select, so neither a flood of
                // inbound responses nor a flood of outbound audio can starve
                // the other direction.
                select! {
                    step = inbound => step,
                    message = message_rx.next() => Step::Outbound(message),
                }
            } else {
                inbound.await
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
                    Some(Ok(Message::Ping(value))) => {
                        // We don't really care if the server receives the pong.
                        let _ = ws_stream_send.send(Message::Pong(value)).await;
                    }
                    Some(Ok(Message::Close(closeframe))) => {
                        // Forward abnormal closes to the consumer as errors —
                        // otherwise the stream silently ends and an abnormal
                        // shutdown is indistinguishable from a normal one.
                        if let Some(closeframe) = closeframe {
                            if closeframe.code != CloseCode::Normal {
                                let _ =
                                    response_tx.start_send(Err(DeepgramError::WebsocketClose {
                                        code: closeframe.code.into(),
                                        reason: closeframe.reason.to_string(),
                                    }));
                            }
                        }
                        // Reading a Close frame queues the acknowledgement on
                        // the shared websocket context; flush the write half so
                        // the closing handshake completes now rather than at
                        // socket teardown.
                        let _ = ws_stream_send.flush().await;
                        // The server is closing the connection; don't send
                        // CloseStream during cleanup.
                        is_open = false;
                        break;
                    }
                    Some(Ok(Message::Frame(frame))) => {
                        match frame.header().opcode {
                            OpCode::Data(Data::Text) => {
                                partial_frame.extend(frame.payload());
                            }
                            // We know we're continuing a text frame because otherwise
                            // partial_frame would be empty.
                            OpCode::Data(Data::Continue) if !partial_frame.is_empty() => {
                                partial_frame.extend(frame.payload())
                            }
                            _ => {
                                // Ignore other partial frames.
                            }
                        }
                        if frame.header().is_final {
                            let response = std::mem::take(&mut partial_frame);
                            let response =
                                serde_json::from_slice(&response).map_err(|err| err.into());
                            if response_tx.start_send(response).is_err() {
                                // Responses are no longer being received; close the stream.
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_) | Message::Pong(_))) => {
                        // We don't expect binary messages or pongs from the API.
                        // They can be safely ignored.
                    }
                    Some(Err(err)) => {
                        // A read error is terminal for the transport: forward
                        // it once and end the worker, rather than keep polling
                        // a broken socket (which can surface duplicate errors)
                        // or accept further commands doomed to fail.
                        let _ = response_tx.start_send(Err(err.into()));
                        is_open = false;
                        break;
                    }
                    None => {
                        // Upstream is closed; there is no socket to send
                        // CloseStream to during cleanup.
                        is_open = false;
                        break;
                    }
                }
            }
            Step::Outbound(message) => {
                if message.is_none() {
                    // The command channel is exhausted; stop polling it so the
                    // remaining inbound responses drain without busy-spinning.
                    commands_open = false;
                }
                if is_open {
                    match message {
                        // A failed write means the transport is broken: forward
                        // the first terminal error and end the worker, rather
                        // than accept further commands doomed to fail the same
                        // way.
                        Some(WsMessage::Audio(audio)) => {
                            if let Err(err) = ws_stream_send
                                .send(Message::Binary(Bytes::from(audio)))
                                .await
                            {
                                let _ = response_tx.send(Err(err.into())).await;
                                is_open = false;
                                break;
                            }
                        }
                        Some(WsMessage::Configure(json)) => {
                            if let Err(err) = ws_stream_send
                                .send(Message::Text(Utf8Bytes::from(json)))
                                .await
                            {
                                let _ = response_tx.send(Err(err.into())).await;
                                is_open = false;
                                break;
                            }
                        }
                        Some(WsMessage::ForceEndTurn) => {
                            if let Err(err) = ws_stream_send
                                .send(Message::Text(Utf8Bytes::from(
                                    serde_json::to_string(&ControlMessage::ForceEndTurn)
                                        .unwrap_or_default(),
                                )))
                                .await
                            {
                                let _ = response_tx.send(Err(err.into())).await;
                                is_open = false;
                                break;
                            }
                        }
                        Some(WsMessage::CloseStream) | None => {
                            is_open = false;
                            if let Err(err) = ws_stream_send
                                .send(Message::Text(Utf8Bytes::from(
                                    serde_json::to_string(&ControlMessage::CloseStream)
                                        .unwrap_or_default(),
                                )))
                                .await
                            {
                                let _ = response_tx.send(Err(err.into())).await;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    // Post-loop cleanup: ensure CloseStream is sent if connection is still open
    if is_open {
        if let Err(err) = ws_stream_send
            .send(Message::Text(Utf8Bytes::from(
                serde_json::to_string(&ControlMessage::CloseStream).unwrap_or_default(),
            )))
            .await
        {
            // If the response channel is closed, there's nothing to be done about it now.
            let _ = response_tx.send(Err(err.into())).await;
        }
    }
    response_tx.close_channel();
    // Return immediately, dropping `message_rx`: once the session is over,
    // later sends from the handle fail fast instead of being silently
    // discarded, and the socket is torn down promptly.
    Ok(())
}

#[derive(Debug)]
#[pin_project]
pub struct FluxStream {
    #[pin]
    rx: Receiver<Result<FluxResponse>>,
    request_id: Uuid,
}

impl Stream for FluxStream {
    type Item = Result<FluxResponse, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.rx.poll_next(cx)
    }
}

impl FluxStream {
    /// Returns the Deepgram request ID for the Flux streaming request.
    ///
    /// A request ID needs to be provided to Deepgram as part of any support
    /// or troubleshooting assistance related to a specific request.
    pub fn request_id(&self) -> Uuid {
        self.request_id
    }
}

mod file_chunker {
    use bytes::{Bytes, BytesMut};
    use futures::Stream;
    use pin_project::pin_project;
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::fs::File;
    use tokio_util::io::ReaderStream;

    use crate::{DeepgramError, Result};

    #[pin_project]
    pub(super) struct FileChunker {
        chunk_size: usize,
        buf: BytesMut,
        #[pin]
        file: ReaderStream<File>,
    }

    impl FileChunker {
        pub(super) fn new(file: File, chunk_size: usize) -> Self {
            FileChunker {
                chunk_size,
                buf: BytesMut::with_capacity(2 * chunk_size),
                file: ReaderStream::new(file),
            }
        }
    }

    impl Stream for FileChunker {
        type Item = Result<Bytes>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
            let mut this = self.project();

            while this.buf.len() < *this.chunk_size {
                match Pin::new(&mut this.file).poll_next(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(next) => match next.transpose() {
                        Err(e) => return Poll::Ready(Some(Err(DeepgramError::from(e)))),
                        Ok(None) => {
                            if this.buf.is_empty() {
                                return Poll::Ready(None);
                            } else {
                                return Poll::Ready(Some(Ok(this
                                    .buf
                                    .split_to(this.buf.len())
                                    .freeze())));
                            }
                        }
                        Ok(Some(next)) => {
                            this.buf.extend_from_slice(&next);
                        }
                    },
                }
            }

            Poll::Ready(Some(Ok(this.buf.split_to(*this.chunk_size).freeze())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigureRequest, ConfigureWireMessage, ControlMessage};
    use crate::common::flux_response::ConfigureThresholds;
    use crate::common::options::Options;

    /// Build the wire message exactly the way `FluxHandle::configure`
    /// builds it, then serialize. Centralized so the tests stay in
    /// sync with the production path.
    fn wire_json(req: &ConfigureRequest) -> String {
        let wire = ConfigureWireMessage::Configure {
            thresholds: req.thresholds.as_ref(),
            keyterms: req.keyterms.as_deref(),
            language_hints: req.language_hints.as_deref(),
        };
        serde_json::to_string(&wire).unwrap()
    }

    #[test]
    fn configure_wire_empty_skips_all_fields() {
        let req = ConfigureRequest::new();
        assert_eq!(wire_json(&req), r#"{"type":"Configure"}"#);
    }

    #[test]
    fn configure_wire_omit_vs_clear_distinguishable() {
        // `None` → field omitted entirely (leaves server-side value untouched).
        let omit = ConfigureRequest::new();
        assert_eq!(wire_json(&omit), r#"{"type":"Configure"}"#);

        // `Some(vec![])` → field serialized as `[]` (clears the
        // server-side list). The two cases MUST produce different JSON.
        let clear = ConfigureRequest::new().with_keyterms(Vec::<String>::new());
        assert_eq!(wire_json(&clear), r#"{"type":"Configure","keyterms":[]}"#);

        let clear_hints = ConfigureRequest::new().with_language_hints(Vec::<String>::new());
        assert_eq!(
            wire_json(&clear_hints),
            r#"{"type":"Configure","language_hints":[]}"#
        );
    }

    #[test]
    fn configure_wire_all_fields_populated() {
        let req = ConfigureRequest::new()
            .with_thresholds(
                ConfigureThresholds::new()
                    .with_eot_threshold(0.85)
                    .with_eot_timeout_ms(5_000),
            )
            .with_keyterms(["weather", "forecast"])
            .with_language_hints(["en", "es"]);
        assert_eq!(
            wire_json(&req),
            r#"{"type":"Configure","thresholds":{"eot_threshold":0.85,"eot_timeout_ms":5000},"keyterms":["weather","forecast"],"language_hints":["en","es"]}"#
        );
    }

    #[test]
    fn force_end_turn_wire_format() {
        let json = serde_json::to_string(&ControlMessage::ForceEndTurn).unwrap();
        assert_eq!(json, r#"{"type":"ForceEndTurn"}"#);
    }

    #[test]
    fn close_stream_wire_format() {
        let json = serde_json::to_string(&ControlMessage::CloseStream).unwrap();
        assert_eq!(json, r#"{"type":"CloseStream"}"#);
    }

    #[test]
    fn test_flux_url() {
        let dg = crate::Deepgram::new("token").unwrap();
        assert_eq!(
            dg.transcription().flux_url().to_string(),
            "wss://api.deepgram.com/v2/listen",
        );
    }

    #[test]
    fn test_flux_url_custom_host() {
        let dg =
            crate::Deepgram::with_base_url_and_api_key("http://localhost:8080", "token").unwrap();
        assert_eq!(
            dg.transcription().flux_url().to_string(),
            "ws://localhost:8080/v2/listen",
        );
    }

    #[test]
    fn query_escaping() {
        let dg = crate::Deepgram::new("token").unwrap();
        let opts = Options::builder()
            .keyterms(["test&value", "another"])
            .build();
        let transcription = dg.transcription();
        let builder = transcription.flux_request_with_options(opts.clone());
        assert_eq!(builder.urlencoded().unwrap(), opts.urlencoded().unwrap())
    }
}
