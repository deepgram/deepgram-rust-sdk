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

/// Returned by [`Read::analyze_text`](crate::read::Read::analyze_text) and
/// [`Read::analyze_url`](crate::read::Read::analyze_url).
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadMetadata {
    #[allow(missing_docs)]
    pub request_id: Uuid,

    #[allow(missing_docs)]
    pub created: String,

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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnalysisInfo {
    #[allow(missing_docs)]
    pub model_uuid: String,

    #[allow(missing_docs)]
    pub input_tokens: u32,

    #[allow(missing_docs)]
    pub output_tokens: u32,
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
    pub text: String,
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
        assert_eq!(response.metadata.language.as_deref(), Some("en"));
        assert_eq!(response.results.summary.unwrap().text, "A short summary.");
        let sentiments = response.results.sentiments.unwrap();
        assert_eq!(sentiments.average.sentiment, "positive");
        assert_eq!(sentiments.segments.len(), 1);
        // topics/intents were not requested.
        assert!(response.results.topics.is_none());
        assert!(response.results.intents.is_none());
    }
}
