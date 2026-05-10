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

    /// Top-level language. Not in the current
    /// `ListenV1ResponseMetadata` schema (the language is on each
    /// channel via `ChannelResult.detected_language`); kept for
    /// backward compatibility, will be removed in 0.10.0 (Phase 8e).
    pub language: Option<String>,

    /// Model UUIDs that served the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,

    /// Per-model metadata, keyed by model UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_info: Option<HashMap<String, ModelInfoEntry>>,

    /// Token usage for the summarization step (when `summarize` was set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_info: Option<TokenInfo>,

    /// Token usage for the sentiment-analysis step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentiment_info: Option<TokenInfo>,

    /// Token usage for the topic-detection step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics_info: Option<TokenInfo>,

    /// Token usage for the intent-detection step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intents_info: Option<TokenInfo>,

    /// Tags echoed back from the request's `tag` query param(s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Per-model metadata entry inside [`ListenMetadata::model_info`].
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelInfoEntry {
    /// Display name of the model.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Model architecture (e.g. `nova-2`).
    pub arch: String,
}

/// Token usage and model identifier for one analytics feature
/// (summarize / sentiment / topics / intents). Shared with
/// [`crate::read::response::TokenInfo`].
#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TokenInfo {
    /// UUID of the model that produced this output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_uuid: Option<String>,

    /// Number of input tokens consumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,

    /// Number of output tokens produced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
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

/// Sentence
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Sentence {
    text: String,
    start: f64,
    end: f64,
}

/// Paragraph
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Paragraph {
    sentences: Vec<Sentence>,
    num_words: usize,
    start: f64,
    end: f64,
    /// Speaker label when diarization is enabled. None otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<usize>,
}

/// Paragraph results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/paragraphs
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Paragraphs {
    transcript: String,
    paragraphs: Vec<Paragraph>,
}

/// Entity Detection results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/detect-entities
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Entity {
    label: String,
    value: String,
    confidence: f64,
    start_word: usize,
    end_word: usize,
    /// Original spoken text of the entity, present when smart formatting is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_value: Option<String>,
}

/// Intent
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Intent {
    intent: String,
    confidence_score: f64,
}

/// Segment
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Segment {
    text: String,
    start_word: usize,
    end_word: usize,
    intents: Vec<Intent>,
}

/// Intent Recognition results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/intent-recognition
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Intents {
    segments: Vec<Segment>,
}

/// SentimentSegment
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SentimentSegment {
    text: String,
    start_word: usize,
    end_word: usize,
    sentiment: String,
    sentiment_score: f64,
}

/// SentimentAverage
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SentimentAverage {
    sentiment: String,
    sentiment_score: f64,
}

/// Sentiment Analysis results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/sentiment-analysis
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Sentiments {
    segments: Vec<SentimentSegment>,
    average: SentimentAverage,
}

/// TopicDetail
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TopicDetail {
    topic: String,
    confidence_score: f64,
}

/// TopicSegment
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TopicSegment {
    text: String,
    start_word: usize,
    end_word: usize,
    topics: Vec<TopicDetail>,
}

/// Topics Detection results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/topic-detection
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Topics {
    segments: Vec<TopicSegment>,
}

/// Summary results.
///
/// See the [Deepgram API Reference][api]
/// and the [Deepgram Search feature docs][docs] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
/// [docs]: https://developers.deepgram.com/docs/summarization
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Summary {
    result: String,
    short: String,
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

    /// Channel-level summaries (when `summarize` was set). Distinct
    /// from [`ListenResults::summary`] (document-level).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summaries: Option<Vec<ChannelSummary>>,

    /// Channel-level topic detections (when `topics` was set).
    /// Distinct from [`ListenResults::topics`] (document-level).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<ChannelTopic>>,
}

/// One channel-level summary entry on a [`ResultAlternative`].
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChannelSummary {
    /// Summary text.
    pub summary: String,
    /// Index of the first word covered by this summary.
    pub start_word: f64,
    /// Index of the last word covered by this summary.
    pub end_word: f64,
}

