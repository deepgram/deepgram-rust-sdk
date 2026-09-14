//! Agent Listen (speech-to-text) provider settings.
//!
//! Mirrors the `agent.listen` block on `AgentV1SettingsMessage` in
//! `asyncapi/schemas/schemas.agent.v1.yml`. The provider is a `oneOf`
//! between two Deepgram-typed shapes that share `type: "deepgram"` but
//! differ on `version` (`v1` vs `v2`/Flux STT). Discrimination here is on
//! the `version` field via a custom [`Deserialize`] impl.

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// `agent.listen` block — wraps a single provider configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentListenSettings {
    /// STT provider.
    pub provider: AgentListenProvider,
}

impl AgentListenSettings {
    /// Construct with the given provider.
    pub fn new(provider: AgentListenProvider) -> Self {
        Self { provider }
    }
}

/// Speech-to-text provider for the Voice Agent. Currently only Deepgram is
/// supported, with two API versions (V1 and V2/Flux STT).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AgentListenProvider {
    /// V1 Deepgram STT (Nova/Nova-2/Nova-3).
    DeepgramV1(DeepgramListenV1Provider),
    /// V2 Deepgram STT (Flux STT). `model` is required.
    DeepgramV2(DeepgramListenV2Provider),
}

impl Serialize for AgentListenProvider {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::DeepgramV1(p) => p.serialize(ser),
            Self::DeepgramV2(p) => p.serialize(ser),
        }
    }
}

impl<'de> Deserialize<'de> for AgentListenProvider {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(de)?;
        let version = value.get("version").and_then(Value::as_str).unwrap_or("v1");
        match version {
            "v2" => serde_json::from_value::<DeepgramListenV2Provider>(value)
                .map(Self::DeepgramV2)
                .map_err(D::Error::custom),
            "v1" => serde_json::from_value::<DeepgramListenV1Provider>(value)
                .map(Self::DeepgramV1)
                .map_err(D::Error::custom),
            other => Err(D::Error::custom(format!(
                "unknown agent listen provider version {other:?} (expected `v1` or `v2`)"
            ))),
        }
    }
}

/// Wire-level discriminator for the Deepgram STT provider type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeepgramProviderType {
    /// Always serializes as `"deepgram"`.
    #[default]
    #[serde(rename = "deepgram")]
    Deepgram,
}

/// Wire-level discriminator for V1 of the Deepgram STT API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeepgramListenV1Version {
    /// Always serializes as `"v1"`.
    #[default]
    #[serde(rename = "v1")]
    V1,
}

/// Wire-level discriminator for V2 (Flux STT) of the Deepgram STT API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeepgramListenV2Version {
    /// Always serializes as `"v2"`.
    #[serde(rename = "v2")]
    V2,
}

/// Deepgram V1 STT provider (Nova-3, Nova-2, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeepgramListenV1Provider {
    /// Always [`DeepgramProviderType::Deepgram`].
    #[serde(rename = "type", default)]
    pub provider_type: DeepgramProviderType,

    /// API version. Defaults to v1 if absent on the wire.
    #[serde(default)]
    pub version: DeepgramListenV1Version,

    /// Model name (e.g. `nova-3`, `nova-2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Spoken-language hint. BCP-47 tag (e.g. `en`) or `multi` for
    /// code-switching transcription. Spec default is `en-US`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Keyterms to boost recognition for specialized terminology.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyterms: Vec<String>,

    /// Whether to apply smart formatting to the transcript.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smart_format: Option<bool>,
}

impl DeepgramListenV1Provider {
    /// Construct an empty V1 provider config — model defaults to whatever
    /// the agent server selects.
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(missing_docs)]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    #[allow(missing_docs)]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    #[allow(missing_docs)]
    pub fn with_keyterms<I, S>(mut self, keyterms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keyterms = keyterms.into_iter().map(Into::into).collect();
        self
    }

    #[allow(missing_docs)]
    pub fn with_smart_format(mut self, smart_format: bool) -> Self {
        self.smart_format = Some(smart_format);
        self
    }
}

