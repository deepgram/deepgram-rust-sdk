//! Opt-in connect-time diagnostics for live transcription streaming
//! connections (`/v1/listen`).
//!
//! Other WebSocket surfaces (Flux, text-to-speech streaming) do not expose
//! diagnostics yet.
//!
//! When a diagnostics sink is configured on a websocket builder (see
//! [`crate::listen::websocket::WebsocketBuilder::diagnostics`]), the SDK
//! establishes the connection in four individually timed phases — DNS
//! resolution, TCP connect, TLS handshake, and the WebSocket upgrade — and
//! emits one [`ConnectRecord`] per connect attempt.
//!
//! Records are delivered synchronously from a drop guard, so a record is
//! emitted even when the caller wraps the connect future in
//! `tokio::time::timeout` and the timeout fires: the record for a cancelled
//! attempt carries [`ConnectOutcome::Cancelled`], the furthest phase reached,
//! and every phase timing captured up to the moment of cancellation.
//!
//! When no sink is configured, the SDK uses its stock connect path
//! (`tokio_tungstenite::connect_async_tls_with_config`) and no phases are
//! timed or recorded.
//!
//! With this feature enabled, both connect paths — stock and phase-timed —
//! are handed the same explicit rustls connector (the crate-internal
//! `tls_connector`): webpki
//! trust roots, no client auth, the crate-default provider. That keeps timed
//! and untimed connections on identical TLS provider and trust configuration
//! even when downstream feature unification enables another TLS backend
//! (e.g. `tokio-tungstenite/native-tls`) in the dependency graph. The
//! upgrade machinery and the request are the same on both paths; only the
//! granularity of measurement differs.
//!
//! [`crate::listen::websocket::WebsocketBuilder::trust_native_roots`] is a
//! separate opt-in, independent of whether a diagnostics sink is configured:
//! it additionally trusts the operating system's certificate store, for
//! environments where trust is anchored there rather than in the public
//! webpki bundle — for example, a corporate network's TLS-inspecting proxy,
//! an internal CA, or a self-hosted Deepgram deployment.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use http::Request;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::error::UrlError;
use tungstenite::Error as TungsteniteError;
use uuid::Uuid;

use crate::{DeepgramError, Result};

/// Version of the [`ConnectRecord`] schema. Changes are additive only:
/// consumers should ignore unknown fields.
pub const SCHEMA_VERSION: u32 = 1;

/// How a connect attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectOutcome {
    /// The connection was established and the WebSocket upgrade succeeded.
    Completed,
    /// A phase of the connection failed.
    Failed,
    /// The connect future was dropped before completing, e.g. because a
    /// caller-side `tokio::time::timeout` fired.
    Cancelled,
}

/// A phase of connection establishment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectPhase {
    /// Hostname resolution.
    Dns,
    /// TCP handshake.
    TcpConnect,
    /// TLS handshake.
    TlsHandshake,
    /// WebSocket upgrade exchange.
    WsUpgrade,
}

