// TODO: Remove this lint
// Currently not documented because interface of this module is still changing
#![allow(missing_docs)]

//! Types used for live audio transcription.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-streaming

use std::{
    error::Error,
    fmt,
    ops::Deref,
    path::Path,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use anyhow::anyhow;
use bytes::Bytes;
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    future::{pending, FutureExt},
    select_biased,
    stream::StreamExt,
    SinkExt, Stream,
};
use http::Request;
use pin_project::pin_project;
use serde_urlencoded;
use tokio::fs::File;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::{
    handshake::client,
    protocol::frame::coding::{Data, OpCode},
};
use url::Url;
use uuid::Uuid;

use self::file_chunker::FileChunker;
use crate::{
    common::{
        options::{Encoding, Endpointing, Options},
        stream_response::StreamResponse,
    },
    Deepgram, DeepgramError, Result, Transcription,
};

static LIVE_LISTEN_URL_PATH: &str = "v1/listen";

#[derive(Clone, Debug)]
pub struct WebsocketBuilder<'a> {
    deepgram: &'a Deepgram,
    options: Options,
    encoding: Option<Encoding>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
    endpointing: Option<Endpointing>,
    utterance_end_ms: Option<u16>,
    interim_results: Option<bool>,
    no_delay: Option<bool>,
    vad_events: Option<bool>,
    stream_url: Url,
    keep_alive: Option<bool>,
    callback: Option<Url>,
}

impl Transcription<'_> {
    /// Begin to configure a websocket request with common options
    /// set to their default values.
    ///
    /// Once configured, the connection can be initiated with any of
    /// [`WebsocketBuilder::file`], [`WebsocketBuilder::stream`], or
    /// [`WebsocketBuilder::handle`].
    ///
    /// ```
    /// use deepgram::{
    ///     Deepgram,
    ///     DeepgramError,
    ///     common::options::{
    ///         DetectLanguage,
    ///         Encoding,
    ///         Model,
    ///         Options,
    ///     },
    ///     listen::websocket::WebsocketBuilder,
    /// };
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap_or_default()).unwrap();
    /// let transcription = dg.transcription();
    /// let builder: WebsocketBuilder<'_> = transcription
    ///     .stream_request()
    ///     .no_delay(true);
    /// ```
    pub fn stream_request(&self) -> WebsocketBuilder<'_> {
        self.stream_request_with_options(Options::default())
    }

    /// Construct a websocket request with common options
    /// specified in [`Options`].
    ///
    /// Once configured, the connection can be initiated with any of
    /// [`WebsocketBuilder::file`], [`WebsocketBuilder::stream`], or
    /// [`WebsocketBuilder::handle`].
    ///
    /// ```
    /// use deepgram::{
    ///     Deepgram,
    ///     DeepgramError,
    ///     common::options::{
    ///         DetectLanguage,
    ///         Encoding,
    ///         Model,
    ///         Options,
    ///     },
    /// };
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap_or_default()).unwrap();
    /// let transcription = dg.transcription();
    /// let options = Options::builder()
    ///     .model(Model::Nova2)
    ///     .detect_language(DetectLanguage::Enabled)
    ///     .build();
    /// let builder = transcription
    ///     .stream_request_with_options(
    ///         options,
    ///     )
    ///     .no_delay(true);
    ///
    /// assert_eq!(&builder.urlencoded().unwrap(), "model=nova-2&detect_language=true&no_delay=true")
    /// ```
    pub fn stream_request_with_options(&self, options: Options) -> WebsocketBuilder<'_> {
        WebsocketBuilder {
            deepgram: self.0,
            options,
            encoding: None,
            sample_rate: None,
            channels: None,
            endpointing: None,
            utterance_end_ms: None,
            interim_results: None,
            no_delay: None,
            vad_events: None,
            stream_url: self.listen_stream_url(),
            keep_alive: None,
            callback: None,
        }
    }

    fn listen_stream_url(&self) -> Url {
        // base
        let mut url =
            self.0.base_url.join(LIVE_LISTEN_URL_PATH).expect(
                "base_url is checked to be a valid base_url when constructing Deepgram client",
            );

        match url.scheme() {
            "http" | "ws" => url.set_scheme("ws").expect("a valid conversion according to the .set_scheme docs"),
            "https" | "wss" => url.set_scheme("wss").expect("a valid conversion according to the .set_scheme docs"),
            _ => unreachable!("base_url is validated to have a scheme of http, https, ws, or wss when constructing Deepgram client"),
        }
        url
    }
}

