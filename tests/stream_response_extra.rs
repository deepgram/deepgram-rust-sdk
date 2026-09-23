//! Deserialization tests for the streaming `Metadata` (terminal) message,
//! exercised from outside the defining module so they prove the `extra`
//! field is readable by SDK consumers (PR #164 review, B1).
//!
//! Run with: cargo test --test stream_response_extra --features listen

#![cfg(feature = "listen")]

use deepgram::common::stream_response::StreamResponse;

const TERMINAL_WITH_EXTRA: &str = r#"{
    "type": "Metadata",
    "transaction_key": "deprecated",
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "sha256": "abc123",
    "created": "2026-09-11T10:00:00.000Z",
    "duration": 12.5,
    "channels": 1,
    "extra": {
        "customer_id": "cust-42",
        "session": "abc"
    }
}"#;

const TERMINAL_WITHOUT_EXTRA: &str = r#"{
    "type": "Metadata",
    "transaction_key": "deprecated",
    "request_id": "550e8400-e29b-41d4-a716-446655440000",
    "sha256": "abc123",
    "created": "2026-09-11T10:00:00.000Z",
    "duration": 12.5,
    "channels": 1
}"#;

#[test]
fn terminal_response_exposes_extra() {
    let response: StreamResponse = serde_json::from_str(TERMINAL_WITH_EXTRA).unwrap();

    match response {
        StreamResponse::TerminalResponse {
            request_id, extra, ..
        } => {
            assert_eq!(request_id, "550e8400-e29b-41d4-a716-446655440000");
            let extra = extra.expect("extra should be present");
            assert_eq!(extra.len(), 2);
            assert_eq!(
                extra.get("customer_id").map(String::as_str),
                Some("cust-42")
            );
            assert_eq!(extra.get("session").map(String::as_str), Some("abc"));
        }
        other => panic!("expected TerminalResponse, got {other:?}"),
    }
}

#[test]
fn terminal_response_without_extra_is_none() {
    let response: StreamResponse = serde_json::from_str(TERMINAL_WITHOUT_EXTRA).unwrap();

    match response {
        StreamResponse::TerminalResponse {
            duration,
            channels,
            extra,
            ..
        } => {
            assert_eq!(duration, 12.5);
            assert_eq!(channels, 1);
            assert!(extra.is_none());
        }
        other => panic!("expected TerminalResponse, got {other:?}"),
    }
}

#[test]
fn terminal_response_extra_round_trips() {
    let response: StreamResponse = serde_json::from_str(TERMINAL_WITH_EXTRA).unwrap();
    let json = serde_json::to_value(&response).unwrap();

    assert_eq!(json["extra"]["customer_id"], "cust-42");
    assert_eq!(json["extra"]["session"], "abc");

    let without: StreamResponse = serde_json::from_str(TERMINAL_WITHOUT_EXTRA).unwrap();
    let json = serde_json::to_value(&without).unwrap();
    assert!(
        json.get("extra").is_none(),
        "absent extra must not serialize as null"
    );
}
