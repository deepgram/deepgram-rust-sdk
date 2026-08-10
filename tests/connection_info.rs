//! Mock WebSocket server tests verifying that `connection_info()` is populated
//! and propagated for live speech-to-text and Flux connections.
//!
//! Run with: cargo test --test connection_info --features listen

#[cfg(feature = "listen")]
mod mock {
    use std::net::SocketAddr;

    use deepgram::Deepgram;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite;

    const FAKE_REQUEST_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

    /// Spin up a local WebSocket server that completes the upgrade (returning a
    /// `dg-request-id` header) then closes. Returns the address to connect to.
    async fn mock_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();

            #[allow(clippy::result_large_err)]
            let callback =
                |_req: &tungstenite::handshake::server::Request,
                 mut resp: tungstenite::handshake::server::Response| {
                    resp.headers_mut()
                        .insert("dg-request-id", FAKE_REQUEST_ID.parse().unwrap());
                    Ok(resp)
                };

            let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
                .await
                .unwrap();

            futures::SinkExt::close(&mut ws).await.ok();
        });

        addr
    }

    fn make_client(addr: SocketAddr) -> Deepgram {
        let base_url = format!("ws://{}", addr);
        Deepgram::with_base_url(base_url.as_str()).unwrap()
    }

    #[tokio::test]
    async fn stt_handle_exposes_connection_info() {
        let addr = mock_server().await;
        let dg = make_client(addr);

        let handle = dg
            .transcription()
            .stream_request()
            .handle()
            .await
            .expect("failed to connect to mock server");

        let info = handle.connection_info();
        assert_eq!(info.request_id.to_string(), FAKE_REQUEST_ID);
        assert_eq!(info.request_id, handle.request_id());
        assert_eq!(info.peer_addr, Some(addr));
        assert!(info.local_addr.is_some(), "local_addr should be captured");
        assert!(
            info.url.starts_with("ws://127.0.0.1"),
            "url should be the final request URL, got {}",
            info.url
        );
    }

    #[tokio::test]
    async fn stt_stream_propagates_connection_info() {
        let addr = mock_server().await;
        let dg = make_client(addr);

        let audio = futures::stream::empty::<Result<bytes::Bytes, std::io::Error>>();
        let stream = dg
            .transcription()
            .stream_request()
            .stream(audio)
            .await
            .expect("failed to connect to mock server");

        let info = stream.connection_info();
        assert_eq!(info.request_id.to_string(), FAKE_REQUEST_ID);
        assert_eq!(info.request_id, stream.request_id());
        assert_eq!(info.peer_addr, Some(addr));
        assert!(info.local_addr.is_some(), "local_addr should be captured");
    }

    #[tokio::test]
    async fn flux_handle_exposes_connection_info() {
        let addr = mock_server().await;
        let dg = make_client(addr);

        let handle = dg
            .transcription()
            .flux_request()
            .handle()
            .await
            .expect("failed to connect to mock server");

        let info = handle.connection_info();
        assert_eq!(info.request_id.to_string(), FAKE_REQUEST_ID);
        assert_eq!(info.request_id, handle.request_id());
        assert_eq!(info.peer_addr, Some(addr));
        assert!(info.local_addr.is_some(), "local_addr should be captured");
    }
}
