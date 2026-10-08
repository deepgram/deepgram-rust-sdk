//! CI-runnable regression tests for the Flux TTS worker's transport
//! behavior, using a localhost WebSocket server so no network access or
//! API key is required.

#![cfg(feature = "speak")]

use std::time::Duration;

use deepgram::{
    speak::flux::{
        options::{Model, Options},
        response::FluxSpeakResponse,
    },
    Deepgram,
};
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

/// The worker's bounded response channel, as the client sizes it.
const RESPONSE_CHANNEL_CAPACITY: usize = 256;

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

async fn connect(port: u16) -> deepgram::speak::flux::websocket::FluxSpeakHandle {
    let dg = Deepgram::with_base_url_and_api_key(
        format!("http://127.0.0.1:{port}").as_str(),
        "fake-key",
    )
    .expect("client");
    let options = Options::builder(Model::FluxHaleyEn).build();
    let speak = dg.text_to_speech();
    speak.flux_request(options).handle().await.expect("connect")
}

/// PR #171 review, B1: `Interrupt` must reach the wire even when the
/// inbound response channel is full of audio and nothing is draining it.
#[tokio::test]
async fn interrupt_reaches_wire_while_audio_responses_are_backpressured() {
    let (interrupt_tx, interrupt_rx) = tokio::sync::oneshot::channel::<String>();

    let port = spawn_mock_server(move |mut ws| async move {
        // Flood the client with audio frames without reading anything, so
        // the client's bounded response channel fills long before the
        // control message is sent.
        for _ in 0..QUEUED_RESPONSES {
            ws.send(Message::Binary(vec![0u8; 160].into()))
                .await
                .expect("server send");
        }
        // Only now start reading, waiting for the Interrupt control
        // message. The queued audio stays undrained on the client.
        let mut interrupt_tx = Some(interrupt_tx);
        while let Some(Ok(message)) = ws.next().await {
            if let Message::Text(text) = message {
                if text.contains("Interrupt") {
                    if let Some(tx) = interrupt_tx.take() {
                        let _ = tx.send(text.to_string());
                    }
                    break;
                }
            }
        }
    })
    .await;

    let mut handle = connect(port).await;

    // Earlier outbound text, then time for the worker to fill its bounded
    // response channel. Nothing consumes responses in this test.
    handle.speak("Hello there!").await.expect("send text");
    tokio::time::sleep(Duration::from_millis(200)).await;

    handle
        .interrupt(Some(1_500))
        .await
        .expect("interrupt enqueues");

    let received = tokio::time::timeout(Duration::from_secs(5), interrupt_rx)
        .await
        .expect("Interrupt must reach the server while responses are undrained")
        .expect("server saw Interrupt");
    assert_eq!(
        received,
        r#"{"type":"Interrupt","playback_offset":{"type":"time_ms","value":1500}}"#
    );
}

/// PR #171 review, B4: after the server closes the connection, the client
/// must flush its close acknowledgement promptly — completing the closing
/// handshake while the handle is still alive — and `speak()` must return an
/// error instead of silently discarding the message.
#[tokio::test]
async fn peer_close_fails_later_sends_and_completes_handshake() {
    let (done_tx, done_rx) = tokio::sync::oneshot::channel::<bool>();

    let port = spawn_mock_server(|mut ws| async move {
        ws.close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await
        .expect("server close");
        // Drain until the connection ends. A clean end means the client's
        // close acknowledgement arrived and the handshake completed; an
        // `Err` here means the socket was torn down without one.
        let mut clean = true;
        while let Some(message) = ws.next().await {
            if message.is_err() {
                clean = false;
            }
        }
        let _ = done_tx.send(clean);
    })
    .await;

    let mut handle = connect(port).await;

    while handle.receive().await.is_some() {}

    // The handle is deliberately still alive here: the closing handshake
    // must not wait for it to be dropped.
    let clean = tokio::time::timeout(Duration::from_secs(2), done_rx)
        .await
        .expect("close handshake must complete while the handle is alive")
        .expect("server task reports");
    assert!(
        clean,
        "the close acknowledgement must be flushed to the server"
    );

    // The worker has ended: sends after peer close must fail rather than
    // be silently discarded.
    assert!(
        handle.speak("too late").await.is_err(),
        "speak() after peer close must return an error"
    );
}

/// PR #171 review, B4 (worker-lifetime half, matching STT S5): the first
/// terminal transport error must end the worker — exactly one error is
/// forwarded (no duplicates), and later commands fail instead of being
/// accepted by a dead session.
#[tokio::test]
async fn terminal_read_error_ends_worker_after_single_error() {
    let port = spawn_mock_server(|ws| async move {
        // Abrupt teardown without a closing handshake produces a terminal
        // read error on the client.
        drop(ws);
    })
    .await;

    let mut handle = connect(port).await;

    let mut errors = 0usize;
    while let Some(response) = tokio::time::timeout(Duration::from_secs(5), handle.receive())
        .await
        .expect("stream must end promptly after a terminal error")
    {
        assert!(
            response.is_err(),
            "only the terminal error is expected, got: {response:?}"
        );
        errors += 1;
    }
    assert_eq!(
        errors, 1,
        "exactly one terminal error must be forwarded, without duplicates"
    );

    assert!(
        handle.speak("too late").await.is_err(),
        "speak() after a terminal error must return an error"
    );
}

