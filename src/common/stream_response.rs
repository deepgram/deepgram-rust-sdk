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
