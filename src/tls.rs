//! TLS trust for `wss://` WebSocket connections.
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
//!    is used verbatim, for every `wss://` connection that client opens.
//! 2. Otherwise the bundled public roots ([`webpki-roots`](webpki_roots)),
//!    plus — with the `rustls-tls-native-roots` cargo feature — the operating
//!    system's certificate store merged on top. This default is built lazily
//!    on the client's first WebSocket connect and reused, so TLS sessions
//!    can be resumed across connections.
//!
//! The default trusts only the public roots so that `wss://` connections
//! need no OS certificate store at all. Behind a
//! TLS-inspecting proxy (Zscaler, Netskope, …), an internal CA, or a
//! self-hosted deployment, enable `rustls-tls-native-roots` or supply your
//! own config. A connection rejected for an unknown issuer fails with
//! [`DeepgramError::UntrustedTlsCertificate`],
//! whose message names the applicable fix.
//!
//! # Plaintext `ws://` is not covered
//!
//! None of this applies to a client built from an `http://` (or `ws://`)
//! base URL: its WebSocket connections are plaintext `ws://`, with no TLS
//! handshake and no certificate verification at all, so neither the
//! `rustls-tls-native-roots` feature nor [`Deepgram::tls_config`] has any
//! effect on them, and no trust roots are resolved or loaded for them (a
//! `ws://` client never reads the OS certificate store). Credentials and
//! audio travel unencrypted. Keep `ws://`
//! to local testing (`http://localhost`) and use an `https://` base URL
//! whenever an API key, a temporary token, or private traffic is involved,
//! including self-hosted deployments.
//!
//! [`Deepgram::tls_config`]: crate::Deepgram::tls_config

use std::sync::Arc;

use rustls::{ClientConfig, RootCertStore};
use serde::Serialize;
use tokio::sync::OnceCell;
use tungstenite::Error as TungsteniteError;

use crate::DeepgramError;

/// Which trust roots a connection verifies the server certificate against.
///
/// This is the trust that actually resulted, not merely what was configured:
/// with the `rustls-tls-native-roots` feature, [`WebpkiAndNative`] means the
/// OS certificate store contributed at least one root, while
/// [`WebpkiNativeUnavailable`] means it could not be loaded and only the
/// bundled roots were checked. Carried by
/// [`DeepgramError::UntrustedTlsCertificate`] so the message can name the
/// applicable fix, and recorded on connect-diagnostics records so telemetry
/// from environments with different trust setups can be told apart.
///
/// [`WebpkiAndNative`]: TlsTrust::WebpkiAndNative
/// [`WebpkiNativeUnavailable`]: TlsTrust::WebpkiNativeUnavailable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TlsTrust {
    /// The bundled public roots only (the default).
    Webpki,
    /// The bundled public roots plus the OS certificate store
    /// (`rustls-tls-native-roots` feature).
    WebpkiAndNative,
    /// The `rustls-tls-native-roots` feature is enabled, but no native root
    /// certificates could be loaded, so only the bundled public roots were
    /// checked. Typical causes: an `SSL_CERT_FILE` / `SSL_CERT_DIR` override
    /// that is missing, unreadable, or not PEM; a system with no OS
    /// certificate store. The load errors are logged at `tracing` WARN
    /// level when the default config is built.
    WebpkiNativeUnavailable,
    /// A caller-supplied [`rustls::ClientConfig`] via
    /// [`Deepgram::tls_config`](crate::Deepgram::tls_config).
    Custom,
}

/// The trust the default config is configured to provide, before it has been
/// built. Once built, the trust that actually resulted is on the
/// [`ResolvedTls`]; the two differ only when the native roots could not be
/// loaded.
///
/// Only the connect-diagnostics guard needs the trust before the config is
/// built (to stamp a record that may be cancelled before TLS runs).
#[cfg_attr(not(feature = "connect-diagnostics"), allow(dead_code))]
pub(crate) const fn default_trust() -> TlsTrust {
    if cfg!(feature = "rustls-tls-native-roots") {
        TlsTrust::WebpkiAndNative
    } else {
        TlsTrust::Webpki
    }
}

/// The TLS a connect attempt actually uses: the config handed to the
/// connector and the trust roots it embodies. The two are resolved together
/// so that an error hint or a diagnostics record can never describe roots
/// other than the ones the handshake verified against.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedTls {
    pub(crate) config: Arc<ClientConfig>,
    pub(crate) trust: TlsTrust,
}

