//! CI-runnable end-to-end proof that `WebsocketBuilder::trust_native_roots`
//! actually changes which TLS certificates the client accepts, rather than
//! just threading a flag through the builder.
//!
//! A localhost TLS server presents a self-signed certificate — nothing in
//! `deepgram`'s default webpki root bundle would ever trust it, the same
//! shape as trust anchored in the OS store instead of the public CA bundle
//! this crate ships by default: a corporate network's TLS-inspecting proxy,
//! an internal CA, or a self-hosted deployment, all terminate TLS with a
//! root CA the OS trusts but the public bundle does not.
//!
//! Rather than installing a CA into this machine's real keychain/cert store,
//! these tests point the `SSL_CERT_FILE` env var at the test CA.
//! `rustls-native-certs` reads that env var in place of the platform store
//! on every OS, so this exercises the exact code path a real OS-trusted CA
//! would take, deterministically and without touching the host's trust
//! store. See `connect_tls_provider_local.rs` for the server/cert helpers
//! this borrows the shape of.
//!
//! The SDK caches the native portion of the trust store (see
//! `cached_native_root_cert_store` in `src/diagnostics.rs`) for a multi-
//! minute TTL, refreshed in the background once stale -- reading it is
//! genuinely blocking OS I/O, so it isn't re-read on every connect. This
//! test binary runs in well under that TTL, so in practice it behaves like
//! a single process-wide read: only the *first* `SSL_CERT_FILE` value this
//! binary's process ever sees while `trust_native_roots` is in play takes
//! effect. Every test below that needs an OS-trusted cert therefore shares
//! one fixture (one cert, one server) rather than each pointing
//! `SSL_CERT_FILE` at its own -- a second, different value would silently
//! be ignored for the duration of this test run, exactly like a real
//! long-lived process serving its cached trust store between refreshes.

#![cfg(feature = "connect-diagnostics")]

use std::{io::Write, sync::Arc};

use deepgram::{common::options::Options, diagnostics::ConnectRecord, Deepgram};
use tokio::{net::TcpListener, sync::OnceCell};
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::tungstenite;

const FAKE_REQUEST_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

/// A self-signed certificate's DER and key material, cloneable so several
/// independently-spawned servers can present the identical certificate.
struct SharedCert {
    cert_der: Vec<u8>,
    key_der: Vec<u8>,
}

/// Lazily generates one certificate and points `SSL_CERT_FILE` at it, exactly
/// once for this binary's process -- matching the SDK's own native-roots
/// cache, which (within this short test run, well under its multi-minute
/// TTL) likewise only ever reads that env var once. This caches
/// certificate *data*, not a server: `tokio::spawn` binds a task's lifetime
/// to whichever runtime called it, and `#[tokio::test]` gives each test
/// function its own runtime, so a server spawned inside one test's runtime
/// would be torn down the moment that test function returns -- breaking any
/// other test still trying to connect to it. Each test below spawns its own
/// server (own runtime, own lifetime) but they all present this same
/// certificate, so they all validate against the one CA the SDK cached.
static SHARED_CERT: OnceCell<SharedCert> = OnceCell::const_new();

async fn shared_cert() -> &'static SharedCert {
    SHARED_CERT
        .get_or_init(|| async {
            let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
                .expect("generate cert");
            let cert_pem = cert.cert.pem();
            let cert_der = cert.cert.der().to_vec();
            let key_der = cert.key_pair.serialize_der();

            // Simulate the OS trust store containing an extra root CA -- the
            // way a corporate network's TLS-inspecting proxy or an internal
            // CA gets installed on a managed machine. Leaked deliberately:
            // the SDK's cache only reads this path once, early in the test
            // run, so it must outlive that read, and a leaked temp file in a
            // short-lived test binary is harmless.
            let mut cert_file = tempfile::NamedTempFile::new().unwrap();
            cert_file.write_all(cert_pem.as_bytes()).unwrap();
            let cert_path = cert_file.into_temp_path();
            let cert_path = Box::leak(Box::new(cert_path));
            // SAFETY: this runs inside `OnceCell::get_or_init`, so it
            // executes at most once for this process, before any test
            // observes `SHARED_CERT` as initialized.
            unsafe {
                std::env::set_var("SSL_CERT_FILE", &*cert_path);
            }

            SharedCert { cert_der, key_der }
        })
        .await
}

/// Bind a localhost TLS listener presenting the given certificate, accepting
/// handshakes until dropped.
async fn spawn_tls_server(cert_der: Vec<u8>, key_der: Vec<u8>) -> u16 {
    let cert_der = tokio_rustls::rustls::pki_types::CertificateDer::from(cert_der);
    let key_der = tokio_rustls::rustls::pki_types::PrivateKeyDer::Pkcs8(key_der.into());

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
                // A client that doesn't trust our self-signed cert aborts the
                // TLS handshake here; that's the expected outcome for one of
                // the tests below, not a bug in the test server.
                let Ok(tls_stream) = acceptor.accept(stream).await else {
                    return;
                };

                // A client that does trust the cert proceeds to a real
                // WebSocket upgrade, which must actually be answered.
                #[allow(clippy::result_large_err)]
                let callback =
                    |_req: &tungstenite::handshake::server::Request,
                     mut resp: tungstenite::handshake::server::Response| {
                        resp.headers_mut()
                            .insert("dg-request-id", FAKE_REQUEST_ID.parse().unwrap());
                        Ok(resp)
                    };
                let Ok(mut ws) = tokio_tungstenite::accept_hdr_async(tls_stream, callback).await
                else {
                    return;
                };
                let _ = futures::StreamExt::next(&mut ws).await;
                let _ = futures::SinkExt::close(&mut ws).await;
            });
        }
    });

    port
}

/// A fresh, ad-hoc self-signed certificate unrelated to [`SHARED_CERT`], for
/// the one test that expects the connection to be rejected.
async fn spawn_untrusted_tls_server() -> u16 {
    let cert =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).expect("generate cert");
    spawn_tls_server(cert.cert.der().to_vec(), cert.key_pair.serialize_der()).await
}

fn client(port: u16) -> Deepgram {
    Deepgram::with_base_url_and_api_key(format!("https://localhost:{port}").as_str(), "fake-key")
        .expect("client")
}

#[tokio::test]
async fn stock_path_rejects_untrusted_cert_without_opt_in() {
    // This test doesn't opt into trust_native_roots, so it's unaffected by
    // that cache either way, and its own untrusted cert must never be
    // trusted regardless of what runs elsewhere in this binary.
    let port = spawn_untrusted_tls_server().await;

    let result = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await;

    assert!(
        result.is_err(),
        "expected the default (webpki-only) trust store to reject an untrusted \
         self-signed certificate"
    );
}

#[tokio::test]
async fn trust_native_roots_accepts_a_cert_trusted_via_the_os_store() {
    let cert = shared_cert().await;
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der.clone()).await;

    let result = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .trust_native_roots()
        .handle()
        .await;

    result.expect("connection should succeed once the cert is trusted via native roots");
}

#[tokio::test]
async fn trust_native_roots_also_applies_to_the_phase_timed_diagnostics_path() {
    let cert = shared_cert().await;
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der.clone()).await;

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let result = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .trust_native_roots()
        .diagnostics(diag_tx)
        .handle()
        .await;

    result.expect("phase-timed connect should also succeed once the cert is trusted");
    let record = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(
        record.outcome,
        deepgram::diagnostics::ConnectOutcome::Completed
    );
}
