// TODO: Remove this lint
// Currently not documented because interface of this module is still changing
#![allow(missing_docs)]

//! Types used for live audio transcription.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-streaming

use crate::common::stream_response::StreamResponse;
use serde_urlencoded;
use std::borrow::Cow;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use futures::channel::mpsc as futures_mpsc;
use futures::channel::mpsc::{self, Receiver};
use futures::stream::{SplitSink, StreamExt};
use futures::{SinkExt, Stream};
use http::Request;
use pin_project::pin_project;
use tokio::fs::File;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_util::io::ReaderStream;
use tungstenite::handshake::client;
use url::Url;

use crate::common::options::{Encoding, Endpointing, Options, SerializableOptions};
use crate::{Deepgram, DeepgramError, Result, Transcription};

static LIVE_LISTEN_URL_PATH: &str = "v1/listen";

// Define event types
#[derive(Debug)]
pub enum Event {
    Open,
    Close,
    Error(DeepgramError),
    Result(String),
}

#[derive(Debug)]
pub struct StreamRequestBuilder<'a, S, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
{
    config: &'a Deepgram,
    options: Option<&'a Options>,
    source: Option<S>,
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
    #[allow(clippy::type_complexity)]
    write_arc: Option<Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>>,
}

#[pin_project]
struct FileChunker {
    chunk_size: usize,
    buf: BytesMut,
    #[pin]
    file: ReaderStream<File>,
}

