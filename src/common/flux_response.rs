//! Flux streaming response types for turn-based conversations.
//!
//! See the [Deepgram Flux API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/speech-to-text/listen-flux

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

/// Flux WebSocket message types
#[derive(Debug)]
#[non_exhaustive]
pub enum FluxResponse {
    /// Initial connection confirmation
    Connected {
        #[allow(missing_docs)]
        request_id: Uuid,

        #[allow(missing_docs)]
        sequence_id: u32,
    },

    /// Turn information with transcript
    TurnInfo {
        #[allow(missing_docs)]
        request_id: Uuid,

        #[allow(missing_docs)]
        sequence_id: u32,

        /// Event type: EndOfTurn, EagerEndOfTurn, or TurnResumed
        event: TurnEvent,

        #[allow(missing_docs)]
        turn_index: u32,

        #[allow(missing_docs)]
        audio_window_start: f64,

        #[allow(missing_docs)]
        audio_window_end: f64,

        #[allow(missing_docs)]
        transcript: String,

        #[allow(missing_docs)]
        words: Vec<FluxWord>,

        /// Confidence that this is end of turn
        end_of_turn_confidence: f64,
    },

    /// Fatal error from server
    FatalError {
        #[allow(missing_docs)]
        sequence_id: u32,

        #[allow(missing_docs)]
        code: String,

        #[allow(missing_docs)]
        description: String,
    },

    /// An unknown message type received from the server.
    ///
    /// This variant is used for forward-compatibility when the server sends
    /// a message type that this version of the SDK does not recognize.
    /// The raw JSON value is preserved for inspection and logging.
    Unknown(serde_json::Value),
}

/// Private helper enum for deserializing/serializing known FluxResponse variants
/// using serde's internally-tagged representation.
#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
enum TaggedFluxResponse {
    Connected {
        request_id: Uuid,
        sequence_id: u32,
    },
    TurnInfo {
        request_id: Uuid,
        sequence_id: u32,
        event: TurnEvent,
        turn_index: u32,
        audio_window_start: f64,
        audio_window_end: f64,
        transcript: String,
        words: Vec<FluxWord>,
        end_of_turn_confidence: f64,
    },
    #[serde(rename = "Error")]
    FatalError {
        sequence_id: u32,
        code: String,
        description: String,
    },
}

impl From<TaggedFluxResponse> for FluxResponse {
    fn from(tagged: TaggedFluxResponse) -> Self {
        match tagged {
            TaggedFluxResponse::Connected {
                request_id,
                sequence_id,
            } => FluxResponse::Connected {
                request_id,
                sequence_id,
            },
            TaggedFluxResponse::TurnInfo {
                request_id,
                sequence_id,
                event,
                turn_index,
                audio_window_start,
                audio_window_end,
                transcript,
                words,
                end_of_turn_confidence,
            } => FluxResponse::TurnInfo {
                request_id,
                sequence_id,
                event,
                turn_index,
                audio_window_start,
                audio_window_end,
                transcript,
                words,
                end_of_turn_confidence,
            },
            TaggedFluxResponse::FatalError {
                sequence_id,
                code,
                description,
            } => FluxResponse::FatalError {
                sequence_id,
                code,
                description,
            },
        }
    }
}

impl<'de> Deserialize<'de> for FluxResponse {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let type_str = value.get("type").and_then(|t| t.as_str());

        match type_str {
            Some("Connected" | "TurnInfo" | "Error") => {
                serde_json::from_value::<TaggedFluxResponse>(value)
                    .map(FluxResponse::from)
                    .map_err(de::Error::custom)
            }
            _ => Ok(FluxResponse::Unknown(value)),
        }
    }
}

impl Serialize for FluxResponse {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FluxResponse::Connected {
                request_id,
                sequence_id,
            } => {
                let tagged = TaggedFluxResponse::Connected {
                    request_id: *request_id,
                    sequence_id: *sequence_id,
                };
                tagged.serialize(serializer)
            }
            FluxResponse::TurnInfo {
                request_id,
                sequence_id,
                event,
                turn_index,
                audio_window_start,
                audio_window_end,
                transcript,
                words,
                end_of_turn_confidence,
            } => {
                let tagged = TaggedFluxResponse::TurnInfo {
                    request_id: *request_id,
                    sequence_id: *sequence_id,
                    event: event.clone(),
                    turn_index: *turn_index,
                    audio_window_start: *audio_window_start,
                    audio_window_end: *audio_window_end,
                    transcript: transcript.clone(),
                    words: words.clone(),
                    end_of_turn_confidence: *end_of_turn_confidence,
                };
                tagged.serialize(serializer)
            }
            FluxResponse::FatalError {
                sequence_id,
                code,
                description,
            } => {
                let tagged = TaggedFluxResponse::FatalError {
                    sequence_id: *sequence_id,
                    code: code.clone(),
                    description: description.clone(),
                };
                tagged.serialize(serializer)
            }
            FluxResponse::Unknown(value) => value.serialize(serializer),
        }
    }
}

/// Turn event types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TurnEvent {
    /// Start of a new turn
    StartOfTurn,

    /// Normal end of turn
    EndOfTurn,

    /// Eager end of turn (when confidence threshold met early)
    EagerEndOfTurn,

    /// Turn resumed after eager end
    TurnResumed,

    /// Turn update (interim transcript update)
    Update,

    /// An unrecognized turn event from the server.
    #[serde(other)]
    Unknown,
}

/// A word in a Flux turn with confidence
#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct FluxWord {
    #[allow(missing_docs)]
    pub word: String,

    #[allow(missing_docs)]
    pub confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_connected() {
        let json = r#"{"type": "Connected", "request_id": "550e8400-e29b-41d4-a716-446655440000", "sequence_id": 0}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, FluxResponse::Connected { .. }));
    }

    #[test]
    fn deserialize_fatal_error() {
        let json = r#"{"type": "Error", "sequence_id": 1, "code": "ERR_001", "description": "test error"}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, FluxResponse::FatalError { .. }));
    }

    #[test]
    fn deserialize_unknown_type() {
        let json = r#"{"type": "NewFeature", "some_field": 42, "data": [1, 2, 3]}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxResponse::Unknown(value) => {
                assert_eq!(value["type"], "NewFeature");
                assert_eq!(value["some_field"], 42);
            }
            _ => panic!("expected Unknown variant"),
        }
    }

    #[test]
    fn deserialize_missing_type_field() {
        let json = r#"{"some_random": "message"}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, FluxResponse::Unknown(_)));
    }

    #[test]
    fn deserialize_unknown_turn_event() {
        let json = r#"{"type": "TurnInfo", "request_id": "550e8400-e29b-41d4-a716-446655440000", "sequence_id": 1, "event": "NewEvent", "turn_index": 0, "audio_window_start": 0.0, "audio_window_end": 1.0, "transcript": "hello", "words": [], "end_of_turn_confidence": 0.5}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxResponse::TurnInfo { event, .. } => {
                assert_eq!(event, TurnEvent::Unknown);
            }
            _ => panic!("expected TurnInfo variant"),
        }
    }

    #[test]
    fn serialize_roundtrip_connected() {
        let json = r#"{"type":"Connected","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":0}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&response).unwrap();
        assert_eq!(serialized, json);
    }

    #[test]
    fn serialize_unknown_preserves_original() {
        let json = r#"{"type":"NewFeature","some_field":42}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&response).unwrap();
        let roundtrip: serde_json::Value = serde_json::from_str(&serialized).unwrap();
        let original: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(roundtrip, original);
    }
}
