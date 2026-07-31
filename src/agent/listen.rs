//! Agent Listen (speech-to-text) provider settings.
//!
//! Mirrors the `agent.listen` block on `AgentV1SettingsMessage` in
//! `asyncapi/schemas/schemas.agent.v1.yml`. The provider is a `oneOf`
//! between two Deepgram-typed shapes that share `type: "deepgram"` but
//! differ on `version` (`v1` vs `v2`/Flux). Discrimination here is on
//! the `version` field via a custom [`Deserialize`] impl.

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

/// `agent.listen` block — wraps a single provider configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
/// supported, with two API versions (V1 and V2/Flux).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentListenProvider {
    /// V1 Deepgram STT (Nova/Nova-2/Nova-3).
    DeepgramV1(DeepgramListenV1Provider),
    /// V2 Deepgram STT (Flux). `model` is required.
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

/// Wire-level discriminator for V2 (Flux) of the Deepgram STT API.
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

/// Deepgram V2 (Flux) STT provider. `model` is required per spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeepgramListenV2Provider {
    /// Always [`DeepgramProviderType::Deepgram`].
    #[serde(rename = "type", default)]
    pub provider_type: DeepgramProviderType,

    /// API version — V2 (Flux).
    pub version: DeepgramListenV2Version,

    /// Flux model identifier (e.g. `flux-general-en`, `flux-general-multi`).
    pub model: String,

    /// Language hints for `flux-general-multi`. Single string or array on
    /// the wire; modeled here as `Vec<String>` with custom serde so a
    /// single-element list deserializes from either form.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_string_one_or_many",
        deserialize_with = "deserialize_string_one_or_many"
    )]
    pub language_hint: Vec<String>,

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
            language_hint: Vec::new(),
            keyterms: Vec::new(),
        }
    }

    #[allow(missing_docs)]
    pub fn with_language_hint<I, S>(mut self, hints: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.language_hint = hints.into_iter().map(Into::into).collect();
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

/// Serialize a `Vec<String>` as a single string when length is 1, otherwise as an array.
fn serialize_string_one_or_many<S>(values: &[String], ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if values.len() == 1 {
        ser.serialize_str(&values[0])
    } else {
        values.serialize(ser)
    }
}

/// Deserialize either a single string or an array of strings into `Vec<String>`.
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
                assert!(p.language_hint.is_empty());
                assert!(p.keyterms.is_empty());
            }
            _ => panic!("expected V2"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn v2_with_language_hint_array_round_trip() {
        let raw = json!({
            "type": "deepgram",
            "version": "v2",
            "model": "flux-general-multi",
            "language_hint": ["en", "es", "fr"]
        });
        let provider: AgentListenProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV2(p) => {
                assert_eq!(p.language_hint, vec!["en", "es", "fr"]);
            }
            _ => panic!("expected V2"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn v2_language_hint_single_string_deserializes_to_vec() {
        let raw = json!({
            "type": "deepgram",
            "version": "v2",
            "model": "flux-general-multi",
            "language_hint": "es"
        });
        let provider: AgentListenProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            AgentListenProvider::DeepgramV2(p) => {
                assert_eq!(p.language_hint, vec!["es"]);
            }
            _ => panic!("expected V2"),
        }
        // Single-element vec serializes back as a scalar string for symmetry with the spec.
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
                "language_hint": ["en", "fr"]
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
