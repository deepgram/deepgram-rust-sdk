//! CI-runnable proof that when the `rustls-tls-native-roots` feature is on
//! but the OS certificate store cannot be loaded, the SDK says so instead of
//! claiming the store was checked.
//!
//! `rustls-native-certs` reads `SSL_CERT_FILE` in place of the platform store
//! on every OS. Pointing it at a file that is not a certificate bundle is the
//! deterministic stand-in for every failed-load shape a deployment can hit:
//! a typo in the override, an unreadable path, a minimal container with no
//! store at all. The client must fall back to the bundled webpki roots (it
//! does; this is the additive contract) and a certificate those roots reject
//! must produce `TlsTrust::WebpkiNativeUnavailable` with a remedy that names
//! a *valid* `SSL_CERT_FILE` or `tls_config`, not "the OS store was checked".
//!
//! The env var is process-global and the SDK reads it once per client, so
//! this lives in its own test binary (its own process), apart from
//! `connect_native_roots_local.rs`, whose tests share a *valid*
//! `SSL_CERT_FILE`. Every test here shares the one garbage value.

#![cfg(all(feature = "listen", feature = "rustls-tls-native-roots"))]

mod common;

use common::{client, self_signed, spawn_tls_server};
use deepgram::{common::options::Options, tls::TlsTrust, DeepgramError};
use tokio::sync::OnceCell;

static GARBAGE_CERT_FILE: OnceCell<()> = OnceCell::const_new();

/// Point `SSL_CERT_FILE` at a file that parses as PEM but yields no usable
/// certificate. Runs at most once per process, inside `get_or_init`.
async fn with_garbage_cert_file() {
    GARBAGE_CERT_FILE
        .get_or_init(|| async {
            let path = std::env::temp_dir().join(format!(
                "deepgram-rust-sdk-test-garbage-ca-{}.pem",
                std::process::id()
            ));
            std::fs::write(
                &path,
                "-----BEGIN CERTIFICATE-----\nthis is not base64 and not a certificate!\n-----END CERTIFICATE-----\n",
            )
            .expect("write garbage CA file");
            std::env::set_var("SSL_CERT_FILE", &path);
        })
        .await;
}

#[tokio::test]
async fn failed_native_load_is_reported_with_a_matching_remedy() {
    with_garbage_cert_file().await;
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der, cert.key_der).await;

    let err = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect_err("with no native roots loaded, only webpki is trusted, which rejects this");

    let DeepgramError::UntrustedTlsCertificate { host, trust, .. } = &err else {
        panic!("expected UntrustedTlsCertificate, got {err:?}");
    };
    assert_eq!(host, "localhost");
    assert_eq!(*trust, TlsTrust::WebpkiNativeUnavailable);

    let message = err.to_string();
    assert!(message.contains("UnknownIssuer"), "{message}");
    assert!(
        message.contains("no native root certificates could be loaded"),
        "{message}"
    );
    assert!(
        !message.contains("were both checked"),
        "must not claim the OS store was checked: {message}"
    );
    assert!(message.contains("SSL_CERT_FILE"), "{message}");
    assert!(message.contains("Deepgram::tls_config"), "{message}");
}

#[tokio::test]
async fn failed_native_load_still_trusts_the_public_roots_via_tls_config_fallback() {
    // The additive contract holds even when the native load fails: the
    // client is usable, and the documented remedy (`tls_config`) works.
    with_garbage_cert_file().await;
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der).await;

    client(port)
        .tls_config(common::config_trusting(&cert.cert_der))
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect("tls_config bypasses the default roots entirely");
}

#[cfg(feature = "connect-diagnostics")]
#[tokio::test]
async fn failed_native_load_is_recorded_on_the_phase_timed_path() {
    use deepgram::diagnostics::{ConnectOutcome, ConnectPhase, ConnectRecord};

    with_garbage_cert_file().await;
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der, cert.key_der).await;

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let err = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await
        .expect_err("phase-timed path rejects the certificate too");
    assert!(
        matches!(
            err,
            DeepgramError::UntrustedTlsCertificate {
                trust: TlsTrust::WebpkiNativeUnavailable,
                ..
            }
        ),
        "{err:?}"
    );

    let record = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(record.outcome, ConnectOutcome::Failed);
    assert_eq!(record.last_phase, ConnectPhase::TlsHandshake);
    // The record says which roots were actually verified against, not
    // which were configured.
    assert_eq!(record.tls_trust, Some(TlsTrust::WebpkiNativeUnavailable));
    assert_eq!(
        serde_json::to_value(&record).unwrap()["tls_trust"],
        "webpki_native_unavailable"
    );
}
