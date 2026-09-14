//! CI-runnable proof that TLS trust is one client-level setting applied to
//! every WebSocket surface, and that a rejected certificate produces an
//! error naming the fix.
//!
//! A localhost TLS server presents a self-signed certificate that the
//! bundled webpki roots will never trust. Without `Deepgram::tls_config`
//! every surface must refuse it with `DeepgramError::UntrustedTlsCertificate`;
//! with a config that trusts it, live transcription, Flux speech-to-text,
//! Flux text-to-speech, streaming text-to-speech, and the phase-timed
//! diagnostics path must all connect.

#![cfg(feature = "listen")]

mod common;

use common::{client, config_trusting, self_signed, spawn_tls_server};
use deepgram::{common::options::Options, tls::TlsTrust, DeepgramError};

#[tokio::test]
async fn default_trust_rejects_an_unknown_issuer_with_an_actionable_error() {
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der, cert.key_der).await;

    let err = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect_err("self-signed certificate must be rejected by default");

    let DeepgramError::UntrustedTlsCertificate { host, trust, .. } = &err else {
        panic!("expected UntrustedTlsCertificate, got {err:?}");
    };
    assert_eq!(host, "localhost");
    let message = err.to_string();
    assert!(message.contains("UnknownIssuer"), "{message}");
    assert!(message.contains("Deepgram::tls_config"), "{message}");
    if cfg!(feature = "rustls-tls-native-roots") {
        match trust {
            TlsTrust::WebpkiAndNative => {
                assert!(message.contains("SSL_CERT_FILE"), "{message}");
            }
            TlsTrust::WebpkiNativeUnavailable => {
                assert!(
                    message.contains("no native root certificates could be loaded"),
                    "{message}"
                );
                assert!(message.contains("SSL_CERT_FILE"), "{message}");
            }
            other => panic!("unexpected native-roots trust: {other:?}"),
        }
    } else {
        assert_eq!(*trust, TlsTrust::Webpki);
        assert!(message.contains("rustls-tls-native-roots"), "{message}");
    }
}

#[tokio::test]
async fn tls_config_applies_to_live_transcription() {
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der).await;

    client(port)
        .tls_config(config_trusting(&cert.cert_der))
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect("connect with a config that trusts the server");
}

#[tokio::test]
async fn tls_config_applies_to_flux_speech_to_text() {
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der).await;

    client(port)
        .tls_config(config_trusting(&cert.cert_der))
        .transcription()
        .flux_request()
        .handle()
        .await
        .expect("flux connect with a config that trusts the server");
}

#[cfg(feature = "speak")]
#[tokio::test]
async fn tls_config_applies_to_flux_text_to_speech() {
    use deepgram::speak::flux::options::{Model, Options as SpeakOptions};

    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der).await;

    client(port)
        .tls_config(config_trusting(&cert.cert_der))
        .text_to_speech()
        .flux_request(SpeakOptions::builder(Model::FluxHaleyEn).build())
        .handle()
        .await
        .expect("flux TTS connect with a config that trusts the server");
}

#[cfg(feature = "speak")]
#[tokio::test]
async fn tls_config_applies_to_streaming_text_to_speech() {
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der).await;

    client(port)
        .tls_config(config_trusting(&cert.cert_der))
        .text_to_speech()
        .speak_stream()
        .handle()
        .await
        .expect("streaming TTS connect with a config that trusts the server");
}

#[tokio::test]
async fn a_custom_config_that_does_not_trust_the_server_says_so() {
    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der, cert.key_der).await;

    let err = client(port)
        .tls_config(
            deepgram::rustls::ClientConfig::builder()
                .with_root_certificates(deepgram::rustls::RootCertStore::empty())
                .with_no_client_auth(),
        )
        .transcription()
        .stream_request_with_options(Options::default())
        .handle()
        .await
        .expect_err("empty trust store must reject everything");

    assert!(
        matches!(
            err,
            DeepgramError::UntrustedTlsCertificate {
                trust: TlsTrust::Custom,
                ..
            }
        ),
        "{err:?}"
    );
    let message = err.to_string();
    assert!(message.contains("does not trust"), "{message}");
    assert!(!message.contains("rustls-tls-native-roots"), "{message}");
}

#[cfg(feature = "connect-diagnostics")]
#[tokio::test]
async fn tls_config_applies_to_the_phase_timed_path_and_is_recorded() {
    use deepgram::diagnostics::{ConnectOutcome, ConnectRecord};

    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der.clone(), cert.key_der).await;
    let dg = client(port).tls_config(config_trusting(&cert.cert_der));

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    dg.transcription()
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx.clone())
        .handle()
        .await
        .expect("phase-timed connect with a trusting config");

    let first = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(first.outcome, ConnectOutcome::Completed);
    assert_eq!(first.tls_trust, Some(TlsTrust::Custom));
    assert_eq!(first.tls_resumed, Some(false), "first handshake is full");
    assert!(first.tls_handshake_ms.is_some());

    // The config lives on the client, so a second connection from the same
    // client can resume the TLS session — and the record says so, because
    // resumed handshakes are not comparable to full ones.
    dg.transcription()
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await
        .expect("second connect from the same client");

    let second = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(second.outcome, ConnectOutcome::Completed);
    assert_eq!(second.tls_resumed, Some(true), "second handshake resumes");
}

#[cfg(feature = "connect-diagnostics")]
#[tokio::test]
async fn a_rejected_certificate_is_recorded_as_a_failed_tls_phase() {
    use deepgram::diagnostics::{ConnectOutcome, ConnectPhase, ConnectRecord};

    let cert = self_signed();
    let port = spawn_tls_server(cert.cert_der, cert.key_der).await;

    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let err = client(port)
        .transcription()
        .stream_request_with_options(Options::default())
        .diagnostics(diag_tx)
        .handle()
        .await
        .expect_err("self-signed certificate must be rejected by default");
    assert!(matches!(err, DeepgramError::UntrustedTlsCertificate { .. }));

    let record = diag_rx.try_recv().expect("one record per attempt");
    assert_eq!(record.outcome, ConnectOutcome::Failed);
    assert_eq!(record.last_phase, ConnectPhase::TlsHandshake);
    assert!(record.tls_resumed.is_none());
    assert!(
        record
            .error
            .as_deref()
            .unwrap_or("")
            .contains("UnknownIssuer"),
        "{record:?}"
    );
}
