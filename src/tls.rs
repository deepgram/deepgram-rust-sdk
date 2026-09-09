//! TLS trust for WebSocket connections.
//!
//! Every WebSocket surface in this crate — live transcription (`/v1/listen`),
//! Flux speech-to-text, and Flux text-to-speech — connects through one
//! explicit rustls connector built here. Trust is therefore identical across
//! surfaces and cannot be changed by which TLS features other crates in your
//! dependency graph happen to enable on `tokio-tungstenite`.
//!
//! Trust roots are resolved once per [`Deepgram`](crate::Deepgram) client:
//!
//! 1. A config passed to [`Deepgram::tls_config`](crate::Deepgram::tls_config)
//!    is used verbatim, for every connection that client opens.
//! 2. Otherwise the bundled public roots ([`webpki-roots`](webpki_roots)),
//!    plus — with the `rustls-tls-native-roots` cargo feature — the operating
//!    system's certificate store merged on top. This default is built lazily
//!    on the client's first WebSocket connect and reused, so TLS sessions
//!    can be resumed across connections.
//!
//! The default trusts only the public roots because that works everywhere,
//! including containers with no OS certificate store at all. Behind a
//! TLS-inspecting proxy (Zscaler, Netskope, …), an internal CA, or a
//! self-hosted deployment, enable `rustls-tls-native-roots` or supply your
//! own config. A connection rejected for an unknown issuer fails with
//! [`DeepgramError::UntrustedTlsCertificate`],
//! whose message names the applicable fix.

use std::sync::Arc;

use rustls::{ClientConfig, RootCertStore};
use serde::Serialize;
use tokio::sync::OnceCell;
use tungstenite::Error as TungsteniteError;

use crate::DeepgramError;

/// Which trust roots a connection verifies the server certificate against.
///
/// Recorded on connect-diagnostics records so telemetry from environments
/// with different trust setups can be told apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TlsTrust {
    /// The bundled public roots only (the default).
    Webpki,
    /// The bundled public roots plus the OS certificate store
    /// (`rustls-tls-native-roots` feature).
    WebpkiAndNative,
    /// A caller-supplied [`rustls::ClientConfig`] via
    /// [`Deepgram::tls_config`](crate::Deepgram::tls_config).
    Custom,
}

/// The trust in effect when no custom config is supplied.
pub(crate) const fn default_trust() -> TlsTrust {
    if cfg!(feature = "rustls-tls-native-roots") {
        TlsTrust::WebpkiAndNative
    } else {
        TlsTrust::Webpki
    }
}

/// Per-client TLS settings: either a caller-supplied config, or the lazily
/// built default. Cheap to clone; clones share the same lazily built default.
#[derive(Debug, Clone)]
pub(crate) struct TlsSettings {
    custom: Option<Arc<ClientConfig>>,
    default: Arc<OnceCell<Arc<ClientConfig>>>,
}

impl TlsSettings {
    pub(crate) fn new() -> Self {
        TlsSettings {
            custom: None,
            default: Arc::new(OnceCell::new()),
        }
    }

    pub(crate) fn custom(config: Arc<ClientConfig>) -> Self {
        TlsSettings {
            custom: Some(config),
            default: Arc::new(OnceCell::new()),
        }
    }

    pub(crate) fn trust(&self) -> TlsTrust {
        if self.custom.is_some() {
            TlsTrust::Custom
        } else {
            default_trust()
        }
    }

    /// The config every connection from this client uses. The default is
    /// built on first use; reading the OS certificate store (when enabled) is
    /// blocking I/O and runs off the async worker threads.
    pub(crate) async fn client_config(&self) -> Arc<ClientConfig> {
        if let Some(config) = &self.custom {
            return config.clone();
        }
        self.default
            .get_or_init(|| async { Arc::new(build_default_config().await) })
            .await
            .clone()
    }

    pub(crate) async fn connector(&self) -> tokio_tungstenite::Connector {
        tokio_tungstenite::Connector::Rustls(self.client_config().await)
    }
}

