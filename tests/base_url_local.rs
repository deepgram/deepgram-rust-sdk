//! Every management request, and the token grant, goes to the base URL the
//! client was built with.
//!
//! A client pointed at a proxy, a private deployment, or a mock server must
//! not send management or auth traffic — and the credential attached to it —
//! to `https://api.deepgram.com`. These tests point the client at a local
//! capture server and assert the request line every such method produces, so
//! a method that rebuilds a hardcoded URL fails here: its request never
//! reaches the capture server and the assertion times out.
//!
//! `deepgram::auth` is always compiled, so the auth test runs in every
//! feature combination; the management test is gated on `manage`.
//!
//! Run with: cargo test --test base_url_local --features manage

use std::net::SocketAddr;
use std::time::Duration;

use deepgram::Deepgram;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, UnboundedReceiver};

/// How long to wait for a request to arrive before declaring the method sent
/// it somewhere else.
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(10);

/// A minimal HTTP server that records the request line of every request it
/// receives and answers each one with `{}` on a connection it then closes.
///
/// The bodies the management methods deserialize are not the point here, so
/// every response is the same and the methods' return values are ignored;
/// what is asserted is what the server saw.
async fn capture_server() -> (SocketAddr, UnboundedReceiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let tx = tx.clone();

            tokio::spawn(async move {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 1024];
                loop {
                    match stream.read(&mut chunk).await {
                        Ok(0) | Err(_) => break,
                        Ok(n) => buf.extend_from_slice(&chunk[..n]),
                    }
                    if buf.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }

                let text = String::from_utf8_lossy(&buf);
                let request_line = text.lines().next().unwrap_or_default().to_owned();
                // "GET /v1/projects HTTP/1.1" -> "GET /v1/projects"
                let captured = match request_line.rfind(" HTTP/") {
                    Some(end) => request_line[..end].to_owned(),
                    None => request_line,
                };
                let _ = tx.send(captured);

                let response = "HTTP/1.1 200 OK\r\n\
                     content-type: application/json\r\n\
                     content-length: 2\r\n\
                     connection: close\r\n\
                     \r\n\
                     {}";
                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.flush().await;
            });
        }
    });

    (addr, rx)
}

/// Assert the next request the server saw, failing rather than hanging when
/// the method under test sent its request to some other host.
async fn assert_captured(rx: &mut UnboundedReceiver<String>, expected: &str) {
    let captured = tokio::time::timeout(CAPTURE_TIMEOUT, rx.recv())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "no request reached the capture server within {CAPTURE_TIMEOUT:?}; \
                 expected {expected:?}. The method under test did not use the \
                 client's configured base URL."
            )
        })
        .expect("capture server stopped");

    assert_eq!(captured, expected);
}

