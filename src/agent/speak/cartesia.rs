//! Cartesia Speak provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/speak-providers/cartesia.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Cartesia as a Speak provider for the Voice Agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CartesiaSpeakProvider {
    /// API version header for Cartesia TTS. Spec value is the date string `2025-03-17`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<CartesiaVersion>,

    /// Cartesia model ID.
    pub model_id: CartesiaModelId,

    /// Voice configuration (mode + ID).
    pub voice: CartesiaVoice,

    /// Cartesia language code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Volume multiplier (0.5 – 2.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
}

impl CartesiaSpeakProvider {
    /// Construct with the given model ID and voice.
    pub fn new(model_id: CartesiaModelId, voice: CartesiaVoice) -> Self {
        Self {
            version: None,
            model_id,
            voice,
            language: None,
            volume: None,
        }
    }
}

/// API version for Cartesia TTS. The wire value is a quoted date string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CartesiaVersion {
    /// `2025-03-17`
    #[serde(rename = "2025-03-17")]
    V2025_03_17,
}

/// Cartesia model ID. Use [`CartesiaModelId::Other`] for unrecognized values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CartesiaModelId {
    /// `sonic-2`
    Sonic2,
    /// `sonic-multilingual`
    SonicMultilingual,
    /// Forward-compatibility escape.
    Other(String),
}

impl CartesiaModelId {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Sonic2 => "sonic-2",
            Self::SonicMultilingual => "sonic-multilingual",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for CartesiaModelId {
    fn from(value: String) -> Self {
        match value.as_str() {
            "sonic-2" => Self::Sonic2,
            "sonic-multilingual" => Self::SonicMultilingual,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for CartesiaModelId {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for CartesiaModelId {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

/// Cartesia voice block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CartesiaVoice {
    /// Voice mode (e.g. `id`, `embedding`).
    pub mode: String,
    /// Voice ID.
    pub id: String,
}

impl CartesiaVoice {
    /// Convenience constructor.
    pub fn new(mode: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            mode: mode.into(),
            id: id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip_full() {
        let raw = json!({
            "version": "2025-03-17",
            "model_id": "sonic-2",
            "voice": { "mode": "id", "id": "voice_abc" },
            "language": "en",
            "volume": 1.4
        });
        let p: CartesiaSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model_id, CartesiaModelId::Sonic2);
        assert_eq!(p.voice.mode, "id");
        assert_eq!(p.voice.id, "voice_abc");
        assert_eq!(p.volume, Some(1.4));
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn round_trip_minimal() {
        let raw = json!({
            "model_id": "sonic-multilingual",
            "voice": { "mode": "id", "id": "v1" }
        });
        let p: CartesiaSpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model_id, CartesiaModelId::SonicMultilingual);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }
}
