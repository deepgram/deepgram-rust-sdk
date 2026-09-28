//! Mock WebSocket server tests that verify `/v1/listen` preserves unknown
//! message types without stopping the response stream.

#![cfg(feature = "listen")]

use deepgram::{common::stream_response::StreamResponse, Deepgram};
use futures::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{self, protocol::Message};

const REQUEST_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

async fn mock_listen_server(messages: Vec<&'static str>) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        #[allow(clippy::result_large_err)]
        let callback =
            |_request: &tungstenite::handshake::server::Request,
             mut response: tungstenite::handshake::server::Response| {
                response
                    .headers_mut()
                    .insert("dg-request-id", REQUEST_ID.parse().unwrap());
                Ok(response)
            };
        let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .expect("upgrade");

        for message in messages {
            ws.send(Message::Text(message.into())).await.expect("send");
        }
        ws.close(None).await.expect("close");
    });

    port
}

fn client(port: u16) -> Deepgram {
    Deepgram::with_base_url_and_api_key(format!("http://127.0.0.1:{port}").as_str(), "fake-key")
        .expect("client")
}

#[tokio::test]
async fn unknown_error_frame_does_not_stop_listen_stream() {
    let port = mock_listen_server(vec![
        r#"{"type":"Error","variant":"SchemaError","description":"request schema is invalid","message":"invalid request"}"#,
        r#"{"type":"Results","channel_index":[0,1],"duration":1.0,"start":0.0,"is_final":true,"from_finalize":false,"channel":{"alternatives":[{"transcript":"hello","words":[],"confidence":0.99}]},"metadata":{"request_id":"request-123","model_info":{"name":"nova-3","version":"2026-09-01","arch":"2"},"model_uuid":"model-123"}}"#,
        r#"{"type":"Metadata","request_id":"request-123","created":"2026-09-28T00:00:00Z","duration":1.0,"channels":1}"#,
    ])
    .await;

    let mut handle = client(port)
        .transcription()
        .stream_request()
        .handle()
        .await
        .expect("connect");

    let error = handle
        .receive()
        .await
        .expect("error frame")
        .expect("unknown response");
    match error {
        StreamResponse::Unknown(value) => {
            assert_eq!(value["variant"], "SchemaError");
            assert_eq!(value["message"], "invalid request");
        }
        other => panic!("expected unknown response, got {other:?}"),
    }

    let results = handle
        .receive()
        .await
        .expect("results frame")
        .expect("transcript response");
    match results {
        StreamResponse::TranscriptResponse {
            speech_final,
            channel,
            ..
        } => {
            assert!(!speech_final);
            assert_eq!(channel.alternatives[0].transcript, "hello");
        }
        other => panic!("expected transcript response, got {other:?}"),
    }

    assert!(matches!(
        handle.receive().await,
        Some(Ok(StreamResponse::TerminalResponse { .. }))
    ));
    assert!(handle.receive().await.is_none());
}
