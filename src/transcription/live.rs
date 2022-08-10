//! Types used for live audio transcription.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-streaming

// TODO: Remove this lint
// Missing docs allowed while this module is under development
#![allow(missing_docs)]

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{stream::FusedStream, Sink, Stream};
use pin_project::pin_project;
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use super::Transcription;
use crate::DeepgramError;

static DEEPGRAM_API_URL_LISTEN: &str = "wss://api.deepgram.com/v1/listen";

#[derive(Debug)]
#[pin_project]
pub struct DeepgramLive {
    #[pin]
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

#[derive(Debug)]
pub struct Options {
    // TODO
}

#[derive(Debug)]
pub struct OptionsBuilder(Options);

#[derive(Debug, Deserialize)]
pub struct Response {
    // TODO
}

impl Options {
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder(Self {})
    }
}

impl OptionsBuilder {
    pub fn build(self) -> Options {
        self.0
    }
}

impl<K: AsRef<str>> Transcription<'_, K> {
    pub async fn live(&self, _options: &Options) -> crate::Result<DeepgramLive> {
        let (websocket, response) = connect_async(DEEPGRAM_API_URL_LISTEN).await?;

        Ok(DeepgramLive { websocket })
    }
}

impl Stream for DeepgramLive {
    type Item = crate::Result<Response>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!()
    }
}

impl<B: Into<Vec<u8>>> Sink<B> for DeepgramLive {
    type Error = DeepgramError;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn start_send(self: Pin<&mut Self>, item: B) -> Result<(), Self::Error> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }
}

impl FusedStream for DeepgramLive {
    fn is_terminated(&self) -> bool {
        todo!()
    }
}