impl Default for DeepgramListenV1Provider {
    fn default() -> Self {
        Self {
            provider_type: DeepgramProviderType::Deepgram,
            version: DeepgramListenV1Version::V1,
            model: None,
            language: None,
            keyterms: Vec::new(),
            smart_format: None,
        }
    }
}

/// Deepgram V2 (Flux STT) provider. `model` is required per spec.
///
/// Mirrors `DeepgramListenProviderV2` in
/// `asyncapi/schemas/agent/listen-providers/deepgram-v2.yml`. The
/// end-of-turn fields (`eot_threshold`, `eager_eot_threshold`,
/// `eot_timeout_ms`) tune Flux STT's built-in turn detection and can also be
/// changed mid-session with `UpdateListen`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeepgramListenV2Provider {
    /// Always [`DeepgramProviderType::Deepgram`].
    #[serde(rename = "type", default)]
    pub provider_type: DeepgramProviderType,

    /// API version — V2 (Flux STT).
    pub version: DeepgramListenV2Version,

    /// Flux model identifier (e.g. `flux-general-en`, `flux-general-multi`).
    pub model: String,

    /// BCP-47 language codes that bias `flux-general-multi` toward specific
    /// languages. Always serialized as a JSON array under `language_hints`,
    /// even for a single hint. Without hints the model auto-detects the
    /// spoken language; single-language models ignore the field.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_string_one_or_many"
    )]
    pub language_hints: Vec<String>,

    /// End-of-turn confidence required to finish a turn. Valid range
    /// `0.5`–`1.0`; server default `0.7`. Set to `1.0` to fully suppress
    /// natural end-of-turn detection and end turns only with
    /// `ForceEndTurn` (see `AgentHandle::force_end_turn`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eot_threshold: Option<f64>,

    /// End-of-turn confidence required to fire an eager end-of-turn event.
    /// When set, enables eager end-of-turn / turn-resumed behavior. Valid
    /// range `0.3`–`0.9`.
    ///
    /// Note: Flux STT sessions can also emit an `EndOfTurn` server message
    /// (`{"type":"EndOfTurn","trigger":"model"|"timeout"}`) that is not in
    /// the published AsyncAPI spec yet; it currently surfaces as
    /// `AgentResponse::Unknown`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eager_eot_threshold: Option<f64>,

    /// A turn is finished once this many milliseconds have elapsed after
    /// speech, regardless of end-of-turn confidence. Server default `5000`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eot_timeout_ms: Option<u32>,

    /// Keyterms to boost recognition for specialized terminology.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyterms: Vec<String>,
}

impl DeepgramListenV2Provider {
    /// Construct with the given model and no other fields set.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            provider_type: DeepgramProviderType::Deepgram,
            version: DeepgramListenV2Version::V2,
            model: model.into(),
            language_hints: Vec::new(),
            eot_threshold: None,
            eager_eot_threshold: None,
            eot_timeout_ms: None,
            keyterms: Vec::new(),
        }
    }

    /// Replace the language hints (`language_hints`). Serializes as a JSON
    /// array even when a single hint is given.
    pub fn with_language_hints<I, S>(mut self, hints: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.language_hints = hints.into_iter().map(Into::into).collect();
        self
    }

    /// Append a single language hint. Convenience over
    /// [`Self::with_language_hints`]; the field still serializes as a JSON
    /// array.
    pub fn with_language_hint(mut self, hint: impl Into<String>) -> Self {
        self.language_hints.push(hint.into());
        self
    }

    /// Set `eot_threshold` (valid range `0.5`–`1.0`; server default `0.7`).
    pub fn with_eot_threshold(mut self, threshold: f64) -> Self {
        self.eot_threshold = Some(threshold);
        self
    }

    /// Set `eager_eot_threshold` (valid range `0.3`–`0.9`).
    pub fn with_eager_eot_threshold(mut self, threshold: f64) -> Self {
        self.eager_eot_threshold = Some(threshold);
        self
    }

    /// Set `eot_timeout_ms` (server default `5000`).
    pub fn with_eot_timeout_ms(mut self, timeout_ms: u32) -> Self {
        self.eot_timeout_ms = Some(timeout_ms);
        self
    }

    #[allow(missing_docs)]
    pub fn with_keyterms<I, S>(mut self, keyterms: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.keyterms = keyterms.into_iter().map(Into::into).collect();
        self
    }
}

