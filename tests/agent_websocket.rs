//! Localhost mock-server tests for the Voice Agent WebSocket client.
//!
//! These exercise connection-level behavior that unit tests on the message
//! types cannot: fair scheduling between inbound frames and outbound
//! commands, prompt failure of retained handles after the session ends, the
//! TLS rule on `start_at_url`, and the `Host` header sent on the handshake.
//!
//! Run with: cargo test --test agent_websocket --features agent

#[cfg(feature = "agent")]
mod mock {
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use deepgram::agent::messages::FunctionCallResponseMessage;
    use deepgram::agent::{AgentEvent, AgentResponse, InsecureAgentUrl};
    use deepgram::{Deepgram, DeepgramError};
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio::sync::{mpsc, oneshot};
    use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
    use tokio_tungstenite::tungstenite::protocol::CloseFrame;
    use tokio_tungstenite::tungstenite::{self, protocol::Message};

    const WELCOME: &str =
        r#"{"type":"Welcome","request_id":"550e8400-e29b-41d4-a716-446655440000"}"#;

    /// Upper bound for "delivered promptly" assertions. Generous so CI
    /// jitter cannot flake the tests; the failure modes under test are
    /// indefinite starvation / indefinite acceptance, not slowness.
    const BOUND: Duration = Duration::from_secs(5);

    fn client() -> Deepgram {
        Deepgram::new("test-api-key").unwrap()
    }

    async fn bind() -> (TcpListener, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        (listener, addr)
    }

    fn agent_url(addr: SocketAddr) -> String {
        format!("ws://{addr}/v1/agent/converse")
    }

    /// Accept one connection, then flood the client with binary audio
    /// frames as fast as the socket allows while forwarding every inbound
    /// text frame to `text_tx`.
    async fn flooding_server(listener: TcpListener, text_tx: mpsc::UnboundedSender<String>) {
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut sink, mut source) = ws.split();

