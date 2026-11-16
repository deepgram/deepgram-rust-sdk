//! Local coverage for the custom streaming transport factory.

#![cfg(all(feature = "agent", feature = "listen", feature = "speak"))]

use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use deepgram::{
    transport::{
        BoxedStreamingTransport, StreamingService, StreamingTransportFactory,
        TransportConnectFuture, TransportMessage, TransportRequest,
    },
    Deepgram, DeepgramError,
};
use futures::{Sink, Stream};

#[derive(Debug, Default)]
struct RecordingFactory {
    requests: Mutex<Vec<TransportRequest>>,
}

impl StreamingTransportFactory for RecordingFactory {
    fn connect(&self, request: TransportRequest) -> TransportConnectFuture<'_> {
        self.requests
            .lock()
            .expect("factory request list is not poisoned")
            .push(request);
        Box::pin(async { Ok(Box::new(PendingTransport) as BoxedStreamingTransport) })
    }
}

/// A connected transport that accepts writes and stays open until the SDK
/// drops it. This lets the test assert factory selection without a network.
#[derive(Debug)]
struct PendingTransport;

impl Stream for PendingTransport {
    type Item = Result<TransportMessage, DeepgramError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

impl Sink<TransportMessage> for PendingTransport {
    type Error = DeepgramError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, _item: TransportMessage) -> Result<(), Self::Error> {
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[tokio::test]
async fn factory_receives_every_streaming_service_request() {
    let factory = Arc::new(RecordingFactory::default());
    let client = Deepgram::new("test-api-key")
        .expect("valid client")
        .with_streaming_transport_factory(factory.clone());

    let listen_v1 = client
        .transcription()
        .stream_request()
        .handle()
        .await
        .expect("Listen V1 uses factory");
    let listen_v2 = client
        .transcription()
        .flux_request()
        .handle()
        .await
        .expect("Flux STT uses factory");
    let speak_v1 = client
        .text_to_speech()
        .speak_stream()
        .handle()
        .await
        .expect("Aura streaming TTS uses factory");
    let speak_v2 = client
        .text_to_speech()
        .flux_request(
            deepgram::speak::flux::options::Options::builder(
                deepgram::speak::flux::options::Model::FluxHaleyEn,
            )
            .build(),
        )
        .handle()
        .await
        .expect("Flux TTS uses factory");
    let agent = client
        .agent()
        .start()
        .await
        .expect("Voice Agent uses factory");

    drop((listen_v1, listen_v2, speak_v1, speak_v2, agent));

    let requests = factory
        .requests
        .lock()
        .expect("factory request list is not poisoned");
    assert_eq!(
        requests
            .iter()
            .map(|request| request.service)
            .collect::<Vec<_>>(),
        vec![
            StreamingService::Listen,
            StreamingService::FluxListen,
            StreamingService::Speak,
            StreamingService::FluxSpeak,
            StreamingService::Agent,
        ]
    );

    for request in requests.iter() {
        assert_eq!(
            request
                .headers
                .get("authorization")
                .expect("SDK forwards Authorization to custom transports"),
            "Token test-api-key"
        );
    }
    assert_eq!(requests[0].url.path(), "/v1/listen");
    assert_eq!(requests[1].url.path(), "/v2/listen");
    assert_eq!(requests[2].url.path(), "/v1/speak");
    assert_eq!(requests[3].url.path(), "/v2/speak");
    assert_eq!(requests[4].url.path(), "/v1/agent/converse");
}