/// Deserialize either a single string or an array of strings into `Vec<String>`.
///
/// The spec defines `language_hints` as an array; accepting a bare string
/// on input is a leniency for hand-written configs. Output is always an
/// array.
fn deserialize_string_one_or_many<'de, D>(de: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }

    Ok(match OneOrMany::deserialize(de)? {
        OneOrMany::One(s) => vec![s],
        OneOrMany::Many(v) => v,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn v1_minimal_round_trip() {
        let raw = json!({ "type": "deepgram" });
        let provider: AgentListenProvider = serde_json::from_value(raw).unwrap();
        let v1 = match &provider {
            AgentListenProvider::DeepgramV1(p) => p,
            _ => panic!("expected V1"),
        };
        // version defaults to v1 even when absent.
        assert_eq!(v1.version, DeepgramListenV1Version::V1);
        assert!(v1.model.is_none());
    }

    #[test]
    fn v1_full_round_trip() {
        let raw = json!({
            "type": "deepgram",
            "version": "v1",
            "model": "nova-3",
            "language": "en-US",
            "keyterms": ["RAG", "MCP"],
            "smart_format": true
        });
        let provider: AgentListenProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV1(p) => {
                assert_eq!(p.model.as_deref(), Some("nova-3"));
                assert_eq!(p.language.as_deref(), Some("en-US"));
                assert_eq!(p.keyterms, vec!["RAG", "MCP"]);
                assert_eq!(p.smart_format, Some(true));
            }
            _ => panic!("expected V1"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn v2_minimal_round_trip() {
        let raw = json!({
            "type": "deepgram",
            "version": "v2",
            "model": "flux-general-en"
        });
        let provider: AgentListenProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV2(p) => {
                assert_eq!(p.model, "flux-general-en");
                assert!(p.language_hints.is_empty());
                assert!(p.eot_threshold.is_none());
                assert!(p.eager_eot_threshold.is_none());
                assert!(p.eot_timeout_ms.is_none());
                assert!(p.keyterms.is_empty());
            }
            _ => panic!("expected V2"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn v2_with_language_hints_array_round_trip() {
        let raw = json!({
            "type": "deepgram",
            "version": "v2",
            "model": "flux-general-multi",
            "language_hints": ["en", "es", "fr"]
        });
        let provider: AgentListenProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV2(p) => {
                assert_eq!(p.language_hints, vec!["en", "es", "fr"]);
            }
            _ => panic!("expected V2"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn v2_single_language_hint_exact_json_is_array() {
        // Exact wire JSON: key is `language_hints`, value is always an
        // array — even with one hint.
        let provider = AgentListenProvider::DeepgramV2(
            DeepgramListenV2Provider::new("flux-general-multi").with_language_hints(["es"]),
        );
        assert_eq!(
            serde_json::to_string(&provider).unwrap(),
            r#"{"type":"deepgram","version":"v2","model":"flux-general-multi","language_hints":["es"]}"#
        );

        // The single-hint convenience builder appends and still emits an array.
        let provider = AgentListenProvider::DeepgramV2(
            DeepgramListenV2Provider::new("flux-general-multi").with_language_hint("es"),
        );
        assert_eq!(
            serde_json::to_string(&provider).unwrap(),
            r#"{"type":"deepgram","version":"v2","model":"flux-general-multi","language_hints":["es"]}"#
        );
    }

    #[test]
    fn v2_multiple_language_hints_exact_json() {
        let provider = AgentListenProvider::DeepgramV2(
            DeepgramListenV2Provider::new("flux-general-multi")
                .with_language_hint("en")
                .with_language_hint("es")
                .with_language_hint("fr"),
        );
        assert_eq!(
            serde_json::to_string(&provider).unwrap(),
            r#"{"type":"deepgram","version":"v2","model":"flux-general-multi","language_hints":["en","es","fr"]}"#
        );
    }

    #[test]
    fn v2_language_hints_bare_string_input_is_accepted() {
        // Leniency on input only: a bare string deserializes to a one-element
        // vec and re-serializes as an array.
        let raw = json!({
            "type": "deepgram",
            "version": "v2",
            "model": "flux-general-multi",
            "language_hints": "es"
        });
        let provider: AgentListenProvider = serde_json::from_value(raw).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV2(p) => assert_eq!(p.language_hints, vec!["es"]),
            _ => panic!("expected V2"),
        }
        assert_eq!(
            serde_json::to_value(&provider).unwrap()["language_hints"],
            json!(["es"])
        );
    }

    #[test]
    fn v2_legacy_scalar_language_hint_key_is_not_emitted() {
        // The old (incorrect) scalar `language_hint` key is not the wire
        // field; the SDK neither reads nor writes it.
        let raw = json!({
            "type": "deepgram",
            "version": "v2",
            "model": "flux-general-multi",
            "language_hint": "es"
        });
        let provider: AgentListenProvider = serde_json::from_value(raw).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV2(p) => assert!(p.language_hints.is_empty()),
            _ => panic!("expected V2"),
        }
        let out = serde_json::to_value(&provider).unwrap();
        assert!(out.get("language_hint").is_none());
        assert!(out.get("language_hints").is_none());
    }

    #[test]
    fn v2_eot_fields_exact_json() {
        let provider = AgentListenProvider::DeepgramV2(
            DeepgramListenV2Provider::new("flux-general-en")
                .with_eot_threshold(0.8)
                .with_eager_eot_threshold(0.5)
                .with_eot_timeout_ms(3000),
        );
        assert_eq!(
            serde_json::to_string(&provider).unwrap(),
            r#"{"type":"deepgram","version":"v2","model":"flux-general-en","eot_threshold":0.8,"eager_eot_threshold":0.5,"eot_timeout_ms":3000}"#
        );
    }

    #[test]
    fn v2_eot_fields_round_trip() {
        // `eot_threshold: 1.0` is the documented way to suppress natural
        // end-of-turn detection in favor of `ForceEndTurn`.
        let raw = json!({
            "type": "deepgram",
            "version": "v2",
            "model": "flux-general-en",
            "eot_threshold": 1.0,
            "eot_timeout_ms": 5000
        });
        let provider: AgentListenProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV2(p) => {
                assert_eq!(p.eot_threshold, Some(1.0));
                assert!(p.eager_eot_threshold.is_none());
                assert_eq!(p.eot_timeout_ms, Some(5000));
            }
            _ => panic!("expected V2"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn v2_with_keyterms() {
        let provider = AgentListenProvider::DeepgramV2(
            DeepgramListenV2Provider::new("flux-general-en").with_keyterms(["transactional"]),
        );
        let value = serde_json::to_value(&provider).unwrap();
        assert_eq!(value["keyterms"], json!(["transactional"]));
    }

    #[test]
    fn unknown_version_rejected() {
        let raw = json!({ "type": "deepgram", "version": "v3" });
        let err = serde_json::from_value::<AgentListenProvider>(raw).unwrap_err();
        assert!(err.to_string().contains("v3"), "got: {err}");
    }

    #[test]
    fn settings_wrapper_round_trip() {
        let raw = json!({
            "provider": {
                "type": "deepgram",
                "version": "v2",
                "model": "flux-general-multi",
                "language_hints": ["en", "fr"]
            }
        });
        let settings: AgentListenSettings = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(
            settings.provider,
            AgentListenProvider::DeepgramV2(_)
        ));
        assert_eq!(serde_json::to_value(&settings).unwrap(), raw);
    }

    #[test]
    fn v1_builder_chain() {
        let p = DeepgramListenV1Provider::new()
            .with_model("nova-3")
            .with_language("en-US")
            .with_smart_format(true)
            .with_keyterms(["one", "two"]);
        assert_eq!(p.model.as_deref(), Some("nova-3"));
        assert_eq!(p.language.as_deref(), Some("en-US"));
        assert_eq!(p.smart_format, Some(true));
        assert_eq!(p.keyterms, vec!["one", "two"]);
    }
}
