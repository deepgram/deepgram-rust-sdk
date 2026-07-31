//! Groq Think provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/think-providers/groq.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::agent::think::OpenAiReasoningMode;

/// Groq as a Think provider for the Voice Agent. Mostly OpenAI-compatible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GroqThinkProvider {
    /// REST API version of Groq's chat completions API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<GroqVersion>,

    /// Groq model identifier.
    pub model: GroqModel,

    /// Sampling temperature (0–2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Reasoning effort. Reuses the same enum as OpenAI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_mode: Option<OpenAiReasoningMode>,
}

impl GroqThinkProvider {
    /// Construct with the given model and defaults for all other fields.
    pub fn new(model: GroqModel) -> Self {
        Self {
            version: None,
            model,
            temperature: None,
            reasoning_mode: None,
        }
    }
}

/// Version of Groq's chat completions REST API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GroqVersion {
    /// REST API v1.
    #[serde(rename = "v1")]
    V1,
}

/// Groq model identifier. Use [`GroqModel::Other`] for unrecognized values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GroqModel {
    /// `openai/gpt-oss-20b`
    OpenAiGptOss20b,
    /// Forward-compatibility escape.
    Other(String),
}

impl GroqModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::OpenAiGptOss20b => "openai/gpt-oss-20b",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for GroqModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "openai/gpt-oss-20b" => Self::OpenAiGptOss20b,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for GroqModel {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for GroqModel {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip() {
        let raw = json!({
            "version": "v1",
            "model": "openai/gpt-oss-20b",
            "temperature": 0.8,
            "reasoning_mode": "high",
        });
        let p: GroqThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, GroqModel::OpenAiGptOss20b);
        assert_eq!(p.reasoning_mode, Some(OpenAiReasoningMode::High));
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }
}