        let flood = tokio::spawn(async move {
            // 20ms of 16kHz linear16 audio per frame.
            let frame = vec![0u8; 640];
            loop {
                if sink
                    .send(Message::Binary(frame.clone().into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        while let Some(Ok(msg)) = source.next().await {
            if let Message::Text(text) = msg {
                let _ = text_tx.send(text.to_string());
            }
        }
        flood.abort();
    }

    /// Accept one connection, capture the `Host` header from the upgrade
    /// request, send `Welcome`, then close from the server side.
    async fn closing_server(listener: TcpListener, host_tx: oneshot::Sender<String>) {
        let (stream, _) = listener.accept().await.unwrap();
        let mut host_tx = Some(host_tx);

        #[allow(clippy::result_large_err)]
        let callback = |req: &tungstenite::handshake::server::Request,
                        resp: tungstenite::handshake::server::Response| {
            let host = req
                .headers()
                .get("host")
                .and_then(|h| h.to_str().ok())
                .unwrap_or_default()
                .to_owned();
            if let Some(tx) = host_tx.take() {
                let _ = tx.send(host);
            }
            Ok(resp)
        };

        let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .unwrap();
        ws.send(Message::Text(WELCOME.into())).await.unwrap();
        ws.close(None).await.ok();
        // Drain until the client's close handshake completes.
        while let Some(Ok(_)) = ws.next().await {}
    }

    /// Accept one connection, send `Welcome`, then close from the server
    /// side with an explicit close frame carrying `code` and `reason`.
    async fn closing_with_frame_server(
        listener: TcpListener,
        code: CloseCode,
        reason: &'static str,
    ) {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        ws.send(Message::Text(WELCOME.into())).await.unwrap();
        ws.close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))
        .await
        .ok();
        // Drain until the client's close handshake completes.
        while let Some(Ok(_)) = ws.next().await {}
    }

    /// Accept one connection, send `Welcome`, and forward every inbound
    /// text frame to `text_tx` verbatim until the client goes away.
    async fn recording_server(listener: TcpListener, text_tx: mpsc::UnboundedSender<String>) {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        ws.send(Message::Text(WELCOME.into())).await.unwrap();
        while let Some(Ok(msg)) = ws.next().await {
            if let Message::Text(text) = msg {
                let _ = text_tx.send(text.to_string());
            }
        }
    }

    /// Accept one connection and emit a JSON event every few milliseconds
    /// until the client goes away.
    async fn chatty_server(listener: TcpListener) {
        let (stream, _) = listener.accept().await.unwrap();
        let ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let (mut sink, mut source) = ws.split();
        let reader = tokio::spawn(async move { while let Some(Ok(_)) = source.next().await {} });
        loop {
            if sink
                .send(Message::Text(r#"{"type":"UserStartedSpeaking"}"#.into()))
                .await
                .is_err()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        reader.abort();
    }

    // ---------- B5: fair scheduling ----------

    /// Guards bounded delivery of outbound controls while the server
    /// streams audio without pause.
    ///
    /// Note on what this does and does not prove: the worker previously
    /// used `select_biased!` favoring the inbound socket. On tokio that
    /// bias is *bounded* rather than indefinite — cooperative budgeting
    /// forces the socket branch to yield every 128 resource ops — so this
    /// test passes against both the old and the new worker. The switch to
    /// unbiased `select!` removes the dependence on that runtime detail;
    /// this test exists so a future change that blocks or serializes the
    /// outbound path behind inbound reads fails loudly.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn outbound_controls_are_delivered_under_continuous_inbound_audio() {
        let (listener, addr) = bind().await;
        let (text_tx, mut text_rx) = mpsc::unbounded_channel();
        tokio::spawn(flooding_server(listener, text_tx));

        let dg = client();
        let (mut handle, mut events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();

        // Honor the send-and-drain contract: keep consuming inbound audio
        // on another task and count the frames so we can prove the flood
        // was actually running while we sent.
        let frames = Arc::new(AtomicUsize::new(0));
        let drain = {
            let frames = Arc::clone(&frames);
            tokio::spawn(async move {
                while let Some(Ok(AgentEvent::Audio(_))) = events.next().await {
                    frames.fetch_add(1, Ordering::Relaxed);
                }
            })
        };

        // Let the flood establish itself.
        tokio::time::sleep(Duration::from_millis(200)).await;
        let before = frames.load(Ordering::Relaxed);
        assert!(before > 0, "server flood never reached the client");

        // Each control must reach the server within the bound even though
        // the inbound side is continuously readable.
        for i in 0..10 {
            handle.keep_alive().await.unwrap();
            let got = tokio::time::timeout(BOUND, text_rx.recv())
                .await
                .unwrap_or_else(|_| panic!("KeepAlive #{i} starved by inbound audio"))
                .expect("server text channel open");
            assert_eq!(got, r#"{"type":"KeepAlive"}"#);
        }

        handle
            .send_function_call_response(FunctionCallResponseMessage::with_id(
                "fc_1",
                "get_weather",
                r#"{"temp":72}"#,
            ))
            .await
            .unwrap();
        let got = tokio::time::timeout(BOUND, text_rx.recv())
            .await
            .expect("FunctionCallResponse starved by inbound audio")
            .expect("server text channel open");
        assert!(
            got.contains(r#""type":"FunctionCallResponse""#),
            "got: {got}"
        );
        assert!(got.contains(r#""id":"fc_1""#), "got: {got}");

        // Binary audio in the other direction is subject to the same rule.
        handle.send_data(vec![1, 2, 3, 4]).await.unwrap();
        handle.keep_alive().await.unwrap();
        let got = tokio::time::timeout(BOUND, text_rx.recv())
            .await
            .expect("KeepAlive after audio starved by inbound audio")
            .expect("server text channel open");
        assert_eq!(got, r#"{"type":"KeepAlive"}"#);

        assert!(
            frames.load(Ordering::Relaxed) > before,
            "inbound audio should have kept flowing while we sent"
        );

        handle.close().await.unwrap();
        let _ = tokio::time::timeout(BOUND, drain).await;
    }

    // ---------- B6: terminal shutdown ----------

    #[tokio::test]
    async fn retained_handle_fails_sends_after_server_close() {
        let (listener, addr) = bind().await;
        let (host_tx, _host_rx) = oneshot::channel();
        tokio::spawn(closing_server(listener, host_tx));

        let dg = client();
        let (mut handle, mut events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();

        // Drain to end-of-stream. The server sent Welcome then Close.
        let mut saw_welcome = false;
        while let Some(event) = tokio::time::timeout(BOUND, events.next())
            .await
            .expect("stream should end after server close")
        {
            if let Ok(AgentEvent::Json(AgentResponse::Welcome(_))) = event {
                saw_welcome = true;
            }
        }
        assert!(saw_welcome, "expected the Welcome event before close");

        // The worker closed the command channel before ending the event
        // stream, so a retained handle fails immediately — no send is
        // accepted into a dead session.
        assert!(handle.keep_alive().await.is_err());
        assert!(handle.send_data(vec![0u8; 320]).await.is_err());
        assert!(handle.force_end_turn().await.is_err());
        // Closing an already-ended session is a no-op, not an error.
        assert!(handle.close().await.is_ok());
    }

    #[tokio::test]
    async fn retained_handle_fails_sends_after_event_stream_dropped() {
        let (listener, addr) = bind().await;
        tokio::spawn(chatty_server(listener));

        let dg = client();
        let (mut handle, events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();
        drop(events);

        // The worker notices the dropped consumer on its next inbound
        // delivery, closes the command channel, and exits. From then on the
        // retained handle must fail rather than accept sends forever.
        let deadline = Instant::now() + BOUND;
        loop {
            match handle.keep_alive().await {
                Err(_) => break,
                Ok(()) => {
                    assert!(
                        Instant::now() < deadline,
                        "handle kept accepting sends after the event stream was dropped"
                    );
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
        assert!(handle.send_data(vec![0u8; 320]).await.is_err());
        assert!(handle.close().await.is_ok());
    }

    // ---------- close-frame semantics ----------

    /// A normal close (code 1000) — how the server ends every session,
    /// including at the 2-hour cap — is end-of-stream, not an error, even
    /// when the frame carries a reason body.
    #[tokio::test]
    async fn normal_close_with_reason_ends_stream_without_error() {
        let (listener, addr) = bind().await;
        tokio::spawn(closing_with_frame_server(
            listener,
            CloseCode::Normal,
            "session complete",
        ));

        let dg = client();
        let (_handle, mut events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();

        let mut saw_welcome = false;
        while let Some(event) = tokio::time::timeout(BOUND, events.next())
            .await
            .expect("stream should end after the normal close")
        {
            match event {
                Ok(AgentEvent::Json(AgentResponse::Welcome(_))) => saw_welcome = true,
                Ok(other) => panic!("unexpected event before close: {other:?}"),
                Err(err) => panic!("a code-1000 close must not surface as an error: {err}"),
            }
        }
        assert!(saw_welcome, "expected the Welcome event before close");
    }

    /// An abnormal close code is still surfaced as `WebsocketClose` with
    /// the server's code and reason, then the stream ends.
    #[tokio::test]
    async fn abnormal_close_is_surfaced_as_error_then_stream_ends() {
        let (listener, addr) = bind().await;
        tokio::spawn(closing_with_frame_server(
            listener,
            CloseCode::Policy,
            "client message timeout",
        ));

        let dg = client();
        let (_handle, mut events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();

        let mut close_errors = Vec::new();
        while let Some(event) = tokio::time::timeout(BOUND, events.next())
            .await
            .expect("stream should end after the abnormal close")
        {
            if let Err(err) = event {
                close_errors.push(err);
            }
        }
        assert_eq!(
            close_errors.len(),
            1,
            "exactly one error item: {close_errors:?}"
        );
        match &close_errors[0] {
            DeepgramError::WebsocketClose { code, reason } => {
                assert_eq!(*code, 1008);
                assert_eq!(reason, "client message timeout");
            }
            other => panic!("expected WebsocketClose, got {other:?}"),
        }
    }

    // ---------- send_raw_json escape hatch ----------

    /// A raw JSON value reaches the server verbatim, on the same channel
    /// (and therefore in order) with the typed messages around it.
    #[tokio::test]
    async fn send_raw_json_reaches_server_verbatim() {
        let (listener, addr) = bind().await;
        let (text_tx, mut text_rx) = mpsc::unbounded_channel::<String>();
        tokio::spawn(recording_server(listener, text_tx));

        let dg = client();
        let (mut handle, mut events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();
        // Wait for Welcome so the server is definitely reading.
        let first = tokio::time::timeout(BOUND, events.next())
            .await
            .expect("welcome within bound")
            .expect("stream open");
        assert!(matches!(
            first,
            Ok(AgentEvent::Json(AgentResponse::Welcome(_)))
        ));

        let raw = serde_json::json!({
            "type": "Settings",
            "agent": {
                "think": {
                    "provider": { "type": "nvidia", "model": "nemotron-3-nano-30B-A3B" },
                    "some_future_field": [1, 2, 3]
                }
            }
        });
        handle.keep_alive().await.unwrap();
        handle.send_raw_json(raw.clone()).await.unwrap();
        handle.keep_alive().await.unwrap();

        let mut received = Vec::new();
        for _ in 0..3 {
            let text = tokio::time::timeout(BOUND, text_rx.recv())
                .await
                .expect("server should receive every frame within bound")
                .expect("server alive");
            received.push(text);
        }
        assert_eq!(received[0], r#"{"type":"KeepAlive"}"#);
        assert_eq!(received[2], r#"{"type":"KeepAlive"}"#);
        // Byte-for-byte what serde_json emits for the value the caller gave us,
        // and structurally identical to it.
        assert_eq!(received[1], serde_json::to_string(&raw).unwrap());
        let parsed: serde_json::Value = serde_json::from_str(&received[1]).unwrap();
        assert_eq!(parsed, raw);
        assert_eq!(parsed["agent"]["think"]["provider"]["type"], "nvidia");

        handle.close().await.unwrap();
    }

    // ---------- B8: TLS rule on start_at_url ----------

    #[tokio::test]
    async fn start_at_url_allows_cleartext_loopback_ip() {
        let (listener, addr) = bind().await;
        let (host_tx, _host_rx) = oneshot::channel();
        tokio::spawn(closing_server(listener, host_tx));

        let dg = client();
        let (_handle, mut events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();
        let first = tokio::time::timeout(BOUND, events.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(first, AgentEvent::Json(AgentResponse::Welcome(_))));
    }

    #[tokio::test]
    async fn start_at_url_allows_cleartext_localhost_name() {
        let (listener, addr) = bind().await;
        let (host_tx, host_rx) = oneshot::channel();
        tokio::spawn(closing_server(listener, host_tx));

        let dg = client();
        let url = format!("ws://localhost:{}/v1/agent/converse", addr.port());
        let (_handle, mut events) = dg.agent().start_at_url(&url).await.unwrap();
        let first = tokio::time::timeout(BOUND, events.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert!(matches!(first, AgentEvent::Json(AgentResponse::Welcome(_))));
        assert_eq!(host_rx.await.unwrap(), format!("localhost:{}", addr.port()));
    }

    #[tokio::test]
    async fn start_at_url_rejects_remote_cleartext_before_connecting() {
        let dg = client();
        // Port 9 (discard) on a documentation-range address: if the SDK
        // tried to connect, this would hang or fail with a transport error
        // rather than the typed rejection asserted below.
        for url in [
            "ws://203.0.113.10:9/v1/agent/converse",
            "ws://agent.deepgram.com/v1/agent/converse",
            "ws://[2001:db8::10]:9/v1/agent/converse",
        ] {
            let started = Instant::now();
            let err = dg.agent().start_at_url(url).await.expect_err(url);
            assert!(
                started.elapsed() < Duration::from_secs(1),
                "{url}: rejection should not involve the network"
            );
            match &err {
                DeepgramError::InternalClientError(inner) => {
                    let insecure = inner
                        .downcast_ref::<InsecureAgentUrl>()
                        .unwrap_or_else(|| panic!("{url}: expected InsecureAgentUrl, got {inner}"));
                    assert!(insecure.url.starts_with("ws://"), "{url}");
                }
                other => panic!("{url}: expected InternalClientError, got {other:?}"),
            }
            let text = err.to_string();
            assert!(text.contains("cleartext"), "{url}: {text}");
            assert!(text.contains("wss://"), "{url}: {text}");
        }
    }

    #[tokio::test]
    async fn start_at_url_wss_passes_the_tls_rule() {
        // We cannot stand up a TLS server here, so assert that a `wss://`
        // URL to a loopback port with no listener gets *past* the URL check
        // and fails at the transport layer instead of with InsecureAgentUrl.
        let dg = client();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let err = dg
            .agent()
            .start_at_url(&format!("wss://127.0.0.1:{port}/v1/agent/converse"))
            .await
            .expect_err("no TLS listener, so the connection must fail");
        assert!(
            !matches!(&err, DeepgramError::InternalClientError(inner)
                if inner.downcast_ref::<InsecureAgentUrl>().is_some()),
            "wss:// must not be rejected by the TLS rule, got {err}"
        );
        assert!(!matches!(err, DeepgramError::InvalidUrl), "got {err}");
    }

    // ---------- S1: Host header ----------

    #[tokio::test]
    async fn handshake_host_header_includes_non_default_port() {
        let (listener, addr) = bind().await;
        let (host_tx, host_rx) = oneshot::channel();
        tokio::spawn(closing_server(listener, host_tx));

        let dg = client();
        let (_handle, _events) = dg.agent().start_at_url(&agent_url(addr)).await.unwrap();
        assert_eq!(
            tokio::time::timeout(BOUND, host_rx).await.unwrap().unwrap(),
            format!("127.0.0.1:{}", addr.port())
        );
    }
}
