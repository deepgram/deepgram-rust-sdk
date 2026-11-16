//! Custom bidirectional transports for streaming Deepgram APIs.
//!
//! A [`StreamingTransportFactory`] can replace the SDK's built-in WebSocket
//! connection for Listen V1, Flux STT, Aura streaming TTS, Flux TTS, and
//! Voice Agent. This is useful for a proxy, a self-hosted transport, or a
//! protocol adapter such as bidirectional HTTP/2.
//!
//! The SDK does not retry a factory-provided transport and never replays a
//! message after a transport failure. A transport that reconnects internally
//! owns that lifecycle and must preserve the ordering and delivery semantics it
//! promises to its caller.

use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use anyhow::anyhow;
use bytes::Bytes;
use futures::{Sink, Stream};
use http::HeaderMap;
use tokio_tungstenite::{tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};
use tungstenite::{
    protocol::frame::{
        coding::{CloseCode, Data, OpCode},
        CloseFrame as NativeCloseFrame,
    },
    Utf8Bytes,
};
use url::Url;

use crate::{DeepgramError, Result};

/// The streaming product a [`TransportRequest`] targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StreamingService {
    /// Live speech-to-text over `/v1/listen`.
    Listen,
    /// Flux STT over `/v2/listen`.
    FluxListen,
    /// Aura streaming text-to-speech over `/v1/speak`.
    Speak,
    /// Flux TTS over `/v2/speak`.
    FluxSpeak,
    /// Voice Agent over `/v1/agent/converse`.
    Agent,
}

/// Fully resolved connection information passed to a
/// [`StreamingTransportFactory`].
///
/// `url` includes the encoded streaming query parameters. `headers` includes
/// the SDK User-Agent and, when the client was configured with a credential,
/// the Authorization header. Treat those headers as secrets: do not log them.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TransportRequest {
    /// The streaming product this connection targets.
    pub service: StreamingService,
    /// Full target URL, including encoded query parameters.
    pub url: Url,
    /// Request headers resolved by the SDK.
    pub headers: HeaderMap,
}

impl TransportRequest {
    pub(crate) fn new(service: StreamingService, url: Url, headers: HeaderMap) -> Self {
        Self {
            service,
            url,
            headers,
        }
    }
}

/// A WebSocket close status reported by a [`StreamingTransport`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TransportCloseFrame {
    /// WebSocket close code.
    pub code: u16,
    /// Peer-provided reason text.
    pub reason: String,
}

impl TransportCloseFrame {
    /// Construct a close frame from a code and reason.
    pub fn new(code: u16, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }
}

/// A complete frame exchanged by a [`StreamingTransport`].
///
/// Transport implementations must reassemble fragmented protocol frames before
/// yielding them. The SDK owns protocol-level JSON serialization,
/// deserialization, ping/pong replies, and close handling.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportMessage {
    /// A complete UTF-8 text frame, generally a JSON control message.
    Text(String),
    /// A complete binary frame, generally audio.
    Binary(Bytes),
    /// A transport ping frame.
    Ping(Bytes),
    /// A transport pong frame.
    Pong(Bytes),
    /// A peer close frame, optionally without a status.
    Close(Option<TransportCloseFrame>),
}

/// Object-safe transport type returned by a [`StreamingTransportFactory`].
///
/// The transport is a bidirectional stream of complete
/// [`TransportMessage`] values. It must surface a terminal failure once as
/// [`DeepgramError`], then end the stream. Its sink must reject sends after it
/// has ended. `Sink::poll_close` must perform the transport's graceful-close
/// action.
pub trait StreamingTransport:
    Stream<Item = Result<TransportMessage>>
    + Sink<TransportMessage, Error = DeepgramError>
    + Send
    + Unpin
{
}

impl<T> StreamingTransport for T where
    T: Stream<Item = Result<TransportMessage>>
        + Sink<TransportMessage, Error = DeepgramError>
        + Send
        + Unpin
{
}

/// Type-erased [`StreamingTransport`] returned by a transport factory.
pub type BoxedStreamingTransport = Box<dyn StreamingTransport>;

/// Future returned by [`StreamingTransportFactory::connect`].
pub type TransportConnectFuture<'a> =
    Pin<Box<dyn Future<Output = Result<BoxedStreamingTransport>> + Send + 'a>>;

/// Factory for custom bidirectional transports.
///
/// Store the factory on a client with
/// [`Deepgram::with_streaming_transport_factory`](crate::Deepgram::with_streaming_transport_factory).
/// A factory is called once per streaming connection. It may establish a
/// transport immediately or return a connecting transport, provided the
/// returned object observes the [`StreamingTransport`] contract.
///
/// The SDK does not wrap a custom transport in retry logic. This prevents
/// duplicate retry loops when an adapter already owns reconnect and backoff.
pub trait StreamingTransportFactory: fmt::Debug + Send + Sync {
    /// Create the transport for one streaming request.
    fn connect(&self, request: TransportRequest) -> TransportConnectFuture<'_>;
}

