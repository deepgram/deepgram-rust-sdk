//! Diagnostics describing an established live WebSocket connection.
//!
//! These types surface connection metadata that is useful when logging or
//! troubleshooting live streaming requests -- for example, correlating a
//! Deepgram request ID with the local and remote socket addresses actually used
//! by a production connection.

use std::net::SocketAddr;
use std::time::Duration;

use tokio_tungstenite::MaybeTlsStream;
use uuid::Uuid;

/// Metadata captured from a successfully established live WebSocket connection.
///
/// This is available on both the low-level handle and the high-level stream for
/// live speech-to-text and Flux via their `connection_info()` accessors.
///
/// It only describes connections that completed the WebSocket upgrade: a
/// connection that fails or is cancelled (for example, by a caller-side connect
/// timeout) before the upgrade completes never produces a
/// `WebSocketConnectionInfo`.
#[derive(Debug, Clone)]
pub struct WebSocketConnectionInfo {
    /// The Deepgram request ID parsed from the `dg-request-id` upgrade header.
    pub request_id: Uuid,

    /// The final request URL used for the connection, including model and
    /// feature query parameters. Authentication is sent as a header and is not
    /// part of this URL.
    pub url: String,

    /// The local (source) socket address of the underlying TCP connection, if it
    /// could be determined.
    pub local_addr: Option<SocketAddr>,

    /// The resolved peer (destination) socket address of the underlying TCP
    /// connection, if it could be determined.
    pub peer_addr: Option<SocketAddr>,

    /// The total time taken to establish the connection, measured around the
    /// connect-and-upgrade call (DNS resolution, TCP connect, TLS handshake, and
    /// WebSocket upgrade combined).
    pub connect_duration: Duration,
}

/// Extract the local and peer socket addresses from an established WebSocket
/// stream's underlying TCP socket.
///
/// Returns `(None, None)` if the addresses can't be read (for example, an
/// unrecognized transport variant).
pub(crate) fn socket_addrs(
    stream: &MaybeTlsStream<tokio::net::TcpStream>,
) -> (Option<SocketAddr>, Option<SocketAddr>) {
    let tcp = match stream {
        MaybeTlsStream::Plain(tcp) => Some(tcp),
        MaybeTlsStream::Rustls(tls) => Some(tls.get_ref().0),
        // `MaybeTlsStream` is `#[non_exhaustive]`; any other transport variant
        // simply yields no addresses rather than failing the connection.
        _ => None,
    };

    match tcp {
        Some(tcp) => (tcp.local_addr().ok(), tcp.peer_addr().ok()),
        None => (None, None),
    }
}