/// The configuration `tokio-tungstenite` builds for its `rustls-tls-webpki-roots`
/// feature (and, with `rustls-tls-native-roots`, its native+webpki merge):
/// no client auth, the crate-default provider.
async fn build_default_config() -> ClientConfig {
    ClientConfig::builder()
        .with_root_certificates(default_root_store().await)
        .with_no_client_auth()
}

/// The bundled webpki roots, with the OS store merged in first when the
/// `rustls-tls-native-roots` feature is enabled. The webpki roots are always
/// present, so enabling the feature never removes trust — it only adds.
pub(crate) async fn default_root_store() -> RootCertStore {
    #[cfg(feature = "rustls-tls-native-roots")]
    let mut store = match tokio::task::spawn_blocking(native_root_store).await {
        Ok(store) => store,
        Err(join_error) => {
            tracing::warn!(
                "loading native root certificates panicked; continuing with the bundled \
                 webpki roots only: {join_error}"
            );
            RootCertStore::empty()
        }
    };
    #[cfg(not(feature = "rustls-tls-native-roots"))]
    let mut store = RootCertStore::empty();

    store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    store
}

/// Load the OS certificate store. Mirrors the merge `tokio-tungstenite` 0.28
/// performs for its own `rustls-tls-native-roots` feature: warn and continue
/// past per-certificate errors, keep whatever parsed. `rustls-native-certs`
/// honors `SSL_CERT_FILE` / `SSL_CERT_DIR` in place of the platform store.
#[cfg(feature = "rustls-tls-native-roots")]
fn native_root_store() -> RootCertStore {
    let mut store = RootCertStore::empty();
    let rustls_native_certs::CertificateResult { certs, errors, .. } =
        rustls_native_certs::load_native_certs();
    if !errors.is_empty() {
        tracing::warn!("errors while loading native root certificates: {errors:?}");
    }
    let total = certs.len();
    let (added, ignored) = store.add_parsable_certificates(certs);
    if added == 0 {
        tracing::warn!(
            "no native root certificates were found; continuing with the bundled webpki \
             roots only"
        );
    } else {
        tracing::debug!("added {added}/{total} native root certificates ({ignored} ignored)");
    }
    store
}

/// Convert a connect-time error into a [`DeepgramError`], upgrading a
/// certificate rejected for an unknown issuer into
/// [`DeepgramError::UntrustedTlsCertificate`] so the message can name the
/// fix. Every other error maps to [`DeepgramError::WsError`] unchanged.
pub(crate) fn connect_error(err: TungsteniteError, host: &str, trust: TlsTrust) -> DeepgramError {
    if is_unknown_issuer(&err) {
        DeepgramError::UntrustedTlsCertificate {
            host: host.to_owned(),
            trust,
            source: Box::new(err),
        }
    } else {
        DeepgramError::from(err)
    }
}

/// `tokio-rustls` surfaces handshake failures as an `io::Error` wrapping the
/// `rustls::Error`; both connect paths in this crate go through it.
fn is_unknown_issuer(err: &TungsteniteError) -> bool {
    let TungsteniteError::Io(io) = err else {
        return false;
    };
    matches!(
        io.get_ref().and_then(|e| e.downcast_ref::<rustls::Error>()),
        Some(rustls::Error::InvalidCertificate(
            rustls::CertificateError::UnknownIssuer
        ))
    )
}

