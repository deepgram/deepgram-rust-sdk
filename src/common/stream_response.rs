//! Stream Response module

use serde::{Deserialize, Serialize};

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

/// One entity hit on a streaming `Results` event when `detect_entities`
/// is enabled. Mirrors `entities[]` on `ListenV1ResultsEvent` in
/// `asyncapi/schemas/schemas.listen.v1.yml`.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EntityHit {
    /// Type/category of the entity (e.g. `NAME`, `PHONE_NUMBER`).
    pub label: String,
    /// Formatted text representation of the entity.
    pub value: String,
    /// Original spoken text of the entity, when smart formatting is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_value: Option<String>,
    /// Confidence score.
    pub confidence: f64,
    /// Index of the first word of the entity in the transcript (inclusive).
    pub start_word: i32,
    /// Index of the last word of the entity in the transcript (exclusive).
    pub end_word: i32,
}

/// Possible websocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum StreamResponse {
    #[allow(missing_docs)]
    TranscriptResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
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

        /// Extracted entities, present on is_final messages when
        /// `detect_entities` is enabled. Empty otherwise.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        entities: Vec<EntityHit>,
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

        /// Deprecated transaction key echoed by the server.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        transaction_key: Option<String>,

        /// SHA-256 of the audio (or empty for streaming).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
    },
    #[allow(missing_docs)]
    SpeechStartedResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        channel: Vec<u8>,

        #[allow(missing_docs)]
        timestamp: f64,
    },
    #[allow(missing_docs)]
    UtteranceEndResponse {
        #[allow(missing_docs)]
        #[serde(rename = "type")]
        type_field: String,

        #[allow(missing_docs)]
        channel: Vec<u8>,

        #[allow(missing_docs)]
        last_word_end: f64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn transcript_response_with_entities_round_trip() {
        let raw = json!({
            "type": "Results",
            "start": 0.0,
            "duration": 1.0,
            "is_final": true,
            "speech_final": true,
            "from_finalize": false,
            "channel": {"alternatives": []},
            "metadata": {
                "request_id": "rid",
                "model_info": {"name": "n", "version": "v", "arch": "a"},
                "model_uuid": "uuid"
            },
            "channel_index": [0, 1],
            "entities": [{
                "label": "NAME",
                "value": "Alice",
                "raw_value": "alice",
                "confidence": 0.9,
                "start_word": 0,
                "end_word": 1
            }]
        });
        let resp: StreamResponse = serde_json::from_value(raw.clone()).unwrap();
        match &resp {
            StreamResponse::TranscriptResponse { entities, .. } => {
                assert_eq!(entities.len(), 1);
                assert_eq!(entities[0].label, "NAME");
                assert_eq!(entities[0].raw_value.as_deref(), Some("alice"));
            }
            _ => panic!("expected TranscriptResponse"),
        }
        assert_eq!(serde_json::to_value(&resp).unwrap(), raw);
    }

    #[test]
    fn transcript_response_without_entities_back_compat() {
        // Existing payloads without `entities` still deserialize; the
        // field defaults to an empty Vec and is skipped on serialize.
        let raw = json!({
            "type": "Results",
            "start": 0.0,
            "duration": 1.0,
            "is_final": false,
            "speech_final": false,
            "from_finalize": false,
            "channel": {"alternatives": []},
            "metadata": {
                "request_id": "rid",
                "model_info": {"name": "n", "version": "v", "arch": "a"},
                "model_uuid": "uuid"
            },
            "channel_index": [0]
        });
        let resp: StreamResponse = serde_json::from_value(raw.clone()).unwrap();
        match &resp {
            StreamResponse::TranscriptResponse { entities, .. } => {
                assert!(entities.is_empty());
            }
            _ => panic!("expected TranscriptResponse"),
        }
        assert_eq!(serde_json::to_value(&resp).unwrap(), raw);
    }

    #[test]
    fn terminal_response_with_transaction_key_and_sha256() {
        let raw = json!({
            "request_id": "rid",
            "created": "2026-05-08T12:00:00Z",
            "duration": 12.5,
            "channels": 2,
            "transaction_key": "deprecated",
            "sha256": "abc123"
        });
        let resp: StreamResponse = serde_json::from_value(raw.clone()).unwrap();
        match &resp {
            StreamResponse::TerminalResponse {
                transaction_key,
                sha256,
                ..
            } => {
                assert_eq!(transaction_key.as_deref(), Some("deprecated"));
                assert_eq!(sha256.as_deref(), Some("abc123"));
            }
            _ => panic!("expected TerminalResponse"),
        }
        assert_eq!(serde_json::to_value(&resp).unwrap(), raw);
    }
}