impl ResolvedTls {
    pub(crate) fn connector(&self) -> tokio_tungstenite::Connector {
        tokio_tungstenite::Connector::Rustls(self.config.clone())
    }
}

/// What a connect attempt hands `tokio-tungstenite`, chosen by the URL
/// scheme. A plaintext `ws://` attempt has no TLS handshake, so nothing is
/// resolved for it: the client's default config is not built and the OS
/// certificate store (with `rustls-tls-native-roots`) is not read.
#[derive(Debug, Clone)]
pub(crate) enum ConnectTls {
    /// Plaintext `ws://`: no TLS, no trust roots.
    Plain,
    /// `wss://`: the client's resolved config and the trust it embodies.
    Tls(ResolvedTls),
}

impl ConnectTls {
    pub(crate) fn connector(&self) -> tokio_tungstenite::Connector {
        match self {
            ConnectTls::Plain => tokio_tungstenite::Connector::Plain,
            ConnectTls::Tls(tls) => tls.connector(),
        }
    }

    /// The trust roots in effect; `None` for a plaintext connection.
    #[cfg_attr(not(feature = "connect-diagnostics"), allow(dead_code))]
    pub(crate) fn trust(&self) -> Option<TlsTrust> {
        match self {
            ConnectTls::Plain => None,
            ConnectTls::Tls(tls) => Some(tls.trust),
        }
    }

    /// Classify a connect-time error (see [`connect_error`]). A plaintext
    /// connection cannot fail certificate verification, so its errors map to
    /// [`DeepgramError::WsError`] unchanged.
    pub(crate) fn connect_error(&self, err: TungsteniteError, host: &str) -> DeepgramError {
        match self {
            ConnectTls::Plain => DeepgramError::from(err),
            ConnectTls::Tls(tls) => connect_error(err, host, tls.trust),
        }
    }
}

/// Per-client TLS settings: either a caller-supplied config, or the lazily
/// built default. Cheap to clone; clones share the same lazily built default.
#[derive(Debug, Clone)]
pub(crate) struct TlsSettings {
    custom: Option<Arc<ClientConfig>>,
    default: Arc<OnceCell<ResolvedTls>>,
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

    /// The trust roots this client uses, without building anything. Until
    /// the default has been built this is the configured trust; afterwards
    /// it is the trust that actually resulted (see [`TlsTrust`]). For the
    /// definitive answer alongside the config, use [`resolve`](Self::resolve).
    #[cfg_attr(not(feature = "connect-diagnostics"), allow(dead_code))]
    pub(crate) fn trust(&self) -> TlsTrust {
        if self.custom.is_some() {
            TlsTrust::Custom
        } else if let Some(default) = self.default.get() {
            default.trust
        } else {
            default_trust()
        }
    }

    /// The config every `wss://` connection from this client uses, with the
    /// trust it embodies. The default is built on first use; reading the OS
    /// certificate store (when enabled) is blocking I/O and runs off the
    /// async worker threads.
    pub(crate) async fn resolve(&self) -> ResolvedTls {
        if let Some(config) = &self.custom {
            return ResolvedTls {
                config: config.clone(),
                trust: TlsTrust::Custom,
            };
        }
        self.default.get_or_init(build_default).await.clone()
    }

    /// The TLS for a connect attempt to `url`: [`resolve`](Self::resolve)d
    /// for `wss://`, [`ConnectTls::Plain`] for anything else, so a plaintext
    /// `ws://` client never builds a config or reads the OS store.
    pub(crate) async fn resolve_for(&self, url: &url::Url) -> ConnectTls {
        if url.scheme() == "wss" {
            ConnectTls::Tls(self.resolve().await)
        } else {
            ConnectTls::Plain
        }
    }
}

/// The configuration `tokio-tungstenite` builds for its `rustls-tls-webpki-roots`
/// feature (and, with `rustls-tls-native-roots`, its native+webpki merge):
/// no client auth, the crate-default provider.
async fn build_default() -> ResolvedTls {
    let (roots, trust) = default_root_store().await;
    ResolvedTls {
        config: Arc::new(
            ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth(),
        ),
        trust,
    }
}

