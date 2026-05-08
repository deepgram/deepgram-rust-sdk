//! Server-to-client event types for the Speak WebSocket.
//!
//! Mirrors the `SpeakV1*Event` schemas in
//! `asyncapi/schemas/schemas.speak.v1.yml`. [`SpeakResponse`] is the
//! discriminated union over every JSON event the streaming TTS server
//! can emit, plus an [`SpeakResponse::Unknown`] catch-all for
//! forward-compatibility with future event types.
//!
//! Wire dispatch is structural via each variant's own `type` field —
//! same approach as [`crate::agent::response::AgentResponse`]. Audio
//! frames are binary WebSocket frames, surfaced via the
//! [`SpeakResponse::Audio`] variant by the connection layer.

use bytes::Bytes;
use serde::{Deserialize, Serialize};

/// Discriminated union of every server-emitted Speak WebSocket event.
///
/// Variants are tried in order during JSON deserialization; the
/// [`SpeakResponse::Unknown`] tail variant matches anything that didn't
/// fit a typed shape and exposes the raw JSON for inspection or logging.
//
// Same `large_enum_variant` rationale as `AgentResponse`: events are
// constructed once during deserialization and immediately consumed by
// a match — boxing the largest variants would only add a heap
// allocation per received WebSocket frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum SpeakResponse {
    /// A raw binary audio frame.
    ///
    /// JSON deserialization will never produce this variant — it's
    /// constructed by the connection layer from binary WebSocket frames.
    /// We list it on the enum so audio chunks can be delivered on the
    /// same stream as JSON events.
    #[serde(skip)]
    Audio(Bytes),
    /// Metadata about the current generation, sent shortly after the handshake.
    Metadata(MetadataEvent),
    /// `Flush` was applied; emitted with a sequence ID.
    Flushed(FlushedEvent),
    /// `Clear` was applied; emitted with a sequence ID.
    Cleared(ClearedEvent),
    /// Non-fatal warning.
    Warning(WarningEvent),
    /// Forward-compatibility catch-all for unknown JSON events.
    Unknown(serde_json::Value),
}

// ---------- Metadata ----------

/// Marker for the `"Metadata"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MetadataType {
    /// Always serializes as `"Metadata"`.
    #[default]
    Metadata,
}

/// Mirrors `SpeakV1MetadataEvent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MetadataEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: MetadataType,

    /// Unique identifier for the streaming request.
    pub request_id: String,

    /// Name of the model serving the request.
    pub model_name: String,

    /// Version of the primary model.
    pub model_version: String,

    /// Unique identifier for the primary model.
    pub model_uuid: String,

    /// Identifiers for any additional models used to serve the request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_model_uuids: Vec<String>,
}

// ---------- Flushed ----------

/// Marker for the `"Flushed"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FlushedType {
    /// Always serializes as `"Flushed"`.
    #[default]
    Flushed,
}

/// Mirrors `SpeakV1Flushed` — confirms a `Flush` control message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FlushedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: FlushedType,

    /// Sequence ID of this control acknowledgement.
    pub sequence_id: u64,
}

// ---------- Cleared ----------

/// Marker for the `"Cleared"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClearedType {
    /// Always serializes as `"Cleared"`.
    #[default]
    Cleared,
}

/// Mirrors `SpeakV1Cleared` — confirms a `Clear` control message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ClearedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: ClearedType,

    /// Sequence ID of this control acknowledgement.
    pub sequence_id: u64,
}

// ---------- Warning ----------

/// Marker for the `"Warning"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WarningType {
    /// Always serializes as `"Warning"`.
    #[default]
    Warning,
}

/// Mirrors `SpeakV1WarningEvent` — non-fatal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WarningEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: WarningType,

    /// Human-readable description of the warning.
    pub description: String,

    /// Warning code identifying the issue.
    pub code: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn metadata_round_trip() {
        let raw = json!({
            "type": "Metadata",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "model_name": "aura-2-thalia-en",
            "model_version": "1.0.0",
            "model_uuid": "11111111-2222-3333-4444-555555555555",
            "additional_model_uuids": [
                "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
            ]
        });
        let event: SpeakResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            SpeakResponse::Metadata(m) => {
                assert_eq!(m.model_name, "aura-2-thalia-en");
                assert_eq!(m.additional_model_uuids.len(), 1);
            }
            _ => panic!("expected Metadata"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn flushed_round_trip() {
        let raw = json!({ "type": "Flushed", "sequence_id": 7 });
        let event: SpeakResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            SpeakResponse::Flushed(f) => assert_eq!(f.sequence_id, 7),
            _ => panic!("expected Flushed"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn cleared_round_trip() {
        let raw = json!({ "type": "Cleared", "sequence_id": 3 });
        let event: SpeakResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            SpeakResponse::Cleared(c) => assert_eq!(c.sequence_id, 3),
            _ => panic!("expected Cleared"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn warning_round_trip() {
        let raw = json!({
            "type": "Warning",
            "description": "deprecated voice",
            "code": "DEPRECATION"
        });
        let event: SpeakResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            SpeakResponse::Warning(w) => {
                assert_eq!(w.code, "DEPRECATION");
                assert_eq!(w.description, "deprecated voice");
            }
            _ => panic!("expected Warning"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn unknown_type_falls_into_unknown_variant() {
        let raw = json!({
            "type": "FutureSpeakEvent",
            "extra": [1, 2, 3]
        });
        let event: SpeakResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            SpeakResponse::Unknown(value) => {
                assert_eq!(value["type"], "FutureSpeakEvent");
            }
            _ => panic!("expected Unknown"),
        }
        // Unknown round-trips its raw value verbatim.
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn audio_variant_constructs_via_bytes() {
        // Audio is constructed by the connection layer from binary
        // frames — never deserialized from JSON. Sanity-check the
        // type compiles and round-trips its bytes.
        let bytes = Bytes::from_static(&[1, 2, 3]);
        let event = SpeakResponse::Audio(bytes.clone());
        match event {
            SpeakResponse::Audio(b) => assert_eq!(b, bytes),
            _ => panic!("expected Audio"),
        }
    }
}