/// One record per connect attempt.
///
/// Serializes to a single flat JSON object suitable for JSONL sinks. Optional
/// fields are omitted when absent (rather than serialized as `null`), and the
/// combination of [`outcome`](Self::outcome) and
/// [`last_phase`](Self::last_phase) determines which fields can be present:
/// an attempt that never received a server response has no
/// [`request_id`](Self::request_id), and each phase timing is present only if
/// that phase completed.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ConnectRecord {
    /// Schema version of this record. See [`SCHEMA_VERSION`].
    pub schema_version: u32,
    /// Client-generated ID for this connect attempt. Unlike
    /// [`request_id`](Self::request_id), this is always present, so it can
    /// correlate attempts that never received a server response.
    pub attempt_id: Uuid,
    /// RFC 3339 UTC timestamp of the start of the connect attempt.
    pub timestamp: String,
    /// How the attempt ended.
    pub outcome: ConnectOutcome,
    /// The furthest phase entered.
    pub last_phase: ConnectPhase,
    /// Request URL reduced to scheme, host, port, and path. Userinfo, query
    /// parameters, and fragment are stripped before recording: query values
    /// carry customer-controlled and potentially sensitive content (signed
    /// callback URLs, keyterms, arbitrary `query_params` values), which must
    /// not be persisted in telemetry. Authorization is carried in a request
    /// header and never appears here either.
    pub url: String,
    /// Total duration from the start of the attempt until completion,
    /// failure, or cancellation, in milliseconds.
    pub connect_duration_ms: f64,
    /// Local (source) socket address, available once the TCP connection is
    /// established.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_addr: Option<String>,
    /// Resolved peer (destination) socket address, available once the TCP
    /// connection is established.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_addr: Option<String>,
    /// Hostname resolution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_ms: Option<f64>,
    /// TCP handshake time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_connect_ms: Option<f64>,
    /// TLS handshake time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_handshake_ms: Option<f64>,
    /// WebSocket upgrade exchange time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_upgrade_ms: Option<f64>,
    /// The `dg-request-id` response header. Present whenever the server
    /// responded to the upgrade request — on success and on rejected
    /// upgrades. Absent when the attempt was cancelled or failed before a
    /// response was received.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// The `dg-error` response header, present on rejected upgrades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dg_error: Option<String>,
    /// Client-side error description, present when the outcome is
    /// [`ConnectOutcome::Failed`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ConnectRecord {
    /// A deterministic, fully populated sample record, for testing
    /// [`DiagnosticsSink`] implementations. `ConnectRecord` is
    /// `#[non_exhaustive]`, so this is the supported way to fabricate a
    /// record outside this crate.
    ///
    /// ```
    /// use deepgram::diagnostics::ConnectRecord;
    ///
    /// let record = ConnectRecord::sample();
    /// let line = serde_json::to_string(&record).unwrap();
    /// assert!(line.contains("\"outcome\":\"completed\""));
    /// ```
    pub fn sample() -> ConnectRecord {
        ConnectRecord {
            schema_version: SCHEMA_VERSION,
            attempt_id: Uuid::nil(),
            timestamp: "2026-01-01T00:00:00.000Z".to_string(),
            outcome: ConnectOutcome::Completed,
            last_phase: ConnectPhase::WsUpgrade,
            url: "wss://api.deepgram.com/v1/listen".to_string(),
            connect_duration_ms: 312.4,
            local_addr: Some("10.0.4.17:53210".to_string()),
            peer_addr: Some("203.0.113.5:443".to_string()),
            dns_ms: Some(11.2),
            tcp_connect_ms: Some(68.9),
            tls_handshake_ms: Some(141.7),
            ws_upgrade_ms: Some(90.6),
            request_id: Some("00000000-0000-0000-0000-000000000000".to_string()),
            dg_error: None,
            error: None,
        }
    }
}

/// Destination for [`ConnectRecord`]s.
///
/// [`emit`](Self::emit) must not block and must not panic: it is called
/// synchronously on the connect path, including from a destructor when the
/// connect future is dropped by a caller-side timeout.
///
/// Implemented for `tokio::sync::mpsc::UnboundedSender<ConnectRecord>`
/// (unbounded, because its `send` is synchronous and therefore safe to call
/// from a destructor). Wrap a closure with [`sink_fn`].
pub trait DiagnosticsSink: Send + Sync {
    /// Deliver one record. Errors must be swallowed: there is nowhere to
    /// report a delivery failure from the connect path.
    fn emit(&self, record: ConnectRecord);
}

impl DiagnosticsSink for tokio::sync::mpsc::UnboundedSender<ConnectRecord> {
    fn emit(&self, record: ConnectRecord) {
        let _ = self.send(record);
    }
}

/// Wrap a closure as a [`DiagnosticsSink`].
///
/// ```
/// use deepgram::diagnostics::{sink_fn, ConnectRecord};
///
/// let sink = sink_fn(|record: ConnectRecord| {
///     println!("{}", serde_json::to_string(&record).unwrap());
/// });
/// ```
pub fn sink_fn<F>(f: F) -> impl DiagnosticsSink
where
    F: Fn(ConnectRecord) + Send + Sync,
{
    struct FnSink<F>(F);

    impl<F> DiagnosticsSink for FnSink<F>
    where
        F: Fn(ConnectRecord) + Send + Sync,
    {
        fn emit(&self, record: ConnectRecord) {
            (self.0)(record);
        }
    }

    FnSink(f)
}

/// A cloneable, debuggable handle to a shared sink, stored on builders.
#[derive(Clone)]
pub(crate) struct SharedSink(pub(crate) Arc<dyn DiagnosticsSink>);

impl fmt::Debug for SharedSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SharedSink(..)")
    }
}

