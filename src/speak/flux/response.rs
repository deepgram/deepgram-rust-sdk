//! Flux TTS streaming response types for turn-based speech synthesis.
//!
//! See the [Deepgram Flux TTS API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/text-to-speech/speak-flux
//!
//! These are the messages the `/v2/speak` WebSocket server sends. Audio
//! arrives as binary frames ([`FluxSpeakResponse::Audio`]); everything
//! else arrives as JSON text frames.

use bytes::Bytes;
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

/// Flux TTS WebSocket message types.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum FluxSpeakResponse {
    /// A chunk of synthesized audio, in the format specified by the
    /// connection parameters. Sent as a binary WebSocket frame.
    Audio(Bytes),

    /// Sent immediately on a successful connection.
    #[non_exhaustive]
    Connected {
        /// The unique identifier of the `/v2/speak` request
        request_id: Uuid,

        /// Resolved model name. May be the short voice name rather than
        /// the full model string (e.g. `haley` for `flux-haley-en`).
        model_name: String,

        /// Resolved model version
        model_version: String,

        /// Resolved model UUIDs. A list, because a resolved model may
        /// be backed by more than one underlying model.
        model_uuids: Vec<Uuid>,
    },

    /// Emitted at the start of each new turn, before audio streaming
    /// begins. Carries the server-assigned `speech_id` that identifies
    /// the turn.
    #[non_exhaustive]
    SpeechStarted {
        /// Server-minted identifier for this turn, of the form
        /// `dg_sp_<12 hex digits>`.
        speech_id: String,
    },

    /// Immediate echo on receipt of a manual `Flush`, confirming the
    /// server received it before synthesis completes.
    #[non_exhaustive]
    Flushed {
        /// Server-assigned turn identifier
        speech_id: String,
    },

    /// Emitted at turn boundaries (manual `Flush`), after all audio for
    /// the turn has been sent. Reports billing and timing for the
    /// completed turn.
    SpeechMetadata(TurnMetadata),

    /// Sent in reply to an `Interrupt`, once synthesis has concluded and
    /// audio generation has stopped. Reports what the user heard, what
    /// they did not, and the billing and timing of the turn the
    /// interrupt landed in.
    #[non_exhaustive]
    SpeechInterrupted {
        /// How much audio the client had played when the interrupt
        /// landed, in milliseconds from the start of the session. Echoes
        /// the `Interrupt`'s `playback_offset` when one was supplied;
        /// otherwise it is the server's own total of audio generated so
        /// far.
        audio_played_ms: u64,

        /// The portion of the turn's text the user heard. `None` when
        /// the `Interrupt` carried no `playback_offset`.
        text_spoken: Option<String>,

        /// The portion of the turn's text the user did not hear. `None`
        /// when the `Interrupt` carried no `playback_offset`.
        text_remaining: Option<String>,

        /// Billing and timing for the turn the interrupt landed in.
        metadata: TurnMetadata,
    },

    /// Final server message before the WebSocket closes. Reports
    /// cumulative session totals across all turns.
    #[non_exhaustive]
    SessionMetadata {
        /// Cumulative audio duration produced across the session, in
        /// milliseconds. An `Interrupt` rebases this onto the audio the
        /// client actually played.
        total_audio_duration_ms: u64,

        /// Cumulative raw input character count across the session
        total_input_character_count: u64,

        /// Cumulative billable character count across the session
        total_billable_character_count: u64,
    },

    /// Sent when a `Configure` was accepted and applied. Echoes the
    /// configuration now in effect.
    #[non_exhaustive]
    ConfigureSuccess {
        /// The synthesis configuration now in effect. A field is present
        /// only when it has been set on this session.
        applied: AppliedConfiguration,
    },

    /// Sent when a `Configure` was rejected. The stream is unaffected
    /// and the previous configuration stays in force.
    #[non_exhaustive]
    ConfigureFailure {
        /// Failure code, in `SCREAMING_SNAKE_CASE`: `SPEED_OUT_OF_RANGE`,
        /// `SPEED_INCREMENT_INVALID`, `SPEED_NOT_SUPPORTED`, or
        /// `INTERNAL_ERROR`.
        code: String,

        /// The configuration field the failure is about (e.g. `speed`).
        /// `None` when the failure is not tied to one field.
        field: Option<String>,

        /// The rejected value for `field`. `None` when there is no
        /// offending value to echo.
        value: Option<f64>,

        /// A human-readable description of the failure
        description: String,
    },

    /// Informational message; synthesis continues and the connection is
    /// unaffected.
    #[non_exhaustive]
    Warning {
        /// Warning code identifying the condition, in
        /// `SCREAMING_SNAKE_CASE` (e.g. `NO_ACTIVE_SPEECH`,
        /// `NO_SYNTHESIZABLE_TEXT`, `SYNTHESIS_RETRYING`,
        /// `NO_AUDIO_GENERATED`, `INTERRUPT_IN_PROGRESS`,
        /// `INVALID_INTERRUPT_OFFSET`). This is an open set — new codes
        /// may be added over time.
        code: String,

        /// A human-readable description of the warning
        description: String,
    },

    /// A fatal, server-originated error. Always followed by a WebSocket
    /// close. The wire `type` tag is `Error`.
    #[non_exhaustive]
    FatalError {
        /// A code identifying the error, using Deepgram's `DOMAIN-NNNN`
        /// convention (e.g. `MESSAGE-0000`, `NET-0000`).
        code: String,

        /// Prose description of the error
        description: String,
    },

    /// An unknown message type received from the server.
    ///
    /// This variant is used for forward-compatibility when the server
    /// sends a message type that this version of the SDK does not
    /// recognize. The raw JSON value is preserved for inspection and
    /// logging.
    Unknown(serde_json::Value),
}