/// The remedy an [`DeepgramError::UntrustedTlsCertificate`] message ends
/// with, chosen by which trust roots were in effect.
pub(crate) fn untrusted_hint(trust: &TlsTrust) -> &'static str {
    match trust {
        TlsTrust::Webpki => {
            "By default the SDK trusts only the bundled public (webpki) roots. If this \
             connection goes through a TLS-inspecting proxy or to a server with a private \
             CA, enable the `rustls-tls-native-roots` cargo feature to also trust the OS \
             certificate store, or pass your own rustls config to `Deepgram::tls_config`."
        }
        TlsTrust::WebpkiAndNative => {
            "The bundled public roots and the OS certificate store were both checked and \
             neither contains this certificate's issuer. Install the issuing CA in the OS \
             store (or point `SSL_CERT_FILE` at it), or pass your own rustls config to \
             `Deepgram::tls_config`."
        }
        TlsTrust::Custom => {
            "The rustls config passed to `Deepgram::tls_config` does not trust this \
             certificate's issuer."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_store_always_contains_the_public_roots() {
        // With `rustls-tls-native-roots`, the OS store is additive: the store
        // can never have fewer roots than webpki alone, and it is never
        // empty even where the OS store yields nothing (a minimal container).
        let store = default_root_store().await;
        assert!(store.len() >= webpki_roots::TLS_SERVER_ROOTS.len());
    }

    #[tokio::test]
    async fn default_config_matches_tokio_tungstenite_defaults() {
        // The stock tokio-tungstenite path builds webpki roots, no client
        // auth, crate-default provider. Our default must be equivalent so
        // that adopting the explicit connector changes nothing for users who
        // did not opt into anything.
        let config = build_default_config().await;
        assert!(!config.client_auth_cert_resolver.has_certs());
        let roots = RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        assert_eq!(
            config.crypto_provider().cipher_suites,
            ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth()
                .crypto_provider()
                .cipher_suites,
        );
    }

    #[tokio::test]
    async fn default_config_is_built_once_per_client_and_shared_by_clones() {
        let settings = TlsSettings::new();
        let clone = settings.clone();
        let a = settings.client_config().await;
        let b = clone.client_config().await;
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(settings.trust(), default_trust());
    }

    #[tokio::test]
    async fn custom_config_is_used_verbatim() {
        let custom = Arc::new(
            ClientConfig::builder()
                .with_root_certificates(RootCertStore::empty())
                .with_no_client_auth(),
        );
        let settings = TlsSettings::custom(custom.clone());
        assert!(Arc::ptr_eq(&settings.client_config().await, &custom));
        assert_eq!(settings.trust(), TlsTrust::Custom);
    }

    #[test]
    fn unknown_issuer_is_classified_and_hinted() {
        let rustls_err = rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer);
        let io = std::io::Error::new(std::io::ErrorKind::InvalidData, rustls_err);
        let err = connect_error(TungsteniteError::Io(io), "proxy.example", TlsTrust::Webpki);

        match &err {
            DeepgramError::UntrustedTlsCertificate { host, trust, .. } => {
                assert_eq!(host, "proxy.example");
                assert_eq!(*trust, TlsTrust::Webpki);
            }
            other => panic!("expected UntrustedTlsCertificate, got {other:?}"),
        }
        let message = err.to_string();
        assert!(message.contains("proxy.example"), "{message}");
        assert!(message.contains("UnknownIssuer"), "{message}");
        assert!(message.contains("rustls-tls-native-roots"), "{message}");
        assert!(message.contains("Deepgram::tls_config"), "{message}");
    }

    #[test]
    fn other_errors_stay_ws_errors() {
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let err = connect_error(TungsteniteError::Io(io), "h", TlsTrust::Webpki);
        assert!(matches!(err, DeepgramError::WsError(_)), "{err:?}");

        let not_for_name =
            rustls::Error::InvalidCertificate(rustls::CertificateError::NotValidForName);
        let io = std::io::Error::new(std::io::ErrorKind::InvalidData, not_for_name);
        let err = connect_error(TungsteniteError::Io(io), "h", TlsTrust::Webpki);
        assert!(matches!(err, DeepgramError::WsError(_)), "{err:?}");
    }

    #[test]
    fn hint_follows_trust_in_effect() {
        assert!(untrusted_hint(&TlsTrust::Webpki).contains("rustls-tls-native-roots"));
        assert!(untrusted_hint(&TlsTrust::WebpkiAndNative).contains("SSL_CERT_FILE"));
        assert!(untrusted_hint(&TlsTrust::Custom).contains("tls_config"));
    }

    #[test]
    fn trust_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&TlsTrust::WebpkiAndNative).unwrap(),
            "\"webpki_and_native\""
        );
    }
}
