//! Response shape for `POST /v1/read`.
//!
//! Mirrors `ReadV1Response` in `openapi/schemas/schemas.read.v1.yml`.
//!
//! Two pieces of the wire shape are unusually nested (matching what
//! Python and JS SDKs auto-generate from the same spec):
//!
//! - **`metadata.metadata`**: the outer `metadata` field wraps an
//!   inner `metadata` object containing the actual fields. Use
//!   [`ReadResponse::metadata_inner`] to reach the inner directly.
//! - **`results.summary.results.summary.text`**: the summary text
//!   sits four levels deep. Use [`ReadResponse::summary_text`] to
//!   skip the wrappers.

use serde::{Deserialize, Serialize};

use crate::common::batch_response::{Intents, Sentiments, Topics};

/// Top-level response from `POST /v1/read`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadResponse {
    /// Outer wrapper around the actual metadata.
    pub metadata: ReadMetadataWrapper,
    /// Analysis results.
    pub results: ReadResults,
}

impl ReadResponse {
    /// Convenience accessor for the metadata's inner fields.
    pub fn metadata_inner(&self) -> Option<&ReadMetadata> {
        self.metadata.metadata.as_ref()
    }

    /// Convenience accessor for the summary text — climbs the
    /// `results.summary.results.summary.text` nested chain.
    pub fn summary_text(&self) -> Option<&str> {
        self.results
            .summary
            .as_ref()?
            .results
            .as_ref()?
            .summary
            .as_ref()?
            .text
            .as_deref()
    }
}

/// The outer wrapper on `metadata`. The actual fields live one level
/// deeper at [`ReadMetadataWrapper::metadata`] — this matches the spec
/// shape (and the Python/JS auto-generated SDKs).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadMetadataWrapper {
    /// Inner metadata block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ReadMetadata>,
}

/// Inner metadata for a Read response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadMetadata {
    /// Unique identifier for the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,

    /// ISO 8601 creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// Input language as detected/declared.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Token usage and model UUID for the summarization step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary_info: Option<TokenInfo>,

    /// Token usage and model UUID for the sentiment analysis step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentiment_info: Option<TokenInfo>,

    /// Token usage and model UUID for the topic detection step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics_info: Option<TokenInfo>,

    /// Token usage and model UUID for the intent detection step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intents_info: Option<TokenInfo>,
}

/// Per-feature token usage and model identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TokenInfo {
    /// UUID of the model that produced this output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_uuid: Option<String>,

    /// Number of tokens of input consumed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,

    /// Number of tokens of output produced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
}

/// Results block on a [`ReadResponse`].
///
/// `topics`, `intents`, and `sentiments` are reused from
/// [`crate::common::batch_response`] since they share the
/// `schemas.shared.yml` definitions with the Listen API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadResults {
    /// Summary output (when `summarize=true`). Three levels of nesting
    /// per spec — prefer [`ReadResponse::summary_text`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReadSummaryWrapper>,

    /// Detected topics (when `topics=true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topics: Option<Topics>,

    /// Detected intents (when `intents=true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intents: Option<Intents>,

    /// Sentiment analysis (when `sentiment=true`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentiments: Option<Sentiments>,
}

/// Outer wrapper for the summary block: `results.summary.results....`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadSummaryWrapper {
    /// Inner `results` object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<ReadSummaryInner>,
}

/// Middle wrapper: `results.summary.results.summary.text`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadSummaryInner {
    /// Inner `summary` object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReadSummaryText>,
}

/// Innermost summary block — finally just the text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReadSummaryText {
    /// Summary text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn metadata_inner_round_trip() {
        let raw = json!({
            "metadata": {
                "metadata": {
                    "request_id": "abc-123",
                    "created": "2026-05-08T12:00:00Z",
                    "language": "en",
                    "summary_info": {
                        "model_uuid": "uuid-1",
                        "input_tokens": 100,
                        "output_tokens": 30
                    }
                }
            },
            "results": {}
        });
        let resp: ReadResponse = serde_json::from_value(raw.clone()).unwrap();
        let inner = resp.metadata_inner().expect("inner metadata");
        assert_eq!(inner.request_id.as_deref(), Some("abc-123"));
        assert_eq!(inner.language.as_deref(), Some("en"));
        let summary_info = inner.summary_info.as_ref().unwrap();
        assert_eq!(summary_info.input_tokens, Some(100));
        assert_eq!(summary_info.output_tokens, Some(30));
        assert_eq!(serde_json::to_value(&resp).unwrap(), raw);
    }

    #[test]
    fn summary_text_climbs_quadruple_nesting() {
        let raw = json!({
            "metadata": {},
            "results": {
                "summary": {
                    "results": {
                        "summary": {
                            "text": "A short summary."
                        }
                    }
                }
            }
        });
        let resp: ReadResponse = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(resp.summary_text(), Some("A short summary."));
        assert_eq!(serde_json::to_value(&resp).unwrap(), raw);
    }

    #[test]
    fn missing_metadata_inner_yields_none() {
        let raw = json!({
            "metadata": {},
            "results": {}
        });
        let resp: ReadResponse = serde_json::from_value(raw).unwrap();
        assert!(resp.metadata_inner().is_none());
        assert!(resp.summary_text().is_none());
    }

    #[test]
    fn missing_summary_path_yields_none() {
        let raw = json!({
            "metadata": {},
            "results": { "summary": {} }
        });
        let resp: ReadResponse = serde_json::from_value(raw).unwrap();
        assert!(resp.summary_text().is_none());
    }
}
