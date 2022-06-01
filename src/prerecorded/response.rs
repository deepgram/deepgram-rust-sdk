use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Response {
    pub metadata: ListenMetadata,
    pub results: ListenResults,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Deserialize)]
pub struct CallbackResponse {
    pub request_id: Uuid,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ListenMetadata {
    pub request_id: Uuid,
    pub transaction_key: String,
    pub sha256: String,
    pub created: String,
    pub duration: f64,
    pub channels: usize,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ListenResults {
    pub channels: Vec<ChannelResult>,
    pub utterances: Option<Vec<Utterance>>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ChannelResult {
    pub search: Option<Vec<SearchResults>>,
    pub alternatives: Vec<ResultAlternative>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Utterance {
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
    pub channel: usize,
    pub transcript: String,
    pub words: Vec<Word>,
    pub speaker: Option<usize>,
    pub id: Uuid,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub hits: Vec<Hit>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ResultAlternative {
    pub transcript: String,
    pub confidence: f64,
    pub words: Vec<Word>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
    pub speaker: Option<usize>,
    pub punctuated_word: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Hit {
    pub confidence: f64,
    pub start: f64,
    pub end: f64,
    pub snippet: String,
}
