use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
    pub speaker: Option<i32>,
    pub punctuated_word: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alternatives {
    pub transcript: String,
    pub words: Vec<Word>,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    pub alternatives: Vec<Alternatives>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub arch: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub request_id: String,
    pub model_info: ModelInfo,
    pub model_uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StreamResponse {
    TranscriptResponse {
        #[serde(rename = "type")]
        type_field: String,
        start: f64,
        duration: f64,
        is_final: bool,
        speech_final: bool,
        from_finalize: bool,
        channel: Channel,
        metadata: Metadata,
        channel_index: Vec<i32>,
    },
    TerminalResponse {
        request_id: String,
        created: String,
        duration: f64,
        channels: u32,
    },
    SpeechStartedResponse {
        #[serde(rename = "type")]
        type_field: String,
        channel: Vec<u8>,
        timestamp: f64,
    },
    UtteranceEndResponse {
        #[serde(rename = "type")]
        type_field: String,
        channel: Vec<u8>,
        last_word_end: f64,
    },
}