impl WebsocketBuilder<'_> {
    /// Return the options in urlencoded format. If serialization would
    /// fail, this will also return an error.
    ///
    /// This is intended primarily to help with debugging API requests.
    ///
    /// ```
    /// use deepgram::{
    ///     Deepgram,
    ///     DeepgramError,
    ///     common::options::{
    ///         DetectLanguage,
    ///         Encoding,
    ///         Model,
    ///         Options,
    ///     },
    /// };
    ///
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap_or_default()).unwrap();
    /// let transcription = dg.transcription();
    /// let options = Options::builder()
    ///     .model(Model::Nova2)
    ///     .detect_language(DetectLanguage::Enabled)
    ///     .build();
    /// let builder = transcription
    ///     .stream_request_with_options(
    ///         options,
    ///     )
    ///     .no_delay(true);
    ///
    /// assert_eq!(&builder.urlencoded().unwrap(), "model=nova-2&detect_language=true&no_delay=true")
    /// ```
    ///
    pub fn urlencoded(&self) -> std::result::Result<String, serde_urlencoded::ser::Error> {
        Ok(self.as_url()?.query().unwrap_or_default().to_string())
    }

    fn as_url(&self) -> std::result::Result<Url, serde_urlencoded::ser::Error> {
        // Destructuring ensures we don't miss new fields if they get added
        let Self {
            deepgram: _,
            keep_alive: _,
            options,
            encoding,
            sample_rate,
            channels,
            endpointing,
            utterance_end_ms,
            interim_results,
            no_delay,
            vad_events,
            stream_url,
            callback,
        } = self;

        let mut url = stream_url.clone();
        {
            let mut pairs = url.query_pairs_mut();

            // Add standard pre-recorded options.
            //
            // Here we serialize the options and then deserialize
            // in order to avoid duplicating serialization logic.
            //
            // TODO: We should be able to lean on the serde more
            // to avoid multiple serialization rounds.
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
            if let Some(channels) = channels {
                pairs.append_pair("channels", &channels.to_string());
            }
            if let Some(endpointing) = endpointing {
                pairs.append_pair("endpointing", &endpointing.to_string());
            }
            if let Some(utterance_end_ms) = utterance_end_ms {
                pairs.append_pair("utterance_end_ms", &utterance_end_ms.to_string());
            }
            if let Some(interim_results) = interim_results {
                pairs.append_pair("interim_results", &interim_results.to_string());
            }
            if let Some(no_delay) = no_delay {
                pairs.append_pair("no_delay", &no_delay.to_string());
            }
            if let Some(vad_events) = vad_events {
                pairs.append_pair("vad_events", &vad_events.to_string());
            }
            if let Some(callback) = callback {
                pairs.append_pair("callback", callback.as_ref());
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

    pub fn channels(mut self, channels: u16) -> Self {
        self.channels = Some(channels);

        self
    }

    pub fn endpointing(mut self, endpointing: Endpointing) -> Self {
        self.endpointing = Some(endpointing);

        self
    }

    pub fn utterance_end_ms(mut self, utterance_end_ms: u16) -> Self {
        self.utterance_end_ms = Some(utterance_end_ms);

        self
    }

    pub fn interim_results(mut self, interim_results: bool) -> Self {
        self.interim_results = Some(interim_results);

        self
    }

    pub fn no_delay(mut self, no_delay: bool) -> Self {
        self.no_delay = Some(no_delay);

        self
    }

    pub fn vad_events(mut self, vad_events: bool) -> Self {
        self.vad_events = Some(vad_events);

        self
    }

    pub fn keep_alive(mut self) -> Self {
        self.keep_alive = Some(true);

        self
    }

    pub fn callback(mut self, callback: Url) -> Self {
        self.callback = Some(callback);

        self
    }
}

impl WebsocketBuilder<'_> {
    pub async fn file(
        self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
    ) -> Result<TranscriptionStream, DeepgramError> {
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

    pub async fn stream<S, E>(self, stream: S) -> Result<TranscriptionStream>
    where
        S: Stream<Item = Result<Bytes, E>> + Send + Unpin + 'static,
        E: Error + Send + Sync + 'static,
    {
        let handle = self.handle().await?;

        let (tx, rx) = mpsc::channel(1);
        let mut is_done = false;
        let request_id = handle.request_id();
        tokio::task::spawn(async move {
            let mut handle = handle;
            let mut tx = tx;
            let mut stream = stream.fuse();

            loop {
                select_biased! {
                    // Receiving messages from WebsocketHandle
                    response = handle.response_rx.next() => {
                        // eprintln!("<stream> got response");
                        match response {
                            Some(Ok(response)) if matches!(response, StreamResponse::TerminalResponse { .. }) => {
                               // eprintln!( "<stream> got terminal response");
                                if tx.send(Ok(response)).await.is_err() {
                                    // Receiver has been dropped.
                                    break;
                                }
                            }
                            Some(response) => {
                                if tx.send(response).await.is_err() {
                                    // Receiver has been dropped.
                                    break;
                                }
                            }
                            None => {
                                // eprintln!("<stream> got none from handle");
                                tx.close_channel();
                                // No more responses
                                break;
                            }
                        }
                    }
                    // Receiving audio data from stream.
                    chunk = stream.next() => {
                        match chunk {
                            Some(Ok(audio)) => if let Err(err) = handle.send_data(audio.to_vec()).await {
                                // eprintln!("<stream> got audio");
                                if tx.send(Err(err)).await.is_err() {
                                    break;
                                }
                            },
                            Some(Err(err)) => {
                                // eprintln!("<stream> got error");
                                if tx.send(Err(DeepgramError::from(Box::new(err) as Box<dyn Error + Send + Sync + 'static>))).await.is_err() {
                                    break;
                                }
                            }
                            None => {
                                if is_done {

                                    continue;
                                }
                                if let Err(err) = handle.finalize().await {
                                    if tx.send(Err(err)).await.is_err() {
                                        break;
                                    }
                                }

                                if let Err(err) = handle.close_stream().await {
                                    if tx.send(Err(err)).await.is_err() {
                                        break;
                                    }
                                }
                                is_done = true;
                            }
                        }
                    }

                }
            }
        });
        Ok(TranscriptionStream {
            rx,
            done: false,
            request_id,
        })
    }

    /// A low level interface to the Deepgram websocket transcription API.
    pub async fn handle(self) -> Result<WebsocketHandle> {
        WebsocketHandle::new(self).await
    }
}

macro_rules! send_message {
    ($stream:expr, $response_tx:expr, $msg:expr) => {
        if let Err(err) = $stream.send($msg).await {
            if $response_tx.send(Err(err.into())).await.is_err() {
                // Responses are no longer being received; close the stream.
                break;
            }
        }
    };
}
async fn run_worker(
    ws_stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    mut message_tx: Sender<WsMessage>,
    message_rx: Receiver<WsMessage>,
    mut response_tx: Sender<Result<StreamResponse>>,
    keep_alive: bool,
) -> Result<()> {
    // We use Vec<u8> for partial frames because we don't know if a fragment of a string is valid utf-8.
    let mut partial_frame: Vec<u8> = Vec::new();
    let (mut ws_stream_send, ws_stream_recv) = ws_stream.split();
    let mut ws_stream_recv = ws_stream_recv.fuse();
    let mut is_open: bool = true;
    let mut last_sent_message = tokio::time::Instant::now();
    let mut message_rx = message_rx.fuse();
    loop {
        // eprintln!("<worker> loop");
        let sleep = tokio::time::sleep_until(last_sent_message + Duration::from_secs(3));
        // Primary event loop.
        select_biased! {
            _ = sleep.fuse() => {
                // eprintln!("<worker> sleep");
                if keep_alive && is_open {
                    message_tx.send(WsMessage::ControlMessage(ControlMessage::KeepAlive)).await.expect("we hold the receiver, so we know it hasn't been dropped");
                    last_sent_message = tokio::time::Instant::now();
                } else {
                    pending::<()>().await;
                }
            }
            response = ws_stream_recv.next() => {
                match response {
                    Some(Ok(Message::Text(response))) => {
                        // eprintln!("<worker> received dg response");
                        match serde_json::from_str(&response) {
                            Ok(response) => {
                                if (response_tx.send(Ok(response)).await).is_err() {
                                    // Responses are no longer being received; close the stream.
                                    break;
                                }
                            }
                            Err(err) =>{
                                if (response_tx.send(Err(err.into())).await).is_err() {
                                    // Responses are no longer being received; close the stream.
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(value))) => {
                        // We don't really care if the server receives the pong.
                        let _ = ws_stream_send.send(Message::Pong(value)).await;
                    }
                    Some(Ok(Message::Close(None))) => {
                        // eprintln!("<worker> received websocket close");
                        return Ok(());
                    }
                    Some(Ok(Message::Close(Some(closeframe)))) => {
                        // eprintln!("<worker> received websocket close");
                        return Err(DeepgramError::WebsocketClose {
                            code: closeframe.code.into(),
                            reason: closeframe.reason.into_owned(),
                        });
                    }

                    Some(Ok(Message::Frame(frame))) => {
                        match frame.header().opcode
                        {
                            OpCode::Data(Data::Text) => {
                                partial_frame.extend(frame.payload());
                            }
                            OpCode::Data(Data::Continue) => {
                                // We know we're continuing a text frame because otherwise
                                // partial_frame would be empty.
                                if !partial_frame.is_empty() {
                                    partial_frame.extend(frame.payload())
                                }
                            }
                            _ => {
                                // Ignore other partial frames.
                            }
                        }
                        if frame.header().is_final {
                            let response = std::mem::take(&mut partial_frame);
                            let response = serde_json::from_slice(&response).map_err(|err| err.into());
                            if (response_tx.send(response).await).is_err() {
                                // Responses are no longer being received; close the stream.
                                break
                            }
                        }
                    }
                    Some(Ok(Message::Binary(_) | Message::Pong(_))) => {
                        // We don't expect binary messages or pongs from the API.
                        // They can be safely ignored.
                    }

                    Some(Err(err)) => {
                        if (response_tx.send(Err(err.into())).await).is_err() {
                            // Responses are no longer being received; close the stream.
                            break;
                        }

                    }
                    None => {
                        // Upstream is closed
                        // eprintln!("<worker> received None");
                        return Ok(())
                    }
                }
            }
            message = message_rx.next().fuse() => {
                // eprintln!("<worker> received message: {message:?}, {is_open:?}");
                if is_open {
                    match message {
                        Some(WsMessage::Audio(audio))=> {
                            send_message!(ws_stream_send, response_tx, Message::Binary(audio.0));
                            last_sent_message = tokio::time::Instant::now();

                        }
                        Some(WsMessage::ControlMessage(msg)) => {
                            send_message!(ws_stream_send, response_tx, Message::Text(
                                serde_json::to_string(&msg).unwrap_or_default()
                            ));
                            last_sent_message = tokio::time::Instant::now();
                            if msg == ControlMessage::CloseStream {
                                is_open = false;
                            }
                        }
                        None => {
                            // Input stream is shut down.  Keep processing responses.
                            send_message!(ws_stream_send, response_tx, Message::Text(
                                serde_json::to_string(&ControlMessage::CloseStream).unwrap_or_default()
                            ));
                            is_open = false;
                        }
                    }
                }
            }
        };
    }
    // eprintln!("<worker> post loop");
    if let Err(err) = ws_stream_send
        .send(Message::Text(
            serde_json::to_string(&ControlMessage::CloseStream).unwrap_or_default(),
        ))
        .await
    {
        // If the response channel is closed, there's nothing to be done about it now.
        let _ = response_tx.send(Err(err.into())).await;
    }
    response_tx.close_channel();
    // Waiting for message_tx to be dropped before exiting
    while message_rx.next().await.is_some() {
        // Receiving messages after closing down. Ignore them.
    }
    // eprintln!("<worker> exit");
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum WsMessage {
    Audio(Audio),
    ControlMessage(ControlMessage),
}

#[derive(Clone, PartialEq, Eq)]
struct Audio(Vec<u8>);

impl fmt::Debug for Audio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("audio")
            .field(&format!(
                "<{} bytes (sha256:{})>",
                self.0.len(),
                &sha256::digest(&self.0)[..12]
            ))
            .finish()
    }
}

impl Deref for Audio {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct WebsocketHandle {
    message_tx: Sender<WsMessage>,
    response_rx: futures::stream::Fuse<Receiver<Result<StreamResponse>>>,
    request_id: Uuid,
}

impl WebsocketHandle {
    async fn new(builder: WebsocketBuilder<'_>) -> Result<WebsocketHandle> {
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
                .header("sec-websocket-version", "13");

            let builder = if let Some(api_key) = builder.deepgram.api_key.as_deref() {
                http_builder.header("authorization", format!("Token {}", api_key))
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

        tokio::task::spawn({
            let message_tx = message_tx.clone();
            run_worker(
                ws_stream,
                message_tx,
                message_rx,
                response_tx,
                builder.keep_alive.unwrap_or(false),
            )
        });

        Ok(WebsocketHandle {
            message_tx,
            response_rx: response_rx.fuse(),
            request_id,
        })
    }

    pub async fn send_data(&mut self, data: Vec<u8>) -> Result<()> {
        let audio = Audio(data);
        // eprintln!("<handle> sending audio: {audio:?}");

        self.message_tx
            .send(WsMessage::Audio(audio))
            .await
            .map_err(|err| DeepgramError::InternalClientError(err.into()))?;
        Ok(())
    }

    /// Send a Finalize message to the Deepgram API to force the server to process
    /// all the audio it has already received.
    pub async fn finalize(&mut self) -> Result<()> {
        self.send_control_message(ControlMessage::Finalize).await
    }

    /// Send a KeepAlive message to the Deepgram API to ensure the connection
    /// isn't closed due to long idle times.
    pub async fn keep_alive(&mut self) -> Result<()> {
        self.send_control_message(ControlMessage::KeepAlive).await
    }

    /// Close the websocket stream. No more data should be sent after this is called.
    pub async fn close_stream(&mut self) -> Result<()> {
        if !self.message_tx.is_closed() {
            self.send_control_message(ControlMessage::CloseStream)
                .await?;
            self.message_tx.close_channel();
        }
        Ok(())
    }

    async fn send_control_message(&mut self, message: ControlMessage) -> Result<()> {
        // eprintln!("<handle> sending control message: {message:?}");
        self.message_tx
            .send(WsMessage::ControlMessage(message.clone()))
            .await
            .map_err(|err| {
                // eprintln!("<handle> error sending control message: {message:?}");
                DeepgramError::InternalClientError(err.into())
            })?;
        // eprintln!("<handle> sent control message");
        Ok(())
    }

    #[allow(clippy::let_and_return)]
    pub async fn receive(&mut self) -> Option<Result<StreamResponse>> {
        let resp = self.response_rx.next().await;
        // eprintln!("<handle> receiving response: {resp:?}");
        resp
    }

    pub fn request_id(&self) -> Uuid {
        self.request_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "type")]
enum ControlMessage {
    Finalize,
    KeepAlive,
    CloseStream,
}

#[derive(Debug)]
#[pin_project]
pub struct TranscriptionStream {
    #[pin]
    rx: Receiver<Result<StreamResponse>>,
    done: bool,
    request_id: Uuid,
}

impl Stream for TranscriptionStream {
    type Item = Result<StreamResponse, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.rx.poll_next(cx)
    }
}

impl TranscriptionStream {
    /// Returns the Deepgram request ID for the speech-to-text live request.
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
    use super::ControlMessage;
    use crate::common::options::Options;

    #[test]
    fn test_stream_url() {
        let dg = crate::Deepgram::new("token").unwrap();
        assert_eq!(
            dg.transcription().listen_stream_url().to_string(),
            "wss://api.deepgram.com/v1/listen",
        );
    }

    #[test]
    fn test_stream_url_custom_host() {
        let dg =
            crate::Deepgram::with_base_url_and_api_key("http://localhost:8080", "token").unwrap();
        assert_eq!(
            dg.transcription().listen_stream_url().to_string(),
            "ws://localhost:8080/v1/listen",
        );
    }

    #[test]
    fn query_escaping() {
        let dg = crate::Deepgram::new("token").unwrap();
        let opts = Options::builder().custom_topics(["A&R"]).build();
        let transcription = dg.transcription();
        let builder = transcription.stream_request_with_options(opts.clone());
        assert_eq!(builder.urlencoded().unwrap(), opts.urlencoded().unwrap())
    }

    #[test]
    fn control_message_format() {
        assert_eq!(
            &serde_json::to_string(&ControlMessage::CloseStream).unwrap(),
            r#"{"type":"CloseStream"}"#
        );
    }
}