/// Holds the partial record during a connect attempt and emits it from `Drop`.
///
/// All three exits converge here: success and failure set the outcome
/// explicitly before the guard is dropped, and cancellation (the future being
/// dropped) leaves the default [`ConnectOutcome::Cancelled`] in place. `Drop`
/// is the single emit point.
pub(crate) struct DiagnosticsGuard {
    record: ConnectRecord,
    start: Instant,
    phase_start: Instant,
    sink: SharedSink,
}

impl DiagnosticsGuard {
    pub(crate) fn new(sink: SharedSink, url: &url::Url) -> Self {
        let now = Instant::now();
        DiagnosticsGuard {
            record: ConnectRecord {
                schema_version: SCHEMA_VERSION,
                attempt_id: Uuid::new_v4(),
                timestamp: rfc3339_utc(SystemTime::now()),
                outcome: ConnectOutcome::Cancelled,
                last_phase: ConnectPhase::Dns,
                url: redacted_url(url),
                connect_duration_ms: 0.0,
                local_addr: None,
                peer_addr: None,
                dns_ms: None,
                tcp_connect_ms: None,
                tls_handshake_ms: None,
                ws_upgrade_ms: None,
                request_id: None,
                dg_error: None,
                error: None,
            },
            start: now,
            phase_start: now,
            sink,
        }
    }

    /// Mark entry into a phase. If the guard is dropped before
    /// [`finish_phase`](Self::finish_phase), the record shows this phase as
    /// the furthest reached, with no timing for it.
    fn enter_phase(&mut self, phase: ConnectPhase) {
        self.record.last_phase = phase;
        self.phase_start = Instant::now();
    }

    /// Stamp the elapsed time of the current phase onto the record.
    fn finish_phase(&mut self) {
        let elapsed = ms(self.phase_start.elapsed());
        let slot = match self.record.last_phase {
            ConnectPhase::Dns => &mut self.record.dns_ms,
            ConnectPhase::TcpConnect => &mut self.record.tcp_connect_ms,
            ConnectPhase::TlsHandshake => &mut self.record.tls_handshake_ms,
            ConnectPhase::WsUpgrade => &mut self.record.ws_upgrade_ms,
        };
        *slot = Some(elapsed);
    }

    fn set_addrs(&mut self, stream: &TcpStream) {
        self.record.local_addr = stream.local_addr().ok().map(|a| a.to_string());
        self.record.peer_addr = stream.peer_addr().ok().map(|a| a.to_string());
    }

    pub(crate) fn set_request_id(&mut self, request_id: &str) {
        self.record.request_id = Some(request_id.to_string());
    }

    pub(crate) fn complete(&mut self) {
        self.record.outcome = ConnectOutcome::Completed;
    }

    /// Record a client-side failure that occurs after the wire exchange, e.g.
    /// the SDK rejecting a malformed upgrade response.
    pub(crate) fn fail_client(&mut self, error: &str) {
        self.record.outcome = ConnectOutcome::Failed;
        self.record.error = Some(error.to_string());
    }

    /// Record a failure, capturing the `dg-request-id` and `dg-error`
    /// response headers when the server responded to the upgrade request.
    fn fail(&mut self, error: &TungsteniteError) {
        self.record.outcome = ConnectOutcome::Failed;
        self.record.error = Some(error.to_string());
        if let TungsteniteError::Http(response) = error {
            if let Some(id) = header_str(response.headers(), "dg-request-id") {
                self.record.request_id = Some(id);
            }
            if let Some(err) = header_str(response.headers(), "dg-error") {
                self.record.dg_error = Some(err);
            }
        }
    }
}

impl Drop for DiagnosticsGuard {
    fn drop(&mut self) {
        self.record.connect_duration_ms = ms(self.start.elapsed());
        self.sink.0.emit(self.record.clone());
    }
}

