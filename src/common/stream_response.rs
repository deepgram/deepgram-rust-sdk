//! Stream Response module

use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A single transcribed word.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    #[allow(missing_docs)]
    pub word: String,

    #[allow(missing_docs)]
    pub start: f64,

    #[allow(missing_docs)]
    pub end: f64,

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    pub speaker: Option<i32>,

    #[allow(missing_docs)]
    pub punctuated_word: Option<String>,

    #[allow(missing_docs)]
    pub language: Option<String>,
}

/// Transcript alternatives.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, Serialize, Deserialize)]
pub struct Alternatives {
    #[allow(missing_docs)]
    pub transcript: String,

    #[allow(missing_docs)]
    pub words: Vec<Word>,

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    #[serde(default)]
    pub languages: Vec<String>,
}

/// Transcription results for a single audio channel.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Multichannel feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/documentation/features/multichannel/
#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    #[allow(missing_docs)]
    pub alternatives: Vec<Alternatives>,
}

/// Modle info
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    #[allow(missing_docs)]
    pub name: String,

    #[allow(missing_docs)]
    pub version: String,

    #[allow(missing_docs)]
    pub arch: String,
}

/// Metadata about the transcription.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[allow(missing_docs)]
    pub request_id: String,

    #[allow(missing_docs)]
    pub model_info: ModelInfo,

    #[allow(missing_docs)]
    pub model_uuid: String,
}

/// Possible websocket message types
#[derive(Debug)]
#[non_exhaustive]
pub enum StreamResponse {
    #[allow(missing_docs)]
    TranscriptResponse {
        #[allow(missing_docs)]
        type_field: String,

        #[allow(missing_docs)]
        start: f64,

        #[allow(missing_docs)]
        duration: f64,

        #[allow(missing_docs)]
        is_final: bool,

        #[allow(missing_docs)]
        speech_final: bool,

        #[allow(missing_docs)]
        from_finalize: bool,

        #[allow(missing_docs)]
        channel: Channel,

        #[allow(missing_docs)]
        metadata: Metadata,

        #[allow(missing_docs)]
        channel_index: Vec<i32>,
    },
    #[allow(missing_docs)]
    TerminalResponse {
        #[allow(missing_docs)]
        request_id: String,

        #[allow(missing_docs)]
        created: String,

        #[allow(missing_docs)]
        duration: f64,

        #[allow(missing_docs)]
        channels: u32,
    },
    #[allow(missing_docs)]
    SpeechStartedResponse {
        #[allow(missing_docs)]
        type_field: String,

        #[allow(missing_docs)]
        channel: Vec<u8>,

        #[allow(missing_docs)]
        timestamp: f64,
    },
    #[allow(missing_docs)]
    UtteranceEndResponse {
        #[allow(missing_docs)]
        type_field: String,

        #[allow(missing_docs)]
        channel: Vec<u8>,

        #[allow(missing_docs)]
        last_word_end: f64,
    },
    /// An unrecognized server message.
    ///
    /// This final fallback preserves raw JSON frames, including server error
    /// frames, so new message types do not end a streaming session in older
    /// SDK releases.
    Unknown(serde_json::Value),
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum TaggedStreamResponse {
    Results {
        start: f64,
        duration: f64,
        is_final: bool,
        #[serde(default)]
        speech_final: bool,
        from_finalize: bool,
        channel: Channel,
        metadata: Metadata,
        channel_index: Vec<i32>,
    },
    Metadata {
        request_id: String,
        created: String,
        duration: f64,
        channels: u32,
    },
    SpeechStarted {
        channel: Vec<u8>,
        timestamp: f64,
    },
    UtteranceEnd {
        channel: Vec<u8>,
        last_word_end: f64,
    },
}

impl From<TaggedStreamResponse> for StreamResponse {
    fn from(tagged: TaggedStreamResponse) -> Self {
        match tagged {
            TaggedStreamResponse::Results {
                start,
                duration,
                is_final,
                speech_final,
                from_finalize,
                channel,
                metadata,
                channel_index,
            } => Self::TranscriptResponse {
                type_field: "Results".to_string(),
                start,
                duration,
                is_final,
                speech_final,
                from_finalize,
                channel,
                metadata,
                channel_index,
            },
            TaggedStreamResponse::Metadata {
                request_id,
                created,
                duration,
                channels,
            } => Self::TerminalResponse {
                request_id,
                created,
                duration,
                channels,
            },
            TaggedStreamResponse::SpeechStarted { channel, timestamp } => {
                Self::SpeechStartedResponse {
                    type_field: "SpeechStarted".to_string(),
                    channel,
                    timestamp,
                }
            }
            TaggedStreamResponse::UtteranceEnd {
                channel,
                last_word_end,
            } => Self::UtteranceEndResponse {
                type_field: "UtteranceEnd".to_string(),
                channel,
                last_word_end,
            },
        }
    }
}

impl<'de> Deserialize<'de> for StreamResponse {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        match value.get("type").and_then(|value| value.as_str()) {
            Some("Results" | "Metadata" | "SpeechStarted" | "UtteranceEnd") => {
                serde_json::from_value::<TaggedStreamResponse>(value)
                    .map(Self::from)
                    .map_err(de::Error::custom)
            }
            _ => Ok(Self::Unknown(value)),
        }
    }
}

