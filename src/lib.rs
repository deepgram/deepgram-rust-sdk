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

#[derive(Debug)]
pub struct Deepgram<'a> {
    api_key: &'a str,
}

// TODO sub-errors for the different types?
#[derive(Debug, Error)]
pub enum DeepgramError {
    #[error("No source was provided to the request builder.")]
    NoSource,
    #[error("Something went wrong when generating the http request: {0}")]
    HttpError(#[from] http::Error),
    #[error("Something went wrong during I/O: {0}")]
    IoError(#[from] io::Error),
    #[error("Something went wrong with WS: {0}")]
    WsError(#[from] tungstenite::Error),
    #[error("Something went wrong during serialization/deserialization: {0}")]
    SerdeError(#[from] serde_json::Error),
}

pub struct StreamRequestBuilder<'a, S>
where
    S: Stream<Item = Result<Bytes>>,
{
    config: &'a Deepgram<'a>,
    source: Option<S>,
}

#[derive(Debug, Deserialize)]
pub struct StreamResponse {
    pub duration: f64,
    pub is_final: bool,
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
                        if this.buf.len() == 0 {
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

impl<'a> Deepgram<'a> {
    // AsRef<str>
    pub fn new(api_key: &'a str) -> Self {
        Deepgram { api_key }
    }

    pub fn stream_request<S: Stream<Item = Result<Bytes>>>(&self) -> StreamRequestBuilder<S> {
        StreamRequestBuilder {
            config: self,
            source: None,
        }
    }
}

impl<'a> StreamRequestBuilder<'a, Receiver<Result<Bytes>>> {
    // Into<Path>
    pub async fn file(
        mut self,
        filename: impl AsRef<Path>,
        frame_size: usize,
        frame_delay: Duration,
    ) -> Result<StreamRequestBuilder<'a, Receiver<Result<Bytes>>>> {
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

impl<S> StreamRequestBuilder<'_, S>
where
    S: Stream<Item = Result<Bytes>> + Send + Unpin + 'static,
{
    pub async fn start(self) -> Result<Receiver<Result<StreamResponse>>> {
        let StreamRequestBuilder { config, source } = self;
        let mut source = source
            .ok_or(DeepgramError::NoSource)?
            .map(|res| res.map(|bytes| Message::binary(Vec::from(bytes.as_ref()))));

        let request = Request::builder()
            .method("GET")
            .uri("wss://api.deepgram.com/v1/listen")
            .header("authorization", format!("token {}", config.api_key))
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
            while let Some(Ok(frame)) = source.next().await {
                // This unwrap is not safe.
                write.send(frame).await.unwrap();
            }

            // This unwrap is not safe.
            println!("sending close...");
            write.send(Message::binary([])).await.unwrap();
        };

        let recv_task = async move {
            while let Some(Ok(msg)) = read.next().await {
                println!("receiving...");
                match dbg!(msg) {
                    Message::Text(txt) => {
                        tx.send(serde_json::from_str(&txt).map_err(DeepgramError::from))
                            .await
                            // This unwrap is probably not safe.
                            .unwrap();
                    }
                    _ => {}
                }
            }
        };

        tokio::join!(send_task, recv_task);

        Ok(rx)
    }
}
