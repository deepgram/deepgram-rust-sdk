//! Deepgram pre-recorded transcription API response types.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded-responses

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Returned by [`Transcription::prerecorded`](crate::Transcription::prerecorded).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Response {
    #[allow(missing_docs)]
    pub metadata: ListenMetadata,

    #[allow(missing_docs)]
    pub results: ListenResults,
}

/// Returned by [`Transcription::prerecorded_callback`](crate::Transcription::prerecorded_callback).
///
/// See the [Deepgram Callback feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/callback/
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

    #[allow(missing_docs)]
    pub language: Option<String>,

    /// Arbitrary key-value pairs echoed back from the `extra` request
    /// parameter, for use in downstream processing.
    ///
    /// [`None`] unless the [Extra Metadata feature][docs] is set.
    ///
    /// [docs]: https://developers.deepgram.com/docs/extra-metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, String>>,
}

/// Transcription results.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ListenResults {
    #[allow(missing_docs)]
    pub channels: Vec<ChannelResult>,

    /// [`None`] unless the [Utterances feature][docs] is set.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/utterances/
    pub utterances: Option<Vec<Utterance>>,

    #[allow(missing_docs)]
    pub intents: Option<Intents>,

    #[allow(missing_docs)]
    pub sentiments: Option<Sentiments>,

    #[allow(missing_docs)]
    pub topics: Option<Topics>,

    #[allow(missing_docs)]
    pub summary: Option<Summary>,
}

/// Transcription results for a single audio channel.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Multichannel feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/documentation/features/multichannel/
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChannelResult {
    /// [`None`] unless the [Search feature][docs] is set.
    ///
    /// [docs]: https://developers.deepgram.com/docs/search/
    pub search: Option<Vec<SearchResults>>,

    #[allow(missing_docs)]
    pub alternatives: Vec<ResultAlternative>,

    ///  [BCP-47][bcp47] language tag for the dominant language identified in the channel.
    ///
    /// [`None`] unless the [Language Detection feature][docs] is set.
    ///
    /// [bcp47]: https://tools.ietf.org/html/bcp47
    /// [docs]: https://developers.deepgram.com/docs/language-detection/
    pub detected_language: Option<String>,
}

/// Transcription results for a single utterance.
///
/// See the [Deepgram Utterance feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/utterances/
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
    ///
    /// [docs]: https://developers.deepgram.com/docs/diarization
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SearchResults {
    #[allow(missing_docs)]
    pub query: String,

    #[allow(missing_docs)]
    pub hits: Vec<Hit>,
}

/// A single sentence within a [`Paragraph`].
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Sentence {
    #[allow(missing_docs)]
    pub text: String,

    #[allow(missing_docs)]
    pub start: f64,

    #[allow(missing_docs)]
    pub end: f64,
}

/// A single paragraph of the transcript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Paragraph {
    #[allow(missing_docs)]
    pub sentences: Vec<Sentence>,

    #[allow(missing_docs)]
    pub num_words: usize,

    #[allow(missing_docs)]
    pub start: f64,

    #[allow(missing_docs)]
    pub end: f64,
}

/// Paragraph results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/paragraphs
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Paragraphs {
    #[allow(missing_docs)]
    pub transcript: String,

    #[allow(missing_docs)]
    pub paragraphs: Vec<Paragraph>,
}

/// Entity Detection results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/detect-entities
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Entity {
    #[allow(missing_docs)]
    pub label: String,

    #[allow(missing_docs)]
    pub value: String,

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    pub start_word: usize,

    #[allow(missing_docs)]
    pub end_word: usize,
}

/// A single detected intent.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Intent {
    #[allow(missing_docs)]
    pub intent: String,

    #[allow(missing_docs)]
    pub confidence_score: f64,
}

/// A segment of the transcript with detected intents.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Segment {
    #[allow(missing_docs)]
    pub text: String,

    #[allow(missing_docs)]
    pub start_word: usize,

    #[allow(missing_docs)]
    pub end_word: usize,

    #[allow(missing_docs)]
    pub intents: Vec<Intent>,
}

/// Intent Recognition results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/intent-recognition
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Intents {
    #[allow(missing_docs)]
    pub segments: Vec<Segment>,
}

