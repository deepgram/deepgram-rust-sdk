//! Deepgram Text Intelligence (`/v1/read`) response types.
//!
//! See the [Deepgram Text Intelligence docs][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/docs/text-intelligence

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// The per-segment sentiment, topic, and intent types are identical to the
// pre-recorded Audio Intelligence response, so we reuse them here rather than
// duplicating.
pub use crate::common::batch_response::{
    Intent, Intents, Segment, SentimentAverage, SentimentSegment, Sentiments, TopicDetail,
    TopicSegment, Topics,
};

/// Returned by [`TextIntelligence::analyze_text`](crate::read::TextIntelligence::analyze_text) and
/// [`TextIntelligence::analyze_url`](crate::read::TextIntelligence::analyze_url).
///
/// See the [Deepgram Text Intelligence docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/text-intelligence
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Response {
    #[allow(missing_docs)]
    pub metadata: ReadMetadata,

    #[allow(missing_docs)]
    pub results: ReadResults,
}

/// Metadata about a Text Intelligence request.
///
/// The `/v1/read` reference marks every field optional, so each is an
/// [`Option`]: a contract-valid response that omits one still deserializes.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadMetadata {
    #[allow(missing_docs)]
    pub request_id: Option<Uuid>,

    #[allow(missing_docs)]
    pub created: Option<String>,

    /// The language of the analyzed text.
    pub language: Option<String>,

    /// Model/token usage info for the sentiment analysis, if requested.
    pub sentiment_info: Option<AnalysisInfo>,

    /// Model/token usage info for the summarization, if requested.
    pub summary_info: Option<AnalysisInfo>,

    /// Model/token usage info for the topic detection, if requested.
    pub topics_info: Option<AnalysisInfo>,

    /// Model/token usage info for the intent recognition, if requested.
    pub intents_info: Option<AnalysisInfo>,
}

/// Per-feature model and token usage information.
///
/// The `/v1/read` reference marks every field optional, so each is an
/// [`Option`]: a contract-valid response that omits one still deserializes.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalysisInfo {
    #[allow(missing_docs)]
    pub model_uuid: Option<String>,

    #[allow(missing_docs)]
    pub input_tokens: Option<u32>,

    #[allow(missing_docs)]
    pub output_tokens: Option<u32>,
}

/// Text Intelligence results.
///
/// Each field is populated only when the corresponding feature was requested.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadResults {
    #[allow(missing_docs)]
    pub sentiments: Option<Sentiments>,

    #[allow(missing_docs)]
    pub summary: Option<Summary>,

    #[allow(missing_docs)]
    pub topics: Option<Topics>,

    #[allow(missing_docs)]
    pub intents: Option<Intents>,
}