#[cfg(feature = "manage")]
#[tokio::test]
async fn every_management_request_uses_the_configured_base_url() {
    use deepgram::manage::{keys, projects, usage};

    let (addr, mut rx) = capture_server().await;
    let base_url = format!("http://{addr}");
    let dg = Deepgram::with_base_url_and_api_key(base_url.as_str(), "token").unwrap();

    // projects
    let _ = dg.projects().list().await;
    assert_captured(&mut rx, "GET /v1/projects").await;

    let _ = dg.projects().get("proj").await;
    assert_captured(&mut rx, "GET /v1/projects/proj").await;

    let options = projects::options::Options::builder()
        .name("renamed")
        .build();
    let _ = dg.projects().update("proj", &options).await;
    assert_captured(&mut rx, "PATCH /v1/projects/proj").await;

    let _ = dg.projects().delete("proj").await;
    assert_captured(&mut rx, "DELETE /v1/projects/proj").await;

    // keys
    let _ = dg.keys().list("proj").await;
    assert_captured(&mut rx, "GET /v1/projects/proj/keys").await;

    let _ = dg.keys().get("proj", "key").await;
    assert_captured(&mut rx, "GET /v1/projects/proj/keys/key").await;

    let options = keys::options::Options::builder("New Key", ["member"]).build();
    let _ = dg.keys().create("proj", &options).await;
    assert_captured(&mut rx, "POST /v1/projects/proj/keys").await;

    let _ = dg.keys().delete("proj", "key").await;
    assert_captured(&mut rx, "DELETE /v1/projects/proj/keys/key").await;

    // members
    let _ = dg.members().list_members("proj").await;
    assert_captured(&mut rx, "GET /v1/projects/proj/members").await;

    let _ = dg.members().remove_member("proj", "mem").await;
    assert_captured(&mut rx, "DELETE /v1/projects/proj/members/mem").await;

    // scopes
    let _ = dg.scopes().get_scope("proj", "mem").await;
    assert_captured(&mut rx, "GET /v1/projects/proj/members/mem/scopes").await;

    let _ = dg.scopes().update_scope("proj", "mem", "member").await;
    assert_captured(&mut rx, "PUT /v1/projects/proj/members/mem/scopes").await;

    // invitations
    let _ = dg.invitations().leave_project("proj").await;
    assert_captured(&mut rx, "DELETE /v1/projects/proj/leave").await;

    // billing
    let _ = dg.billing().list_balance("proj").await;
    assert_captured(&mut rx, "GET /v1/projects/proj/balances").await;

    let _ = dg.billing().get_balance("proj", "bal").await;
    assert_captured(&mut rx, "GET /v1/projects/proj/balances/bal").await;

    // usage
    let options = usage::list_requests_options::Options::builder().build();
    let _ = dg.usage().list_requests("proj", &options).await;
    assert_captured(&mut rx, "GET /v1/projects/proj/requests").await;

    let _ = dg.usage().get_request("proj", "req").await;
    assert_captured(&mut rx, "GET /v1/projects/proj/requests/req").await;

    let options = usage::get_usage_options::Options::builder().build();
    let _ = dg.usage().get_usage("proj", &options).await;
    assert_captured(&mut rx, "GET /v1/projects/proj/usage").await;

    let options = usage::get_fields_options::Options::builder().build();
    let _ = dg.usage().get_fields("proj", &options).await;
    assert_captured(&mut rx, "GET /v1/projects/proj/usage/fields").await;
}

/// A base URL that carries a path prefix keeps it, and query parameters are
/// still appended to the joined path.
#[cfg(feature = "manage")]
#[tokio::test]
async fn base_url_path_prefix_and_query_parameters_are_preserved() {
    use deepgram::manage::usage;

    let (addr, mut rx) = capture_server().await;
    let base_url = format!("http://{addr}/gateway/");
    let dg = Deepgram::with_base_url_and_api_key(base_url.as_str(), "token").unwrap();

    let _ = dg.projects().list().await;
    assert_captured(&mut rx, "GET /gateway/v1/projects").await;

    let options = usage::list_requests_options::Options::builder()
        .start("2024-01-01")
        .build();
    let _ = dg.usage().list_requests("proj", &options).await;
    assert_captured(
        &mut rx,
        "GET /gateway/v1/projects/proj/requests?start=2024-01-01",
    )
    .await;
}

/// `POST /v1/auth/grant` goes to the configured base URL too, with and
/// without a path prefix. `deepgram::auth` has no Cargo feature, so this runs
/// even with `--no-default-features`.
#[tokio::test]
async fn the_token_grant_uses_the_configured_base_url() {
    use deepgram::auth::options::Options;

    let (addr, mut rx) = capture_server().await;
    let base_url = format!("http://{addr}");
    let dg = Deepgram::with_base_url_and_api_key(base_url.as_str(), "token").unwrap();

    let _ = dg.auth().grant(None).await;
    assert_captured(&mut rx, "POST /v1/auth/grant").await;

    let options = Options::builder().ttl_seconds(300.0).build();
    let _ = dg.auth().grant(Some(&options)).await;
    assert_captured(&mut rx, "POST /v1/auth/grant").await;

    let prefixed = format!("http://{addr}/gateway/");
    let dg = Deepgram::with_base_url_and_api_key(prefixed.as_str(), "token").unwrap();

    let _ = dg.auth().grant(None).await;
    assert_captured(&mut rx, "POST /gateway/v1/auth/grant").await;
}
