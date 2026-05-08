//! Google (Gemini) Think provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/think-providers/google.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Google as a Think provider for the Voice Agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GoogleThinkProvider {
    /// REST API version of Google's generative language API.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<GoogleVersion>,

    /// Google model identifier.
    pub model: GoogleModel,

    /// Sampling temperature (0–2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
}

impl GoogleThinkProvider {
    /// Construct with the given model and no other fields set.
    pub fn new(model: GoogleModel) -> Self {
        Self {
            version: None,
            model,
            temperature: None,
        }
    }
}

/// Version of Google's generative language REST API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GoogleVersion {
    /// REST API v1beta.
    #[serde(rename = "v1beta")]
    V1Beta,
}

/// Google model identifier. Use [`GoogleModel::Other`] for unrecognized values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GoogleModel {
    /// `gemini-2.0-flash`
    Gemini20Flash,
    /// `gemini-2.0-flash-lite`
    Gemini20FlashLite,
    /// `gemini-2.5-flash`
    Gemini25Flash,
    /// Forward-compatibility escape.
    Other(String),
}

impl GoogleModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Gemini20Flash => "gemini-2.0-flash",
            Self::Gemini20FlashLite => "gemini-2.0-flash-lite",
            Self::Gemini25Flash => "gemini-2.5-flash",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for GoogleModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "gemini-2.0-flash" => Self::Gemini20Flash,
            "gemini-2.0-flash-lite" => Self::Gemini20FlashLite,
            "gemini-2.5-flash" => Self::Gemini25Flash,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for GoogleModel {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for GoogleModel {
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
            "version": "v1beta",
            "model": "gemini-2.5-flash",
            "temperature": 1.2,
        });
        let p: GoogleThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, GoogleModel::Gemini25Flash);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }
}
