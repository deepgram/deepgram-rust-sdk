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
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{
        client::IntoClientRequest,
        handshake::client::Request,
        protocol::{frame::coding::CloseCode, CloseFrame, Message},
    },
    MaybeTlsStream, WebSocketStream,
};

use super::Transcription;
use crate::DeepgramError;

pub mod options;
pub mod response;

use options::{Options, SerializableOptions};
use response::Response;

static DEEPGRAM_API_URL_LISTEN: &str = "wss://api.deepgram.com/v1/listen";

// The traits `futures::{stream::FusedStream, Sink, Stream}` were chosen for `DeepgramLive`
// because tokio_tungstenite::WebSocketStream implements them, and DeepgramLive is essentially
// just a wrapper around tokio_tungstenite::WebSocketStream
#[derive(Debug)]
pub struct DeepgramLive {
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl<K: AsRef<str>> Transcription<'_, K> {
    pub async fn live(&self, options: &Options) -> crate::Result<DeepgramLive> {
        let request = self.make_streaming_request(options)?;

        let (websocket, _response) = connect_async(request).await?;

        Ok(DeepgramLive { websocket })
    }

    fn make_streaming_request(&self, options: &Options) -> crate::Result<Request> {
        // The reqwest::Request used here is *not* sent
        // It only exists to build a URL, which is passed to into_client_request
        // Since only the URL is used, headers aren't set here
        let mut request = self
            .0
            .client
            .get(DEEPGRAM_API_URL_LISTEN)
            .query(&SerializableOptions(options))
            .build()?
            .url()
            .into_client_request()?;

        request
            .headers_mut()
            .insert("Authorization", self.0.api_key_header.clone());

        Ok(request)
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
        match self.websocket.poll_next_unpin(cx) {
            // Deserialize the Response
            Poll::Ready(Some(Ok(Message::Text(message)))) => {
                let response = serde_json::from_str(&message)?;
                Poll::Ready(Some(Ok(response)))
            }

            // I'm pretty sure that Deepgram API doesn't send binary frames,
            // but it's good to make sure that this case is covered
            Poll::Ready(Some(Ok(Message::Binary(message)))) => {
                let response = serde_json::from_slice(&message)?;
                Poll::Ready(Some(Ok(response)))
            }

            // No need to give this message to the user of DeepgramLive
            // Skipping it and polling again
            //
            // The tokio_tungstenite::WebSocketStream will automatically
            // respond to the Ping/Close appropriately when we call poll_next
            // on it again (as we are doing below).
            //
            // It's important to call poll_next again, *even* if we know
            // that the DeepgramLive stream won't be producing any more values,
            // because otherwise the tokio_tungstenite::WebSocketStream won't
            // get a chance to respond to the Ping/Close.
            Poll::Ready(Some(Ok(
                Message::Ping(_)
                | Message::Pong(_)
                | Message::Close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: _,
                }))
                | Message::Close(None),
            ))) => self.poll_next(cx),

            // Occurs in response errors from Deepgram
            // https://developers.deepgram.com/api-reference/#error-handling-str
            Poll::Ready(Some(Ok(Message::Close(Some(not_normal_close))))) => Poll::Ready(Some(
                Err(DeepgramError::DeepgramLiveError(not_normal_close)),
            )),

            // tungstenite guarantees "that you're not going to get this value while reading the message"
            // Source: https://docs.rs/tungstenite/0.17/tungstenite/enum.Message.html#variant.Frame
            Poll::Ready(Some(Ok(Message::Frame(_)))) => {
                unreachable!("Unexpected raw WebSocket frame");
            }

            // tungstenite::Error occured
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(DeepgramError::WsError(err)))),

            // Stream has terminated
            Poll::Ready(None) => Poll::Ready(None),

            // Stream' next value is not ready yet
            Poll::Pending => Poll::Pending,
        }
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