impl Serialize for StreamResponse {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::TranscriptResponse {
                type_field,
                start,
                duration,
                is_final,
                speech_final,
                from_finalize,
                channel,
                metadata,
                channel_index,
            } => serde_json::json!({
                "type": type_field,
                "start": start,
                "duration": duration,
                "is_final": is_final,
                "speech_final": speech_final,
                "from_finalize": from_finalize,
                "channel": channel,
                "metadata": metadata,
                "channel_index": channel_index,
            }),
            Self::TerminalResponse {
                request_id,
                created,
                duration,
                channels,
            } => serde_json::json!({
                "request_id": request_id,
                "created": created,
                "duration": duration,
                "channels": channels,
            }),
            Self::SpeechStartedResponse {
                type_field,
                channel,
                timestamp,
            } => serde_json::json!({
                "type": type_field,
                "channel": channel,
                "timestamp": timestamp,
            }),
            Self::UtteranceEndResponse {
                type_field,
                channel,
                last_word_end,
            } => serde_json::json!({
                "type": type_field,
                "channel": channel,
                "last_word_end": last_word_end,
            }),
            Self::Unknown(value) => return value.serialize(serializer),
        };

        value.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_frame_deserializes_as_unknown() {
        let json = r#"{"type":"Error","variant":"SchemaError","description":"request schema is invalid","message":"invalid request"}"#;
        let expected: serde_json::Value = serde_json::from_str(json).unwrap();

        let response: StreamResponse = serde_json::from_str(json).unwrap();
        match &response {
            StreamResponse::Unknown(value) => {
                assert_eq!(value, &expected);
                assert_eq!(value["type"], "Error");
                assert_eq!(value["variant"], "SchemaError");
                assert_eq!(value["description"], "request schema is invalid");
                assert_eq!(value["message"], "invalid request");
            }
            _ => panic!("expected Unknown response"),
        }
    }

    #[test]
    fn unknown_response_serialization_preserves_fields() {
        let expected = serde_json::json!({
            "type": "Error",
            "variant": "SchemaError",
            "description": "request schema is invalid",
            "message": "invalid request",
        });
        let response = StreamResponse::Unknown(expected.clone());

        assert_eq!(serde_json::to_value(response).unwrap(), expected);
    }

    #[test]
    fn results_without_speech_final_deserializes_as_transcript() {
        let json = r#"{"type":"Results","channel_index":[0,1],"duration":1.0,"start":0.0,"is_final":true,"from_finalize":false,"channel":{"alternatives":[{"transcript":"hello","words":[],"confidence":0.99}]},"metadata":{"request_id":"request-123","model_info":{"name":"nova-3","version":"2026-09-01","arch":"2"},"model_uuid":"model-123"}}"#;

        let response: StreamResponse = serde_json::from_str(json).unwrap();
        match response {
            StreamResponse::TranscriptResponse {
                speech_final,
                channel,
                ..
            } => {
                assert!(!speech_final);
                assert_eq!(channel.alternatives[0].transcript, "hello");
            }
            _ => panic!("expected transcript response"),
        }
    }

    #[test]
    fn malformed_known_response_is_an_error() {
        let json = r#"{"type":"Results","channel_index":[0]}"#;

        assert!(serde_json::from_str::<StreamResponse>(json).is_err());
    }
}
