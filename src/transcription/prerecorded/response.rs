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
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Response {
    #[allow(missing_docs)]
    pub metadata: ListenMetadata,

    #[allow(missing_docs)]
    pub results: ListenResults,
}

/// Returned by [`Transcription::prerecorded_callback`](crate::transcription::Transcription::prerecorded_callback).
///
/// See the [Deepgram Callback feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/callback/
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct CallbackResponse {
    #[allow(missing_docs)]
    pub request_id: Uuid,
}

/// Metadata about the transcription.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ListenMetadata {
    #[allow(missing_docs)]
    pub request_id: Uuid,

    #[allow(missing_docs)]
    pub transaction_key: String,

    #[allow(missing_docs)]
    pub sha256: String,

    #[allow(missing_docs)]
    pub created: String,

    #[allow(missing_docs)]
    pub duration: f64,

    #[allow(missing_docs)]
    pub channels: usize,
}

/// Transcription results.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ListenResults {
    #[allow(missing_docs)]
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
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ChannelResult {
    /// [`None`] unless the [Search feature][docs] is set.
    /// Features can be set using an [`OptionsBuilder`](`super::options::OptionsBuilder`).
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/search/
    pub search: Option<Vec<SearchResults>>,

    #[allow(missing_docs)]
    pub alternatives: Vec<ResultAlternative>,
}

/// Transcription results for a single utterance.
///
/// See the [Deepgram Utterance feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/utterances/
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Utterance {
    #[allow(missing_docs)]
    pub start: f64,

    #[allow(missing_docs)]
    pub end: f64,

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    pub channel: usize,

    #[allow(missing_docs)]
    pub transcript: String,

    #[allow(missing_docs)]
    pub words: Vec<Word>,

    /// [`None`] unless the [Diarization feature][docs] is set.
    /// Features can be set using an [`OptionsBuilder`](`super::options::OptionsBuilder`).
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/diarize/
    pub speaker: Option<usize>,

    #[allow(missing_docs)]
    pub id: Uuid,
}

/// Search results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/documentation/features/search/
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct SearchResults {
    #[allow(missing_docs)]
    pub query: String,

    #[allow(missing_docs)]
    pub hits: Vec<Hit>,
}

/// Transcript alternatives.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ResultAlternative {
    #[allow(missing_docs)]
    pub transcript: String,

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    pub words: Vec<Word>,
}

/// A single transcribed word.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Word {
    #[allow(missing_docs)]
    pub word: String,

    #[allow(missing_docs)]
    pub start: f64,

    #[allow(missing_docs)]
    pub end: f64,

    #[allow(missing_docs)]
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
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Hit {
    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    pub start: f64,

    #[allow(missing_docs)]
    pub end: f64,

    #[allow(missing_docs)]
    pub snippet: String,
}
