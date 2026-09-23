//! CI-runnable regression tests for the streaming (`/v1/listen`) worker's
//! transport behavior, using a localhost WebSocket server so no network
//! access or API key is required. The Flux speech-to-text equivalents live in
//! `tests/flux_backpressure_local.rs`.

#![cfg(feature = "listen")]

use std::time::Duration;

use deepgram::Deepgram;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::Message;

const REQUEST_ID: &str = "0193b1c8-6d3f-7a4e-b8f0-1234567890ab";

/// The worker's bounded response channel, as the client sizes it. A `futures`
/// mpsc channel holds `buffer + num_senders`, so the worker's own sender can
/// queue one more than the buffer before it must wait for room.
const RESPONSE_CHANNEL_CAPACITY: usize = 256;
const QUEUEABLE_RESPONSES: usize = RESPONSE_CHANNEL_CAPACITY + 1;

/// Comfortably more responses than the channel can queue, so it is provably
/// full while they remain undrained.
const QUEUED_RESPONSES: usize = RESPONSE_CHANNEL_CAPACITY + 50;

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

fn speech_started(timestamp: usize) -> String {
    format!(r#"{{"type":"SpeechStarted","channel":[0],"timestamp":{timestamp}.0}}"#)
}

/// Audio must keep reaching the wire while the consumer is behind on
/// responses, on a socket that is perfectly healthy.
///
/// The worker forwarded each inbound response with a blocking send, so a full
/// response channel parked it mid-forward: it stopped writing audio, the
/// caller filled the bounded command channel, and `send_data` parked too —
/// with no transport failure anywhere. Reserving response-channel capacity
/// before reading the socket pauses inbound reads instead (backpressure
/// propagates to the socket), so the outbound direction keeps running.
#[tokio::test]
async fn audio_reaches_the_wire_while_responses_are_undrained() {
    const AUDIO_FRAMES: usize = 200;

    let (filled_tx, filled_rx) = tokio::sync::oneshot::channel::<()>();
    let (audio_tx, audio_rx) = tokio::sync::oneshot::channel::<usize>();

    let port = spawn_mock_server(move |mut ws| async move {
        for timestamp in 0..QUEUED_RESPONSES {
            ws.send(Message::Text(speech_started(timestamp).into()))
                .await
                .expect("server send");
        }
        let _ = filled_tx.send(());
        // Stay connected and keep reading: the only backpressure in play is
        // the client's own undrained response channel.
        let mut audio_frames = 0;
        let mut audio_tx = Some(audio_tx);
        while let Some(Ok(message)) = ws.next().await {
            if let Message::Binary(_) = message {
                audio_frames += 1;
                if audio_frames == AUDIO_FRAMES {
                    if let Some(tx) = audio_tx.take() {
                        let _ = tx.send(audio_frames);
                    }
                }
            }
        }
    })
    .await;

    let dg = client(port);
    let transcription = dg.transcription();
    let mut handle = transcription
        .stream_request()
        .handle()
        .await
        .expect("connect");

    // Nothing consumes responses yet, so the worker fills its bounded
    // response channel and leaves the rest of the flood in the socket.
    filled_rx.await.expect("server sent the flood");
    tokio::time::sleep(Duration::from_millis(500)).await;

    for _ in 0..AUDIO_FRAMES {
        handle.send_data(vec![0u8; 320]).await.expect("send audio");
    }

    let audio_frames = tokio::time::timeout(Duration::from_secs(15), audio_rx)
        .await
        .expect("audio must reach the server while responses are undrained")
        .expect("server counted the audio");
    assert_eq!(audio_frames, AUDIO_FRAMES);

    // The response channel really was saturated throughout: everything it
    // can queue is already waiting, with no further help from the server.
    for queued in 0..QUEUEABLE_RESPONSES {
        let response = tokio::time::timeout(Duration::from_secs(5), handle.receive())
            .await
            .unwrap_or_else(|_| {
                panic!("response {queued} of {QUEUEABLE_RESPONSES} was already queued")
            });
        assert!(
            matches!(response, Some(Ok(_))),
            "the queued responses must be the server's, not a transport error"
        );
    }
}

/// A write failure while the response channel is *full* must both end the
/// session promptly and still deliver the terminal error.
///
/// Two failure modes meet here. Forwarding the error with a blocking send
/// parks the worker on the full response channel, so it never ends the
/// session: the caller keeps handing audio to a worker that will never write
/// it again, fills the bounded command channel, and parks too — a deadlock in
/// which the error never arrives. Forwarding it with a plain `try_send` on
/// the worker's own sender cures the deadlock but drops the error, because
/// that sender is exactly the one parked on the full channel — the consumer
/// then sees the stream end with no explanation. A sender dedicated to the
/// error carries its own guaranteed slot, so the forward neither blocks nor
/// drops.
#[tokio::test]
async fn write_error_with_undrained_responses_is_delivered_without_stalling() {
    let (filled_tx, filled_rx) = tokio::sync::oneshot::channel::<()>();
    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();

    let port = spawn_mock_server(move |mut ws| async move {
        // Just over what the response channel can queue, so it fills while
        // everything still fits in the socket buffers.
        for timestamp in 0..QUEUED_RESPONSES {
            ws.send(Message::Text(speech_started(timestamp).into()))
                .await
                .expect("server send");
        }
        let _ = filled_tx.send(());
        // Vanish without a closing handshake, but only once the client has
        // had the chance to fill its response channel, so the write below
        // fails while there is provably no room to forward the error into.
        let _ = abort_rx.await;
        drop(ws);
    })
    .await;

    let dg = client(port);
    let transcription = dg.transcription();
    let mut handle = transcription
        .stream_request()
        .handle()
        .await
        .expect("connect");

    // Never receive anything yet: the worker forwards the flood until its
    // bounded response channel is full and then stops reading the socket.
    filled_rx.await.expect("server sent the flood");
    tokio::time::sleep(Duration::from_millis(500)).await;
    let _ = abort_tx.send(());

    // Large payloads, so the socket's send buffer is exhausted quickly and
    // the worker's write actually reaches the dead peer.
    let audio = vec![0u8; 8192];
    let sending = async {
        for _ in 0..2_000 {
            if handle.send_data(audio.clone()).await.is_err() {
                return true;
            }
        }
        false
    };

    let failed = tokio::time::timeout(Duration::from_secs(15), sending)
        .await
        .expect("a write error with undrained responses must not deadlock `send_data`");
    assert!(
        failed,
        "once the transport is broken the worker must end the session, so `send_data` reports an \
         error instead of hanging or succeeding forever"
    );

    // Now drain: the terminal error must be in the stream, exactly once and
    // last, and the stream must end.
    let mut responses = 0usize;
    let mut errors = 0usize;
    let mut responses_after_error = 0usize;
    while let Some(response) = tokio::time::timeout(Duration::from_secs(5), handle.receive())
        .await
        .expect("the stream must end promptly after a terminal error")
    {
        match response {
            Ok(_) => {
                responses += 1;
                if errors > 0 {
                    responses_after_error += 1;
                }
            }
            Err(_) => errors += 1,
        }
    }
    assert_eq!(
        responses, QUEUEABLE_RESPONSES,
        "the response channel must have been full at the moment the write failed, otherwise this \
         test would pass without exercising the bug"
    );
    assert_eq!(
        errors, 1,
        "the terminal write error must reach the consumer exactly once, even though the response \
         channel was full when the write failed"
    );
    assert_eq!(
        responses_after_error, 0,
        "the terminal error must be the last item in the stream"
    );

    // The session stays ended: a later send fails rather than silently
    // succeeding.
    assert!(
        handle.send_data(vec![0u8; 4]).await.is_err(),
        "send_data after a terminal error must return an error"
    );
}

/// The ordinary session still works end to end: audio reaches the server,
/// responses reach the consumer, `close_stream` reaches the wire, and the
/// stream ends once the server closes. A guard for the scheduling rework
/// above, which touches every branch of the worker's loop.
#[tokio::test]
async fn normal_session_completes() {
    let port = spawn_mock_server(|mut ws| async move {
        while let Some(Ok(message)) = ws.next().await {
            match message {
                Message::Binary(_) => {
                    ws.send(Message::Text(speech_started(1).into()))
                        .await
                        .expect("server send");
                }
                Message::Text(text) if text.contains("CloseStream") => {
                    ws.send(Message::Text(speech_started(2).into()))
                        .await
                        .expect("server send");
                    ws.close(None).await.expect("server close");
                }
                _ => {}
            }
        }
    })
    .await;

    let dg = client(port);
    let transcription = dg.transcription();
    let mut handle = transcription
        .stream_request()
        .handle()
        .await
        .expect("connect");

    handle.send_data(vec![0u8; 320]).await.expect("send audio");
    let first = tokio::time::timeout(Duration::from_secs(5), handle.receive())
        .await
        .expect("a response must arrive");
    assert!(matches!(first, Some(Ok(_))), "got {first:?}");

    handle.close_stream().await.expect("close_stream");

    let mut responses = 0usize;
    while let Some(response) = tokio::time::timeout(Duration::from_secs(5), handle.receive())
        .await
        .expect("the stream must end after the server closes")
    {
        assert!(response.is_ok(), "unexpected error: {response:?}");
        responses += 1;
    }
    assert_eq!(
        responses, 1,
        "the response sent after CloseStream must still be delivered"
    );
}