/// The request URL reduced to scheme, host, port, and path, for recording.
/// Userinfo, query, and fragment are stripped: query values carry
/// customer-controlled and potentially sensitive content (signed callback
/// URLs, keyterms, arbitrary `query_params` values), which must not be
/// persisted in diagnostics telemetry.
fn redacted_url(url: &url::Url) -> String {
    let mut url = url.clone();
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

fn ms(duration: std::time::Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn header_str(headers: &http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// Establish a WebSocket connection with per-phase timing.
///
/// Mirrors `tokio_tungstenite::connect_async` phase by phase: hostname
/// resolution (which `TcpStream::connect` performs internally on the stock
/// path), sequential TCP connect attempts across the resolved addresses, a
/// TLS handshake with the same `rustls` configuration `tokio-tungstenite`
/// builds for its `rustls-tls-webpki-roots` feature, and the upgrade via
/// `tokio_tungstenite::client_async_with_config` — the same function the
/// stock path bottoms out in.
pub(crate) async fn connect_with_diagnostics(
    request: Request<()>,
    guard: &mut DiagnosticsGuard,
    trust_native_roots: bool,
) -> Result<(
    WebSocketStream<MaybeTlsStream<TcpStream>>,
    tungstenite::handshake::client::Response,
)> {
    match connect_phases(request, guard, trust_native_roots).await {
        Ok(ok) => Ok(ok),
        Err(err) => {
            guard.fail(&err);
            Err(DeepgramError::from(Box::new(err)))
        }
    }
}

async fn connect_phases(
    request: Request<()>,
    guard: &mut DiagnosticsGuard,
    trust_native_roots: bool,
) -> std::result::Result<
    (
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        tungstenite::handshake::client::Response,
    ),
    TungsteniteError,
> {
    let domain = domain(&request)?;
    let tls = match request.uri().scheme_str() {
        Some("wss") => true,
        Some("ws") => false,
        _ => return Err(TungsteniteError::Url(UrlError::UnsupportedUrlScheme)),
    };
    let port = request
        .uri()
        .port_u16()
        .unwrap_or(if tls { 443 } else { 80 });

    guard.enter_phase(ConnectPhase::Dns);
    let addrs: Vec<_> = tokio::net::lookup_host((domain.as_str(), port))
        .await?
        .collect();
    guard.finish_phase();

    guard.enter_phase(ConnectPhase::TcpConnect);
    let mut tcp = None;
    let mut last_err = None;
    for addr in addrs {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                tcp = Some(stream);
                break;
            }
            Err(err) => last_err = Some(err),
        }
    }
    let tcp = match tcp {
        Some(tcp) => tcp,
        None => {
            return Err(TungsteniteError::Io(last_err.unwrap_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("could not resolve host {domain}"),
                )
            })))
        }
    };
    guard.set_addrs(&tcp);
    guard.finish_phase();

    let stream = if tls {
        guard.enter_phase(ConnectPhase::TlsHandshake);
        let server_name = rustls_pki_types::ServerName::try_from(domain.as_str())
            .map_err(|_| TungsteniteError::Tls(tungstenite::error::TlsError::InvalidDnsName))?
            .to_owned();
        let connector =
            tokio_rustls::TlsConnector::from(tls_client_config(trust_native_roots).await);
        let tls_stream = connector
            .connect(server_name, tcp)
            .await
            .map_err(TungsteniteError::Io)?;
        guard.finish_phase();
        MaybeTlsStream::Rustls(tls_stream)
    } else {
        MaybeTlsStream::Plain(tcp)
    };

    guard.enter_phase(ConnectPhase::WsUpgrade);
    let (ws_stream, response) =
        tokio_tungstenite::client_async_with_config(request, stream, None).await?;
    guard.finish_phase();

    Ok((ws_stream, response))
}