/// A segment of the transcript with a sentiment score.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct SentimentSegment {
    #[allow(missing_docs)]
    pub text: String,

    #[allow(missing_docs)]
    pub start_word: usize,

    #[allow(missing_docs)]
    pub end_word: usize,

    #[allow(missing_docs)]
    pub sentiment: String,

    #[allow(missing_docs)]
    pub sentiment_score: f64,
}

/// The average sentiment across the transcript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct SentimentAverage {
    #[allow(missing_docs)]
    pub sentiment: String,

    #[allow(missing_docs)]
    pub sentiment_score: f64,
}

/// Sentiment Analysis results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/sentiment-analysis
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Sentiments {
    #[allow(missing_docs)]
    pub segments: Vec<SentimentSegment>,

    #[allow(missing_docs)]
    pub average: SentimentAverage,
}

/// A single detected topic.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct TopicDetail {
    #[allow(missing_docs)]
    pub topic: String,

    #[allow(missing_docs)]
    pub confidence_score: f64,
}

/// A segment of the transcript with detected topics.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct TopicSegment {
    #[allow(missing_docs)]
    pub text: String,

    #[allow(missing_docs)]
    pub start_word: usize,

    #[allow(missing_docs)]
    pub end_word: usize,

    #[allow(missing_docs)]
    pub topics: Vec<TopicDetail>,
}

/// Topics Detection results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/topic-detection
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Topics {
    #[allow(missing_docs)]
    pub segments: Vec<TopicSegment>,
}

/// Summary results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/summarization
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Summary {
    #[allow(missing_docs)]
    pub result: String,

    #[allow(missing_docs)]
    pub short: String,
}

/// Transcript alternatives.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ResultAlternative {
    #[allow(missing_docs)]
    pub transcript: String,

    #[allow(missing_docs)]
    pub confidence: f64,

    #[allow(missing_docs)]
    pub words: Vec<Word>,

    #[allow(missing_docs)]
    pub paragraphs: Option<Paragraphs>,

    #[allow(missing_docs)]
    pub entities: Option<Vec<Entity>>,

    #[allow(missing_docs)]
    #[serde(default)]
    pub languages: Vec<String>,
}

/// A single transcribed word.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/diarize/
    pub speaker: Option<usize>,

    /// [`None`] unless the [Punctuation feature][docs] is set.
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA_WITH_EXTRA: &str = r#"{
        "transaction_key": "deprecated",
        "request_id": "550e8400-e29b-41d4-a716-446655440000",
        "sha256": "abc123",
        "created": "2026-09-11T10:00:00.000Z",
        "duration": 12.5,
        "channels": 1,
        "extra": {
            "customer_id": "cust-42",
            "session": "abc"
        }
    }"#;

    const METADATA_WITHOUT_EXTRA: &str = r#"{
        "transaction_key": "deprecated",
        "request_id": "550e8400-e29b-41d4-a716-446655440000",
        "sha256": "abc123",
        "created": "2026-09-11T10:00:00.000Z",
        "duration": 12.5,
        "channels": 1
    }"#;

    #[test]
    fn listen_metadata_exposes_extra() {
        let metadata: ListenMetadata = serde_json::from_str(METADATA_WITH_EXTRA).unwrap();

        let extra = metadata.extra.expect("extra should be present");
        assert_eq!(extra.len(), 2);
        assert_eq!(
            extra.get("customer_id").map(String::as_str),
            Some("cust-42")
        );
        assert_eq!(extra.get("session").map(String::as_str), Some("abc"));
    }

    #[test]
    fn listen_metadata_without_extra_is_none() {
        let metadata: ListenMetadata = serde_json::from_str(METADATA_WITHOUT_EXTRA).unwrap();

        assert_eq!(metadata.duration, 12.5);
        assert_eq!(metadata.channels, 1);
        assert!(metadata.extra.is_none());
    }

    #[test]
    fn listen_metadata_extra_round_trips() {
        let metadata: ListenMetadata = serde_json::from_str(METADATA_WITH_EXTRA).unwrap();
        let json = serde_json::to_value(&metadata).unwrap();
        assert_eq!(json["extra"]["customer_id"], "cust-42");
        assert_eq!(json["extra"]["session"], "abc");

        let without: ListenMetadata = serde_json::from_str(METADATA_WITHOUT_EXTRA).unwrap();
        let json = serde_json::to_value(&without).unwrap();
        assert!(
            json.get("extra").is_none(),
            "absent extra must not serialize as null"
        );
    }
}