/// Built-in WebSocket transport adapted to the public transport contract.
///
/// Native WebSockets normally reassemble protocol frames themselves, but
/// tungstenite may expose fragments. Keeping their reassembly here gives a
/// factory-provided transport and the built-in transport the same complete-frame
/// contract.
#[derive(Debug)]
pub(crate) struct NativeStreamingTransport {
    stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    partial_frame: Vec<u8>,
    partial_frame_is_text: bool,
}

impl NativeStreamingTransport {
    pub(crate) fn new(stream: WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>) -> Self {
        Self {
            stream,
            partial_frame: Vec::new(),
            partial_frame_is_text: false,
        }
    }

    fn map_message(
        &mut self,
        message: Message,
    ) -> std::result::Result<Option<TransportMessage>, DeepgramError> {
        let message = match message {
            Message::Text(text) => Some(TransportMessage::Text(text.to_string())),
            Message::Binary(bytes) => Some(TransportMessage::Binary(bytes)),
            Message::Ping(bytes) => Some(TransportMessage::Ping(bytes)),
            Message::Pong(bytes) => Some(TransportMessage::Pong(bytes)),
            Message::Close(frame) => Some(TransportMessage::Close(frame.map(|frame| {
                TransportCloseFrame::new(frame.code.into(), frame.reason.to_string())
            }))),
            Message::Frame(frame) => {
                match frame.header().opcode {
                    OpCode::Data(Data::Text) => {
                        self.partial_frame_is_text = true;
                        self.partial_frame.extend(frame.payload());
                    }
                    OpCode::Data(Data::Binary) => {
                        self.partial_frame_is_text = false;
                        self.partial_frame.extend(frame.payload());
                    }
                    OpCode::Data(Data::Continue) if !self.partial_frame.is_empty() => {
                        self.partial_frame.extend(frame.payload());
                    }
                    _ => {}
                }

                if !frame.header().is_final || self.partial_frame.is_empty() {
                    None
                } else {
                    let payload = std::mem::take(&mut self.partial_frame);
                    Some(if self.partial_frame_is_text {
                        TransportMessage::Text(String::from_utf8(payload).map_err(|err| {
                            DeepgramError::UnexpectedServerResponse(anyhow!(
                                "received a fragmented WebSocket text frame that was not UTF-8: {err}"
                            ))
                        })?)
                    } else {
                        TransportMessage::Binary(payload.into())
                    })
                }
            }
        };
        Ok(message)
    }
}

impl Stream for NativeStreamingTransport {
    type Item = Result<TransportMessage>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            let message = match futures::ready!(Pin::new(&mut self.stream).poll_next(cx)) {
                Some(Ok(message)) => message,
                Some(Err(err)) => return Poll::Ready(Some(Err(err.into()))),
                None => return Poll::Ready(None),
            };
            match self.map_message(message) {
                Ok(Some(message)) => return Poll::Ready(Some(Ok(message))),
                Ok(None) => {}
                Err(err) => return Poll::Ready(Some(Err(err))),
            }
        }
    }
}

