//! Configure a streaming client with an in-memory custom transport.
//!
//! This is a minimal factory test double: it accepts outbound frames and
//! remains open without contacting Deepgram. A production factory establishes
//! its own TLS connection and protocol handshake from `TransportRequest`.

use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use deepgram::{
    transport::{
        BoxedStreamingTransport, StreamingTransportFactory, TransportConnectFuture,
        TransportMessage, TransportRequest,
    },
    Deepgram, DeepgramError,
};
use futures::{Sink, Stream};

#[derive(Debug)]
struct InMemoryFactory;

impl StreamingTransportFactory for InMemoryFactory {
    fn connect(&self, _request: TransportRequest) -> TransportConnectFuture<'_> {
        Box::pin(async { Ok(Box::new(InMemoryTransport) as BoxedStreamingTransport) })
    }
}

/// A transport test double that accepts writes and never yields an inbound
/// frame. Implement a real adapter here for a proxy or another streaming
/// protocol.
#[derive(Debug)]
struct InMemoryTransport;

impl Stream for InMemoryTransport {
    type Item = Result<TransportMessage, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

impl Sink<TransportMessage> for InMemoryTransport {
    type Error = DeepgramError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, _message: TransportMessage) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let client = Deepgram::new("replace-with-your-api-key")?
        .with_streaming_transport_factory(Arc::new(InMemoryFactory));

    // This calls InMemoryFactory instead of opening a native WebSocket.
    let handle = client.transcription().stream_request().handle().await?;
    drop(handle);

    println!("Custom transport factory connected.");
    Ok(())
}
