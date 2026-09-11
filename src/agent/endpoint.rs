//! Custom endpoint configuration for non-Deepgram Think and Speak providers.
//!
//! Mirrors the `endpoint` object on `agent.think` and `agent.speak` in the
//! Voice Agent `Settings` message.

use core::fmt;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Custom HTTPS/WSS endpoint with optional headers.
///
/// Used to point a Think or Speak provider at a self-hosted or proxied
/// endpoint instead of the provider's default. On `speak`, required for
/// every non-Deepgram provider and optional for the Deepgram provider. On
/// `think`, required for `groq` and `aws_bedrock` (Deepgram does not
/// manage those LLMs) and optional for `open_ai`, `anthropic`, and
/// `google`, where omitting it selects Deepgram's managed LLM.
///
/// The `Debug` output redacts header values (header names are kept) so
/// that logging a `Settings`, `ThinkSettings`, or `SpeakSettings` never
/// leaks an `Authorization` or API-key header. Serialization is
/// unaffected.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Endpoint")
            .field("url", &self.url)
            .field("headers", &RedactedHeaders(&self.headers))
            .finish()
    }
}

/// `Debug` adapter for a header map that prints header names but replaces
/// every value with `"<redacted>"`.
///
/// Shared by [`Endpoint`] and `FunctionEndpoint`; same spirit as the
/// crate-level `RedactedString` used for the API key.
pub(crate) struct RedactedHeaders<'a>(pub(crate) &'a HashMap<String, String>);

impl fmt::Debug for RedactedHeaders<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut map = f.debug_map();
        for name in self.0.keys() {
            map.entry(name, &REDACTED);
        }
        map.finish()
    }
}

/// Placeholder printed in place of any secret in `Debug` output.
pub(crate) const REDACTED: &str = "<redacted>";

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

    #[test]
    fn debug_redacts_header_values_but_keeps_names_and_url() {
        let endpoint = Endpoint::new("https://llm.internal/v1/chat")
            .with_header("Authorization", "Bearer sk-live-0123456789")
            .with_header("X-Tenant", "acme-tenant-id");
        let debug = format!("{endpoint:?}");
        assert!(!debug.contains("sk-live-0123456789"), "got: {debug}");
        assert!(!debug.contains("acme-tenant-id"), "got: {debug}");
        assert!(debug.contains("Authorization"), "got: {debug}");
        assert!(debug.contains("X-Tenant"), "got: {debug}");
        assert!(debug.contains("<redacted>"), "got: {debug}");
        assert!(
            debug.contains("https://llm.internal/v1/chat"),
            "got: {debug}"
        );
    }

    #[test]
    fn debug_redaction_does_not_affect_serialization() {
        let endpoint = Endpoint::new("https://example.com").with_header("Authorization", "secret");
        let json = serde_json::to_value(&endpoint).unwrap();
        assert_eq!(json["headers"]["Authorization"], "secret");
    }
}
