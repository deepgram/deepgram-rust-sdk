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
    #[non_exhaustive]
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

        /// The cause of the turn ending. Present on every
        /// [`TurnEvent::EndOfTurn`] event and only there.
        trigger: Option<TurnTrigger>,

        /// Languages detected in the user's speech (descending by word
        /// count). Only populated when the listen model is
        /// `flux-general-multi`; empty otherwise.
        languages: Vec<String>,

        /// The currently active language hints. Only populated when
        /// language hints are configured (e.g. via
        /// [`crate::common::options::OptionsBuilder::language_hint`]
        /// or via a [`FluxResponse::ConfigureSuccess`] update).
        languages_hinted: Vec<String>,
    },

    /// Acknowledges that a `Configure` message was successfully applied.
    /// Reflects the post-update settings.
    ConfigureSuccess {
        #[allow(missing_docs)]
        request_id: Uuid,

        #[allow(missing_docs)]
        sequence_id: u32,

        /// Current threshold values after the update.
        thresholds: ConfigureThresholds,

        /// Currently active keyterms.
        keyterms: Vec<String>,

        /// Currently active language hints. Only meaningful with
        /// `flux-general-multi`.
        language_hints: Vec<String>,
    },

    /// The server rejected a `Configure` message. The session continues
    /// with its prior settings.
    ConfigureFailure {
        #[allow(missing_docs)]
        request_id: Uuid,

        #[allow(missing_docs)]
        sequence_id: u32,
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

/// End-of-turn threshold settings reported on
/// [`FluxResponse::ConfigureSuccess`] (and accepted by
/// [`crate::listen::flux::FluxHandle::configure`]).
///
/// All fields are optional — the server only echoes back the values
/// that are currently set.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConfigureThresholds {
    /// Confidence required to fire an eager end-of-turn event (0.3-0.9).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eager_eot_threshold: Option<f64>,

    /// Confidence required to finish a turn (0.5-0.9).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eot_threshold: Option<f64>,

    /// Force-end-of-turn timeout, in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eot_timeout_ms: Option<u32>,
}

impl ConfigureThresholds {
    /// Construct an empty threshold set; populate via the `with_*` builders.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the eager-EOT confidence threshold (0.3-0.9).
    pub fn with_eager_eot_threshold(mut self, value: f64) -> Self {
        self.eager_eot_threshold = Some(value);
        self
    }

    /// Set the EOT confidence threshold (0.5-0.9).
    pub fn with_eot_threshold(mut self, value: f64) -> Self {
        self.eot_threshold = Some(value);
        self
    }

