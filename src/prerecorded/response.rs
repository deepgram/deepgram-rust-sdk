use serde::Deserialize;
use uuid::Uuid;

/// Returned by [`Deepgram::prerecorded_request`](crate::Deepgram::prerecorded_request).
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded) for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Response {
    pub metadata: ListenMetadata,
    pub results: ListenResults,
}

/// Returned by [`Deepgram::callback_request`](crate::Deepgram::callback_request).
///
/// See the [Deepgram Callback feature](https://developers.deepgram.com/documentation/features/callback/) docs for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Deserialize)]
pub struct CallbackResponse {
    pub request_id: Uuid,
}

/// Metadata about the transcription.
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded) for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ListenMetadata {
    pub request_id: Uuid,
    pub transaction_key: String,
    pub sha256: String,
    pub created: String,
    pub duration: f64,
    pub channels: usize,
}

/// Transcription results.
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded) for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ListenResults {
    pub channels: Vec<ChannelResult>,

    /// [`None`] unless the [Utterances feature](https://developers.deepgram.com/documentation/features/utterances/) is set.
    /// Features can be set using an [`OptionsBuilder`](`super::OptionsBuilder`).
    pub utterances: Option<Vec<Utterance>>,
}

/// Transcription results for a single audio channel.
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded)
/// and the [Deepgram Multichannel feature](https://developers.deepgram.com/documentation/features/multichannel/) docs for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ChannelResult {
    /// [`None`] unless the [Search feature](https://developers.deepgram.com/documentation/features/search/) is set.
    /// Features can be set using an [`OptionsBuilder`](`super::OptionsBuilder`).
    pub search: Option<Vec<SearchResults>>,

    pub alternatives: Vec<ResultAlternative>,
}

/// Transcription results for a single utterance.
///
/// See the [Deepgram Utternace feature](https://developers.deepgram.com/documentation/features/utterances/) docs for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Utterance {
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
    pub channel: usize,
    pub transcript: String,
    pub words: Vec<Word>,

    /// [`None`] unless the [Diarization feature](https://developers.deepgram.com/documentation/features/diarize/) is set.
    /// Features can be set using an [`OptionsBuilder`](`super::OptionsBuilder`).
    pub speaker: Option<usize>,

    pub id: Uuid,
}

/// Search results.
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded)
/// and the [Deepgram Search feature](https://developers.deepgram.com/documentation/features/search/) docs for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub hits: Vec<Hit>,
}

/// Transcript alternatives.
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded) for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct ResultAlternative {
    pub transcript: String,
    pub confidence: f64,
    pub words: Vec<Word>,
}

/// A single transcribed word.
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded) for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,

    /// [`None`] unless the [Diarization feature](https://developers.deepgram.com/documentation/features/diarize/) is set.
    /// Features can be set using an [`OptionsBuilder`](`super::OptionsBuilder`).
    pub speaker: Option<usize>,

    /// [`None`] unless the [Punctuation feature](https://developers.deepgram.com/documentation/features/diarize/) is set.
    /// Features can be set using an [`OptionsBuilder`](`super::OptionsBuilder`).
    pub punctuated_word: Option<String>,
}

/// Search result.
///
/// See the [Deepgram API Reference](https://developers.deepgram.com/api-reference/#transcription-prerecorded) for more info.
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
pub struct Hit {
    pub confidence: f64,
    pub start: f64,
    pub end: f64,
    pub snippet: String,
}
