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
    path::Path,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bytes::{Bytes, BytesMut};
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    future::FutureExt,
    select,
    stream::StreamExt,
    SinkExt, Stream,
};
use http::Request;
use pin_project::pin_project;
use serde_urlencoded;
use tokio::fs::File;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tokio_util::io::ReaderStream;
use tungstenite::{
    handshake::client,
    protocol::frame::coding::{Data, OpCode},
};
use url::Url;

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
}

#[pin_project]
struct FileChunker {
    chunk_size: usize,
    buf: BytesMut,
    #[pin]
    file: ReaderStream<File>,
}

impl Transcription<'_> {
    pub fn stream_request(&self) -> WebsocketBuilder<'_> {
        self.stream_request_with_options(Options::default())
    }

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
        }
    }

    fn listen_stream_url(&self) -> Url {
        let mut url = self.0.base_url.join(LIVE_LISTEN_URL_PATH).unwrap();
        match url.scheme() {
            "http" | "ws" => url.set_scheme("ws").unwrap(),
            "https" | "wss" => url.set_scheme("wss").unwrap(),
            _ => panic!("base_url must have a scheme of http, https, ws, or wss"),
        }
        url
    }
}

impl FileChunker {
    fn new(file: File, chunk_size: usize) -> Self {
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

impl<'a> WebsocketBuilder<'a> {
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
    /// # let mut need_token = std::env::var("DEEPGRAM_API_TOKEN").is_err();
    /// # if need_token {
    /// #     std::env::set_var("DEEPGRAM_API_TOKEN", "abc")
    /// # }
    /// let dg = Deepgram::new(std::env::var("DEEPGRAM_API_TOKEN").unwrap());
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
    /// # if need_token {
    /// #     std::env::remove_var("DEEPGRAM_API_TOKEN");
    /// # }
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
}

impl<'a> WebsocketBuilder<'a> {
    pub async fn file(
        self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
    ) -> Result<TranscriptionStream, DeepgramError> {
        let file = File::open(filename).await?;
        let mut chunker = FileChunker::new(file, frame_size);
        let (mut tx, rx) = mpsc::channel(1);
        let task = async move {
            while let Some(frame) = chunker.next().await {
                tokio::time::sleep(frame_delay).await;
                // This unwrap() is safe because application logic dictates that the Receiver won't
                // be dropped before the Sender.
                tx.send(frame).await.unwrap();
            }
        };
        tokio::spawn(task);
        self.stream(rx).await
    }

    pub async fn stream<S, E>(self, stream: S) -> Result<TranscriptionStream>
    where
        S: Stream<Item = Result<Bytes, E>> + Send + Unpin + 'static,
        E: Error + Send + Sync + 'static,
    {
        let handle = self.handle().await?;

        let (tx, rx) = mpsc::channel(1);
        tokio::task::spawn(async move {
            let mut handle = handle;
            let mut tx = tx;
            let mut stream = stream;

            loop {
                select! {
                    result = stream.next().fuse() => {
                        match result {
                            Some(Ok(audio)) => if let Err(err) = handle.send_data(audio.into()).await {
                                if tx.send(Err(err)).await.is_err() {
                                    break;
                                }
                            },
                            Some(Err(err)) => {
                                if tx.send(Err(DeepgramError::from(Box::new(err) as Box<dyn Error + Send + Sync + 'static>))).await.is_err() {
                                    break;
                                }
                            }
                            None => {
                                continue;
                            }
                        }
                    }
                    response = handle.receive().fuse() => {
                        match response {
                            Some(response) => {
                                if tx.send(response).await.is_err() {
                                    // Receiver has been dropped.
                                    break;
                                }
                            }
                            None => {
                                // No more responses
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok(TranscriptionStream { rx })
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
    mut message_rx: Receiver<WsMessage>,
    mut response_tx: Sender<Result<StreamResponse>>,
    keep_alive: bool,
) -> Result<()> {
    // We use Vec<u8> for partial frames because we don't know if a fragment of a string is valid utf-8.
    let mut partial_frame: Vec<u8> = Vec::new();
    let (mut ws_stream_send, mut ws_stream_recv) = ws_stream.split();
    loop {
        // Primary event loop.
        select! {
            _ = tokio::time::sleep(Duration::from_secs(3)).fuse() => {
                if keep_alive {
                    send_message!(ws_stream_send, response_tx, Message::Text(
                        serde_json::to_string(&ControlMessage::KeepAlive).unwrap_or_default(),
                    ));
                }
            }
            message = message_rx.next().fuse() => {
                match message {
                    Some(WsMessage::Audio(audio))=> {
                        send_message!(ws_stream_send, response_tx, Message::Binary(audio));
                    }
                    Some(WsMessage::ControlMessage(msg)) => {
                        send_message!(ws_stream_send, response_tx, Message::Text(
                            serde_json::to_string(&msg).unwrap_or_default()
                        ));
                    }
                    None => {
                        // Input stream is shut down.  Keep processing responses.
                    }
                }
            }
            response = ws_stream_recv.next().fuse() => {
                match response {
                    Some(Ok(Message::Text(response))) => {
                        //
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
                    Some(Ok(Message::Close(_))) => {
                        return Ok(());
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
                        return Ok(())
                    }
                }
            }
        };
        if let Err(err) = ws_stream_send
            .send(Message::Text(
                serde_json::to_string(&ControlMessage::CloseStream).unwrap_or_default(),
            ))
            .await
        {
            // If the response channel is closed, there's nothing to be done about it now.
            let _ = response_tx.send(Err(err.into())).await;
        }
    }
    Ok(())
}

enum WsMessage {
    Audio(Vec<u8>),
    ControlMessage(ControlMessage),
}

#[derive(Debug)]
pub struct WebsocketHandle {
    message_tx: Sender<WsMessage>,
    response_rx: Receiver<Result<StreamResponse>>,
}

impl<'a> WebsocketHandle {
    async fn new(builder: WebsocketBuilder<'a>) -> Result<WebsocketHandle> {
        let url = builder.as_url()?;

        let request = {
            let http_builder = Request::builder()
                .method("GET")
                .uri(url.to_string())
                .header("sec-websocket-key", client::generate_key())
                .header("host", "api.deepgram.com")
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13");

            let builder = if let Some(api_key) = builder.deepgram.api_key.as_deref() {
                http_builder.header("authorization", format!("token {}", api_key))
            } else {
                http_builder
            };
            builder.body(())?
        };

        let (ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (message_tx, message_rx) = mpsc::channel(256);
        let (response_tx, response_rx) = mpsc::channel(256);

        tokio::task::spawn(run_worker(
            ws_stream,
            message_rx,
            response_tx,
            builder.keep_alive.unwrap_or(false),
        ));

        Ok(WebsocketHandle {
            message_tx,
            response_rx,
        })
    }

    pub async fn send_data(&mut self, data: Vec<u8>) -> Result<()> {
        self.message_tx
            .send(WsMessage::Audio(data))
            .await
            .expect("worker does not shutdown before handle");
        Ok(())
    }

    pub async fn finalize(&mut self) -> Result<()> {
        self.send_control_message(ControlMessage::Finalize).await
    }

    pub async fn keep_alive(&mut self) -> Result<()> {
        self.send_control_message(ControlMessage::KeepAlive).await
    }

    pub async fn close_stream(&mut self) -> Result<()> {
        self.send_control_message(ControlMessage::CloseStream).await
    }

    async fn send_control_message(&mut self, message: ControlMessage) -> Result<()> {
        self.message_tx
            .send(WsMessage::ControlMessage(message))
            .await
            .expect("worker does not shutdown before handle");
        Ok(())
    }

    pub async fn receive(&mut self) -> Option<Result<StreamResponse>> {
        self.response_rx.next().await
    }
}

#[derive(serde::Serialize)]
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
}

impl Stream for TranscriptionStream {
    type Item = Result<StreamResponse, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        this.rx.poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::ControlMessage;
    use crate::common::options::Options;

    #[test]
    fn test_stream_url() {
        let dg = crate::Deepgram::new("token");
        assert_eq!(
            dg.transcription().listen_stream_url().to_string(),
            "wss://api.deepgram.com/v1/listen",
        );
    }

    #[test]
    fn test_stream_url_custom_host() {
        let dg = crate::Deepgram::with_base_url_and_api_key("http://localhost:8080", "token");
        assert_eq!(
            dg.transcription().listen_stream_url().to_string(),
            "ws://localhost:8080/v1/listen",
        );
    }

    #[test]
    fn query_escaping() {
        let dg = crate::Deepgram::new("token");
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