/// Billing and timing for a single turn, reported on
/// [`FluxSpeakResponse::SpeechMetadata`] and nested inside
/// [`FluxSpeakResponse::SpeechInterrupted`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TurnMetadata {
    /// Server-assigned turn identifier
    pub speech_id: String,

    /// Audio duration produced for this turn, in milliseconds
    pub audio_duration_ms: u64,

    /// Raw input character count for this turn, before text
    /// normalization
    pub input_character_count: u64,

    /// Billable character count for this turn — the input character
    /// count with stripped control characters removed. Always less than
    /// or equal to `input_character_count`.
    pub billable_character_count: u64,

    /// Counts of the inline controls the server acted on during the turn
    pub controls_applied: ControlsApplied,
}

/// Counts of the inline controls the server acted on during a turn.
///
/// Inline pause and pronunciation controls are not applied at launch —
/// support is coming soon — so every count is currently `0`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ControlsApplied {
    /// Pronunciation overrides successfully applied
    pub pronunciations_applied: u64,

    /// Pause (break) controls successfully applied
    pub breaks_applied: u64,

    /// Pronunciation entries that triggered a warning (invalid IPA, word
    /// too long)
    pub pronunciation_warnings: u64,
}

/// Synthesis configuration echoed on
/// [`FluxSpeakResponse::ConfigureSuccess`]. A field is present only when
/// it has been set on this session.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AppliedConfiguration {
    /// Speech-rate multiplier currently in effect
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

/// Private helper enum for deserializing known JSON server messages
/// using serde's internally-tagged representation.
#[derive(Deserialize)]
#[serde(tag = "type")]
enum TaggedResponse {
    Connected {
        request_id: Uuid,
        model_name: String,
        model_version: String,
        model_uuids: Vec<Uuid>,
    },
    SpeechStarted {
        speech_id: String,
    },
    Flushed {
        speech_id: String,
    },
    SpeechMetadata(TurnMetadata),
    SpeechInterrupted {
        audio_played_ms: u64,
        #[serde(default)]
        text_spoken: Option<String>,
        #[serde(default)]
        text_remaining: Option<String>,
        metadata: TurnMetadata,
    },
    SessionMetadata {
        total_audio_duration_ms: u64,
        total_input_character_count: u64,
        total_billable_character_count: u64,
    },
    ConfigureSuccess {
        applied: AppliedConfiguration,
    },
    ConfigureFailure {
        code: String,
        #[serde(default)]
        field: Option<String>,
        #[serde(default)]
        value: Option<f64>,
        description: String,
    },
    Warning {
        code: String,
        description: String,
    },
    #[serde(rename = "Error")]
    FatalError {
        code: String,
        description: String,
    },
}