/// PR #166 review, applied to the released Flux text-to-speech socket: a
/// write failure while the response channel is *full* must not deadlock.
/// Forwarding the terminal error with a blocking send parks the worker on
/// the full response channel while the caller, which is not draining events,
/// parks on the full command channel — neither side moves and the error
/// never surfaces.
#[tokio::test]
async fn write_error_with_undrained_events_does_not_deadlock() {
    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();

    let port = spawn_mock_server(|mut ws| async move {
        // Just over the response channel's capacity, so the channel fills
        // while everything still fits in the socket buffers.
        for _ in 0..(RESPONSE_CHANNEL_CAPACITY + 50) {
            ws.send(Message::Binary(vec![0u8; 160].into()))
                .await
                .expect("server send");
        }
        // Vanish without a closing handshake, but only once the client has
        // filled its response channel, so the write below fails while there
        // is provably no room to forward the error into.
        let _ = abort_rx.await;
        drop(ws);
    })
    .await;

    let mut handle = connect(port).await;

    // Never receive anything: let the worker forward the flood until its
    // bounded response channel is full, then break the connection.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = abort_tx.send(());

    // Large payloads, so the socket's send buffer is exhausted quickly and
    // the worker's write actually reaches the dead peer.
    let text = "token ".repeat(1024);
    let sending = async {
        for _ in 0..2_000 {
            if handle.speak(text.clone()).await.is_err() {
                return true;
            }
        }
        false
    };

    let failed = tokio::time::timeout(Duration::from_secs(15), sending)
        .await
        .expect("a write error with undrained events must not deadlock `speak`");
    assert!(
        failed,
        "once the transport is broken the worker must end the session, so `speak` reports an \
         error instead of hanging or succeeding forever"
    );
}

/// PR #166 review, S3 (the delivery half of the test above, mirroring
/// `speak_stream_local::write_error_reaches_a_slow_split_consumer`): the
/// terminal write error must not merely unblock `speak()`, it must actually
/// be *delivered*. Forwarding it on the shared response channel drops it
/// when that channel is full, so a consumer that comes back to drain reads
/// the queued audio and then nothing at all — the failure looks like a
/// clean end of stream. `FluxSpeakHandle` cannot be split, so the channel
/// is filled and the write is broken before anything is received, and the
/// drain happens once `speak()` has reported the error.
#[tokio::test]
async fn write_error_reaches_a_consumer_that_drains_after_the_failure() {
    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();

    let port = spawn_mock_server(|mut ws| async move {
        // More audio than the response channel holds, so it is provably
        // full — and has no room for a terminal error — when the write
        // below fails. The drain at the end of the test asserts this.
        for _ in 0..(RESPONSE_CHANNEL_CAPACITY + 50) {
            ws.send(Message::Binary(vec![0u8; 160].into()))
                .await
                .expect("server send");
        }
        // Vanish without a closing handshake, but only once the client's
        // response channel is full.
        let _ = abort_rx.await;
        drop(ws);
    })
    .await;

    let mut handle = connect(port).await;

    // Receive nothing yet: let the worker fill its bounded response channel
    // with the flood, then break the connection.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = abort_tx.send(());

    // Large payloads, so the socket's send buffer is exhausted quickly and
    // the worker's write actually reaches the dead peer.
    let text = "token ".repeat(1024);
    let sending = async {
        for _ in 0..2_000 {
            if handle.speak(text.clone()).await.is_err() {
                return true;
            }
        }
        false
    };
    let failed = tokio::time::timeout(Duration::from_secs(15), sending)
        .await
        .expect("a write error with undrained events must not deadlock `speak`");
    assert!(failed, "the broken transport must fail a later `speak`");

    // Now drain everything the worker queued. The audio comes first, then
    // the terminal error, and then the stream ends.
    let mut audio = 0usize;
    let mut terminal_errors = 0usize;
    let mut error_was_last = false;
    while let Some(response) = tokio::time::timeout(Duration::from_secs(10), handle.receive())
        .await
        .expect("the event stream must end after the terminal error")
    {
        match response {
            Ok(FluxSpeakResponse::Audio(_)) => {
                audio += 1;
                error_was_last = false;
            }
            Ok(other) => panic!("unexpected event {other:?}"),
            Err(_) => {
                terminal_errors += 1;
                error_was_last = true;
            }
        }
    }

    assert!(
        audio >= RESPONSE_CHANNEL_CAPACITY,
        "the response channel must have been full at the write failure, but only {audio} audio \
         events were queued"
    );
    assert_eq!(
        terminal_errors, 1,
        "the terminal write error must be delivered to a consumer that drains after the failure, \
         exactly once (drained {audio} audio events)"
    );
    assert!(
        error_was_last,
        "the terminal error must arrive after the queued audio, as the last item in the stream"
    );
}
