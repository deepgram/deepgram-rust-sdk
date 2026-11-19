//! Deepgram pre-recorded transcription API response types.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded-responses

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// `Transcription` only exists with the `listen` feature, so the link is
// emitted only when it will resolve. docs.rs builds all features.
#[cfg_attr(
    feature = "listen",
    doc = "Returned by [`Transcription::prerecorded`](crate::Transcription::prerecorded)."
)]
#[cfg_attr(
    not(feature = "listen"),
    doc = "Returned by `Transcription::prerecorded`."
)]
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

// `Transcription` only exists with the `listen` feature, so the link is
// emitted only when it will resolve. docs.rs builds all features.
#[cfg_attr(
    feature = "listen",
    doc = "Returned by [`Transcription::prerecorded_callback`](crate::Transcription::prerecorded_callback)."
)]
#[cfg_attr(
    not(feature = "listen"),
    doc = "Returned by `Transcription::prerecorded_callback`."
)]
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

    /// Model UUIDs that processed the request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub models: Vec<String>,

    /// Metadata keyed by each model UUID in [`ListenMetadata::models`].
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub model_info: HashMap<String, ModelInfo>,

    /// The diarization model that produced speaker labels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diarize_info: Option<DiarizeInfo>,

    /// Summarization model usage metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_info: Option<AnalysisInfo>,

    /// Sentiment model usage metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentiment_info: Option<AnalysisInfo>,

    /// Topic model usage metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics_info: Option<AnalysisInfo>,

    /// Intent model usage metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intents_info: Option<AnalysisInfo>,

    /// Request tags echoed by the service.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

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

/// Metadata for a speech-to-text model used in a request.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelInfo {
    /// Model name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Model version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Model architecture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

/// The diarization model that produced speaker labels.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DiarizeInfo {
    /// Diarization model UUID.
    pub model_uuid: String,

    /// Diarization model architecture.
    pub arch: String,
}

/// Token usage metadata for an Audio Intelligence feature.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalysisInfo {
    /// Model UUID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_uuid: Option<String>,

    /// Number of input tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<usize>,

    /// Number of output tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<usize>,
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

    /// Languages detected in the utterance, sorted by word count.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,

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
    /// The text of the sentence.
    pub text: String,

    /// Number of seconds into the audio at which this sentence starts.
    pub start: f64,

    /// Number of seconds into the audio at which this sentence ends.
    pub end: f64,
}

/// A single paragraph of the transcript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Paragraph {
    /// The sentences this paragraph was divided into, in spoken order.
    pub sentences: Vec<Sentence>,

    /// Speaker identifier when diarization is enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<usize>,

    /// Count of the words in this paragraph.
    pub num_words: usize,

    /// Number of seconds into the audio at which this paragraph starts.
    pub start: f64,

    /// Number of seconds into the audio at which this paragraph ends.
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
    /// The transcript for the processed audio, with line breaks inserted
    /// where it is divided into paragraphs.
    pub transcript: String,

    /// The paragraphs the transcript was divided into, in spoken order.
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
    /// The type of entity identified, for example `NAME`, `ORGANIZATION`,
    /// `PHONE_NUMBER`, `EMAIL_ADDRESS`, `ADDRESS`, or `CARDINAL`. The set of
    /// labels the model can return is not a fixed list.
    pub label: String,

    /// The entity text, formatted when Smart Formatting is enabled. Not
    /// necessarily a substring of the transcript: formatting can rewrite the
    /// words (`five five five` becomes `(555`) and trailing punctuation the
    /// transcript carries is dropped.
    pub value: String,

    /// The entity text as it was spoken, before a formatting feature rewrote
    /// it. `None` unless a formatting feature such as `smart_format` is
    /// enabled, and also `None` for an entity that formatting left alone. With
    /// `smart_format=true` a live probe returned it for five of six entities —
    /// `PHONE_NUMBER` as `five five five` against a `value` of `(555` — and
    /// omitted it for the one entity (`NAME`) whose text was unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_value: Option<String>,

    /// The model's confidence in this entity, from `0.0` to `1.0`. Larger
    /// values indicate higher confidence.
    pub confidence: f64,

    /// Index of the entity's first word, inclusive, in the transcript's
    /// word list.
    pub start_word: usize,

    /// Index of the entity's last word, exclusive, in the transcript's
    /// word list.
    pub end_word: usize,
}