/// One channel-level topic entry on a [`ResultAlternative`].
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChannelTopic {
    /// Snippet of text that was classified.
    pub text: String,
    /// Index of the first word in the snippet.
    pub start_word: f64,
    /// Index of the last word in the snippet.
    pub end_word: f64,
    /// Topic labels detected on this snippet.
    #[serde(default)]
    pub topics: Vec<String>,
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

    /// Confidence of the [`speaker`](Word::speaker) assignment, when
    /// diarization is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker_confidence: Option<f64>,

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
    use serde_json::json;

    // Tests below assert deserialization shape only (not strict JSON
    // round-trip equality). Several pre-existing optional fields on
    // batch_response types serialize `None` as `null` rather than
    // omitting them — normalizing that wire behavior is a Phase 8
    // cleanup, not Phase 7.

    #[test]
    fn metadata_with_model_info_and_token_info() {
        let raw = json!({
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "transaction_key": "deprecated",
            "sha256": "abc",
            "created": "2026-05-08T12:00:00Z",
            "duration": 12.5,
            "channels": 1,
            "models": ["30089e05-99d1-4376-b32e-c263170674af"],
            "model_info": {
                "30089e05-99d1-4376-b32e-c263170674af": {
                    "name": "2-general-nova",
                    "version": "2024-01-09.29447",
                    "arch": "nova-2"
                }
            },
            "summary_info": {
                "model_uuid": "67875a7f-c9c4-48a0-aa55-5bdb8a91c34a",
                "input_tokens": 95,
                "output_tokens": 63
            },
            "tags": ["staging"]
        });
        let m: ListenMetadata = serde_json::from_value(raw).unwrap();
        assert_eq!(m.models.as_ref().unwrap().len(), 1);
        let info = m.model_info.as_ref().unwrap();
        assert_eq!(info["30089e05-99d1-4376-b32e-c263170674af"].arch, "nova-2");
        assert_eq!(m.summary_info.as_ref().unwrap().input_tokens, Some(95));
        assert_eq!(m.tags.as_deref().unwrap(), &["staging".to_string()]);
    }

    #[test]
    fn metadata_minimal_deserializes_without_new_fields() {
        let raw = json!({
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "transaction_key": "deprecated",
            "sha256": "abc",
            "created": "2026-05-08T12:00:00Z",
            "duration": 12.5,
            "channels": 1
        });
        let m: ListenMetadata = serde_json::from_value(raw).unwrap();
        assert!(m.models.is_none());
        assert!(m.summary_info.is_none());
        assert!(m.tags.is_none());
    }

    #[test]
    fn word_speaker_confidence_round_trip() {
        let raw = json!({
            "word": "hello",
            "start": 0.0,
            "end": 0.5,
            "confidence": 0.95,
            "speaker": 0,
            "speaker_confidence": 0.88,
            "punctuated_word": "Hello,"
        });
        let w: Word = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(w.speaker_confidence, Some(0.88));
        assert_eq!(serde_json::to_value(&w).unwrap(), raw);
    }

    #[test]
    fn entity_raw_value_round_trip() {
        let raw = json!({
            "label": "PHONE_NUMBER",
            "value": "555-1234",
            "raw_value": "five five five one two three four",
            "confidence": 0.91,
            "start_word": 3,
            "end_word": 6
        });
        let e: Entity = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(
            e.raw_value.as_deref(),
            Some("five five five one two three four")
        );
        assert_eq!(serde_json::to_value(&e).unwrap(), raw);
    }

    #[test]
    fn paragraph_speaker_round_trip() {
        let raw = json!({
            "sentences": [{"text": "Hi.", "start": 0.0, "end": 0.5}],
            "num_words": 1,
            "start": 0.0,
            "end": 0.5,
            "speaker": 2
        });
        let p: Paragraph = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.speaker, Some(2));
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn channel_summaries_and_topics_deserialize() {
        let raw = json!({
            "transcript": "Hello world",
            "confidence": 0.97,
            "words": [],
            "summaries": [
                {"summary": "A greeting.", "start_word": 0.0, "end_word": 1.0}
            ],
            "topics": [
                {"text": "Hello world", "start_word": 0.0, "end_word": 1.0,
                 "topics": ["greeting"]}
            ]
        });
        let alt: ResultAlternative = serde_json::from_value(raw).unwrap();
        assert_eq!(alt.summaries.as_ref().unwrap().len(), 1);
        assert_eq!(
            alt.topics.as_ref().unwrap()[0].topics,
            vec!["greeting".to_string()]
        );
    }
}
