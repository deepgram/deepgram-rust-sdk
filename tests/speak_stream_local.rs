//! CI-runnable regression tests for the `/v1/speak` streaming TTS worker's
//! transport behavior, using a localhost WebSocket server so no network
//! access or API key is required.

#![cfg(feature = "speak")]

use std::time::Duration;

use deepgram::{
    speak::{options::Encoding, SpeakResponse, SpeakStreamHandle},
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
const QUEUED_AUDIO_FRAMES: usize = 600;

/// The worker's bounded response channel, as the client sizes it.
const RESPONSE_CHANNEL_CAPACITY: usize = 256;

/// Far more text messages than the client's bounded outbound channel (256)
/// can hold, so `speak()` would block if the worker stopped draining it.
const SPEAK_MESSAGES: usize = 400;

/// Bind a localhost listener that accepts one upgrade (with a
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

async fn connect(port: u16) -> SpeakStreamHandle {
    let dg = Deepgram::with_base_url_and_api_key(
        format!("http://127.0.0.1:{port}").as_str(),
        "fake-key",
    )
    .expect("client");
    dg.text_to_speech()
        .speak_stream()
        .encoding(Encoding::Linear16)
        .sample_rate(24000)
        .handle()
        .await
        .expect("connect")
}

/// PR #166 review, S2: text must be sendable from one task while audio is
/// consumed in another, with more queued in each direction than either
/// bounded channel holds, without deadlocking.
#[tokio::test]
async fn sends_while_receiving_concurrently_without_deadlock() {
    let port = spawn_mock_server(|mut ws| async move {
        // Flood the client with audio before reading anything, so its
        // bounded response channel is full while text is still being sent.
        for _ in 0..QUEUED_AUDIO_FRAMES {
            ws.send(Message::Binary(vec![0u8; 160].into()))
                .await
                .expect("server send");
        }
        // Now read everything the client sends until its Close, counting
        // the Speak messages, then answer with a Flushed event and close.
        let mut speaks = 0usize;
        let mut flushes = 0usize;
        while let Some(Ok(message)) = ws.next().await {
            if let Message::Text(text) = message {
                if text.contains(r#""type":"Speak""#) {
                    speaks += 1;
                } else if text.contains(r#""type":"Flush""#) {
                    flushes += 1;
                } else if text.contains(r#""type":"Close""#) {
                    break;
                }
            }
        }
        assert_eq!(speaks, SPEAK_MESSAGES, "server must see every Speak");
        assert_eq!(flushes, 1, "server must see the Flush");
        ws.send(Message::Text(
            r#"{"type":"Flushed","sequence_id":0}"#.into(),
        ))
        .await
        .expect("server send flushed");
        ws.close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await
        .expect("server close");
        while ws.next().await.is_some() {}
    })
    .await;

    let handle = connect(port).await;
    assert_eq!(handle.request_id().to_string(), REQUEST_ID);
    let (sender, mut events) = handle.split();

    // Producer task: more text than the outbound channel holds, then flush
    // and close. It must make progress while the consumer drains audio.
    let producer = tokio::spawn(async move {
        for i in 0..SPEAK_MESSAGES {
            sender.speak(format!("token {i} ")).await?;
        }
        sender.flush().await?;
        sender.close().await
    });

    // Consumer: drain everything until the stream ends.
    let consumer = async {
        let mut audio_frames = 0usize;
        let mut flushed = 0usize;
        while let Some(event) = events.next().await {
            match event.expect("no transport errors") {
                SpeakResponse::Audio(chunk) => {
                    assert_eq!(chunk.len(), 160);
                    audio_frames += 1;
                }
                SpeakResponse::Flushed { .. } => flushed += 1,
                other => panic!("unexpected event {other:?}"),
            }
        }
        (audio_frames, flushed)
    };

    let (audio_frames, flushed) = tokio::time::timeout(Duration::from_secs(15), consumer)
        .await
        .expect("sending and receiving concurrently must not deadlock");
    assert_eq!(audio_frames, QUEUED_AUDIO_FRAMES);
    assert_eq!(flushed, 1);

    tokio::time::timeout(Duration::from_secs(5), producer)
        .await
        .expect("producer must finish")
        .expect("producer task panicked")
        .expect("sends succeed");
}

/// PR #166 review, N3: dropping the handle without calling `close()` must
/// still end the session cleanly — the worker sends `Close` and then a
/// WebSocket Close frame, rather than silently tearing down the TCP stream.
#[tokio::test]
async fn dropping_the_handle_sends_close_and_a_websocket_close_frame() {
    let (report_tx, report_rx) = tokio::sync::oneshot::channel::<(bool, bool)>();

    let port = spawn_mock_server(|mut ws| async move {
        let mut saw_close_text = false;
        let mut saw_close_frame = false;
        while let Some(Ok(message)) = ws.next().await {
            match message {
                Message::Text(text) if text.contains(r#""type":"Close""#) => {
                    saw_close_text = true;
                }
                Message::Close(_) => {
                    saw_close_frame = true;
                    break;
                }
                _ => {}
            }
        }
        let _ = report_tx.send((saw_close_text, saw_close_frame));
    })
    .await;

    let handle = connect(port).await;
    handle.speak("Hello").await.expect("speak");
    drop(handle);

    let (saw_close_text, saw_close_frame) = tokio::time::timeout(Duration::from_secs(5), report_rx)
        .await
        .expect("the server must see the connection close promptly")
        .expect("server task reports");
    assert!(saw_close_text, "worker must send the Close text on drop");
    assert!(
        saw_close_frame,
        "worker must complete the WebSocket closing handshake on drop"
    );
}

/// After the server closes the connection, later sends fail instead of being
/// silently discarded, and the closing handshake completes while the handle
/// is still alive.
#[tokio::test]
async fn peer_close_ends_stream_and_fails_later_sends() {
    let (done_tx, done_rx) = tokio::sync::oneshot::channel::<bool>();

    let port = spawn_mock_server(|mut ws| async move {
        ws.close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "".into(),
        }))
        .await
        .expect("server close");
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

    let clean = tokio::time::timeout(Duration::from_secs(2), done_rx)
        .await
        .expect("close handshake must complete while the handle is alive")
        .expect("server task reports");
    assert!(clean, "the close acknowledgement must reach the server");

    assert!(
        handle.speak("too late").await.is_err(),
        "speak() after peer close must return an error"
    );
}

/// PR #166 review (matching the Flux speech-to-text and Flux text-to-speech
/// sockets' `terminal_read_error_ends_worker_after_single_error`): the first
/// terminal transport error must end the worker — exactly one error is
/// forwarded, then the stream ends, and later sends fail instead of being
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

/// PR #166 review: a write failure while the response channel is *full*
/// must not stall the worker. Forwarding the terminal error with a blocking
/// send parks the worker on the full response channel, so it never ends the
/// session: the caller keeps handing text to a worker that will never write
/// it again and is never told the transport died. The error is forwarded
/// without waiting for room instead — the end of the stream, and the failing
/// send, are the signals.
#[tokio::test]
async fn write_error_with_undrained_events_does_not_stall_the_worker() {
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

    let handle = connect(port).await;

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
         error instead of succeeding forever"
    );
}