    /// Set the force-EOT timeout, in milliseconds.
    pub fn with_eot_timeout_ms(mut self, value: u32) -> Self {
        self.eot_timeout_ms = Some(value);
        self
    }
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trigger: Option<TurnTrigger>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        languages: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        languages_hinted: Vec<String>,
    },
    ConfigureSuccess {
        request_id: Uuid,
        sequence_id: u32,
        thresholds: ConfigureThresholds,
        #[serde(default)]
        keyterms: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        language_hints: Vec<String>,
    },
    ConfigureFailure {
        request_id: Uuid,
        sequence_id: u32,
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
                trigger,
                languages,
                languages_hinted,
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
                trigger,
                languages,
                languages_hinted,
            },
            TaggedFluxResponse::ConfigureSuccess {
                request_id,
                sequence_id,
                thresholds,
                keyterms,
                language_hints,
            } => FluxResponse::ConfigureSuccess {
                request_id,
                sequence_id,
                thresholds,
                keyterms,
                language_hints,
            },
            TaggedFluxResponse::ConfigureFailure {
                request_id,
                sequence_id,
            } => FluxResponse::ConfigureFailure {
                request_id,
                sequence_id,
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
            Some("Connected" | "TurnInfo" | "ConfigureSuccess" | "ConfigureFailure" | "Error") => {
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
                trigger,
                languages,
                languages_hinted,
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
                    trigger: trigger.clone(),
                    languages: languages.clone(),
                    languages_hinted: languages_hinted.clone(),
                };
                tagged.serialize(serializer)
            }
            FluxResponse::ConfigureSuccess {
                request_id,
                sequence_id,
                thresholds,
                keyterms,
                language_hints,
            } => {
                let tagged = TaggedFluxResponse::ConfigureSuccess {
                    request_id: *request_id,
                    sequence_id: *sequence_id,
                    thresholds: thresholds.clone(),
                    keyterms: keyterms.clone(),
                    language_hints: language_hints.clone(),
                };
                tagged.serialize(serializer)
            }
            FluxResponse::ConfigureFailure {
                request_id,
                sequence_id,
            } => {
                let tagged = TaggedFluxResponse::ConfigureFailure {
                    request_id: *request_id,
                    sequence_id: *sequence_id,
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

/// The cause of a turn ending, reported on [`TurnEvent::EndOfTurn`] events.
///
/// This is an open enum: new values may be added over time, and
/// unrecognized values deserialize as [`TurnTrigger::Unknown`], which
/// preserves the original wire string and re-serializes to it exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TurnTrigger {
    /// The turn ended by Flux's native end-of-turn detection
    Model,

    /// The turn ended because a `ForceEndTurn` message was sent
    Manual,

    /// The turn ended because `eot_timeout_ms` elapsed
    Timeout,

    /// An unrecognized trigger value from the server, preserved verbatim.
    Unknown(String),
}

impl TurnTrigger {
    /// The wire representation of this trigger.
    pub fn as_str(&self) -> &str {
        match self {
            TurnTrigger::Model => "model",
            TurnTrigger::Manual => "manual",
            TurnTrigger::Timeout => "timeout",
            TurnTrigger::Unknown(value) => value,
        }
    }
}

// Manual scalar (de)serialization: `TurnTrigger` is a plain string on the
// wire, and unknown values must survive an exact round trip — a derived
// `#[serde(other)]` unit variant would collapse them all to one value.
impl Serialize for TurnTrigger {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TurnTrigger {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "model" => TurnTrigger::Model,
            "manual" => TurnTrigger::Manual,
            "timeout" => TurnTrigger::Timeout,
            _ => TurnTrigger::Unknown(value),
        })
    }
}