impl From<TaggedResponse> for FluxSpeakResponse {
    fn from(tagged: TaggedResponse) -> Self {
        match tagged {
            TaggedResponse::Connected {
                request_id,
                model_name,
                model_version,
                model_uuids,
            } => FluxSpeakResponse::Connected {
                request_id,
                model_name,
                model_version,
                model_uuids,
            },
            TaggedResponse::SpeechStarted { speech_id } => {
                FluxSpeakResponse::SpeechStarted { speech_id }
            }
            TaggedResponse::Flushed { speech_id } => FluxSpeakResponse::Flushed { speech_id },
            TaggedResponse::SpeechMetadata(metadata) => FluxSpeakResponse::SpeechMetadata(metadata),
            TaggedResponse::SpeechInterrupted {
                audio_played_ms,
                text_spoken,
                text_remaining,
                metadata,
            } => FluxSpeakResponse::SpeechInterrupted {
                audio_played_ms,
                text_spoken,
                text_remaining,
                metadata,
            },
            TaggedResponse::SessionMetadata {
                total_audio_duration_ms,
                total_input_character_count,
                total_billable_character_count,
            } => FluxSpeakResponse::SessionMetadata {
                total_audio_duration_ms,
                total_input_character_count,
                total_billable_character_count,
            },
            TaggedResponse::ConfigureSuccess { applied } => {
                FluxSpeakResponse::ConfigureSuccess { applied }
            }
            TaggedResponse::ConfigureFailure {
                code,
                field,
                value,
                description,
            } => FluxSpeakResponse::ConfigureFailure {
                code,
                field,
                value,
                description,
            },
            TaggedResponse::Warning { code, description } => {
                FluxSpeakResponse::Warning { code, description }
            }
            TaggedResponse::FatalError { code, description } => {
                FluxSpeakResponse::FatalError { code, description }
            }
        }
    }
}

impl<'de> Deserialize<'de> for FluxSpeakResponse {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        let type_str = value.get("type").and_then(|t| t.as_str());

