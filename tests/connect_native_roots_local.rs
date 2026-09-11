//! CI-runnable proof that the `rustls-tls-native-roots` feature actually
//! changes which certificates the client accepts, on every WebSocket
//! surface, rather than only threading a flag through.
//!
//! A localhost TLS server presents a self-signed certificate — nothing in the
//! bundled webpki roots trusts it, the same shape as trust anchored in the OS
//! store instead of the public bundle: a corporate network's TLS-inspecting
//! proxy, an internal CA, or a self-hosted deployment.
//!
//! Rather than installing a CA into this machine's real keychain or cert
//! store, these tests point `SSL_CERT_FILE` at the test certificate.
//! `rustls-native-certs` reads that env var in place of the platform store on
//! every OS, so this exercises the exact code path a real OS-trusted CA takes,
//! deterministically and without touching the host's trust store.
//!
//! The SDK reads the OS store once per `Deepgram` client, on that client's
//! first WebSocket connect. The env var itself is process-global, so every
//! test shares one certificate fixture and one `SSL_CERT_FILE` value rather
//! than racing to set its own.

#![cfg(all(feature = "listen", feature = "rustls-tls-native-roots"))]

mod common;

use common::{client, self_signed, self_signed_named, spawn_tls_server, SelfSigned};
use deepgram::{common::options::Options, tls::TlsTrust, DeepgramError};
use tokio::sync::OnceCell;

/// One certificate for the whole test binary, written to a PEM file that
/// `SSL_CERT_FILE` points at. Certificate *data* is cached, not a server:
/// each `#[tokio::test]` has its own runtime, so each test spawns its own
/// server presenting this same certificate.
static SHARED_CERT: OnceCell<SelfSigned> = OnceCell::const_new();

async fn shared_cert() -> &'static SelfSigned {
    SHARED_CERT
        .get_or_init(|| async {
            let cert = self_signed();
            let path = std::env::temp_dir().join(format!(
                "deepgram-rust-sdk-test-ca-{}.pem",
                std::process::id()
            ));
            std::fs::write(&path, cert.cert_pem.as_bytes()).expect("write test CA");
            // Simulate the OS trust store containing an extra root CA, the
            // way a managed machine gets a corporate proxy's CA installed.
            // Runs at most once per process, inside `get_or_init`.
            std::env::set_var("SSL_CERT_FILE", &path);
            cert
        })
        .await
}

#[tokio::test]
async fn native_roots_trust_a_certificate_from_the_os_store() {
    let cert = shared_cert().await;
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der.clone()).await;

    client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect("connect once the certificate is trusted via the OS store");
}

#[tokio::test]
async fn native_roots_apply_to_flux_speech_to_text() {
    let cert = shared_cert().await;
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der.clone()).await;

    client(port)
        .transcription()
        .flux_request()
        .handle()
        .await
        .expect("flux connect once the certificate is trusted via the OS store");
}

#[cfg(feature = "speak")]
#[tokio::test]
async fn native_roots_apply_to_flux_text_to_speech() {
    use deepgram::speak::flux::options::{Model, Options as SpeakOptions};

    let cert = shared_cert().await;
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der.clone()).await;

    client(port)
        .text_to_speech()
        .flux_request(SpeakOptions::builder(Model::FluxHaleyEn).build())
        .handle()
        .await
        .expect("flux TTS connect once the certificate is trusted via the OS store");
}

#[tokio::test]
async fn native_roots_still_reject_a_certificate_the_os_store_does_not_have() {
    // Make sure the shared fixture (and SSL_CERT_FILE) is in place, then
    // present a different, unrelated self-signed certificate.
    let _ = shared_cert().await;
    let other = self_signed_named("Some Other CA");
    let port = spawn_tls_server(other.cert_der, other.key_der).await;

    let err = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect_err("a certificate in neither webpki nor the OS store must be rejected");

    assert!(
        matches!(
            err,
            DeepgramError::UntrustedTlsCertificate {
                trust: TlsTrust::WebpkiAndNative,
                ..
            }
        ),
        "{err:?}"
    );
    let message = err.to_string();
    assert!(message.contains("SSL_CERT_FILE"), "{message}");
}

#[cfg(feature = "connect-diagnostics")]
#[tokio::test]
async fn native_roots_apply_to_the_phase_timed_path_and_are_recorded() {
    use deepgram::diagnostics::{ConnectOutcome, ConnectRecord};

    let cert = shared_cert().await;
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der.clone()).await;

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await
        .expect("phase-timed connect once the certificate is trusted via the OS store");

    let record = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(record.outcome, ConnectOutcome::Completed);
    assert_eq!(record.tls_trust, Some(TlsTrust::WebpkiAndNative));
    assert_eq!(record.tls_resumed, Some(false));
}