/// A word in a Flux turn with confidence
#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct FluxWord {
    #[allow(missing_docs)]
    pub word: String,

    #[allow(missing_docs)]
    pub confidence: f64,

    /// The start time of the word, in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<f64>,

    /// The end time of the word, in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
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

    #[test]
    fn turninfo_languages_default_empty_when_absent() {
        let json = r#"{"type":"TurnInfo","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":1,"event":"EndOfTurn","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"hello","words":[],"end_of_turn_confidence":0.9}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxResponse::TurnInfo {
                languages,
                languages_hinted,
                ..
            } => {
                assert!(languages.is_empty());
                assert!(languages_hinted.is_empty());
            }
            _ => panic!("expected TurnInfo"),
        }
    }

    #[test]
    fn turninfo_languages_round_trip_when_present() {
        let json = r#"{"type":"TurnInfo","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":1,"event":"EndOfTurn","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"Hola","words":[],"end_of_turn_confidence":0.9,"languages":["es"],"languages_hinted":["en","es"]}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        match &response {
            FluxResponse::TurnInfo {
                languages,
                languages_hinted,
                ..
            } => {
                assert_eq!(languages, &vec!["es".to_string()]);
                assert_eq!(languages_hinted, &vec!["en".to_string(), "es".to_string()]);
            }
            _ => panic!("expected TurnInfo"),
        }
        let back = serde_json::to_string(&response).unwrap();
        assert_eq!(back, json);
    }

    #[test]
    fn turninfo_trigger_absent_on_non_eot_events() {
        let json = r#"{"type":"TurnInfo","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":1,"event":"Update","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"hello","words":[],"end_of_turn_confidence":0.5}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        match &response {
            FluxResponse::TurnInfo { trigger, .. } => assert_eq!(trigger, &None),
            _ => panic!("expected TurnInfo"),
        }
        // `trigger: None` must serialize back to the same JSON (field omitted).
        assert_eq!(serde_json::to_string(&response).unwrap(), json);
    }

    #[test]
    fn turninfo_trigger_round_trip() {
        for (wire, expected) in [
            ("model", TurnTrigger::Model),
            ("manual", TurnTrigger::Manual),
            ("timeout", TurnTrigger::Timeout),
        ] {
            let json = format!(
                r#"{{"type":"TurnInfo","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":11,"event":"EndOfTurn","turn_index":3,"audio_window_start":4.2,"audio_window_end":6.8,"transcript":"hello","words":[],"end_of_turn_confidence":0.35,"trigger":"{wire}"}}"#
            );
            let response: FluxResponse = serde_json::from_str(&json).unwrap();
            match &response {
                FluxResponse::TurnInfo { trigger, .. } => {
                    assert_eq!(trigger, &Some(expected.clone()))
                }
                _ => panic!("expected TurnInfo"),
            }
            assert_eq!(serde_json::to_string(&response).unwrap(), json);
        }
    }

    #[test]
    fn turninfo_unknown_trigger_round_trips_verbatim() {
        let json = r#"{"type":"TurnInfo","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":11,"event":"EndOfTurn","turn_index":0,"audio_window_start":0.0,"audio_window_end":1.0,"transcript":"hello","words":[],"end_of_turn_confidence":0.9,"trigger":"some_future_trigger"}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        match &response {
            FluxResponse::TurnInfo { trigger, .. } => {
                assert_eq!(
                    trigger,
                    &Some(TurnTrigger::Unknown("some_future_trigger".to_string()))
                );
            }
            _ => panic!("expected TurnInfo"),
        }
        // The unknown wire value must survive an exact round trip.
        assert_eq!(serde_json::to_string(&response).unwrap(), json);
    }

    #[test]
    fn flux_word_timings_round_trip() {
        let json = r#"{"word":"Hello,","confidence":0.96,"start":0.0,"end":0.18}"#;
        let word: FluxWord = serde_json::from_str(json).unwrap();
        assert_eq!(word.start, Some(0.0));
        assert_eq!(word.end, Some(0.18));
        assert_eq!(serde_json::to_string(&word).unwrap(), json);
    }

    #[test]
    fn flux_word_timings_optional() {
        let json = r#"{"word":"Hello,","confidence":0.96}"#;
        let word: FluxWord = serde_json::from_str(json).unwrap();
        assert_eq!(word.start, None);
        assert_eq!(word.end, None);
        // Absent timings must not be invented on re-serialization.
        assert_eq!(serde_json::to_string(&word).unwrap(), json);
    }

    #[test]
    fn configure_success_round_trip() {
        let json = r#"{"type":"ConfigureSuccess","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":5,"thresholds":{"eager_eot_threshold":0.6,"eot_threshold":0.8,"eot_timeout_ms":4000},"keyterms":["activate","cancel"],"language_hints":["en"]}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        match &response {
            FluxResponse::ConfigureSuccess {
                thresholds,
                keyterms,
                language_hints,
                sequence_id,
                ..
            } => {
                assert_eq!(*sequence_id, 5);
                assert_eq!(thresholds.eot_threshold, Some(0.8));
                assert_eq!(
                    keyterms,
                    &vec!["activate".to_string(), "cancel".to_string()]
                );
                assert_eq!(language_hints, &vec!["en".to_string()]);
            }
            _ => panic!("expected ConfigureSuccess"),
        }
        let back = serde_json::to_string(&response).unwrap();
        assert_eq!(back, json);
    }

    #[test]
    fn configure_failure_round_trip() {
        let json = r#"{"type":"ConfigureFailure","request_id":"550e8400-e29b-41d4-a716-446655440000","sequence_id":6}"#;
        let response: FluxResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, FluxResponse::ConfigureFailure { .. }));
        let back = serde_json::to_string(&response).unwrap();
        assert_eq!(back, json);
    }
}
