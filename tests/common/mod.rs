//! Shared helpers for the local TLS tests: a self-signed certificate and a
//! localhost `wss://` server that completes the WebSocket upgrade with a
//! `dg-request-id`, which is all any of the SDK's WebSocket handles need to
//! construct successfully.

#![allow(dead_code)]

use std::sync::Arc;

use deepgram::Deepgram;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_tungstenite::tungstenite;

pub const FAKE_REQUEST_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

/// A self-signed certificate for `localhost`. Nothing in the bundled webpki
/// roots trusts it, which makes it a stand-in for any privately issued
/// certificate: a TLS-inspecting proxy's, an internal CA's, a self-hosted
/// deployment's.
pub struct SelfSigned {
    pub cert_der: Vec<u8>,
    pub key_der: Vec<u8>,
    pub cert_pem: String,
}

pub fn self_signed() -> SelfSigned {
    let cert =
        rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).expect("generate cert");
    SelfSigned {
        cert_der: cert.cert.der().to_vec(),
        key_der: cert.key_pair.serialize_der(),
        cert_pem: cert.cert.pem(),
    }
}

/// Bind a localhost TLS listener presenting the given certificate. Every
/// accepted connection gets a completed WebSocket upgrade carrying
/// `dg-request-id`, then is closed. A client that does not trust the
/// certificate aborts during the TLS handshake instead; the server just
/// moves on.
pub async fn spawn_tls_server(cert_der: Vec<u8>, key_der: Vec<u8>) -> u16 {
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
                let Ok(tls_stream) = acceptor.accept(stream).await else {
                    return;
                };
                // The callback signature (and its large `ErrorResponse` Err
                // variant) is fixed by tungstenite's accept_hdr API.
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

/// A client pointed at the local server. `https` → `wss`: every WebSocket
/// connect performs a real TLS handshake.
pub fn client(port: u16) -> Deepgram {
    Deepgram::with_base_url_and_api_key(format!("https://localhost:{port}").as_str(), "fake-key")
        .expect("client")
}

/// A rustls config that trusts exactly one certificate, built from the
/// SDK's re-exported `rustls` so the version matches.
pub fn config_trusting(cert_der: &[u8]) -> deepgram::rustls::ClientConfig {
    let mut roots = deepgram::rustls::RootCertStore::empty();
    roots
        .add(deepgram::rustls::pki_types::CertificateDer::from(
            cert_der.to_vec(),
        ))
        .expect("add root");
    deepgram::rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth()
}

/// A self-signed certificate for `localhost` with its own subject name, so
/// it is unrelated to [`self_signed`]'s output the way a genuinely different
/// CA would be. (Two rcgen defaults share a subject name; webpki then finds
/// a same-named anchor and reports `BadSignature` rather than the
/// `UnknownIssuer` a truly foreign issuer produces.)
pub fn self_signed_named(common_name: &str) -> SelfSigned {
    let key_pair = rcgen::KeyPair::generate().expect("key pair");
    let mut params = rcgen::CertificateParams::new(vec!["localhost".to_string()]).expect("params");
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, common_name);
    let cert = params.self_signed(&key_pair).expect("self-sign");
    SelfSigned {
        cert_der: cert.der().to_vec(),
        key_der: key_pair.serialize_der(),
        cert_pem: cert.pem(),
    }
}