/// A single detected intent.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Intent {
    /// The name of the intent the model detected. Returned as a verb phrase,
    /// for example `Upgrade phone`; the set of intents is not a fixed list
    /// unless `custom_intent_mode` is `strict`.
    pub intent: String,

    /// The model's confidence in this intent, from `0.0` to `1.0`.
    pub confidence_score: f64,
}

/// A segment of the transcript with detected intents.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct Segment {
    /// The transcript text covered by this segment.
    pub text: String,

    /// Index of this segment's first word, inclusive, in the transcript's
    /// word list.
    pub start_word: usize,

    /// Index of this segment's last word, **inclusive**, in the transcript's
    /// word list. Note that this differs from [`Entity::end_word`], which is
    /// exclusive.
    pub end_word: usize,

    /// The intents the model detected in this segment. A segment can carry
    /// more than one intent.
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
    /// The segments of text the model identified as carrying notable intents.
    /// These segments do not necessarily cover the whole transcript.
    pub segments: Vec<Segment>,
}

/// A segment of the transcript with a sentiment score.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct SentimentSegment {
    /// The transcript text covered by this segment.
    pub text: String,

    /// Index of this segment's first word, inclusive, in the transcript's
    /// word list.
    pub start_word: usize,

    /// Index of this segment's last word, **inclusive**, in the transcript's
    /// word list. Note that this differs from [`Entity::end_word`], which is
    /// exclusive.
    pub end_word: usize,

    /// The sentiment classification for this segment: `positive`, `negative`,
    /// or `neutral`.
    pub sentiment: String,

    /// The sentiment of this segment, from `-1.0` (most negative) to `1.0`
    /// (most positive). Scores within roughly `0.333` of zero are classified
    /// as `neutral`.
    pub sentiment_score: f64,
}

/// The average sentiment across the transcript.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct SentimentAverage {
    /// The sentiment classification for the transcript as a whole:
    /// `positive`, `negative`, or `neutral`.
    pub sentiment: String,

    /// The sentiment of the transcript as a whole, from `-1.0` (most
    /// negative) to `1.0` (most positive).
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
    /// Per-segment sentiment for the transcript, in spoken order.
    pub segments: Vec<SentimentSegment>,

    /// The sentiment aggregated over the whole transcript.
    pub average: SentimentAverage,
}

/// A single detected topic.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct TopicDetail {
    /// The name of the topic the model detected. Topics are generated from
    /// context rather than drawn from a fixed list, unless
    /// `custom_topic_mode` is `strict`.
    pub topic: String,

    /// The model's confidence in this topic, from `0.0` to `1.0`.
    pub confidence_score: f64,
}

/// A segment of the transcript with detected topics.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[non_exhaustive]
pub struct TopicSegment {
    /// The transcript text covered by this segment.
    pub text: String,

    /// Index of this segment's first word, inclusive, in the transcript's
    /// word list.
    pub start_word: usize,

    /// Index of this segment's last word, **inclusive**, in the transcript's
    /// word list. Note that this differs from [`Entity::end_word`], which is
    /// exclusive.
    pub end_word: usize,

    /// The topics the model detected in this segment. A segment can carry
    /// more than one topic.
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
    /// The segments of text in which the model detected topics. These
    /// segments do not necessarily cover the whole transcript.
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
    /// Status of the summarization request: `success` or `failure`. This is
    /// not the summary text -- see [`Summary::short`].
    pub result: String,

    /// The generated summary of the audio.
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

    /// Summaries for transcript spans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summaries: Option<Vec<AlternativeSummary>>,

    /// Topics for transcript spans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<AlternativeTopic>>,

    #[allow(missing_docs)]
    #[serde(default)]
    pub languages: Vec<String>,

