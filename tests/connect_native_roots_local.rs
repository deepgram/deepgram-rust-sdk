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
//! than racing to set its own. The fixture is reference-counted: tests hold
//! it while they run, and when the last holder drops it the PEM file is
//! removed from the temp dir (a later test simply creates a fresh one).

#![cfg(all(feature = "listen", feature = "rustls-tls-native-roots"))]

mod common;

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

use common::{client, self_signed, self_signed_named, spawn_tls_server, SelfSigned};
use deepgram::{common::options::Options, tls::TlsTrust, DeepgramError};

/// One certificate shared by every test currently running, written to a PEM
/// file that `SSL_CERT_FILE` points at. Certificate *data* is shared, not a
/// server: each `#[tokio::test]` has its own runtime, so each test spawns
/// its own server presenting this same certificate.
///
/// Held weakly here and strongly by each running test, so the file lives
/// exactly as long as some test needs it and is removed when the last one
/// finishes; a test that starts after that creates a fresh fixture (with its
/// own file name, so a racing cleanup of the old one cannot remove the new).
static SHARED_CERT: Mutex<Weak<SharedCert>> = Mutex::new(Weak::new());
static FIXTURE_SEQ: AtomicUsize = AtomicUsize::new(0);

struct SharedCert {
    cert: SelfSigned,
    path: PathBuf,
}

impl Drop for SharedCert {
    fn drop(&mut self) {
        // Best effort: a leftover file is harmless, a failed test more so.
        let _ = std::fs::remove_file(&self.path);
    }
}

fn shared_cert() -> Arc<SharedCert> {
    let mut slot = SHARED_CERT.lock().expect("fixture lock");
    if let Some(live) = slot.upgrade() {
        return live;
    }
    let cert = self_signed();
    let path = std::env::temp_dir().join(format!(
        "deepgram-rust-sdk-test-ca-{}-{}.pem",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, cert.cert_pem.as_bytes()).expect("write test CA");
    // Simulate the OS trust store containing an extra root CA, the way a
    // managed machine gets a corporate proxy's CA installed. Set under the
    // lock, before any client that will read it is built.
    std::env::set_var("SSL_CERT_FILE", &path);
    let fixture = Arc::new(SharedCert { cert, path });
    *slot = Arc::downgrade(&fixture);
    fixture
}

#[tokio::test]
async fn native_roots_trust_a_certificate_from_the_os_store() {
    let fixture = shared_cert();
    let cert = &fixture.cert;
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
    let fixture = shared_cert();
    let cert = &fixture.cert;
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

    let fixture = shared_cert();
    let cert = &fixture.cert;
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
    let _fixture = shared_cert();
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

    let fixture = shared_cert();
    let cert = &fixture.cert;
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
