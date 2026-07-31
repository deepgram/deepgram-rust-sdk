//! ElevenLabs Speak provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/speak-providers/eleven-labs.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// ElevenLabs as a Speak provider for the Voice Agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ElevenLabsSpeakProvider {
    /// REST API version of ElevenLabs TTS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<ElevenLabsVersion>,

    /// ElevenLabs model ID.
    pub model_id: ElevenLabsModelId,

    /// Optional language hint, e.g. `en-US`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Deprecated alias for `language`. Prefer `language` for new code.
    #[deprecated(
        since = "0.10.0",
        note = "Use the `language` field instead. Mirrors deprecation in the AsyncAPI spec."
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
}

#[allow(deprecated)]
impl ElevenLabsSpeakProvider {
    /// Construct with the given model ID and no language.
    pub fn new(model_id: ElevenLabsModelId) -> Self {
        Self {
            version: None,
            model_id,
            language: None,
            language_code: None,
        }
    }
}

/// Version of the ElevenLabs TTS REST API used by the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ElevenLabsVersion {
    /// REST API v1.
    #[serde(rename = "v1")]
    V1,
}

/// ElevenLabs model identifier. Use [`ElevenLabsModelId::Other`] for unrecognized values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ElevenLabsModelId {
    /// `eleven_turbo_v2_5`
    ElevenTurboV2_5,
    /// `eleven_monolingual_v1`
    ElevenMonolingualV1,
    /// `eleven_multilingual_v2`
    ElevenMultilingualV2,
    /// Forward-compatibility escape.
    Other(String),
}

impl ElevenLabsModelId {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::ElevenTurboV2_5 => "eleven_turbo_v2_5",
            Self::ElevenMonolingualV1 => "eleven_monolingual_v1",
            Self::ElevenMultilingualV2 => "eleven_multilingual_v2",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for ElevenLabsModelId {
    fn from(value: String) -> Self {
        match value.as_str() {
            "eleven_turbo_v2_5" => Self::ElevenTurboV2_5,
            "eleven_monolingual_v1" => Self::ElevenMonolingualV1,
            "eleven_multilingual_v2" => Self::ElevenMultilingualV2,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for ElevenLabsModelId {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ElevenLabsModelId {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip_with_language() {
        let raw = json!({
            "version": "v1",
            "model_id": "eleven_multilingual_v2",
            "language": "en-US"
        });
        let p: ElevenLabsSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model_id, ElevenLabsModelId::ElevenMultilingualV2);
        assert_eq!(p.language.as_deref(), Some("en-US"));
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn round_trip_minimal() {
        let raw = json!({ "model_id": "eleven_turbo_v2_5" });
        let p: ElevenLabsSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model_id, ElevenLabsModelId::ElevenTurboV2_5);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_language_code_still_round_trips() {
        let raw = json!({
            "model_id": "eleven_monolingual_v1",
            "language_code": "fr-FR"
        });
        let p: ElevenLabsSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.language_code.as_deref(), Some("fr-FR"));
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }
}
