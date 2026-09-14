//! CI-runnable proof that when the `rustls-tls-native-roots` feature is on
//! but the OS certificate store cannot be loaded, the SDK says so instead of
//! claiming the store was checked.
//!
//! `rustls-native-certs` reads `SSL_CERT_FILE` / `SSL_CERT_DIR` in place of the
//! platform store on every OS. Pointing them at a non-PEM file and an empty
//! directory is the deterministic stand-in for every failed-load shape.
//!
//! Two process-global facts shape this fixture. `cargo` exports `SSL_CERT_DIR`
//! (via openssl-probe) to every test process on Linux, so the directory must be
//! overridden too, not only the file. And `reqwest` (rustls-platform-verifier on
//! Linux) reads the same variables when the `Deepgram` client is *built*, while
//! the SDK's WebSocket trust reads them on the client's *first connect*. Each
//! test therefore builds its client with the environment untouched, then poisons
//! the variables only around the connect, and a lock serializes that window.

#![cfg(all(feature = "listen", feature = "rustls-tls-native-roots"))]

mod common;

use std::ffi::OsString;

use common::{client, self_signed, spawn_tls_server};
use deepgram::{common::options::Options, tls::TlsTrust, DeepgramError};
use tokio::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

/// Points `SSL_CERT_FILE` at a file that is PEM-framed but not a certificate
/// and `SSL_CERT_DIR` at an empty directory; restores both on drop.
struct GarbageCertEnv {
    prev_file: Option<OsString>,
    prev_dir: Option<OsString>,
}

impl GarbageCertEnv {
    fn set() -> Self {
        let prev_file = std::env::var_os("SSL_CERT_FILE");
        let prev_dir = std::env::var_os("SSL_CERT_DIR");
        let base = std::env::temp_dir().join(format!(
            "deepgram-rust-sdk-test-garbage-ca-{}",
            std::process::id()
        ));
        let empty_dir = base.join("empty");
        std::fs::create_dir_all(&empty_dir).expect("create empty cert dir");
        let file = base.join("garbage.pem");
        std::fs::write(
            &file,
            "-----BEGIN CERTIFICATE-----\nthis is not base64 and not a certificate!\n-----END CERTIFICATE-----\n",
        )
        .expect("write garbage CA file");
        std::env::set_var("SSL_CERT_FILE", &file);
        std::env::set_var("SSL_CERT_DIR", &empty_dir);
        GarbageCertEnv {
            prev_file,
            prev_dir,
        }
    }
}

impl Drop for GarbageCertEnv {
    fn drop(&mut self) {
        restore("SSL_CERT_FILE", self.prev_file.take());
        restore("SSL_CERT_DIR", self.prev_dir.take());
    }
}

fn restore(name: &str, prev: Option<OsString>) {
    match prev {
        Some(v) => std::env::set_var(name, v),
        None => std::env::remove_var(name),
    }
}

#[tokio::test]
async fn failed_native_load_is_reported_with_a_matching_remedy() {
    let _serial = ENV_LOCK.lock().await;
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der, cert.key_der).await;
    let dg = client(port); // reqwest is built with the real environment
    let _env = GarbageCertEnv::set(); // the first wss connect sees no usable native roots

    let err = dg
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
    let _serial = ENV_LOCK.lock().await;
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der).await;
    let dg = client(port).tls_config(common::config_trusting(&cert.cert_der));
    let _env = GarbageCertEnv::set();

    dg.transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect("tls_config bypasses the default roots entirely");
}

#[cfg(feature = "connect-diagnostics")]
#[tokio::test]
async fn failed_native_load_is_recorded_on_the_phase_timed_path() {
    use deepgram::diagnostics::{ConnectOutcome, ConnectPhase, ConnectRecord};

    let _serial = ENV_LOCK.lock().await;
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der, cert.key_der).await;
    let dg = client(port);
    let _env = GarbageCertEnv::set();

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let err = dg
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
    assert_eq!(record.tls_trust, Some(TlsTrust::WebpkiNativeUnavailable));
    assert_eq!(
        serde_json::to_value(&record).unwrap()["tls_trust"],
        "webpki_native_unavailable"
    );
}
