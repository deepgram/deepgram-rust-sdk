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
/// The `Debug` output redacts header values (header names are kept) and
/// any `user:pass@` userinfo in the URL, so that logging a `Settings`,
/// `ThinkSettings`, or `SpeakSettings` never leaks an `Authorization`
/// header, an API-key header, or a credential embedded in the endpoint
/// URL. Serialization is unaffected.
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
            .field("url", &RedactedUrl(&self.url))
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

/// `Debug` adapter for a URL that prints everything except the parts that
/// carry a credential — `user:pass@` userinfo in the authority, and the
/// value of any query parameter whose name names a secret — each replaced
/// with `"<redacted>"`.
///
/// A credential can live in a URL (`https://user:pass@llm.internal`) or in
/// its query string (`?api-key=…`), so printing `url` verbatim would defeat
/// the header redaction next to it.
/// Works whether or not the value carries a scheme, since the field is a
/// free-form `String`. Shared by [`Endpoint`] and `FunctionEndpoint`;
/// same spirit as the origin-only `url` on `InsecureAgentUrl`.
pub(crate) struct RedactedUrl<'a>(pub(crate) &'a str);

impl fmt::Debug for RedactedUrl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The two steps are independent: a URL can carry userinfo, a secret
        // query parameter, or both.
        let without_userinfo = redact_userinfo(self.0);
        let source = without_userinfo.as_deref().unwrap_or(self.0);
        match redact_query_secrets(source) {
            Some(redacted) => fmt::Debug::fmt(&redacted, f),
            None => fmt::Debug::fmt(source, f),
        }
    }
}

/// Replace a URL's `user[:pass]@` userinfo with `<redacted>@`, or return
/// `None` when there is nothing to redact.
///
/// Deliberately string-based rather than going through `url::Url`: the
/// field is a free-form `String` the caller may not have written as a
/// parseable URL, and a value that fails to parse must still be printed
/// with its userinfo removed.
///
/// A missing scheme is one such value (`bob:hunter2@llm.internal`), so
/// the authority is located relative to whichever prefix is present —
/// `scheme://`, a protocol-relative `//`, or nothing at all — instead of
/// requiring `://`. Only an `@` inside that authority is userinfo: an
/// `@` in a path or query (an email address in a parameter) is left
/// alone either way.
fn redact_userinfo(url: &str) -> Option<String> {
    let (prefix, rest) = match url.split_once("://") {
        // `scheme://…`: keep the scheme, scan from the authority.
        Some((scheme, rest)) => (&url[..scheme.len() + "://".len()], rest),
        // Protocol-relative `//authority/…`, then the schemeless
        // `authority/…` form a free-form field invites.
        None => match url.strip_prefix("//") {
            Some(rest) => ("//", rest),
            None => ("", url),
        },
    };
    // The authority ends at the first `/`, `?`, or `#`; a later `@` (in a
    // path or query) is not userinfo.
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let at = rest[..authority_end].rfind('@')?;
    Some(format!("{prefix}{REDACTED}@{}", &rest[at + 1..]))
}

/// Replace the value of any query parameter whose name names a credential
/// with `<redacted>`, or return `None` when there is nothing to redact.
///
/// A credential is routinely a query parameter rather than a header — Google
/// AI Studio takes `?key=`, Azure OpenAI takes `?api-key=`, and a pre-signed
/// URL carries `?signature=` — and `think.endpoint` / `speak.endpoint` are
/// exactly the fields a caller points at those services. Redacting only the
/// userinfo and the header values would print such a key in full, right next
/// to a header value that reads as redacted.
///
/// Deliberately string-based, for the same reason as [`redact_userinfo`]:
/// the field is a free-form `String` that may not parse as a URL.
fn redact_query_secrets(url: &str) -> Option<String> {
    let (before, query) = url.split_once('?')?;
    // A fragment is not part of the query.
    let (query, fragment) = match query.split_once('#') {
        Some((query, fragment)) => (query, Some(fragment)),
        None => (query, None),
    };
    let mut redacted_any = false;
    let mut out = String::with_capacity(query.len());
    for (index, pair) in query.split('&').enumerate() {
        if index > 0 {
            out.push('&');
        }
        match pair.split_once('=') {
            Some((name, _)) if is_secret_param(name) => {
                redacted_any = true;
                out.push_str(name);
                out.push('=');
                out.push_str(REDACTED);
            }
            _ => out.push_str(pair),
        }
    }
    if !redacted_any {
        return None;
    }
    Some(match fragment {
        Some(fragment) => format!("{before}?{out}#{fragment}"),
        None => format!("{before}?{out}"),
    })
}

