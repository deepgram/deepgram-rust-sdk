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
use futures::{Sink, Stream, StreamExt};

#[test]
fn transport_request_constructor_supports_factory_unit_tests() {
    let request = TransportRequest::new(
        StreamingService::Listen,
        "wss://example.test/v1/listen?model=nova-3"
            .parse()
            .expect("valid test URL"),
        http::HeaderMap::new(),
    );
    assert_eq!(request.service, StreamingService::Listen);
    assert_eq!(request.url.path(), "/v1/listen");
}

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

/// Emits one Ping and then stays readable while rejecting the automatic Pong.
/// This models a custom transport whose write side fails independently from
/// its read side.
#[derive(Debug)]
struct PingThenFailTransport {
    sent_ping: bool,
}

impl Stream for PingThenFailTransport {
    type Item = Result<TransportMessage, DeepgramError>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.sent_ping {
            Poll::Pending
        } else {
            self.sent_ping = true;
            Poll::Ready(Some(Ok(TransportMessage::Ping(Default::default()))))
        }
    }
}

impl Sink<TransportMessage> for PingThenFailTransport {
    type Error = DeepgramError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, message: TransportMessage) -> Result<(), Self::Error> {
        if matches!(message, TransportMessage::Pong(_)) {
            Err(DeepgramError::InvalidOptions(
                "test Pong write failure".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

#[derive(Debug)]
struct PingThenFailFactory;

impl StreamingTransportFactory for PingThenFailFactory {
    fn connect(&self, _request: TransportRequest) -> TransportConnectFuture<'_> {
        Box::pin(async {
            Ok(Box::new(PingThenFailTransport { sent_ping: false }) as BoxedStreamingTransport)
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum ScriptedTransportMode {
    ReadError,
    NormalClose,
    AbnormalClose,
}

/// Delivers exactly one terminal inbound condition through the factory path.
#[derive(Debug)]
struct ScriptedTransport {
    mode: ScriptedTransportMode,
    delivered: bool,
}

impl Stream for ScriptedTransport {
    type Item = Result<TransportMessage, DeepgramError>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.delivered {
            return Poll::Ready(None);
        }
        self.delivered = true;
        Poll::Ready(Some(match self.mode {
            ScriptedTransportMode::ReadError => Err(DeepgramError::InvalidOptions(
                "test custom transport read failure".to_string(),
            )),
            ScriptedTransportMode::NormalClose => Ok(TransportMessage::Close(Some(
                deepgram::transport::TransportCloseFrame::new(1000, "normal close"),
            ))),
            ScriptedTransportMode::AbnormalClose => Ok(TransportMessage::Close(Some(
                deepgram::transport::TransportCloseFrame::new(1011, "test close"),
            ))),
        }))
    }
}

impl Sink<TransportMessage> for ScriptedTransport {
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

#[derive(Debug)]
struct ScriptedTransportFactory(ScriptedTransportMode);

impl StreamingTransportFactory for ScriptedTransportFactory {
    fn connect(&self, _request: TransportRequest) -> TransportConnectFuture<'_> {
        let mode = self.0;
        Box::pin(async move {
            Ok(Box::new(ScriptedTransport {
                mode,
                delivered: false,
            }) as BoxedStreamingTransport)
        })
    }
}

#[derive(Debug)]
struct FailingFactory;

impl StreamingTransportFactory for FailingFactory {
    fn connect(&self, _request: TransportRequest) -> TransportConnectFuture<'_> {
        Box::pin(async {
            Err(DeepgramError::InvalidOptions(
                "test factory connection failure".to_string(),
            ))
        })
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
        let authorization = request
            .headers
            .get("authorization")
            .expect("SDK forwards Authorization to custom transports");
        assert_eq!(authorization, "Token test-api-key");
        assert!(authorization.is_sensitive(), "credentials remain redacted");
        assert!(
            !format!("{request:?}").contains("test-api-key"),
            "debug output must redact the API key"
        );
    }
    assert_eq!(requests[0].url.path(), "/v1/listen");
    assert_eq!(requests[1].url.path(), "/v2/listen");
    assert_eq!(requests[2].url.path(), "/v1/speak");
    assert_eq!(requests[3].url.path(), "/v2/speak");
    assert_eq!(requests[4].url.path(), "/v1/agent/converse");
}

#[tokio::test]
async fn temporary_tokens_are_redacted_in_factory_requests() {
    let factory = Arc::new(RecordingFactory::default());
    let client = Deepgram::with_temp_token("test-temporary-token")
        .expect("valid client")
        .with_streaming_transport_factory(factory.clone());
    let handle = client
        .transcription()
        .stream_request()
        .handle()
        .await
        .expect("Listen V1 uses factory");
    drop(handle);

    let requests = factory
        .requests
        .lock()
        .expect("factory request list is not poisoned");
    let request = requests.first().expect("one factory request");
    let authorization = request
        .headers
        .get("authorization")
        .expect("SDK forwards Authorization to custom transports");
    assert_eq!(authorization, "Bearer test-temporary-token");
    assert!(authorization.is_sensitive(), "credentials remain redacted");
    assert!(
        !format!("{request:?}").contains("test-temporary-token"),
        "debug output must redact the temporary token"
    );
}

#[tokio::test]
async fn failed_automatic_pongs_end_every_factory_backed_session() {
    let client = Deepgram::new("test-api-key")
        .expect("valid client")
        .with_streaming_transport_factory(Arc::new(PingThenFailFactory));

    let mut listen_v1 = client
        .transcription()
        .stream_request()
        .handle()
        .await
        .expect("Listen V1 uses factory");
    assert!(listen_v1
        .receive()
        .await
        .expect("Pong write error is forwarded")
        .is_err());
    assert!(listen_v1.send_data(vec![0]).await.is_err());

    let mut listen_v2 = client
        .transcription()
        .flux_request()
        .handle()
        .await
        .expect("Flux STT uses factory");
    assert!(listen_v2
        .receive()
        .await
        .expect("Pong write error is forwarded")
        .is_err());
    assert!(listen_v2.send_data(vec![0]).await.is_err());

    let mut speak_v1 = client
        .text_to_speech()
        .speak_stream()
        .handle()
        .await
        .expect("Aura streaming TTS uses factory");
    assert!(speak_v1
        .receive()
        .await
        .expect("Pong write error is forwarded")
        .is_err());
    assert!(speak_v1.speak("must fail").await.is_err());

    let mut speak_v2 = client
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
    assert!(speak_v2
        .receive()
        .await
        .expect("Pong write error is forwarded")
        .is_err());
    assert!(speak_v2.speak("must fail").await.is_err());

    let (mut agent, mut events) = client
        .agent()
        .start()
        .await
        .expect("Voice Agent uses factory");
    assert!(events
        .next()
        .await
        .expect("Pong write error is forwarded")
        .is_err());
    assert!(agent.keep_alive().await.is_err());
}

#[tokio::test]
async fn factory_connection_errors_reach_handle_creation() {
    let client = Deepgram::new("test-api-key")
        .expect("valid client")
        .with_streaming_transport_factory(Arc::new(FailingFactory));

    assert!(matches!(
        client.transcription().stream_request().handle().await,
        Err(DeepgramError::InvalidOptions(message)) if message == "test factory connection failure"
    ));
}

#[tokio::test]
async fn factory_read_error_reaches_receive_once_then_ends() {
    let client = Deepgram::new("test-api-key")
        .expect("valid client")
        .with_streaming_transport_factory(Arc::new(ScriptedTransportFactory(
            ScriptedTransportMode::ReadError,
        )));
    let mut handle = client
        .transcription()
        .stream_request()
        .handle()
        .await
        .expect("Listen V1 uses factory");

    assert!(handle
        .receive()
        .await
        .expect("terminal read error is forwarded")
        .is_err());
    assert!(
        handle.receive().await.is_none(),
        "the terminal read error is delivered exactly once"
    );
    assert!(handle.send_data(vec![0]).await.is_err());
}

#[tokio::test]
async fn factory_close_frame_reaches_receive_once_then_ends() {
    let client = Deepgram::new("test-api-key")
        .expect("valid client")
        .with_streaming_transport_factory(Arc::new(ScriptedTransportFactory(
            ScriptedTransportMode::AbnormalClose,
        )));
    let mut handle = client
        .transcription()
        .stream_request()
        .handle()
        .await
        .expect("Listen V1 uses factory");

    assert!(matches!(
        handle.receive().await,
        Some(Err(DeepgramError::WebsocketClose { code: 1011, .. }))
    ));
    assert!(
        handle.receive().await.is_none(),
        "the close error is delivered exactly once"
    );
    assert!(handle.send_data(vec![0]).await.is_err());
}

#[tokio::test]
async fn factory_normal_close_ends_listen_v1_without_an_error() {
    let client = Deepgram::new("test-api-key")
        .expect("valid client")
        .with_streaming_transport_factory(Arc::new(ScriptedTransportFactory(
            ScriptedTransportMode::NormalClose,
        )));
    let mut handle = client
        .transcription()
        .stream_request()
        .handle()
        .await
        .expect("Listen V1 uses factory");

    assert!(
        handle.receive().await.is_none(),
        "a normal 1000 close must end Listen V1 without an error item"
    );
    assert!(handle.send_data(vec![0]).await.is_err());
}
