//! CI-runnable tests for the phase-timed diagnostics connect path, using a
//! localhost WebSocket server so no network access or API key is required.
//!
//! These exercise the real wire path (DNS resolution, TCP connect, WebSocket
//! upgrade, header capture) over plain `ws://`; the TLS leg is covered by the
//! config-equivalence unit test in `src/diagnostics.rs` and by the ignored
//! live tests in `connect_diagnostics_e2e.rs`.

#![cfg(feature = "connect-diagnostics")]

use deepgram::{
    common::options::Options,
    diagnostics::{ConnectOutcome, ConnectPhase, ConnectRecord},
    Deepgram,
};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};

const REQUEST_ID: &str = "0193b1c8-6d3f-7a4e-b8f0-1234567890ab";

/// How the mock server answers the one upgrade request it accepts.
#[derive(Clone, Copy)]
enum Mode {
    /// 101 with a valid `dg-request-id` header.
    Accept,
    /// 101 with no `dg-request-id` header.
    AcceptWithoutRequestId,
    /// 101 with a `dg-request-id` that is not a UUID.
    AcceptWithMalformedRequestId,
    /// 400 carrying `dg-request-id` and `dg-error` headers.
    Reject,
}

/// Bind a localhost listener, serve exactly one upgrade in the given mode,
/// and return the bound port.
async fn spawn_mock_server(mode: Mode) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        // The callback signature (and its large `ErrorResponse` Err variant)
        // is fixed by tungstenite's accept_hdr API.
        #[allow(clippy::result_large_err)]
        let callback = move |_request: &Request, mut response: Response| match mode {
            Mode::Accept => {
                response
                    .headers_mut()
                    .insert("dg-request-id", REQUEST_ID.parse().unwrap());
                Ok(response)
            }
            Mode::AcceptWithoutRequestId => Ok(response),
            Mode::AcceptWithMalformedRequestId => {
                response
                    .headers_mut()
                    .insert("dg-request-id", "not-a-uuid".parse().unwrap());
                Ok(response)
            }
            Mode::Reject => {
                let error_response: ErrorResponse =
                    tokio_tungstenite::tungstenite::http::Response::builder()
                        .status(400)
                        .header("dg-request-id", REQUEST_ID)
                        .header("dg-error", "bad model")
                        .body(None)
                        .expect("error response");
                Err(error_response)
            }
        };
        // Hold the connection until the client goes away, so the client-side
        // worker isn't torn down mid-test.
        if let Ok(mut ws) = tokio_tungstenite::accept_hdr_async(stream, callback).await {
            use futures::StreamExt;
            while let Some(Ok(_)) = ws.next().await {}
        }
    });

    port
}

fn client(port: u16) -> Deepgram {
    Deepgram::with_base_url_and_api_key(format!("http://127.0.0.1:{port}").as_str(), "fake-key")
        .expect("client")
}

fn sink() -> (
    tokio::sync::mpsc::UnboundedSender<ConnectRecord>,
    tokio::sync::mpsc::UnboundedReceiver<ConnectRecord>,
) {
    tokio::sync::mpsc::unbounded_channel()
}

#[tokio::test]
async fn completed_connect_over_plain_ws_emits_full_record() {
    let port = spawn_mock_server(Mode::Accept).await;
    let (diag_tx, mut diag_rx) = sink();

    let dg = client(port);
    let transcription = dg.transcription();
    let handle = transcription
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await
        .expect("connect should succeed");
    assert_eq!(handle.request_id().to_string(), REQUEST_ID);

    let record = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(record.outcome, ConnectOutcome::Completed);
    assert_eq!(record.last_phase, ConnectPhase::WsUpgrade);
    assert_eq!(record.request_id.as_deref(), Some(REQUEST_ID));
    assert!(record
        .url
        .starts_with(&format!("ws://127.0.0.1:{port}/v1/listen")));
    assert!(record
        .local_addr
        .as_deref()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert_eq!(
        record.peer_addr.as_deref(),
        Some(format!("127.0.0.1:{port}").as_str())
    );
    assert!(record.dns_ms.is_some());
    assert!(record.tcp_connect_ms.is_some());
    assert!(
        record.tls_handshake_ms.is_none(),
        "plain ws has no TLS phase"
    );
    assert!(record.ws_upgrade_ms.is_some());
    assert!(record.connect_duration_ms > 0.0);
    assert!(record.error.is_none());
    assert!(record.dg_error.is_none());

    assert!(
        diag_rx.try_recv().is_err(),
        "exactly one record per connect attempt"
    );
}

#[tokio::test]
async fn missing_request_id_yields_failed_record_matching_caller_error() {
    let port = spawn_mock_server(Mode::AcceptWithoutRequestId).await;
    let (diag_tx, mut diag_rx) = sink();

    let dg = client(port);
    let transcription = dg.transcription();
    let result = transcription
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await;
    assert!(result.is_err(), "SDK rejects an upgrade with no request ID");

    let record = diag_rx.try_recv().expect("record emitted on failure");
    assert_eq!(
        record.outcome,
        ConnectOutcome::Failed,
        "outcome must match the error the caller saw"
    );
    assert!(record.request_id.is_none());
    assert!(record
        .error
        .as_deref()
        .expect("error populated")
        .contains("missing request ID"));
}

#[tokio::test]
async fn malformed_request_id_yields_failed_record_with_raw_header() {
    let port = spawn_mock_server(Mode::AcceptWithMalformedRequestId).await;
    let (diag_tx, mut diag_rx) = sink();

    let dg = client(port);
    let transcription = dg.transcription();
    let result = transcription
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await;
    assert!(result.is_err(), "SDK rejects a malformed request ID");

    let record = diag_rx.try_recv().expect("record emitted on failure");
    assert_eq!(record.outcome, ConnectOutcome::Failed);
    assert_eq!(
        record.request_id.as_deref(),
        Some("not-a-uuid"),
        "raw header kept for correlation even when unparsable"
    );
    assert!(record
        .error
        .as_deref()
        .expect("error populated")
        .contains("malformed request ID"));
}

#[tokio::test]
async fn rejected_upgrade_captures_deepgram_headers_from_the_wire() {
    let port = spawn_mock_server(Mode::Reject).await;
    let (diag_tx, mut diag_rx) = sink();

    let dg = client(port);
    let transcription = dg.transcription();
    let result = transcription
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await;
    assert!(result.is_err(), "a 400 upgrade response is an error");

    let record = diag_rx.try_recv().expect("record emitted on rejection");
    assert_eq!(record.outcome, ConnectOutcome::Failed);
    assert_eq!(record.last_phase, ConnectPhase::WsUpgrade);
    assert_eq!(record.request_id.as_deref(), Some(REQUEST_ID));
    assert_eq!(record.dg_error.as_deref(), Some("bad model"));
    assert!(record.error.is_some());
}

#[tokio::test]
async fn stock_path_without_sink_connects_and_emits_nothing() {
    let port = spawn_mock_server(Mode::Accept).await;

    let dg = client(port);
    let transcription = dg.transcription();
    let handle = transcription
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect("stock connect should succeed against the mock");
    assert_eq!(handle.request_id().to_string(), REQUEST_ID);
}
