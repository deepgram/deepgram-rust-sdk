#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

use std::io;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use bytes::{Bytes, BytesMut};
use futures::channel::mpsc::{self, Receiver};
use futures::stream::StreamExt;
use futures::{SinkExt, Stream};
use http::Request;
use pin_project::pin_project;
use serde::Deserialize;
use thiserror::Error;
use tokio::fs::File;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_util::io::ReaderStream;
use tungstenite::handshake::client;
use url::Url;

pub mod transcription;

#[derive(Debug, Clone)]
pub struct Deepgram<K>
where
    K: AsRef<str>,
{
    api_key: K,
    client: reqwest::Client,
}

// TODO sub-errors for the different types?
#[derive(Debug, Error)]
pub enum DeepgramError {
    #[error("No source was provided to the request builder.")]
    NoSource,
    #[error("Something went wrong during transcription.")]
    TranscriptionError { body: String, err: reqwest::Error },
    #[error("Something went wrong when generating the http request: {0}")]
    HttpError(#[from] http::Error),
    #[error("Something went wrong when making the HTTP request: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Something went wrong during I/O: {0}")]
    IoError(#[from] io::Error),
    #[error("Something went wrong with WS: {0}")]
    WsError(#[from] tungstenite::Error),
    #[error("Something went wrong during serialization/deserialization: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug)]
pub struct StreamRequestBuilder<'a, S, K, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    K: AsRef<str>,
{
    config: &'a Deepgram<K>,
    source: Option<S>,
    encoding: Option<String>,
    sample_rate: Option<u32>,
    channels: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct Alternatives {
    pub transcript: String,
    pub words: Vec<Word>,
}

#[derive(Debug, Deserialize)]
pub struct Channel {
    pub alternatives: Vec<Alternatives>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StreamResponse {
    TranscriptResponse {
        duration: f64,
        is_final: bool,
        channel: Channel,
    },
    TerminalResponse {
        request_id: String,
        created: String,
        duration: f64,
        channels: u32,
    },
}

type Result<T> = std::result::Result<T, DeepgramError>;

#[pin_project]
struct FileChunker {
    chunk_size: usize,
    buf: BytesMut,
    #[pin]
    file: ReaderStream<File>,
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

impl<K> Deepgram<K>
where
    K: AsRef<str>,
{
    /// Construct a new Deepgram client.
    ///
    /// Create your first API key on the [Deepgram Console][console].
    ///
    /// [console]: https://console.deepgram.com/
    ///
    /// # Panics
    ///
    /// Panics under the same conditions as [`reqwest::Client::new`].
    pub fn new(api_key: K) -> Self {
        static USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " rust",
        );

        Deepgram {
            api_key,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                // Even though `reqwest::Client::new` is not used here, it will always panic under the same conditions
                .expect("See reqwest::Client::new docs for cause of panic"),
        }
    }

    pub fn stream_request<E, S: Stream<Item = std::result::Result<Bytes, E>>>(
        &self,
    ) -> StreamRequestBuilder<S, K, E> {
        StreamRequestBuilder {
            config: self,
            source: None,
            encoding: None,
            sample_rate: None,
            channels: None,
        }
    }
}

impl<'a, S, K, E> StreamRequestBuilder<'a, S, K, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>>,
    K: AsRef<str>,
{
    pub fn stream(mut self, stream: S) -> Self {
        self.source = Some(stream);

        self
    }

    pub fn encoding(mut self, encoding: String) -> Self {
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
}

impl<'a, K> StreamRequestBuilder<'a, Receiver<Result<Bytes>>, K, DeepgramError>
where
    K: AsRef<str>,
{
    pub async fn file(
        mut self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
    ) -> Result<StreamRequestBuilder<'a, Receiver<Result<Bytes>>, K, DeepgramError>> {
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

        self.source = Some(rx);
        Ok(self)
    }
}

impl<S, K, E> StreamRequestBuilder<'_, S, K, E>
where
    S: Stream<Item = std::result::Result<Bytes, E>> + Send + Unpin + 'static,
    K: AsRef<str>,
    E: Send + std::fmt::Debug,
{
    pub async fn start(self) -> Result<Receiver<Result<StreamResponse>>> {
        let StreamRequestBuilder {
            config,
            source,
            encoding,
            sample_rate,
            channels,
        } = self;
        let mut source = source
            .ok_or(DeepgramError::NoSource)?
            .map(|res| res.map(|bytes| Message::binary(Vec::from(bytes.as_ref()))));

        // This unwrap is safe because we're parsing a static.
        let mut base = Url::parse("wss://api.deepgram.com/v1/listen").unwrap();
        let mut pairs = base.query_pairs_mut();
        if let Some(encoding) = encoding {
            pairs.append_pair("encoding", &encoding);
        }
        if let Some(sample_rate) = sample_rate {
            pairs.append_pair("sample_rate", &sample_rate.to_string());
        }
        if let Some(channels) = channels {
            pairs.append_pair("channels", &channels.to_string());
        }

        let request = Request::builder()
            .method("GET")
            // TODO Hard-coded.
            .uri("wss://api.deepgram.com/v1/listen?encoding=linear16&sample_rate=44100&channels=2")
            .header(
                "authorization",
                format!("token {}", config.api_key.as_ref()),
            )
            .header("sec-websocket-key", client::generate_key())
            .header("host", "api.deepgram.com")
            .header("connection", "upgrade")
            .header("upgrade", "websocket")
            .header("sec-websocket-version", "13")
            .body(())?;
        let (ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (mut write, mut read) = ws_stream.split();
        let (mut tx, rx) = mpsc::channel::<Result<StreamResponse>>(1);

        let send_task = async move {
            loop {
                match source.next().await {
                    None => break,
                    Some(Ok(frame)) => {
                        // This unwrap is not safe.
                        write.send(frame).await.unwrap();
                    }
                    Some(e) => {
                        let _ = dbg!(e);
                        break;
                    }
                }
            }

            // This unwrap is not safe.
            write.send(Message::binary([])).await.unwrap();
        };

        let recv_task = async move {
            loop {
                match read.next().await {
                    None => break,
                    Some(Ok(msg)) => {
                        if let Message::Text(txt) = msg {
                            let resp = serde_json::from_str(&txt).map_err(DeepgramError::from);
                            tx.send(resp)
                                .await
                                // This unwrap is probably not safe.
                                .unwrap();
                        }
                    }
                    Some(e) => {
                        let _ = dbg!(e);
                        break;
                    }
                }
            }
        };

        tokio::spawn(async move {
            tokio::join!(send_task, recv_task);
        });

        Ok(rx)
    }
}
