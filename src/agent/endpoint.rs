//! Custom endpoint configuration for non-Deepgram Think and Speak providers.
//!
//! Mirrors the `endpoint` object on `agent.think` and `agent.speak` in the
//! Voice Agent `Settings` message.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Custom HTTPS/WSS endpoint with optional headers.
///
/// Used to point a Think or Speak provider at a self-hosted or proxied
/// endpoint instead of the provider's default. Required when using a
/// non-Deepgram provider; optional with the Deepgram speak provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Endpoint {
    /// Endpoint URL. `https://` for REST providers; `wss://` is only
    /// supported by ElevenLabs.
    pub url: String,

    /// Custom headers to send on every request to the endpoint.
    /// Empty map serializes as `{}`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

impl Endpoint {
    /// Create an endpoint with the given URL and no extra headers.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: HashMap::new(),
        }
    }

    /// Add a single header. Useful for fluent construction.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_with_headers() {
        let raw = json!({
            "url": "https://llm.internal/v1/chat",
            "headers": {
                "Authorization": "Bearer abc",
                "X-Tenant": "acme",
            }
        });
        let endpoint: Endpoint = serde_json::from_value(raw).unwrap();
        assert_eq!(endpoint.url, "https://llm.internal/v1/chat");
        assert_eq!(endpoint.headers.len(), 2);
        assert_eq!(
            endpoint.headers.get("Authorization").map(String::as_str),
            Some("Bearer abc")
        );
    }

    #[test]
    fn deserialize_without_headers() {
        let raw = json!({ "url": "https://example.com" });
        let endpoint: Endpoint = serde_json::from_value(raw).unwrap();
        assert!(endpoint.headers.is_empty());
    }

    #[test]
    fn serialize_skips_empty_headers() {
        let endpoint = Endpoint::new("https://example.com");
        let json = serde_json::to_value(&endpoint).unwrap();
        assert_eq!(json, json!({ "url": "https://example.com" }));
    }

    #[test]
    fn with_header_chain() {
        let endpoint = Endpoint::new("https://example.com")
            .with_header("X-One", "1")
            .with_header("X-Two", "2");
        assert_eq!(endpoint.headers.len(), 2);
    }
}