/// Summarization result for the Text Intelligence API.
///
/// Note this differs from the pre-recorded transcription
/// [`Summary`](crate::common::batch_response::Summary), which carries `result`
/// and `short`; the `/v1/read` endpoint returns only `text`.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Summary {
    /// A short summary of the submitted text.
    ///
    /// The `/v1/read` reference marks this optional, so it is an [`Option`]:
    /// a contract-valid response carrying an empty `summary` object still
    /// deserializes rather than failing the whole request. When checked on
    /// 2026-09-23 the live endpoint sent it on every successful response,
    /// including for inputs too short to summarize, where it echoed the
    /// input back; treat that as the common case, not a guarantee.
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::Response;

    #[test]
    fn deserializes_documented_response() {
        // Shape taken from the Deepgram Text Intelligence docs (sentiment +
        // summary examples merged).
        let json = serde_json::json!({
            "metadata": {
                "request_id": "7dcd719f-344b-4c72-a194-6bd1019d855c",
                "created": "2023-12-01T15:54:39.681Z",
                "language": "en",
                "sentiment_info": {
                    "model_uuid": "80ab3179-d113-4254-bd6b-4a2f96498695",
                    "input_tokens": 22,
                    "output_tokens": 22
                },
                "summary_info": {
                    "model_uuid": "67875a7f-c9c4-48a0-aa55-5bdb8a91c34a",
                    "input_tokens": 103,
                    "output_tokens": 33
                }
            },
            "results": {
                "sentiments": {
                    "segments": [{
                        "text": "Hi. Thank you for calling.",
                        "start_word": 0,
                        "end_word": 8,
                        "sentiment": "positive",
                        "sentiment_score": 0.738
                    }],
                    "average": { "sentiment": "positive", "sentiment_score": 0.397 }
                },
                "summary": { "text": "A short summary." }
            }
        });

        let response: Response = serde_json::from_value(json).unwrap();
        assert_eq!(
            response.metadata.request_id.map(|id| id.to_string()),
            Some("7dcd719f-344b-4c72-a194-6bd1019d855c".to_string())
        );
        assert_eq!(
            response.metadata.created.as_deref(),
            Some("2023-12-01T15:54:39.681Z")
        );
        assert_eq!(
            response
                .metadata
                .sentiment_info
                .as_ref()
                .and_then(|info| info.input_tokens),
            Some(22)
        );
        assert_eq!(response.metadata.language.as_deref(), Some("en"));
        assert_eq!(
            response.results.summary.unwrap().text.as_deref(),
            Some("A short summary.")
        );
        let sentiments = response.results.sentiments.unwrap();
        assert_eq!(sentiments.average.sentiment, "positive");
        assert_eq!(sentiments.segments.len(), 1);
        // topics/intents were not requested.
        assert!(response.results.topics.is_none());
        assert!(response.results.intents.is_none());
    }

    #[test]
    fn deserializes_metadata_with_omitted_optional_fields() {
        // The `/v1/read` reference marks every `metadata` field optional, and
        // every field of the per-feature `*_info` objects too. A response that
        // omits any of them must still deserialize rather than failing the
        // whole request.
        let json = serde_json::json!({
            "metadata": { "summary_info": {} },
            "results": { "summary": { "text": "A short summary." } }
        });

        let response: Response = serde_json::from_value(json).unwrap();
        assert!(response.metadata.request_id.is_none());
        assert!(response.metadata.created.is_none());
        assert!(response.metadata.language.is_none());
        assert!(response.metadata.sentiment_info.is_none());
        assert!(response.metadata.topics_info.is_none());
        assert!(response.metadata.intents_info.is_none());

        let summary_info = response.metadata.summary_info.unwrap();
        assert!(summary_info.model_uuid.is_none());
        assert!(summary_info.input_tokens.is_none());
        assert!(summary_info.output_tokens.is_none());
    }

    #[test]
    fn deserializes_summary_with_omitted_text() {
        // The `/v1/read` reference marks `results.summary.text` optional, so a
        // successful response carrying an empty `summary` object must
        // deserialize into `Summary { text: None }` rather than failing the
        // whole request before the caller can inspect any other result.
        let json = serde_json::json!({
            "metadata": {},
            "results": { "summary": {} }
        });

        let response: Response = serde_json::from_value(json).unwrap();
        let summary = response.results.summary.expect("summary present");
        assert!(summary.text.is_none());
    }

    #[test]
    fn deserializes_live_response_with_every_feature() {
        // Shape captured from a live `POST /v1/read` on 2026-09-23 with
        // `language=en&summarize=true&sentiment=true&topics=true&intents=true`
        // (identifiers replaced). Every feature was requested, so every
        // `results` member and every `*_info` block is populated.
        let json = serde_json::json!({
            "metadata": {
                "request_id": "00000000-0000-4000-8000-000000000000",
                "created": "2026-09-23T13:56:35.206Z",
                "language": "en",
                "summary_info": {
                    "model_uuid": "11111111-1111-4111-8111-111111111111",
                    "input_tokens": 93,
                    "output_tokens": 52
                },
                "sentiment_info": {
                    "model_uuid": "22222222-2222-4222-8222-222222222222",
                    "input_tokens": 95,
                    "output_tokens": 95
                },
                "topics_info": {
                    "model_uuid": "33333333-3333-4333-8333-333333333333",
                    "input_tokens": 95,
                    "output_tokens": 14
                },
                "intents_info": {
                    "model_uuid": "44444444-4444-4444-8444-444444444444",
                    "input_tokens": 95,
                    "output_tokens": 12
                }
            },
            "results": {
                "summary": {
                    "text": "The customer calls about a cracked phone screen."
                },
                "topics": {
                    "segments": [{
                        "text": "my phone screen cracked last week",
                        "start_word": 26,
                        "end_word": 82,
                        "topics": [{
                            "topic": "Phone screen repair/replacement options",
                            "confidence_score": 0.014938718
                        }]
                    }]
                },
                "intents": {
                    "segments": [{
                        "text": "my phone screen cracked last week",
                        "start_word": 26,
                        "end_word": 82,
                        "intents": [{
                            "intent": "Request repair/replacement information",
                            "confidence_score": 1.25285105e-05
                        }]
                    }]
                },
                "sentiments": {
                    "segments": [{
                        "text": "Hi, thank you for calling Premier Phone Service.",
                        "start_word": 0,
                        "end_word": 7,
                        "sentiment": "positive",
                        "sentiment_score": 0.6423379778862
                    }],
                    "average": {
                        "sentiment": "neutral",
                        "sentiment_score": 0.3200141191482544
                    }
                }
            }
        });

        let response: Response = serde_json::from_value(json).unwrap();
        assert_eq!(
            response
                .results
                .summary
                .as_ref()
                .and_then(|summary| summary.text.as_deref()),
            Some("The customer calls about a cracked phone screen.")
        );
        assert_eq!(response.results.topics.unwrap().segments.len(), 1);
        assert_eq!(response.results.intents.unwrap().segments.len(), 1);
        assert_eq!(
            response.results.sentiments.unwrap().average.sentiment,
            "neutral"
        );
        assert_eq!(
            response
                .metadata
                .intents_info
                .and_then(|info| info.output_tokens),
            Some(12)
        );
    }
}