/// Whether a query-parameter name names a credential. Compared with `-` and
/// `_` removed and ignoring case, so `api-key`, `api_key` and `apiKey` all
/// match the same entry.
fn is_secret_param(name: &str) -> bool {
    let normalized: String = name
        .chars()
        .filter(|c| *c != '-' && *c != '_')
        .flat_map(char::to_lowercase)
        .collect();
    matches!(
        normalized.as_str(),
        "key"
            | "apikey"
            | "apisecret"
            | "accesskey"
            | "secretkey"
            | "secret"
            | "clientsecret"
            | "token"
            | "accesstoken"
            | "idtoken"
            | "refreshtoken"
            | "sessiontoken"
            | "password"
            | "passwd"
            | "pwd"
            | "auth"
            | "authorization"
            | "sig"
            | "signature"
            | "subscriptionkey"
    )
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
    fn debug_redacts_url_userinfo() {
        // A credential can live in the URL itself, so `Debug` must strip
        // `user:pass@` the way it strips header values.
        let endpoint = Endpoint::new("https://alice:s3cret@llm.internal:8443/v1/chat?k=v");
        let debug = format!("{endpoint:?}");
        assert!(!debug.contains("s3cret"), "got: {debug}");
        assert!(!debug.contains("alice"), "got: {debug}");
        // Everything a reader needs to identify the endpoint is kept.
        assert!(
            debug.contains("llm.internal:8443/v1/chat?k=v"),
            "got: {debug}"
        );
        assert!(debug.contains("<redacted>@"), "got: {debug}");
    }

    #[test]
    fn debug_redacts_credentials_carried_in_the_query_string() {
        // Google AI Studio takes `?key=`, Azure OpenAI takes `?api-key=`;
        // those are the services `think.endpoint` points at, so a query
        // credential must not survive into `Debug` just because it is not a
        // header.
        for (url, secret) in [
            (
                "https://generativelanguage.googleapis.com/v1/x?key=AIzaSECRET",
                "AIzaSECRET",
            ),
            (
                "https://acme.openai.azure.com/v1/chat?api-key=azSECRET",
                "azSECRET",
            ),
            ("https://fn.internal/charge?token=FNSECRET&id=7", "FNSECRET"),
            ("https://s3.example/obj?signature=SIGSECRET", "SIGSECRET"),
            ("https://llm.internal/v1?Api_Key=MIXEDSECRET", "MIXEDSECRET"),
        ] {
            let debug = format!("{:?}", Endpoint::new(url));
            assert!(!debug.contains(secret), "leaked {secret} in: {debug}");
            assert!(debug.contains("<redacted>"), "got: {debug}");
        }
    }

    #[test]
    fn debug_keeps_non_secret_query_parameters_and_the_rest_of_the_url() {
        // Redaction is per-parameter: everything a reader needs to identify
        // the endpoint stays.
        let debug = format!(
            "{:?}",
            Endpoint::new("https://llm.internal/v1/chat?model=gpt-4o&api-key=SECRET&stream=true")
        );
        assert!(!debug.contains("SECRET"), "got: {debug}");
        assert!(debug.contains("model=gpt-4o"), "got: {debug}");
        assert!(debug.contains("stream=true"), "got: {debug}");
        assert!(debug.contains("api-key=<redacted>"), "got: {debug}");
        assert!(
            debug.contains("https://llm.internal/v1/chat?"),
            "got: {debug}"
        );
    }

    #[test]
    fn debug_redacts_userinfo_and_a_query_secret_together() {
        let debug = format!(
            "{:?}",
            Endpoint::new("https://alice:s3cret@llm.internal/v1/chat?api-key=qSECRET#frag")
        );
        assert!(!debug.contains("s3cret"), "got: {debug}");
        assert!(!debug.contains("qSECRET"), "got: {debug}");
        assert!(debug.contains("<redacted>@"), "got: {debug}");
        assert!(debug.contains("api-key=<redacted>"), "got: {debug}");
        // The fragment is not part of the query and survives.
        assert!(debug.contains("#frag"), "got: {debug}");
    }

    #[test]
    fn debug_keeps_a_url_without_userinfo_verbatim() {
        for url in [
            "https://llm.internal/v1/chat",
            // An `@` after the authority is not userinfo.
            "https://llm.internal/v1/chat?to=a@b.com",
            // Not a parseable URL at all: still printed, still safe.
            "llm.internal/v1/chat",
        ] {
            let debug = format!("{:?}", Endpoint::new(url));
            assert!(debug.contains(url), "got: {debug}");
        }
    }

    #[test]
    fn debug_redacts_userinfo_in_an_unparseable_url() {
        // `url` is a free-form String; redaction must not depend on the
        // value being a valid URL.
        let debug = format!("{:?}", Endpoint::new("wss://bob:hunter2@"));
        assert!(!debug.contains("hunter2"), "got: {debug}");
        assert!(!debug.contains("bob"), "got: {debug}");
    }

    #[test]
    fn debug_redacts_userinfo_in_a_schemeless_url() {
        // `url` is free-form, so a host:port or userinfo form with no
        // scheme at all is a value a caller can write. The credential
        // must not survive into `Debug` just because `://` is absent.
        for url in [
            "bob:hunter2@llm.internal",
            "bob:hunter2@llm.internal:8443/v1/chat?k=v",
            // Userinfo with no password is still userinfo.
            "bob@llm.internal/v1/chat",
            // Protocol-relative form.
            "//bob:hunter2@llm.internal/v1/chat",
        ] {
            let debug = format!("{:?}", Endpoint::new(url));
            assert!(!debug.contains("hunter2"), "got: {debug}");
            assert!(!debug.contains("bob"), "got: {debug}");
            assert!(debug.contains("<redacted>@"), "got: {debug}");
            // The host is still there to identify the endpoint by.
            assert!(debug.contains("llm.internal"), "got: {debug}");
        }
    }

    #[test]
    fn debug_keeps_an_at_outside_a_schemeless_authority_verbatim() {
        // The no-scheme path uses the same authority boundary as the
        // scheme-bearing one, so an address in a path or query is not
        // mistaken for a credential.
        for url in [
            "llm.internal/v1/chat?to=a@b.com",
            "llm.internal/mail/a@b.com",
            "/v1/chat?to=a@b.com",
            "//llm.internal/v1/chat?to=a@b.com",
        ] {
            let debug = format!("{:?}", Endpoint::new(url));
            assert!(debug.contains(url), "got: {debug}");
        }
    }

    #[test]
    fn debug_redaction_does_not_affect_serialization() {
        let endpoint = Endpoint::new("https://example.com").with_header("Authorization", "secret");
        let json = serde_json::to_value(&endpoint).unwrap();
        assert_eq!(json["headers"]["Authorization"], "secret");
    }
}
