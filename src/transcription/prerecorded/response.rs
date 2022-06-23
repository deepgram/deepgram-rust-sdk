//! Deepgram pre-recorded transcription API response types.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded-responses

use serde::Deserialize;
use uuid::Uuid;

/// Returned by [`Transcription::prerecorded`](crate::transcription::Transcription::prerecorded).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Response {
    pub metadata: ListenMetadata,
    pub results: ListenResults,
}

/// Returned by [`Transcription::prerecorded_callback`](crate::transcription::Transcription::prerecorded_callback).
///
/// See the [Deepgram Callback feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/callback/
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
#[non_exhaustive]
pub struct CallbackResponse {
    pub request_id: Uuid,
}

/// Metadata about the transcription.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
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
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ListenResults {
    pub channels: Vec<ChannelResult>,

    /// [`None`] unless the [Utterances feature][docs] is set.
    /// Features can be set using an [`OptionsBuilder`](`super::options::OptionsBuilder`).
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/utterances/
    pub utterances: Option<Vec<Utterance>>,
}

/// Transcription results for a single audio channel.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Multichannel feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/documentation/features/multichannel/
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ChannelResult {
    /// [`None`] unless the [Search feature][docs] is set.
    /// Features can be set using an [`OptionsBuilder`](`super::options::OptionsBuilder`).
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/search/
    pub search: Option<Vec<SearchResults>>,

    pub alternatives: Vec<ResultAlternative>,
}

/// Transcription results for a single utterance.
///
/// See the [Deepgram Utternace feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/utterances/
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Utterance {
    pub start: f64,
    pub end: f64,
    pub confidence: f64,
    pub channel: usize,
    pub transcript: String,
    pub words: Vec<Word>,

    /// [`None`] unless the [Diarization feature][docs] is set.
    /// Features can be set using an [`OptionsBuilder`](`super::options::OptionsBuilder`).
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/diarize/
    pub speaker: Option<usize>,

    pub id: Uuid,
}

/// Search results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/documentation/features/search/
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct SearchResults {
    pub query: String,
    pub hits: Vec<Hit>,
}

/// Transcript alternatives.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ResultAlternative {
    pub transcript: String,
    pub confidence: f64,
    pub words: Vec<Word>,
}

/// A single transcribed word.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub confidence: f64,

    /// [`None`] unless the [Diarization feature][docs] is set.
    /// Features can be set using an [`OptionsBuilder`](`super::options::OptionsBuilder`).
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/diarize/
    pub speaker: Option<usize>,

    /// [`None`] unless the [Punctuation feature][docs] is set.
    /// Features can be set using an [`OptionsBuilder`](`super::options::OptionsBuilder`).
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/punctuate/
    pub punctuated_word: Option<String>,
}

/// Search result.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/documentation/features/search/
#[allow(missing_docs)] // Struct fields are documented in the API reference
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Hit {
    pub confidence: f64,
    pub start: f64,
    pub end: f64,
    pub snippet: String,
}
