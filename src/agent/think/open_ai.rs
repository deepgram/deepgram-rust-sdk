//! OpenAI Think provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/think-providers/open-ai.yml` in
//! `deepgram-docs`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// OpenAI as a Think provider for the Voice Agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OpenAiThinkProvider {
    /// REST API version of the OpenAI chat completions API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<OpenAiVersion>,

    /// OpenAI model identifier.
    pub model: OpenAiModel,

    /// Sampling temperature (0–2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Reasoning effort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_mode: Option<OpenAiReasoningMode>,
}

impl OpenAiThinkProvider {
    /// Construct with the given model and defaults for all other fields.
    pub fn new(model: OpenAiModel) -> Self {
        Self {
            version: None,
            model,
            temperature: None,
            reasoning_mode: None,
        }
    }
}

/// Version of the OpenAI chat completions REST API used by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OpenAiVersion {
    /// REST API v1.
    #[serde(rename = "v1")]
    V1,
}

/// OpenAI model identifier.
///
/// Use [`OpenAiModel::Other`] to pass any value not yet enumerated by this SDK.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OpenAiModel {
    /// `gpt-5`
    Gpt5,
    /// `gpt-5-mini`
    Gpt5Mini,
    /// `gpt-5-nano`
    Gpt5Nano,
    /// `gpt-4.1`
    Gpt4_1,
    /// `gpt-4.1-mini`
    Gpt4_1Mini,
    /// `gpt-4.1-nano`
    Gpt4_1Nano,
    /// `gpt-4o`
    Gpt4o,
    /// `gpt-4o-mini`
    Gpt4oMini,
    /// Forward-compatibility escape — pass any unrecognized model identifier.
    Other(String),
}

impl OpenAiModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gpt5 => "gpt-5",
            Self::Gpt5Mini => "gpt-5-mini",
            Self::Gpt5Nano => "gpt-5-nano",
            Self::Gpt4_1 => "gpt-4.1",
            Self::Gpt4_1Mini => "gpt-4.1-mini",
            Self::Gpt4_1Nano => "gpt-4.1-nano",
            Self::Gpt4o => "gpt-4o",
            Self::Gpt4oMini => "gpt-4o-mini",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for OpenAiModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "gpt-5" => Self::Gpt5,
            "gpt-5-mini" => Self::Gpt5Mini,
            "gpt-5-nano" => Self::Gpt5Nano,
            "gpt-4.1" => Self::Gpt4_1,
            "gpt-4.1-mini" => Self::Gpt4_1Mini,
            "gpt-4.1-nano" => Self::Gpt4_1Nano,
            "gpt-4o" => Self::Gpt4o,
            "gpt-4o-mini" => Self::Gpt4oMini,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for OpenAiModel {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OpenAiModel {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

/// OpenAI reasoning effort.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OpenAiReasoningMode {
    /// Disable reasoning.
    None,
    /// Minimal reasoning.
    Minimal,
    /// Low reasoning effort.
    Low,
    /// Medium reasoning effort.
    Medium,
    /// High reasoning effort.
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_full_provider() {
        let raw = json!({
            "version": "v1",
            "model": "gpt-5",
            "temperature": 0.7,
            "reasoning_mode": "medium",
        });
        let p: OpenAiThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.version, Some(OpenAiVersion::V1));
        assert_eq!(p.model, OpenAiModel::Gpt5);
        assert_eq!(p.temperature, Some(0.7));
        assert_eq!(p.reasoning_mode, Some(OpenAiReasoningMode::Medium));
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn unknown_model_falls_back_to_other() {
        let raw = json!({ "model": "gpt-9-future" });
        let p: OpenAiThinkProvider = serde_json::from_value(raw).unwrap();
        assert_eq!(p.model, OpenAiModel::Other("gpt-9-future".into()));
    }

    #[test]
    fn other_model_round_trips() {
        let p = OpenAiThinkProvider::new(OpenAiModel::Other("gpt-future".into()));
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v, json!({ "model": "gpt-future" }));
    }

    #[test]
    fn known_model_round_trip_via_from_string() {
        let m: OpenAiModel = "gpt-4.1-nano".to_string().into();
        assert_eq!(m, OpenAiModel::Gpt4_1Nano);
        assert_eq!(m.as_str(), "gpt-4.1-nano");
    }
}
