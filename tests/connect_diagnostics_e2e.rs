//! End-to-end equivalence check between the stock connect path and the
//! phase-timed diagnostics connect path.
//!
//! Requires network access and a `DEEPGRAM_API_KEY`; run explicitly with:
//! `cargo test --features connect-diagnostics --test connect_diagnostics_e2e -- --ignored`

#![cfg(feature = "connect-diagnostics")]

use deepgram::{
    common::options::Options,
    diagnostics::{ConnectOutcome, ConnectPhase, ConnectRecord},
    Deepgram,
};

fn client() -> Deepgram {
    let key = std::env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY must be set");
    Deepgram::new(&key).expect("client")
}

/// Both connect paths must succeed against the same endpoint and surface a
/// request ID, and the diagnostic path must produce a completed record with
/// every phase timed and socket addresses captured.
#[tokio::test]
#[ignore = "requires network access and DEEPGRAM_API_KEY"]
async fn stock_and_diagnostic_connects_are_equivalent() {
    let dg = client();

    let transcription = dg.transcription();

    // Stock path.
    let stock = transcription
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect("stock connect should succeed");
    let stock_request_id = stock.request_id();

    // Diagnostic path.
    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let diagnostic = transcription
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await
        .expect("diagnostic connect should succeed");
    let diagnostic_request_id = diagnostic.request_id();

    // Both produced server-issued request IDs.
    assert_ne!(stock_request_id, diagnostic_request_id);

    let record = diag_rx.try_recv().expect("one record per connect attempt");
    assert_eq!(record.outcome, ConnectOutcome::Completed);
    assert_eq!(record.last_phase, ConnectPhase::WsUpgrade);
    assert_eq!(record.request_id, Some(diagnostic_request_id.to_string()));
    assert!(record.url.starts_with("wss://api.deepgram.com/v1/listen"));
    assert!(record.local_addr.is_some());
    assert!(record.peer_addr.is_some());
    for (name, value) in [
        ("dns_ms", record.dns_ms),
        ("tcp_connect_ms", record.tcp_connect_ms),
        ("tls_handshake_ms", record.tls_handshake_ms),
        ("ws_upgrade_ms", record.ws_upgrade_ms),
    ] {
        assert!(value.is_some(), "{name} should be present on success");
    }
    assert!(record.connect_duration_ms > 0.0);

    assert!(
        diag_rx.try_recv().is_err(),
        "exactly one record per connect attempt"
    );
}

/// A connect cancelled by a caller-side timeout still emits a record.
#[tokio::test]
#[ignore = "requires network access and DEEPGRAM_API_KEY"]
async fn cancelled_connect_still_emits_record() {
    let dg = client();

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let transcription = dg.transcription();
    let connect = transcription
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle();

    // A 1ms budget cannot complete a cross-internet connect; the future is
    // dropped mid-flight.
    let result = tokio::time::timeout(std::time::Duration::from_millis(1), connect).await;
    assert!(result.is_err(), "timeout should fire");

    let record = diag_rx.try_recv().expect("record must survive cancellation");
    assert_eq!(record.outcome, ConnectOutcome::Cancelled);
    assert!(record.request_id.is_none());
}
