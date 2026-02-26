//! Mock WebSocket server tests that verify FluxResponse handles unknown message
//! types gracefully without breaking the stream.
//!
//! Run with: cargo test --test flux_unknown_messages --features listen

#[cfg(feature = "listen")]
mod mock {
    use std::net::SocketAddr;

    use deepgram::{
        common::flux_response::{FluxResponse, TurnEvent},
        Deepgram,
    };
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::{self, protocol::Message};

    const FAKE_REQUEST_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

    /// Spin up a local WebSocket server that sends the given JSON messages
    /// then closes. Returns the address to connect to.
    async fn mock_flux_server(messages: Vec<String>) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();

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

            for msg in messages {
                futures::SinkExt::send(&mut ws, Message::Text(msg.into()))
                    .await
                    .unwrap();
            }

            futures::SinkExt::close(&mut ws).await.ok();
        });

        addr
    }

    fn make_client(addr: SocketAddr) -> Deepgram {
        let base_url = format!("ws://{}", addr);
        Deepgram::with_base_url(base_url.as_str()).unwrap()
    }

    #[tokio::test]
    async fn unknown_message_type_does_not_break_stream() {
        let messages = vec![
            // 1. Normal Connected
            format!(
                r#"{{"type":"Connected","request_id":"{}","sequence_id":0}}"#,
                FAKE_REQUEST_ID
            ),
            // 2. Completely unknown message type
            r#"{"type":"SomeFutureFeature","version":2,"payload":{"nested":true}}"#.to_string(),
            // 3. Another unknown type
            r#"{"type":"DebugInfo","latency_ms":42}"#.to_string(),
            // 4. Normal TurnInfo with a known event
            format!(
                r#"{{"type":"TurnInfo","request_id":"{}","sequence_id":1,"event":"StartOfTurn","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"","words":[],"end_of_turn_confidence":0.0}}"#,
                FAKE_REQUEST_ID
            ),
            // 5. TurnInfo with an unknown event type
            format!(
                r#"{{"type":"TurnInfo","request_id":"{}","sequence_id":2,"event":"BrandNewEvent","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"hello","words":[],"end_of_turn_confidence":0.5}}"#,
                FAKE_REQUEST_ID
            ),
        ];

        let addr = mock_flux_server(messages).await;
        let dg = make_client(addr);

        let mut handle = dg
            .transcription()
            .flux_request()
            .handle()
            .await
            .expect("failed to connect to mock server");

        let mut received: Vec<String> = Vec::new();

        while let Some(result) = handle.receive().await {
            let response = result.expect("stream should not error on unknown messages");
            match &response {
                FluxResponse::Connected { .. } => received.push("Connected".into()),
                FluxResponse::TurnInfo { event, .. } => {
                    received.push(format!("TurnInfo:{:?}", event));
                }
                FluxResponse::FatalError { .. } => received.push("FatalError".into()),
                FluxResponse::Unknown(val) => {
                    received.push(format!("Unknown:{}", val["type"].as_str().unwrap_or("?")));
                }
                _ => received.push("Other".into()),
            }
        }

        assert_eq!(
            received,
            vec![
                "Connected",
                "Unknown:SomeFutureFeature",
                "Unknown:DebugInfo",
                "TurnInfo:StartOfTurn",
                "TurnInfo:Unknown",
            ]
        );
    }

    #[tokio::test]
    async fn message_with_no_type_field_becomes_unknown() {
        let messages = vec![
            format!(
                r#"{{"type":"Connected","request_id":"{}","sequence_id":0}}"#,
                FAKE_REQUEST_ID
            ),
            // Message with no "type" field at all
            r#"{"event":"something","data":123}"#.to_string(),
        ];

        let addr = mock_flux_server(messages).await;
        let dg = make_client(addr);
        let mut handle = dg.transcription().flux_request().handle().await.unwrap();

        let msg1 = handle.receive().await.unwrap().unwrap();
        assert!(matches!(msg1, FluxResponse::Connected { .. }));

        let msg2 = handle.receive().await.unwrap().unwrap();
        match msg2 {
            FluxResponse::Unknown(val) => {
                assert_eq!(val["event"], "something");
                assert_eq!(val["data"], 123);
            }
            other => panic!("expected Unknown, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn unknown_messages_interleaved_with_real_turn_data() {
        let messages = vec![
            format!(
                r#"{{"type":"Connected","request_id":"{}","sequence_id":0}}"#,
                FAKE_REQUEST_ID
            ),
            // Unknown before turn data
            r#"{"type":"Heartbeat","ts":1234567890}"#.to_string(),
            // Start of turn
            format!(
                r#"{{"type":"TurnInfo","request_id":"{}","sequence_id":1,"event":"StartOfTurn","turn_index":0,"audio_window_start":0.0,"audio_window_end":0.5,"transcript":"","words":[],"end_of_turn_confidence":0.0}}"#,
                FAKE_REQUEST_ID
            ),
            // Unknown between updates
            r#"{"type":"Analytics","word_count":5}"#.to_string(),
            // Update
            format!(
                r#"{{"type":"TurnInfo","request_id":"{}","sequence_id":2,"event":"Update","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"hello world","words":[{{"word":"hello","confidence":0.99}},{{"word":"world","confidence":0.95}}],"end_of_turn_confidence":0.3}}"#,
                FAKE_REQUEST_ID
            ),
            // End of turn
            format!(
                r#"{{"type":"TurnInfo","request_id":"{}","sequence_id":3,"event":"EndOfTurn","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.5,"transcript":"hello world","words":[{{"word":"hello","confidence":0.99}},{{"word":"world","confidence":0.95}}],"end_of_turn_confidence":0.95}}"#,
                FAKE_REQUEST_ID
            ),
        ];

        let addr = mock_flux_server(messages).await;
        let dg = make_client(addr);
        let mut handle = dg.transcription().flux_request().handle().await.unwrap();

        let mut turn_events: Vec<TurnEvent> = Vec::new();
        let mut unknown_count = 0u32;
        let mut final_transcript = String::new();

        while let Some(result) = handle.receive().await {
            let response = result.unwrap();
            match response {
                FluxResponse::Connected { .. } => {}
                FluxResponse::TurnInfo {
                    event, transcript, ..
                } => {
                    if event == TurnEvent::EndOfTurn {
                        final_transcript = transcript;
                    }
                    turn_events.push(event);
                }
                FluxResponse::Unknown(_) => unknown_count += 1,
                _ => {}
            }
        }

        assert_eq!(unknown_count, 2, "should have received 2 unknown messages");
        assert_eq!(
            turn_events,
            vec![
                TurnEvent::StartOfTurn,
                TurnEvent::Update,
                TurnEvent::EndOfTurn
            ]
        );
        assert_eq!(final_transcript, "hello world");
    }
}
