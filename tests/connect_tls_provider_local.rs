//! CI-runnable regression test for TLS connector equivalence between the
//! stock (untimed) and phase-timed connect paths (PR #169 review, B2).
//!
//! With `connect-diagnostics` enabled, both paths must be handed the same
//! explicit rustls connector — webpki trust roots, crate-default provider —
//! regardless of which TLS backends downstream feature unification enables
//! on `tokio-tungstenite`. A localhost TLS server presents a self-signed
//! certificate; both paths must reject it with rustls's webpki verification
//! error (`UnknownIssuer`). A backend selected by feature auto-detection
//! (e.g. native-tls) would surface a different, platform-specific error.

#![cfg(feature = "connect-diagnostics")]

use std::sync::Arc;

use deepgram::{
    common::options::Options,
    diagnostics::{ConnectOutcome, ConnectPhase, ConnectRecord},
    Deepgram,
};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

/// Bind a localhost TLS listener with a fresh self-signed certificate for
/// `localhost`, accepting handshakes until dropped. Client-side certificate
/// rejection aborts each handshake; the server just moves on.
async fn spawn_self_signed_tls_server() -> u16 {
    let cert =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).expect("generate cert");
    let cert_der = cert.cert.der().clone();
    let key_der = rustls_key(cert.key_pair.serialize_der());

    let server_config = tokio_rustls::rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert_der], key_der)
        .expect("server config");
    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("local addr").port();

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let acceptor = acceptor.clone();
            tokio::spawn(async move {
                // The client rejects our self-signed certificate, so the
                // handshake is expected to fail; nothing to serve beyond it.
                let _ = acceptor.accept(stream).await;
            });
        }
    });

    port
}

fn rustls_key(pkcs8: Vec<u8>) -> tokio_rustls::rustls::pki_types::PrivateKeyDer<'static> {
    tokio_rustls::rustls::pki_types::PrivateKeyDer::Pkcs8(pkcs8.into())
}

fn client(port: u16) -> Deepgram {
    // https → wss: the connect path performs a real TLS handshake.
    Deepgram::with_base_url_and_api_key(format!("https://localhost:{port}").as_str(), "fake-key")
        .expect("client")
}

/// The rustls webpki verifier rejects an unknown issuer with this exact
/// variant name; native-tls backends produce platform-specific messages
/// that never contain it.
fn is_rustls_unknown_issuer(err: &impl std::fmt::Debug) -> bool {
    format!("{err:?}").contains("UnknownIssuer")
}

#[tokio::test]
async fn stock_and_timed_paths_use_the_same_rustls_connector() {
    let port = spawn_self_signed_tls_server().await;

    // Stock path: no diagnostics sink configured.
    let dg = client(port);
    let transcription = dg.transcription();
    let stock_err = transcription
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect_err("self-signed certificate must be rejected");
    assert!(
        is_rustls_unknown_issuer(&stock_err),
        "stock path must fail with rustls's UnknownIssuer, got: {stock_err:?}"
    );

    // Phase-timed path: diagnostics sink configured.
    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let dg = client(port);
    let transcription = dg.transcription();
    let timed_err = transcription
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await
        .expect_err("self-signed certificate must be rejected");
    assert!(
        is_rustls_unknown_issuer(&timed_err),
        "timed path must fail with rustls's UnknownIssuer, got: {timed_err:?}"
    );

    let record = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(record.outcome, ConnectOutcome::Failed);
    assert_eq!(record.last_phase, ConnectPhase::TlsHandshake);
}
