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
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use super::Transcription;
use crate::DeepgramError;

static DEEPGRAM_API_URL_LISTEN: &str = "wss://api.deepgram.com/v1/listen";

#[derive(Debug)]
pub struct DeepgramLive {
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
        let (websocket, _response) = connect_async(DEEPGRAM_API_URL_LISTEN).await?;

        Ok(DeepgramLive { websocket })
    }
}

impl DeepgramLive {
    pub async fn finish(&mut self) -> crate::Result<()> {
        Ok(self.websocket.send(Message::binary(Vec::new())).await?)
    }
}

impl Stream for DeepgramLive {
    type Item = crate::Result<Response>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.websocket.poll_next_unpin(cx).map(|message| {
            message.map(|message| Ok(serde_json::from_slice(&message?.into_data())?))
        })
    }
}

impl<B: Into<Vec<u8>>> Sink<B> for DeepgramLive {
    type Error = DeepgramError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.websocket
            .poll_ready_unpin(cx)
            .map(|result| Ok(result?))
    }

    fn start_send(mut self: Pin<&mut Self>, item: B) -> Result<(), Self::Error> {
        Ok(self.websocket.start_send_unpin(Message::binary(item))?)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.websocket
            .poll_flush_unpin(cx)
            .map(|result| Ok(result?))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.websocket
            .poll_close_unpin(cx)
            .map(|result| Ok(result?))
    }
}

impl FusedStream for DeepgramLive {
    fn is_terminated(&self) -> bool {
        self.websocket.is_terminated()
    }
}
