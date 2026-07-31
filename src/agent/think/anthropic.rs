//! Anthropic Think provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/think-providers/anthropic.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Anthropic as a Think provider for the Voice Agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AnthropicThinkProvider {
    /// REST API version of the Anthropic Messages API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<AnthropicVersion>,

    /// Anthropic model identifier.
    pub model: AnthropicModel,

    /// Sampling temperature (0–1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

impl AnthropicThinkProvider {
    /// Construct with the given model and defaults for all other fields.
    pub fn new(model: AnthropicModel) -> Self {
        Self {
            version: None,
            model,
            temperature: None,
        }
    }
}

/// Version of the Anthropic Messages REST API used by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AnthropicVersion {
    /// REST API v1.
    #[serde(rename = "v1")]
    V1,
}

/// Anthropic model identifier. Use [`AnthropicModel::Other`] for unrecognized values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AnthropicModel {
    /// `claude-3-5-haiku-latest`
    Claude35HaikuLatest,
    /// `claude-sonnet-4-20250514`
    ClaudeSonnet4_20250514,
    /// Forward-compatibility escape.
    Other(String),
}

impl AnthropicModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Claude35HaikuLatest => "claude-3-5-haiku-latest",
            Self::ClaudeSonnet4_20250514 => "claude-sonnet-4-20250514",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for AnthropicModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "claude-3-5-haiku-latest" => Self::Claude35HaikuLatest,
            "claude-sonnet-4-20250514" => Self::ClaudeSonnet4_20250514,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for AnthropicModel {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AnthropicModel {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_full() {
        let raw = json!({
            "version": "v1",
            "model": "claude-sonnet-4-20250514",
            "temperature": 0.5,
        });
        let p: AnthropicThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, AnthropicModel::ClaudeSonnet4_20250514);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn unknown_model_falls_back_to_other() {
        let raw = json!({ "model": "claude-future" });
        let p: AnthropicThinkProvider = serde_json::from_value(raw).unwrap();
        assert_eq!(p.model, AnthropicModel::Other("claude-future".into()));
    }
}
