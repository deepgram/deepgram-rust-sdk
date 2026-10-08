//! OpenAI Speak (TTS) provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/speak-providers/open-ai.yml`. Distinct
//! from the OpenAI _Think_ provider — same `type: open_ai` discriminator
//! but a different schema in a different settings block.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// OpenAI as a Speak provider for the Voice Agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct OpenAiSpeakProvider {
    /// REST API version of OpenAI TTS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<OpenAiSpeakVersion>,

    /// OpenAI TTS model.
    pub model: OpenAiSpeakModel,

    /// OpenAI voice.
    pub voice: OpenAiVoice,
}

impl OpenAiSpeakProvider {
    /// Construct with the given model and voice.
    pub fn new(model: OpenAiSpeakModel, voice: OpenAiVoice) -> Self {
        Self {
            version: None,
            model,
            voice,
        }
    }
}

/// Version of the OpenAI TTS REST API used by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum OpenAiSpeakVersion {
    /// REST API v1.
    #[serde(rename = "v1")]
    V1,
}

/// OpenAI TTS model.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OpenAiSpeakModel {
    /// `tts-1`
    Tts1,
    /// `tts-1-hd`
    Tts1Hd,
    /// Forward-compatibility escape.
    Other(String),
}

impl OpenAiSpeakModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Tts1 => "tts-1",
            Self::Tts1Hd => "tts-1-hd",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for OpenAiSpeakModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "tts-1" => Self::Tts1,
            "tts-1-hd" => Self::Tts1Hd,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for OpenAiSpeakModel {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for OpenAiSpeakModel {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

/// OpenAI voice identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OpenAiVoice {
    #[allow(missing_docs)]
    Alloy,
    #[allow(missing_docs)]
    Echo,
    #[allow(missing_docs)]
    Fable,
    #[allow(missing_docs)]
    Onyx,
    #[allow(missing_docs)]
    Nova,
    #[allow(missing_docs)]
    Shimmer,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip() {
        let raw = json!({
            "version": "v1",
            "model": "tts-1-hd",
            "voice": "alloy"
        });
        let p: OpenAiSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, OpenAiSpeakModel::Tts1Hd);
        assert_eq!(p.voice, OpenAiVoice::Alloy);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn unknown_model_falls_back_to_other() {
        let raw = json!({
            "model": "tts-future",
            "voice": "nova"
        });
        let p: OpenAiSpeakProvider = serde_json::from_value(raw).unwrap();
        assert_eq!(p.model, OpenAiSpeakModel::Other("tts-future".into()));
    }
}