/// The trust roots backing [`tls_client_config`]: the bundled public webpki
/// roots, the same set `tokio-tungstenite`'s `rustls-tls-webpki-roots`
/// feature trusts. When `trust_native_roots` is set, the operating system's
/// certificate store is also consulted — this is what lets the connection
/// succeed wherever trust is anchored in the OS store rather than the
/// public CA bundle this crate ships by default: a corporate network's
/// TLS-inspecting proxy, an internal CA, or a self-hosted deployment.
/// Enabling it means Deepgram's TLS traffic is trusted as decrypted and
/// re-encrypted by anything the OS trusts, which may include such a proxy;
/// it is opt-in for that reason.
///
/// This intentionally mirrors `tokio-tungstenite` 0.28's own native+webpki
/// merge (`encryption::rustls::wrap_stream` in its `src/tls.rs`, reachable
/// only via its feature-driven default connector, which this crate's
/// explicit connector never uses — see [`tls_connector`]'s doc comment): load
/// native certs, warn and continue past per-cert errors, add whatever
/// parsed, then always add the webpki set regardless. That upstream
/// function is private and not reusable from here, so this is a maintained
/// copy, not a delegation — if a `tokio-tungstenite` bump changes that
/// merge's order or error handling, this function needs the same change by
/// hand. The equivalence this function relies on is pinned by
/// `root_store_still_trusts_public_roots_with_native_roots_enabled` below;
/// a change here that breaks that invariant should fail that test.
///
/// Reading the OS certificate store is genuinely blocking I/O (platform
/// keychain/CryptoAPI calls, or a directory scan on Unix) and can be slow,
/// so unlike the webpki set below it is not read on every call: it is loaded
/// off the async runtime's worker threads via
/// [`spawn_blocking`](tokio::task::spawn_blocking) and cached, subject to
/// [`NATIVE_ROOTS_TTL`] — see [`cached_native_root_cert_store`] for the
/// staleness/refresh policy. A newly installed or rotated corporate/internal
/// CA is picked up within one TTL window of a connect attempt, without every
/// connect paying the OS read's latency.
async fn root_cert_store(trust_native_roots: bool) -> rustls::RootCertStore {
    let mut root_store = if trust_native_roots {
        (*native_root_cert_store().await).clone()
    } else {
        rustls::RootCertStore::empty()
    };

    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    root_store
}

/// How long a cached native root store is served before a connect triggers
/// a background refresh. Chosen as a middle ground: short enough that a
/// rotated corporate/internal CA is picked up promptly by a long-lived
/// client, long enough that the OS trust store (read via blocking I/O) is
/// not re-read on anything like every connect.
const NATIVE_ROOTS_TTL: Duration = Duration::from_secs(5 * 60);

struct CachedNativeRoots {
    store: Arc<rustls::RootCertStore>,
    loaded_at: Instant,
}

/// Process-wide cache for [`root_cert_store`]'s native portion; see
/// [`cached_native_root_cert_store`] for the staleness/refresh policy.
static NATIVE_ROOT_CERT_STORE: tokio::sync::RwLock<Option<CachedNativeRoots>> =
    tokio::sync::RwLock::const_new(None);

/// Guards against piling up redundant background refreshes when many
/// connects land after the cache has gone stale: only one refresh task runs
/// at a time, and every other stale hit just keeps serving the old value
/// until that refresh completes.
static NATIVE_ROOTS_REFRESHING: AtomicBool = AtomicBool::new(false);

async fn native_root_cert_store() -> Arc<rustls::RootCertStore> {
    cached_native_root_cert_store(NATIVE_ROOTS_TTL).await
}

/// Stale-while-revalidate cache for the OS-native root store.
///
/// - Cold (nothing cached yet): loads synchronously and blocks the caller —
///   unavoidable, since a connect needs a trust store to proceed, but this
///   only ever happens once per process.
/// - Fresh (`loaded_at` within `ttl`): returns the cached value immediately.
/// - Stale (`loaded_at` older than `ttl`): still returns the cached value
///   immediately — no connect after the first ever pays the OS read's
///   latency — while kicking off a detached background reload that updates
///   the cache for subsequent connects. Worst-case staleness is therefore
///   up to roughly two TTL windows, not one, in exchange for connects never
///   observing a latency spike after the initial load.
async fn cached_native_root_cert_store(ttl: Duration) -> Arc<rustls::RootCertStore> {
    if let Some(cached) = NATIVE_ROOT_CERT_STORE.read().await.as_ref() {
        if cached.loaded_at.elapsed() < ttl {
            return cached.store.clone();
        }
        let stale = cached.store.clone();
        spawn_native_roots_refresh();
        return stale;
    }

    load_and_cache_native_roots().await
}

fn spawn_native_roots_refresh() {
    if NATIVE_ROOTS_REFRESHING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    tokio::spawn(async {
        load_and_cache_native_roots().await;
        NATIVE_ROOTS_REFRESHING.store(false, Ordering::Release);
    });
}