        match type_str {
            Some(
                "Connected" | "SpeechStarted" | "Flushed" | "SpeechMetadata" | "SpeechInterrupted"
                | "SessionMetadata" | "ConfigureSuccess" | "ConfigureFailure" | "Warning" | "Error",
            ) => serde_json::from_value::<TaggedResponse>(value)
                .map(FluxSpeakResponse::from)
                .map_err(de::Error::custom),
            _ => Ok(FluxSpeakResponse::Unknown(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_connected() {
        let json = r#"{"type":"Connected","request_id":"550e8400-e29b-41d4-a716-446655440000","model_name":"flux-haley-en","model_version":"2026.06.01","model_uuids":["660e8400-e29b-41d4-a716-446655440000"]}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::Connected {
                model_name,
                model_uuids,
                ..
            } => {
                assert_eq!(model_name, "flux-haley-en");
                assert_eq!(model_uuids.len(), 1);
            }
            _ => panic!("expected Connected"),
        }
    }

    #[test]
    fn deserialize_speech_started_and_flushed() {
        let started: FluxSpeakResponse =
            serde_json::from_str(r#"{"type":"SpeechStarted","speech_id":"dg_sp_a1b2c3d4e5f6"}"#)
                .unwrap();
        match started {
            FluxSpeakResponse::SpeechStarted { speech_id } => {
                assert_eq!(speech_id, "dg_sp_a1b2c3d4e5f6");
            }
            _ => panic!("expected SpeechStarted"),
        }

        let flushed: FluxSpeakResponse =
            serde_json::from_str(r#"{"type":"Flushed","speech_id":"dg_sp_a1b2c3d4e5f6"}"#).unwrap();
        assert!(matches!(flushed, FluxSpeakResponse::Flushed { .. }));
    }

    #[test]
    fn deserialize_speech_metadata() {
        let json = r#"{"type":"SpeechMetadata","speech_id":"dg_sp_a1b2c3d4e5f6","audio_duration_ms":2340,"input_character_count":52,"billable_character_count":52,"controls_applied":{"pronunciations_applied":0,"breaks_applied":0,"pronunciation_warnings":0}}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::SpeechMetadata(metadata) => {
                assert_eq!(metadata.speech_id, "dg_sp_a1b2c3d4e5f6");
                assert_eq!(metadata.audio_duration_ms, 2340);
                assert_eq!(metadata.billable_character_count, 52);
                assert_eq!(metadata.controls_applied.breaks_applied, 0);
            }
            _ => panic!("expected SpeechMetadata"),
        }
    }

    #[test]
    fn deserialize_speech_interrupted_with_offset() {
        let json = r#"{"type":"SpeechInterrupted","audio_played_ms":2340,"text_spoken":"Sure, I can help you cancel your subscription.","text_remaining":" Let me pull up your account.","metadata":{"speech_id":"dg_sp_a1b2c3d4e5f6","audio_duration_ms":4200,"input_character_count":75,"billable_character_count":75,"controls_applied":{"pronunciations_applied":0,"breaks_applied":0,"pronunciation_warnings":0}}}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::SpeechInterrupted {
                audio_played_ms,
                text_spoken,
                text_remaining,
                metadata,
            } => {
                assert_eq!(audio_played_ms, 2340);
                assert!(text_spoken.unwrap().starts_with("Sure"));
                assert!(text_remaining.is_some());
                assert_eq!(metadata.audio_duration_ms, 4200);
            }
            _ => panic!("expected SpeechInterrupted"),
        }
    }

    #[test]
    fn deserialize_speech_interrupted_without_offset() {
        let json = r#"{"type":"SpeechInterrupted","audio_played_ms":2340,"metadata":{"speech_id":"dg_sp_a1b2c3d4e5f6","audio_duration_ms":4200,"input_character_count":75,"billable_character_count":75,"controls_applied":{"pronunciations_applied":0,"breaks_applied":0,"pronunciation_warnings":0}}}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::SpeechInterrupted {
                text_spoken,
                text_remaining,
                ..
            } => {
                assert_eq!(text_spoken, None);
                assert_eq!(text_remaining, None);
            }
            _ => panic!("expected SpeechInterrupted"),
        }
    }

    #[test]
    fn deserialize_session_metadata() {
        let json = r#"{"type":"SessionMetadata","total_audio_duration_ms":10500,"total_input_character_count":230,"total_billable_character_count":230}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::SessionMetadata {
                total_audio_duration_ms,
                ..
            } => assert_eq!(total_audio_duration_ms, 10500),
            _ => panic!("expected SessionMetadata"),
        }
    }

    #[test]
    fn deserialize_configure_success_and_failure() {
        let ok: FluxSpeakResponse =
            serde_json::from_str(r#"{"type":"ConfigureSuccess","applied":{"speed":1.05}}"#)
                .unwrap();
        match ok {
            FluxSpeakResponse::ConfigureSuccess { applied } => {
                assert_eq!(applied.speed, Some(1.05));
            }
            _ => panic!("expected ConfigureSuccess"),
        }

        // An accepted Configure that named no field echoes an empty object.
        let empty: FluxSpeakResponse =
            serde_json::from_str(r#"{"type":"ConfigureSuccess","applied":{}}"#).unwrap();
        match empty {
            FluxSpeakResponse::ConfigureSuccess { applied } => assert_eq!(applied.speed, None),
            _ => panic!("expected ConfigureSuccess"),
        }

        let failure: FluxSpeakResponse = serde_json::from_str(
            r#"{"type":"ConfigureFailure","code":"SPEED_OUT_OF_RANGE","field":"speed","value":3.5,"description":"speed must be between 0.85 and 1.15 in 0.05 increments"}"#,
        )
        .unwrap();
        match failure {
            FluxSpeakResponse::ConfigureFailure {
                code, field, value, ..
            } => {
                assert_eq!(code, "SPEED_OUT_OF_RANGE");
                assert_eq!(field, Some("speed".to_string()));
                assert_eq!(value, Some(3.5));
            }
            _ => panic!("expected ConfigureFailure"),
        }
    }

    #[test]
    fn deserialize_warning() {
        let json = r#"{"type":"Warning","code":"NO_ACTIVE_SPEECH","description":"There is no active turn. The request will be ignored."}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::Warning { code, .. } => assert_eq!(code, "NO_ACTIVE_SPEECH"),
            _ => panic!("expected Warning"),
        }
    }

    #[test]
    fn deserialize_fatal_error() {
        let json = r#"{"type":"Error","code":"MESSAGE-0000","description":"The message could not be parsed."}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::FatalError { code, .. } => assert_eq!(code, "MESSAGE-0000"),
            _ => panic!("expected FatalError"),
        }
    }

    #[test]
    fn deserialize_unknown_type_preserved() {
        let json = r#"{"type":"NewFeature","some_field":42}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        match response {
            FluxSpeakResponse::Unknown(value) => {
                assert_eq!(value["type"], "NewFeature");
                assert_eq!(value["some_field"], 42);
            }
            _ => panic!("expected Unknown"),
        }
    }

    #[test]
    fn deserialize_missing_type_field() {
        let json = r#"{"some_random":"message"}"#;
        let response: FluxSpeakResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, FluxSpeakResponse::Unknown(_)));
    }
}
