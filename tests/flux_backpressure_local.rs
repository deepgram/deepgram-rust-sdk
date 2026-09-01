//! CI-runnable regression tests for the Flux worker's transport behavior,
//! using a localhost WebSocket server so no network access or API key is
//! required.

#![cfg(feature = "listen")]

use std::time::Duration;

use deepgram::{common::flux_response::FluxResponse, Deepgram, DeepgramError};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;

const REQUEST_ID: &str = "0193b1c8-6d3f-7a4e-b8f0-1234567890ab";

/// Far more responses than the client's bounded response channel (256) can
/// hold, so the channel is guaranteed to be full while they remain undrained.
const QUEUED_RESPONSES: usize = 600;

/// Bind a localhost listener that accepts one upgrade (with a valid
/// `dg-request-id`), and hand the accepted WebSocket to `serve`.
async fn spawn_mock_server<F, Fut>(serve: F) -> u16
where
    F: FnOnce(tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        // The callback signature (and its large `ErrorResponse` Err variant)
        // is fixed by tungstenite's accept_hdr API.
        #[allow(clippy::result_large_err)]
        let callback = |_request: &Request, mut response: Response| {
            response
                .headers_mut()
                .insert("dg-request-id", REQUEST_ID.parse().unwrap());
            Ok(response)
        };
        let ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .expect("upgrade");
        serve(ws).await;
    });

    port
}

fn client(port: u16) -> Deepgram {
    Deepgram::with_base_url_and_api_key(format!("http://127.0.0.1:{port}").as_str(), "fake-key")
        .expect("client")
}

fn turn_info_update(sequence_id: usize) -> String {
    format!(
        r#"{{"type":"TurnInfo","request_id":"{REQUEST_ID}","sequence_id":{sequence_id},"event":"Update","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"hello","words":[],"end_of_turn_confidence":0.5}}"#
    )
}

/// PR #170 review, B1: `ForceEndTurn` must reach the wire even when the
/// inbound response channel is full and nothing is draining it.
#[tokio::test]
async fn force_end_turn_reaches_wire_while_responses_are_backpressured() {
    let (force_tx, force_rx) = tokio::sync::oneshot::channel::<String>();

    let port = spawn_mock_server(move |mut ws| async move {
        // Flood the client with valid responses without reading anything,
        // so the client's bounded response channel fills long before the
        // control message is sent.
        for sequence_id in 0..QUEUED_RESPONSES {
            ws.send(Message::Text(turn_info_update(sequence_id).into()))
                .await
                .expect("server send");
        }
        // Only now start reading, waiting for the ForceEndTurn control
        // message. The queued responses stay undrained on the client.
        let mut force_tx = Some(force_tx);
        while let Some(Ok(message)) = ws.next().await {
            if let Message::Text(text) = message {
                if text.contains("ForceEndTurn") {
                    if let Some(tx) = force_tx.take() {
                        let _ = tx.send(text.to_string());
                    }
                    break;
                }
            }
        }
    })
    .await;

    let dg = client(port);
    let transcription = dg.transcription();
    let mut handle = transcription
        .flux_request()
        .handle()
        .await
        .expect("connect");

    // Earlier outbound audio, then time for the worker to fill its bounded
    // response channel. Nothing consumes responses in this test.
    handle.send_data(vec![0u8; 320]).await.expect("send audio");
    tokio::time::sleep(Duration::from_millis(200)).await;

    handle
        .force_end_turn()
        .await
        .expect("force_end_turn enqueues");

    let received = tokio::time::timeout(Duration::from_secs(5), force_rx)
        .await
        .expect("ForceEndTurn must reach the server while responses are undrained")
        .expect("server saw ForceEndTurn");
    assert_eq!(received, r#"{"type":"ForceEndTurn"}"#);
}

/// An abnormal server close frame must surface to the consumer as
/// `DeepgramError::WebsocketClose`, not as a silently ended stream.
#[tokio::test]
async fn abnormal_server_close_surfaces_as_error() {
    let port = spawn_mock_server(|mut ws| async move {
        ws.close(Some(CloseFrame {
            code: CloseCode::Error,
            reason: "internal error".into(),
        }))
        .await
        .expect("server close");
        // Drain until the connection winds down.
        while let Some(Ok(_)) = ws.next().await {}
    })
    .await;

    let dg = client(port);
    let transcription = dg.transcription();
    let mut handle = transcription
        .flux_request()
        .handle()
        .await
        .expect("connect");

    let mut saw_close_error = false;
    while let Some(response) = handle.receive().await {
        match response {
            Err(DeepgramError::WebsocketClose { code, reason }) => {
                assert_eq!(code, 1011, "server sent CloseCode::Error");
                assert_eq!(reason, "internal error");
                saw_close_error = true;
            }
            Err(err) => panic!("unexpected error: {err:?}"),
            Ok(response) => panic!("unexpected response: {response:?}"),
        }
    }
    assert!(
        saw_close_error,
        "abnormal close must be forwarded to the consumer"
    );
}

/// A normal server close ends the stream without an error.
#[tokio::test]
async fn normal_server_close_ends_stream_silently() {
    let port = spawn_mock_server(|mut ws| async move {
        ws.close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await
        .expect("server close");
        while let Some(Ok(_)) = ws.next().await {}
    })
    .await;

    let dg = client(port);
    let transcription = dg.transcription();
    let mut handle = transcription
        .flux_request()
        .handle()
        .await
        .expect("connect");

    while let Some(response) = handle.receive().await {
        match response {
            Ok(FluxResponse::Unknown { .. }) | Ok(_) => {}
            Err(err) => panic!("normal close must not produce an error: {err:?}"),
        }
    }
}