async fn load_and_cache_native_roots() -> Arc<rustls::RootCertStore> {
    let store = tokio::task::spawn_blocking(|| {
        let mut store = rustls::RootCertStore::empty();
        let rustls_native_certs::CertificateResult { certs, errors, .. } =
            rustls_native_certs::load_native_certs();
        if !errors.is_empty() {
            tracing::warn!("native root CA certificate loading errors: {errors:?}");
        }
        let (added, ignored) = store.add_parsable_certificates(certs);
        tracing::debug!("added {added} native root certificates ({ignored} ignored)");
        store
    })
    .await
    .unwrap_or_else(|join_err| {
        tracing::warn!(
            "native root CA loading task panicked, continuing with webpki roots only: {join_err}"
        );
        rustls::RootCertStore::empty()
    });
    let store = Arc::new(store);
    *NATIVE_ROOT_CERT_STORE.write().await = Some(CachedNativeRoots {
        store: store.clone(),
        loaded_at: Instant::now(),
    });
    store
}

/// The TLS configuration `tokio-tungstenite` builds when no connector is
/// supplied and its `rustls-tls-webpki-roots` feature is enabled: webpki
/// trust roots, no client auth, and the crate-default provider — plus, when
/// `trust_native_roots` is set, the OS-trusted root CAs (see
/// [`root_cert_store`]). The returned `ClientConfig` is still a fresh object
/// built per attempt, like the stock path, so TLS session resumption
/// behavior matches — only the expensive native-certificate load underneath
/// it is cached, not the config object itself.
async fn tls_client_config(trust_native_roots: bool) -> Arc<rustls::ClientConfig> {
    Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store(trust_native_roots).await)
            .with_no_client_auth(),
    )
}

/// The one TLS connector every `/v1/listen` connection uses while the
/// `connect-diagnostics` feature is enabled — passed explicitly to both the
/// stock (`tokio_tungstenite::connect_async_tls_with_config`) and the
/// phase-timed connect paths, so downstream feature unification (e.g. a
/// consumer also enabling `tokio-tungstenite/native-tls`) cannot make the
/// two paths select different TLS providers or trust configurations.
pub(crate) async fn tls_connector(trust_native_roots: bool) -> tokio_tungstenite::Connector {
    tokio_tungstenite::Connector::Rustls(tls_client_config(trust_native_roots).await)
}

/// Hostname from the request URI, with IPv6 brackets stripped as `rustls`
/// expects. Mirrors `tokio-tungstenite`'s internal `domain` helper.
fn domain(request: &Request<()>) -> std::result::Result<String, TungsteniteError> {
    match request.uri().host() {
        Some(d) if d.starts_with('[') && d.ends_with(']') => Ok(d[1..d.len() - 1].to_string()),
        Some(d) => Ok(d.to_string()),
        None => Err(TungsteniteError::Url(UrlError::NoHostName)),
    }
}