impl Transcription<'_> {
    pub fn stream_request<E, S>(&self) -> StreamRequestBuilder<'_, S, E>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
    {
        self.stream_request_with_options(None)
    }

    pub fn stream_request_with_options<'a, E, S>(
        &'a self,
        options: Option<&'a Options>,
    ) -> StreamRequestBuilder<'a, S, E>
    where
        S: Stream<Item = std::result::Result<Bytes, E>>,
    {
        StreamRequestBuilder {
            config: self.0,
            options,
            source: None,
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
            write_arc: None,
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

impl<'a, S, E> StreamRequestBuilder<'a, S, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
{
    pub fn stream(mut self, stream: S) -> Self {
        self.source = Some(stream);

        self
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
}

impl<'a> StreamRequestBuilder<'a, Receiver<Result<Bytes>>, DeepgramError> {
    pub async fn file(
        mut self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
        event_tx: Sender<Event>,
    ) -> Result<StreamRequestBuilder<'a, Receiver<Result<Bytes>>, DeepgramError>> {
        let file = File::open(filename).await?;
        let mut chunker = FileChunker::new(file, frame_size);
        let (mut tx, rx) = mpsc::channel(1);
        let task = async move {
            while let Some(frame) = chunker.next().await {
                tokio::time::sleep(frame_delay).await;
                if let Err(e) = tx.send(frame).await {
                    let _ = event_tx.send(Event::Error(DeepgramError::from(e))).await;
                    break;
                }
            }
        };
        tokio::spawn(task);

        self.source = Some(rx);
        Ok(self)
    }
}

fn options_to_query_string(options: &Options) -> String {
    let serialized_options = SerializableOptions::from(options);
    serde_urlencoded::to_string(serialized_options).unwrap_or_default()
}

#[derive(Debug)]
pub struct TranscriptionStream {
    write_arc: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
}

impl TranscriptionStream {
    pub async fn finalize(
        &self,
        event_tx: Sender<Event>,
    ) -> std::result::Result<(), DeepgramError> {
        let finalize_message = Message::Text(r#"{"type": "Finalize"}"#.to_string());
        let mut write_guard = self.write_arc.lock().await;
        if let Err(e) = write_guard.send(finalize_message).await {
            let err = DeepgramError::from(e);
            event_tx.send(Event::Error(err)).await.unwrap();
            return Err(DeepgramError::CustomError(
                "Failed to send Finalize message".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn finish(&self, event_tx: Sender<Event>) -> std::result::Result<(), DeepgramError> {
        let finish_message = Message::Text(r#"{"type": "CloseStream"}"#.to_string());
        let mut write_guard = self.write_arc.lock().await;
        if let Err(e) = write_guard.send(finish_message).await {
            let err = DeepgramError::from(e);
            event_tx.send(Event::Error(err)).await.unwrap();
            return Err(DeepgramError::CustomError(
                "Failed to send CloseStream message".to_string(),
            ));
        }
        event_tx.send(Event::Close).await.unwrap();
        Ok(())
    }
}

impl<S, E> StreamRequestBuilder<'_, S, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>> + Send + Unpin + 'static,
    E: Send + std::fmt::Debug,
{
    /*
    // Add an automatic retry on failure to the start function
    pub async fn start_with_retry(
        &self,
        event_tx: Sender<Event>,
    ) -> std::result::Result<
        (
            TranscriptionStream,
            futures_mpsc::Receiver<std::result::Result<StreamResponse, DeepgramError>>,
        ),
        DeepgramError,
    > {
        let mut retry_count = 0;
        let retry_limit = 3;
        loop {
            // Re-create the builder with the same parameters for each retry
            let mut builder = StreamRequestBuilder {
                config: self.config,
                options: self.options,
                source: self.source,
                encoding: self.encoding.clone(),
                sample_rate: self.sample_rate,
                channels: self.channels,
                endpointing: self.endpointing.clone(),
                utterance_end_ms: self.utterance_end_ms,
                interim_results: self.interim_results,
                no_delay: self.no_delay,
                vad_events: self.vad_events,
                stream_url: self.stream_url.clone(),
                keep_alive: self.keep_alive,
                write_arc: None,
            };

            match builder.start(event_tx.clone()).await {
                Ok((stream, response_stream)) => {
                    return Ok((stream, response_stream));
                }
                Err(e) => {
                    event_tx.send(Event::Error(DeepgramError::from(e))).await.unwrap();
                    retry_count += 1;
                    if retry_count >= retry_limit {
                        println!("Retry attempts failed after {} attempts", retry_count);
                        return Err(DeepgramError::CustomError(
                            "Websocket retry attempts failed".to_string(),
                        ));
                    } else {
                        println!("Retry attempt: {}", retry_count);
                    }
                }
            }
        }
    }
    */
    
    pub async fn start(
        mut self,
        event_tx: Sender<Event>,
    ) -> std::result::Result<
        (
            TranscriptionStream,
            futures_mpsc::Receiver<std::result::Result<StreamResponse, DeepgramError>>,
        ),
        DeepgramError,
    > {
        // This unwrap is safe because we're parsing a static.
        let mut url = self.stream_url;
        {
            let mut pairs = url.query_pairs_mut();

            // Add standard pre-recorded options
            if let Some(options) = &self.options {
                let query_string = options_to_query_string(options);
                let query_pairs: Vec<(Cow<str>, Cow<str>)> = query_string
                    .split('&')
                    .map(|s| {
                        let mut split = s.splitn(2, '=');
                        (
                            Cow::from(split.next().unwrap_or_default()),
                            Cow::from(split.next().unwrap_or_default()),
                        )
                    })
                    .collect();

                for (key, value) in query_pairs {
                    pairs.append_pair(&key, &value);
                }
            }
            if let Some(encoding) = &self.encoding {
                pairs.append_pair("encoding", encoding.as_str());
            }
            if let Some(sample_rate) = self.sample_rate {
                pairs.append_pair("sample_rate", &sample_rate.to_string());
            }
            if let Some(channels) = self.channels {
                pairs.append_pair("channels", &channels.to_string());
            }
            if let Some(endpointing) = self.endpointing {
                pairs.append_pair("endpointing", &endpointing.to_str());
            }
            if let Some(utterance_end_ms) = self.utterance_end_ms {
                pairs.append_pair("utterance_end_ms", &utterance_end_ms.to_string());
            }
            if let Some(interim_results) = self.interim_results {
                pairs.append_pair("interim_results", &interim_results.to_string());
            }
            if let Some(no_delay) = self.no_delay {
                pairs.append_pair("no_delay", &no_delay.to_string());
            }
            if let Some(vad_events) = self.vad_events {
                pairs.append_pair("vad_events", &vad_events.to_string());
            }
        }

        let mut source = self
            .source
            .ok_or(DeepgramError::NoSource)?
            .map(|res| res.map(|bytes| Message::binary(Vec::from(bytes.as_ref()))));

        let request = {
            let builder = Request::builder()
                .method("GET")
                .uri(url.to_string())
                .header("sec-websocket-key", client::generate_key())
                .header("host", "api.deepgram.com")
                .header("connection", "upgrade")
                .header("upgrade", "websocket")
                .header("sec-websocket-version", "13");

            let builder = if let Some(api_key) = self.config.api_key.as_deref() {
                builder.header("authorization", format!("token {}", api_key))
            } else {
                builder
            };
            builder.body(())?
        };
        let (ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (write, mut read) = ws_stream.split();
        let write_arc = Arc::new(Mutex::new(write));
        let cloned_write_arc = Arc::clone(&write_arc);
        self.write_arc = Some(cloned_write_arc); // Store the write arc
        let (mut tx, rx) = mpsc::channel::<Result<StreamResponse>>(1);

        let event_tx_keep_alive = event_tx.clone();
        let event_tx_send = event_tx.clone();
        let event_tx_receive = event_tx.clone();

        event_tx.send(Event::Open).await.unwrap();

        // Spawn the keep-alive task
        if self.keep_alive.unwrap_or(false) {
            {
                let write_clone = Arc::clone(&write_arc);
                tokio::spawn(async move {
                    let mut interval = time::interval(Duration::from_secs(10));
                    loop {
                        interval.tick().await;
                        let keep_alive_message =
                            Message::Text("{\"type\": \"KeepAlive\"}".to_string());
                        let mut write = write_clone.lock().await;
                        if let Err(e) = write.send(keep_alive_message).await {
                            let _ = event_tx_keep_alive
                                .send(Event::Error(DeepgramError::from(e)))
                                .await;
                            break;
                        }
                    }
                })
            };
        }

        let write_clone = Arc::clone(&write_arc);
        let send_task = async move {
            while let Some(frame) = source.next().await {
                match frame {
                    Ok(frame) => {
                        let mut write = write_clone.lock().await;
                        if let Err(e) = write.send(frame).await {
                            let _ = event_tx_send
                                .send(Event::Error(DeepgramError::from(e)))
                                .await;
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = event_tx_send
                            .send(Event::Error(DeepgramError::CustomError(format!("{:?}", e))))
                            .await;
                        break;
                    }
                }
            }

            let mut write = write_clone.lock().await;
            if let Err(e) = write.send(Message::binary([])).await {
                let _ = event_tx_send
                    .send(Event::Error(DeepgramError::from(e)))
                    .await;
            }
        };

        let recv_write_clone = Arc::clone(&write_arc);
        let recv_task = async move {
            loop {
                match read.next().await {
                    None => break,
                    Some(Ok(msg)) => {
                        match msg {
                            Message::Text(txt) => {
                                let resp = serde_json::from_str(&txt).map_err(DeepgramError::from);
                                if let Err(e) = tx.send(resp).await {
                                    let _ = event_tx_receive
                                        .send(Event::Error(DeepgramError::from(e)))
                                        .await;
                                    break;
                                }
                            }
                            Message::Close(close_frame) => {
                                // Send a close frame back to acknowledge the close request
                                let mut write = recv_write_clone.lock().await;
                                if let Err(e) = write.send(Message::Close(close_frame)).await {
                                    // Notify the client we could not ack the server initiated close frame?
                                    let _ = event_tx_receive
                                        .send(Event::Error(DeepgramError::from(e)))
                                        .await;
                                }
                                event_tx_receive.send(Event::Close).await.unwrap();
                                break;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => {
                        let _ = dbg!(e);
                        break;
                    }
                }
            }
        };

        tokio::spawn(async move {
            tokio::join!(send_task, recv_task);
        });

        Ok((TranscriptionStream { write_arc }, rx))
    }

    pub fn keep_alive(mut self) -> Self {
        self.keep_alive = Some(true);

        self
    }
}

#[cfg(test)]
mod tests {
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
}