/// The bundled webpki roots, with the OS store merged in first when the
/// `rustls-tls-native-roots` feature is enabled, and the [`TlsTrust`] that
/// describes the result. The webpki roots are always present, so enabling
/// the feature never removes trust — it only adds; when the OS store yields
/// nothing, the trust says so ([`TlsTrust::WebpkiNativeUnavailable`]) rather
/// than claiming roots that were never loaded.
pub(crate) async fn default_root_store() -> (RootCertStore, TlsTrust) {
    #[cfg(feature = "rustls-tls-native-roots")]
    let (mut store, trust) = match tokio::task::spawn_blocking(native_root_store).await {
        Ok(store) if !store.is_empty() => (store, TlsTrust::WebpkiAndNative),
        Ok(store) => (store, TlsTrust::WebpkiNativeUnavailable),
        Err(join_error) => {
            tracing::warn!(
                "loading native root certificates panicked; continuing with the bundled \
                 webpki roots only: {join_error}"
            );
            (RootCertStore::empty(), TlsTrust::WebpkiNativeUnavailable)
        }
    };
    #[cfg(not(feature = "rustls-tls-native-roots"))]
    let (mut store, trust) = (RootCertStore::empty(), TlsTrust::Webpki);

    store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    (store, trust)
}

/// Load the OS certificate store. Mirrors the merge `tokio-tungstenite` 0.28
/// performs for its own `rustls-tls-native-roots` feature: warn and continue
/// past per-certificate errors, keep whatever parsed. `rustls-native-certs`
/// honors `SSL_CERT_FILE` / `SSL_CERT_DIR` in place of the platform store.
///
/// An empty result — load errors, an override pointing at a missing or
/// non-PEM file, a system with no store — is reported here at WARN level and
/// surfaces to the caller as [`TlsTrust::WebpkiNativeUnavailable`], so the
/// remedy in an [`DeepgramError::UntrustedTlsCertificate`] message matches
/// what was actually checked.
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
             store, or point `SSL_CERT_FILE` at a PEM bundle that contains it together \
             with the public roots you rely on (the variable replaces the OS store for \
             these WebSockets and, on Linux, for the REST client too), or pass your own \
             rustls config to `Deepgram::tls_config`."
        }
        TlsTrust::WebpkiNativeUnavailable => {
            "The `rustls-tls-native-roots` feature is enabled, but no native root \
             certificates could be loaded, so only the bundled public (webpki) roots were \
             checked and they do not contain this certificate's issuer. The load errors \
             were logged at `tracing` WARN level; the usual causes are an `SSL_CERT_FILE` / \
             `SSL_CERT_DIR` override that is missing, unreadable, or not PEM, or a system \
             with no OS certificate store. Point `SSL_CERT_FILE` at a valid PEM bundle \
             containing the issuing CA together with the public roots you rely on, or \
             pass your own rustls config to `Deepgram::tls_config`."
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
        let (store, trust) = default_root_store().await;
        assert!(store.len() >= webpki_roots::TLS_SERVER_ROOTS.len());
        // And the reported trust describes what the store holds: native
        // roots are claimed only when at least one was actually merged.
        let native_added = store.len() - webpki_roots::TLS_SERVER_ROOTS.len();
        let expected = match (cfg!(feature = "rustls-tls-native-roots"), native_added) {
            (false, _) => TlsTrust::Webpki,
            (true, 0) => TlsTrust::WebpkiNativeUnavailable,
            (true, _) => TlsTrust::WebpkiAndNative,
        };
        assert_eq!(trust, expected);
    }

    #[tokio::test]
    async fn default_config_matches_tokio_tungstenite_defaults() {
        // The stock tokio-tungstenite path builds webpki roots, no client
        // auth, crate-default provider. Our default must be equivalent so
        // that adopting the explicit connector changes nothing for users who
        // did not opt into anything.
        let config = build_default().await.config;
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
        // Before the default is built, `trust()` can only report what is
        // configured.
        assert_eq!(settings.trust(), default_trust());
        let a = settings.resolve().await;
        let b = clone.resolve().await;
        assert!(Arc::ptr_eq(&a.config, &b.config));
        assert_eq!(a.trust, b.trust);
        assert_ne!(a.trust, TlsTrust::Custom);
        // Once built, `trust()` reports what actually resulted, on every
        // clone, without building again.
        assert_eq!(settings.trust(), a.trust);
        assert_eq!(clone.trust(), a.trust);
    }

    #[tokio::test]
    async fn custom_config_is_used_verbatim() {
        let custom = Arc::new(
            ClientConfig::builder()
                .with_root_certificates(RootCertStore::empty())
                .with_no_client_auth(),
        );
        let settings = TlsSettings::custom(custom.clone());
        let resolved = settings.resolve().await;
        assert!(Arc::ptr_eq(&resolved.config, &custom));
        assert_eq!(resolved.trust, TlsTrust::Custom);
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
    fn ssl_cert_file_hints_ask_for_a_full_bundle() {
        // `SSL_CERT_FILE` replaces the OS store wherever rustls-native-certs
        // reads it (these WebSockets and, on Linux, reqwest's platform
        // verifier), so a file holding only the proxy CA would break REST
        // calls to hosts that CA did not sign. Every hint that names the
        // variable must ask for the public roots alongside the CA and must
        // not suggest pointing it at the CA alone.
        for trust in [TlsTrust::WebpkiAndNative, TlsTrust::WebpkiNativeUnavailable] {
            let hint = untrusted_hint(&trust);
            assert!(hint.contains("SSL_CERT_FILE"), "{hint}");
            assert!(hint.contains("together with the public roots"), "{hint}");
            assert!(!hint.contains("`SSL_CERT_FILE` at it"), "{hint}");
        }
        assert!(untrusted_hint(&TlsTrust::WebpkiAndNative).contains("replaces the OS store"),);
    }

    #[tokio::test]
    async fn plaintext_ws_resolves_nothing() {
        // A `ws://` attempt must not build the default config (and so must
        // not read the OS store): the lazily built default stays unbuilt and
        // the attempt carries no trust roots.
        let settings = TlsSettings::new();
        let ws = url::Url::parse("ws://localhost:8080/v1/listen").unwrap();
        let plain = settings.resolve_for(&ws).await;
        assert!(matches!(plain, ConnectTls::Plain), "{plain:?}");
        assert_eq!(plain.trust(), None);
        assert!(matches!(
            plain.connector(),
            tokio_tungstenite::Connector::Plain
        ));
        assert!(
            settings.default.get().is_none(),
            "default config was built for ws://"
        );

        // The same client on `wss://` resolves as before.
        let wss = url::Url::parse("wss://localhost:8080/v1/listen").unwrap();
        let secure = settings.resolve_for(&wss).await;
        let ConnectTls::Tls(resolved) = &secure else {
            panic!("expected resolved TLS for wss://, got {secure:?}");
        };
        assert_eq!(secure.trust(), Some(resolved.trust));
        assert!(settings.default.get().is_some());

        // Plaintext errors are never upgraded to a certificate error.
        let rustls_err = rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer);
        let io = std::io::Error::new(std::io::ErrorKind::InvalidData, rustls_err);
        let err = plain.connect_error(TungsteniteError::Io(io), "localhost");
        assert!(matches!(err, DeepgramError::WsError(_)), "{err:?}");
    }

    #[test]
    fn failed_native_load_gets_its_own_hint() {
        // A loaded OS store and a failed load must not share a remedy: the
        // failed case says so, still points at `SSL_CERT_FILE` (as the thing
        // to fix, not the thing already checked) and at `tls_config`.
        let loaded = untrusted_hint(&TlsTrust::WebpkiAndNative);
        let unavailable = untrusted_hint(&TlsTrust::WebpkiNativeUnavailable);
        assert_ne!(loaded, unavailable);
        assert!(loaded.contains("were both checked"), "{loaded}");
        assert!(
            !loaded.contains("no native root certificates could be loaded"),
            "{loaded}"
        );
        assert!(
            unavailable.contains("no native root certificates could be loaded"),
            "{unavailable}"
        );
        assert!(!unavailable.contains("were both checked"), "{unavailable}");
        assert!(unavailable.contains("SSL_CERT_FILE"), "{unavailable}");
        assert!(
            unavailable.contains("Deepgram::tls_config"),
            "{unavailable}"
        );

        let rustls_err = rustls::Error::InvalidCertificate(rustls::CertificateError::UnknownIssuer);
        let io = std::io::Error::new(std::io::ErrorKind::InvalidData, rustls_err);
        let err = connect_error(
            TungsteniteError::Io(io),
            "proxy.example",
            TlsTrust::WebpkiNativeUnavailable,
        );
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
        assert!(err.to_string().ends_with(unavailable), "{err}");
    }

    #[test]
    fn trust_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&TlsTrust::WebpkiAndNative).unwrap(),
            "\"webpki_and_native\""
        );
        assert_eq!(
            serde_json::to_string(&TlsTrust::WebpkiNativeUnavailable).unwrap(),
            "\"webpki_native_unavailable\""
        );
    }
}