impl Sink<TransportMessage> for NativeStreamingTransport {
    type Error = DeepgramError;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Pin::new(&mut self.stream)
            .poll_ready(cx)
            .map_err(Into::into)
    }

    fn start_send(mut self: Pin<&mut Self>, message: TransportMessage) -> Result<()> {
        let message = match message {
            TransportMessage::Text(text) => Message::Text(Utf8Bytes::from(text)),
            TransportMessage::Binary(bytes) => Message::Binary(bytes),
            TransportMessage::Ping(bytes) => Message::Ping(bytes),
            TransportMessage::Pong(bytes) => Message::Pong(bytes),
            TransportMessage::Close(frame) => Message::Close(frame.map(|frame| NativeCloseFrame {
                code: CloseCode::from(frame.code),
                reason: Utf8Bytes::from(frame.reason),
            })),
        };
        Pin::new(&mut self.stream)
            .start_send(message)
            .map_err(Into::into)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Pin::new(&mut self.stream)
            .poll_flush(cx)
            .map_err(Into::into)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Pin::new(&mut self.stream)
            .poll_close(cx)
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::accept_async;

    async fn connected_transport() -> (
        NativeStreamingTransport,
        WebSocketStream<tokio::net::TcpStream>,
    ) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local WebSocket server");
        let address = listener.local_addr().expect("listener address");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept client");
            accept_async(stream).await.expect("upgrade local WebSocket")
        });
        let (client, _) = tokio_tungstenite::connect_async(format!("ws://{address}"))
            .await
            .expect("connect local WebSocket");
        (
            NativeStreamingTransport::new(client),
            server.await.expect("server task completes"),
        )
    }

    #[tokio::test]
    async fn native_transport_maps_complete_and_fragmented_messages() {
        let (mut transport, _server) = connected_transport().await;

        assert_eq!(
            transport
                .map_message(Message::Text(Utf8Bytes::from("text")))
                .expect("text is valid"),
            Some(TransportMessage::Text("text".to_string()))
        );
        assert_eq!(
            transport
                .map_message(Message::Binary(Bytes::from_static(b"audio")))
                .expect("binary is valid"),
            Some(TransportMessage::Binary(Bytes::from_static(b"audio")))
        );
        assert_eq!(
            transport
                .map_message(Message::Ping(Bytes::from_static(b"ping")))
                .expect("ping is valid"),
            Some(TransportMessage::Ping(Bytes::from_static(b"ping")))
        );
        assert_eq!(
            transport
                .map_message(Message::Pong(Bytes::from_static(b"pong")))
                .expect("pong is valid"),
            Some(TransportMessage::Pong(Bytes::from_static(b"pong")))
        );
        assert_eq!(
            transport
                .map_message(Message::Close(Some(NativeCloseFrame {
                    code: CloseCode::Normal,
                    reason: Utf8Bytes::from("done"),
                })))
                .expect("close is valid"),
            Some(TransportMessage::Close(Some(TransportCloseFrame::new(
                1000, "done"
            ))))
        );

        assert_eq!(
            transport
                .map_message(Message::Frame(
                    tungstenite::protocol::frame::Frame::message(
                        Bytes::from_static(b"hel"),
                        OpCode::Data(Data::Text),
                        false,
                    )
                ))
                .expect("first text fragment is valid"),
            None
        );
        assert_eq!(
            transport
                .map_message(Message::Frame(
                    tungstenite::protocol::frame::Frame::message(
                        Bytes::from_static(b"lo"),
                        OpCode::Data(Data::Continue),
                        true,
                    )
                ))
                .expect("final text fragment is valid"),
            Some(TransportMessage::Text("hello".to_string()))
        );
        assert_eq!(
            transport
                .map_message(Message::Frame(
                    tungstenite::protocol::frame::Frame::message(
                        Bytes::from_static(b"audio"),
                        OpCode::Data(Data::Binary),
                        true,
                    )
                ))
                .expect("binary fragment is valid"),
            Some(TransportMessage::Binary(Bytes::from_static(b"audio")))
        );
        assert_eq!(
            transport
                .map_message(Message::Frame(
                    tungstenite::protocol::frame::Frame::message(
                        Bytes::new(),
                        OpCode::Data(Data::Continue),
                        true,
                    )
                ))
                .expect("an orphaned continuation frame is ignored"),
            None
        );
        assert!(matches!(
            transport.map_message(Message::Frame(
                tungstenite::protocol::frame::Frame::message(
                    Bytes::from_static(&[0xff]),
                    OpCode::Data(Data::Text),
                    true,
                )
            )),
            Err(DeepgramError::UnexpectedServerResponse(_))
        ));
    }

    #[tokio::test]
    async fn native_transport_writes_every_message_kind_and_closes() {
        let (mut transport, mut server) = connected_transport().await;

        transport
            .send(TransportMessage::Text("text".to_string()))
            .await
            .expect("send text");
        transport
            .send(TransportMessage::Binary(Bytes::from_static(b"audio")))
            .await
            .expect("send binary");
        transport
            .send(TransportMessage::Ping(Bytes::from_static(b"ping")))
            .await
            .expect("send ping");
        transport
            .send(TransportMessage::Pong(Bytes::from_static(b"pong")))
            .await
            .expect("send pong");
        transport
            .send(TransportMessage::Close(Some(TransportCloseFrame::new(
                1000, "done",
            ))))
            .await
            .expect("send close");
        transport.flush().await.expect("flush transport");

        assert!(matches!(
            server.next().await,
            Some(Ok(Message::Text(text))) if text == "text"
        ));
        assert!(matches!(
            server.next().await,
            Some(Ok(Message::Binary(bytes))) if bytes == Bytes::from_static(b"audio")
        ));
        assert!(matches!(
            server.next().await,
            Some(Ok(Message::Ping(bytes))) if bytes == Bytes::from_static(b"ping")
        ));
        assert!(matches!(
            server.next().await,
            Some(Ok(Message::Pong(bytes))) if bytes == Bytes::from_static(b"pong")
        ));
        assert!(matches!(
            server.next().await,
            Some(Ok(Message::Close(Some(frame)))) if frame.code == CloseCode::Normal && frame.reason == "done"
        ));
        transport.close().await.expect("close transport");
    }
}