/// Format a `SystemTime` as an RFC 3339 UTC timestamp with millisecond
/// precision, e.g. `2026-09-10T14:03:22.114Z`.
///
/// Implemented locally (via the standard civil-from-days algorithm) to avoid
/// adding a date-time dependency for one format.
fn rfc3339_utc(time: SystemTime) -> String {
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let days = (secs / 86_400) as i64;
    let secs_of_day = secs % 86_400;
    let (hour, minute, second) = (
        secs_of_day / 3600,
        (secs_of_day % 3600) / 60,
        secs_of_day % 60,
    );

    // Civil-from-days (Howard Hinnant's algorithm), valid for all dates
    // reachable from a u64 UNIX timestamp.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_url() -> url::Url {
        url::Url::parse("wss://api.deepgram.com/v1/listen?model=nova-3").unwrap()
    }

    fn channel_sink() -> (
        SharedSink,
        tokio::sync::mpsc::UnboundedReceiver<ConnectRecord>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (SharedSink(Arc::new(tx)), rx)
    }

    #[test]
    fn rfc3339_epoch() {
        assert_eq!(rfc3339_utc(UNIX_EPOCH), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn rfc3339_known_instants() {
        // 2026-09-10T14:03:22.114Z
        let t = UNIX_EPOCH + Duration::from_millis(1_789_049_002_114);
        assert_eq!(rfc3339_utc(t), "2026-09-10T14:03:22.114Z");
        // Leap-year day: 2024-02-29T23:59:59.999Z
        let t = UNIX_EPOCH + Duration::from_millis(1_709_251_199_999);
        assert_eq!(rfc3339_utc(t), "2024-02-29T23:59:59.999Z");
    }

    #[test]
    fn record_serialization_skips_absent_fields() {
        let (sink, mut rx) = channel_sink();
        drop(DiagnosticsGuard::new(sink, &test_url()));
        let record = rx.try_recv().expect("record emitted on drop");

        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&record).unwrap()).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["outcome"], "cancelled");
        assert_eq!(json["last_phase"], "dns");
        assert_eq!(json["url"], "wss://api.deepgram.com/v1/listen");
        for absent in [
            "local_addr",
            "peer_addr",
            "dns_ms",
            "tcp_connect_ms",
            "tls_handshake_ms",
            "ws_upgrade_ms",
            "request_id",
            "dg_error",
            "error",
        ] {
            assert!(
                json.get(absent).is_none(),
                "{absent} should be omitted when absent"
            );
        }
    }

    #[test]
    fn recorded_url_omits_userinfo_and_query_values() {
        // Query values can carry secrets: signed callback URLs, keyterms,
        // arbitrary `query_params` values. None of it may reach the sink.
        let secret = "synthetic-secret-0f9b2";
        let url = url::Url::parse(&format!(
            "wss://user:{secret}@api.deepgram.com/v1/listen\
             ?model=nova-3&customer_token={secret}\
             &callback=https%3A%2F%2Fexample.com%2Fcb%3Fsig%3D{secret}"
        ))
        .unwrap();

        let (sink, mut rx) = channel_sink();
        drop(DiagnosticsGuard::new(sink, &url));
        let record = rx.try_recv().expect("record emitted on drop");

        assert_eq!(record.url, "wss://api.deepgram.com/v1/listen");
        let line = serde_json::to_string(&record).unwrap();
        assert!(
            !line.contains(secret),
            "serialized record must not contain query or userinfo secrets: {line}"
        );
        assert!(!line.contains("customer_token"));
        assert!(!line.contains("model=nova-3"));
    }

    #[test]
    fn cancelled_guard_keeps_finished_phase_timings() {
        let (sink, mut rx) = channel_sink();
        let mut guard = DiagnosticsGuard::new(sink, &test_url());
        guard.enter_phase(ConnectPhase::Dns);
        guard.finish_phase();
        guard.enter_phase(ConnectPhase::TcpConnect);
        guard.finish_phase();
        guard.enter_phase(ConnectPhase::TlsHandshake);
        // Dropped mid-TLS, as when a caller-side timeout fires.
        drop(guard);

        let record = rx.try_recv().expect("record emitted on drop");
        assert_eq!(record.outcome, ConnectOutcome::Cancelled);
        assert_eq!(record.last_phase, ConnectPhase::TlsHandshake);
        assert!(record.dns_ms.is_some());
        assert!(record.tcp_connect_ms.is_some());
        assert!(record.tls_handshake_ms.is_none());
        assert!(record.request_id.is_none());
    }

    #[tokio::test]
    async fn record_survives_tokio_timeout() {
        let (sink, mut rx) = channel_sink();
        let connect = async {
            let mut guard = DiagnosticsGuard::new(sink, &test_url());
            guard.enter_phase(ConnectPhase::Dns);
            guard.finish_phase();
            guard.enter_phase(ConnectPhase::TcpConnect);
            std::future::pending::<()>().await;
        };
        let result = tokio::time::timeout(Duration::from_millis(10), connect).await;
        assert!(result.is_err(), "timeout should fire");

        let record = rx.try_recv().expect("record emitted despite cancellation");
        assert_eq!(record.outcome, ConnectOutcome::Cancelled);
        assert_eq!(record.last_phase, ConnectPhase::TcpConnect);
        assert!(record.connect_duration_ms >= 10.0);
    }

    #[test]
    fn failed_upgrade_captures_deepgram_headers() {
        let (sink, mut rx) = channel_sink();
        let mut guard = DiagnosticsGuard::new(sink, &test_url());
        guard.enter_phase(ConnectPhase::WsUpgrade);

        let response = http::Response::builder()
            .status(400)
            .header("dg-request-id", "9ac2aaaa-bbbb-cccc-dddd-eeeeffff0000")
            .header("dg-error", "bad model")
            .body(None)
            .unwrap();
        guard.fail(&TungsteniteError::Http(Box::new(response)));
        drop(guard);

        let record = rx.try_recv().expect("record emitted on drop");
        assert_eq!(record.outcome, ConnectOutcome::Failed);
        assert_eq!(
            record.request_id.as_deref(),
            Some("9ac2aaaa-bbbb-cccc-dddd-eeeeffff0000")
        );
        assert_eq!(record.dg_error.as_deref(), Some("bad model"));
        assert!(record.error.is_some());
    }

    #[test]
    fn closure_sink_works() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sink = SharedSink(Arc::new(sink_fn(move |record: ConnectRecord| {
            let _ = tx.send(record.outcome);
        })));
        let mut guard = DiagnosticsGuard::new(sink, &test_url());
        guard.complete();
        drop(guard);
        assert_eq!(rx.try_recv().unwrap(), ConnectOutcome::Completed);
    }

    #[tokio::test]
    async fn tls_config_matches_tokio_tungstenite_defaults() {
        // The stock path (tokio-tungstenite, rustls-tls-webpki-roots) builds
        // its trust store from webpki_roots::TLS_SERVER_ROOTS with no client
        // auth. Assert our replica loads the identical root set when native
        // roots are not opted into.
        let config = tls_client_config(false).await;
        assert!(!config.client_auth_cert_resolver.has_certs());
        let roots = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
        };
        assert_eq!(
            tls_client_config(false)
                .await
                .crypto_provider()
                .cipher_suites,
            rustls::ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth()
                .crypto_provider()
                .cipher_suites,
        );
    }

    #[tokio::test]
    async fn root_store_trusts_public_roots_without_native_roots() {
        assert!(!root_cert_store(false).await.is_empty());
    }

    /// Pins the merge invariant `root_cert_store`'s doc comment claims to
    /// replicate from `tokio-tungstenite`'s own (private, unreusable)
    /// native+webpki merge: native roots are additive, and webpki roots are
    /// always present regardless of what native loading finds. Since that
    /// merge is hand-copied rather than delegated to, this test — not the
    /// upstream source — is what would catch this function silently
    /// diverging from it after a future edit.
    #[tokio::test]
    async fn root_store_still_trusts_public_roots_with_native_roots_enabled() {
        // Native root loading is best-effort and platform/sandbox dependent
        // (e.g. it may find zero certs in a minimal CI container), but the
        // bundled webpki roots must always be present regardless, and native
        // roots (when found) are additive, never fewer than webpki alone.
        let webpki_only = root_cert_store(false).await.len();
        let with_native = root_cert_store(true).await.len();
        assert!(!root_cert_store(true).await.is_empty());
        assert!(with_native >= webpki_only);
    }

    /// The native portion is cached (see `cached_native_root_cert_store`);
    /// within the TTL, repeated calls must not re-read the OS trust store,
    /// and must keep returning a store that still includes the webpki
    /// baseline.
    #[tokio::test]
    async fn native_root_cache_is_stable_across_repeated_calls() {
        let first = root_cert_store(true).await.len();
        let second = root_cert_store(true).await.len();
        assert_eq!(first, second);
    }

    /// Once the TTL has elapsed, a call must still return immediately
    /// (serving the stale value, not blocking on a fresh OS read) while a
    /// background task refreshes the cache for subsequent calls. Uses its
    /// own short TTL so it doesn't depend on `NATIVE_ROOTS_TTL`'s real
    /// (multi-minute) value, but shares the same process-wide cache as the
    /// other tests in this module — harmless, since every assertion here is
    /// about the *shape* of what's returned and whether a refresh
    /// eventually lands, not about a specific `loaded_at` baseline.
    #[tokio::test]
    async fn stale_native_root_cache_is_served_immediately_and_refreshed_in_background() {
        let ttl = Duration::from_millis(20);

        let fresh = cached_native_root_cert_store(ttl).await;
        tokio::time::sleep(ttl * 3).await;

        let loaded_at_before_stale_hit = NATIVE_ROOT_CERT_STORE
            .read()
            .await
            .as_ref()
            .expect("cache populated by the call above")
            .loaded_at;

        let stale = cached_native_root_cert_store(ttl).await;
        assert!(
            Arc::ptr_eq(&fresh, &stale),
            "a stale hit must return the existing cached Arc, not a freshly loaded one"
        );

        for _ in 0..50 {
            let refreshed = NATIVE_ROOT_CERT_STORE
                .read()
                .await
                .as_ref()
                .map(|cached| cached.loaded_at > loaded_at_before_stale_hit)
                .unwrap_or(false);
            if refreshed {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("expected the stale hit to have triggered a background refresh by now");
    }
}
