//! Request body for `POST /v1/read`.
//!
//! Mirrors `oneOf [ReadV1RequestUrl, ReadV1RequestText]` in
//! `openapi/schemas/schemas.read.v1.yml`.

use serde::{Deserialize, Serialize};

/// The text to analyze: either a remote URL or an inline string.
///
/// Serializes as one of:
/// - `{"url": "https://example.com/story.txt"}`
/// - `{"text": "Some text to analyze."}`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ReadRequest {
    /// Fetch and analyze a document from a URL.
    Url {
        /// URL pointing to the text document.
        url: String,
    },
    /// Analyze the given inline text.
    Text {
        /// Plain text to analyze.
        text: String,
    },
}

impl ReadRequest {
    /// Construct a URL-source request.
    pub fn url(url: impl Into<String>) -> Self {
        Self::Url { url: url.into() }
    }

    /// Construct a text-source request.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn url_serializes_with_url_field() {
        let req = ReadRequest::url("https://example.com/story.txt");
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v, json!({"url": "https://example.com/story.txt"}));
    }

    #[test]
    fn text_serializes_with_text_field() {
        let req = ReadRequest::text("Hello, world.");
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v, json!({"text": "Hello, world."}));
    }

    #[test]
    fn deserialize_routes_by_field() {
        let req: ReadRequest = serde_json::from_value(json!({"url": "https://x.example"})).unwrap();
        assert!(matches!(req, ReadRequest::Url { .. }));
        let req: ReadRequest = serde_json::from_value(json!({"text": "hi"})).unwrap();
        assert!(matches!(req, ReadRequest::Text { .. }));
    }
}