    /// Normalized `language_hint` values active for this response.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages_hinted: Vec<String>,
}

/// A summary for a span of transcript words.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AlternativeSummary {
    /// Generated summary text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Inclusive position of the first summarized word.
    ///
    /// The service documents this as a number, so decimal values are
    /// preserved when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_word: Option<f64>,

    /// Inclusive position of the last summarized word.
    ///
    /// The service documents this as a number, so decimal values are
    /// preserved when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_word: Option<f64>,
}

/// Topics detected in a span of transcript words.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AlternativeTopic {
    /// Transcript text covered by this topic result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Inclusive position of the first covered word.
    ///
    /// The service documents this as a number, so decimal values are
    /// preserved when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_word: Option<f64>,

    /// Inclusive position of the last covered word.
    ///
    /// The service documents this as a number, so decimal values are
    /// preserved when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_word: Option<f64>,

    /// Topic strings detected in this span.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

    /// Detected BCP-47 language tag, present for multilingual transcription.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// [`None`] unless the [Diarization feature][docs] is set.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/diarize/
    pub speaker: Option<usize>,

    /// Confidence in the speaker assignment, present for pre-recorded
    /// diarization.
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

    // Contract fixture for documented metadata fields. It is not a captured
    // live payload; the live-captured entity and sentiment fixtures below
    // retain their provenance separately.
    #[test]
    fn listen_metadata_exposes_documented_model_and_analysis_fields() {
        let raw = serde_json::json!({
            "transaction_key": "deprecated",
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "sha256": "abc123",
            "created": "2026-11-16T10:00:00.000Z",
            "duration": 12.5,
            "channels": 1,
            "language": null,
            "models": ["30089e05-99d1-4376-b32e-c263170674af"],
            "model_info": {
                "30089e05-99d1-4376-b32e-c263170674af": {
                    "name": "nova-3",
                    "version": "2026-01-01",
                    "arch": "nova-3"
                }
            },
            "diarize_info": { "model_uuid": "diarizer-id", "arch": "v2" },
            "summary_info": { "model_uuid": "summary-id", "input_tokens": 95, "output_tokens": 63 },
            "sentiment_info": { "model_uuid": "sentiment-id", "input_tokens": 105, "output_tokens": 105 },
            "topics_info": { "model_uuid": "topics-id", "input_tokens": 105, "output_tokens": 7 },
            "intents_info": { "model_uuid": "intents-id", "input_tokens": 105, "output_tokens": 4 },
            "tags": ["production", "customer:42"]
        });
        let metadata: ListenMetadata = serde_json::from_value(raw.clone()).unwrap();

        assert_eq!(
            metadata.models,
            vec!["30089e05-99d1-4376-b32e-c263170674af"]
        );
        assert_eq!(
            metadata.model_info["30089e05-99d1-4376-b32e-c263170674af"]
                .arch
                .as_deref(),
            Some("nova-3")
        );
        assert_eq!(metadata.diarize_info.as_ref().unwrap().arch, "v2");
        assert_eq!(
            metadata.summary_info.as_ref().unwrap().output_tokens,
            Some(63)
        );
        assert_eq!(metadata.tags, vec!["production", "customer:42"]);
        assert_eq!(serde_json::to_value(metadata).unwrap(), raw);
    }

    // Contract fixture based on the documented alternative-scoped summary and
    // topic response shape. It is not a captured live payload: the available
    // live sample did not produce summaries or topics.
    #[test]
    fn alternatives_and_utterances_expose_multilingual_and_analysis_fields() {
        let raw_alternative = serde_json::json!({
            "transcript": "Hola Deepgram",
            "confidence": 0.99,
            "languages": ["es"],
            "languages_hinted": ["en", "es"],
            "words": [{
                "word": "Hola",
                "start": 0.0,
                "end": 0.3,
                "confidence": 0.99,
                "language": "es",
                "speaker": 0,
                "speaker_confidence": 0.98,
                "punctuated_word": "Hola"
            }],
            "paragraphs": {
                "transcript": "Hola Deepgram",
                "paragraphs": [{
                    "sentences": [{ "text": "Hola Deepgram", "start": 0.0, "end": 0.8 }],
                    "speaker": 0,
                    "num_words": 2,
                    "start": 0.0,
                    "end": 0.8
                }]
            },
            "summaries": [{ "summary": "A greeting.", "start_word": 0.0, "end_word": 1.0 }],
            "topics": [{ "text": "Hola Deepgram", "start_word": 0.0, "end_word": 1.0, "topics": ["greeting"] }]
            ,"entities": null
        });
        let alternative: ResultAlternative =
            serde_json::from_value(raw_alternative.clone()).unwrap();
        assert_eq!(alternative.languages_hinted, vec!["en", "es"]);
        assert_eq!(alternative.words[0].language.as_deref(), Some("es"));
        assert_eq!(alternative.words[0].speaker_confidence, Some(0.98));
        assert_eq!(
            alternative.paragraphs.as_ref().unwrap().paragraphs[0].speaker,
            Some(0)
        );
        assert_eq!(
            alternative.summaries.as_ref().unwrap()[0]
                .summary
                .as_deref(),
            Some("A greeting.")
        );
        assert_eq!(
            alternative.topics.as_ref().unwrap()[0].topics,
            vec!["greeting"]
        );
        assert_eq!(serde_json::to_value(alternative).unwrap(), raw_alternative);

        let raw_utterance = serde_json::json!({
            "start": 0.0,
            "end": 0.8,
            "confidence": 0.99,
            "channel": 0,
            "transcript": "Hola Deepgram",
            "languages": ["es"],
            "words": [],
            "speaker": null,
            "id": "550e8400-e29b-41d4-a716-446655440000"
        });
        let utterance: Utterance = serde_json::from_value(raw_utterance.clone()).unwrap();
        assert_eq!(utterance.languages, vec!["es"]);
        assert_eq!(serde_json::to_value(utterance).unwrap(), raw_utterance);
    }

    #[test]
    fn alternative_scoped_analysis_entries_accept_sparse_server_payloads() {
        let summary: AlternativeSummary = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(summary.summary.is_none());
        assert!(summary.start_word.is_none());
        assert!(summary.end_word.is_none());
        assert_eq!(
            serde_json::to_value(summary).unwrap(),
            serde_json::json!({})
        );

        let topic: AlternativeTopic = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(topic.text.is_none());
        assert!(topic.start_word.is_none());
        assert!(topic.end_word.is_none());
        assert!(topic.topics.is_empty());
        assert_eq!(serde_json::to_value(topic).unwrap(), serde_json::json!({}));
    }

    #[test]
    fn alternative_scoped_analysis_entries_accept_decimal_word_positions() {
        let summary: AlternativeSummary = serde_json::from_value(serde_json::json!({
            "summary": "A greeting.",
            "start_word": 5.25,
            "end_word": 8.75
        }))
        .unwrap();
        assert_eq!(summary.start_word, Some(5.25));
        assert_eq!(summary.end_word, Some(8.75));

        let integer_summary: AlternativeSummary = serde_json::from_value(serde_json::json!({
            "start_word": 29,
            "end_word": 30
        }))
        .unwrap();
        assert_eq!(integer_summary.start_word, Some(29.0));
        assert_eq!(integer_summary.end_word, Some(30.0));

        let topic: AlternativeTopic = serde_json::from_value(serde_json::json!({
            "text": "Hello Deepgram",
            "start_word": 5.25,
            "end_word": 8.75,
            "topics": ["greeting"]
        }))
        .unwrap();
        assert_eq!(topic.start_word, Some(5.25));
        assert_eq!(topic.end_word, Some(8.75));
    }

    // Captured from a live `POST /v1/listen` response on 2026-09-15
    // (nova-3, smart_format + detect_entities + sentiment). These fixtures pin
    // what the SDK's types decode off a real payload, and that the two
    // word-index conventions differ the way the docs on those fields say --
    // they cannot detect the server changing convention, only the SDK drifting
    // from the payload recorded here.
    const LIVE_ENTITY: &str = r#"{
        "label": "ORDINAL",
        "value": "first",
        "confidence": 0.9988257,
        "start_word": 9,
        "end_word": 10
    }"#;

    // Same session, a closed-loop probe: text with a phone number synthesized
    // with aura-2-thalia-en, then transcribed with
    // `smart_format=true&detect_entities=true`. Smart Formatting rewrote
    // "five five five" as "(555", so the entity carries both forms.
    const LIVE_ENTITY_WITH_RAW_VALUE: &str = r#"{
        "label": "PHONE_NUMBER",
        "value": "(555",
        "raw_value": "five five five",
        "confidence": 0.99934894,
        "start_word": 9,
        "end_word": 10
    }"#;

    // The one entity of the six in that same response that formatting left
    // alone: the server omits `raw_value` entirely.
    const LIVE_ENTITY_WITHOUT_RAW_VALUE: &str = r#"{
        "label": "NAME",
        "value": "Jane Doe",
        "confidence": 0.9998472,
        "start_word": 3,
        "end_word": 5
    }"#;

    #[test]
    fn entity_word_indices_are_end_exclusive() {
        let entity: Entity = serde_json::from_str(LIVE_ENTITY).unwrap();
        // Live transcript word 9 is "first", and the entity covers exactly it.
        assert_eq!(entity.start_word, 9);
        assert_eq!(entity.end_word, 10);
        assert_eq!(
            entity.end_word - entity.start_word,
            entity.value.split_whitespace().count(),
            "Entity::end_word is exclusive, so the span equals the word count"
        );
    }

    #[test]
    fn entity_raw_value_is_read_when_formatting_rewrote_the_entity() {
        let entity: Entity = serde_json::from_str(LIVE_ENTITY_WITH_RAW_VALUE).unwrap();

        assert_eq!(entity.value, "(555");
        assert_eq!(entity.raw_value.as_deref(), Some("five five five"));

        // Round-trip: a present raw_value must survive serialization, or a
        // caller re-emitting a response would silently drop it.
        let json = serde_json::to_value(&entity).unwrap();
        assert_eq!(json["raw_value"], "five five five");
    }

    #[test]
    fn entity_raw_value_is_none_when_the_server_omits_it() {
        let entity: Entity = serde_json::from_str(LIVE_ENTITY_WITHOUT_RAW_VALUE).unwrap();

        assert_eq!(entity.value, "Jane Doe");
        assert!(entity.raw_value.is_none());
        // Two words, and the exclusive end index gives exactly two.
        assert_eq!(
            entity.end_word - entity.start_word,
            entity.value.split_whitespace().count()
        );

        let json = serde_json::to_value(&entity).unwrap();
        assert!(
            json.get("raw_value").is_none(),
            "absent raw_value must not serialize as null"
        );
    }

    // The sentiment segment from the same live response as LIVE_ENTITY.
    // `end_word` is 61 and the text carries 62 words -- one more than an
    // exclusive end index would give.
    const LIVE_SENTIMENT_SEGMENT: &str = r#"{
        "text": "Yeah. As as much as, it's worth celebrating, the first, spacewalk, with an all female team, I think many of us are looking forward to it just being normal. And, I think if it signifies anything, it is, to honor the the women who came before us who, were skilled and qualified, and didn't get the same opportunities that we have today.",
        "start_word": 0,
        "end_word": 61,
        "sentiment": "positive",
        "sentiment_score": 0.6363217830657959
    }"#;

    #[test]
    fn intelligence_segment_word_indices_are_end_inclusive() {
        let segment: SentimentSegment = serde_json::from_str(LIVE_SENTIMENT_SEGMENT).unwrap();

        assert_eq!(segment.start_word, 0);
        assert_eq!(segment.end_word, 61);
        assert_eq!(segment.text.split_whitespace().count(), 62);
        assert_eq!(
            segment.end_word - segment.start_word + 1,
            segment.text.split_whitespace().count(),
            "segment end_word is inclusive, unlike Entity::end_word"
        );
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
