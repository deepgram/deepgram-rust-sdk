//! Deepgram-hosted Think provider settings.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Deepgram-hosted language model settings for a Voice Agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeepgramThinkProvider {
    /// Deepgram Think model identifier.
    pub model: DeepgramThinkModel,
    /// API version. The service defaults this to `v1`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<DeepgramThinkVersion>,
    /// Sampling temperature, from 0 to 2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Reasoning effort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_mode: Option<DeepgramReasoningMode>,
    /// Opt out of Deepgram's spoken-style prompt prefix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_prompt_opt_out: Option<bool>,
}

impl DeepgramThinkProvider {
    /// Construct settings for a Deepgram-hosted Think model.
    pub fn new(model: DeepgramThinkModel) -> Self {
        Self {
            model,
            version: None,
            temperature: None,
            reasoning_mode: None,
            voice_prompt_opt_out: None,
        }
    }

    /// Set the sampling temperature.
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the reasoning effort.
    pub fn with_reasoning_mode(mut self, reasoning_mode: DeepgramReasoningMode) -> Self {
        self.reasoning_mode = Some(reasoning_mode);
        self
    }

    /// Opt out of the spoken-style prompt prefix added only by `think-fast`.
    pub fn with_voice_prompt_opt_out(mut self, voice_prompt_opt_out: bool) -> Self {
        self.voice_prompt_opt_out = Some(voice_prompt_opt_out);
        self
    }
}

/// Deepgram Think API version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DeepgramThinkVersion {
    /// Version 1.
    #[serde(rename = "v1")]
    V1,
}

/// Deepgram-hosted Think model identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DeepgramThinkModel {
    /// `think-fast`.
    ///
    /// Early Access. Requires Model Improvement Program participation: the
    /// service rejects a session with `mip_opt_out` set to `true`. Use
    /// [`DeepgramThinkModel::PhonellmMiniAlpha2Fp8`] for an opt-out session.
    ThinkFast,
    /// `phonellm-mini-alpha-2-fp8`.
    ///
    /// Use this model when the Voice Agent session opts out of the Model
    /// Improvement Program. Omit reasoning, or use
    /// [`DeepgramReasoningMode::None`], so the agent does not speak
    /// reasoning text.
    PhonellmMiniAlpha2Fp8,
    /// Forward-compatible custom model identifier.
    Other(String),
}

impl DeepgramThinkModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::ThinkFast => "think-fast",
            Self::PhonellmMiniAlpha2Fp8 => "phonellm-mini-alpha-2-fp8",
            Self::Other(value) => value,
        }
    }
}

impl From<String> for DeepgramThinkModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "think-fast" => Self::ThinkFast,
            "phonellm-mini-alpha-2-fp8" => Self::PhonellmMiniAlpha2Fp8,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for DeepgramThinkModel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DeepgramThinkModel {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

/// Reasoning effort for a Deepgram Think model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DeepgramReasoningMode {
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
    fn provider_round_trips_all_settings() {
        let raw = json!({
            "model": "think-fast",
            "version": "v1",
            "temperature": 0.7,
            "reasoning_mode": "medium",
            "voice_prompt_opt_out": true
        });
        let provider: DeepgramThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(provider.model, DeepgramThinkModel::ThinkFast);
        assert_eq!(provider.version, Some(DeepgramThinkVersion::V1));
        assert_eq!(provider.reasoning_mode, Some(DeepgramReasoningMode::Medium));
        assert_eq!(serde_json::to_value(provider).unwrap(), raw);
    }

    #[test]
    fn model_preserves_unknown_identifiers() {
        let model: DeepgramThinkModel = "future-think-model".to_owned().into();
        assert_eq!(
            model,
            DeepgramThinkModel::Other("future-think-model".to_owned())
        );
        assert_eq!(
            serde_json::to_value(model).unwrap(),
            json!("future-think-model")
        );
    }
}
