//! Response shapes for the model-listing endpoints.
//!
//! Mirrors `schemas.models.v1.yml`. The `oneOf [STT, TTS]` shape from
//! the spec is flattened into a single [`ModelInfo`] with feature
//! fields optional — STT models populate `batch`/`streaming`/
//! `formatted_output`, TTS models populate `metadata`.

use serde::{Deserialize, Serialize};

/// A single model entry returned by the listing endpoints. Fields
/// specific to STT or TTS are optional and only one set is populated
/// at a time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelInfo {
    /// Short product name, e.g. `nova-3` or `zeus`.
    pub name: String,
    /// Canonical model identifier, e.g. `nova-3` or `aura-2-zeus-en`.
    pub canonical_name: String,
    /// Underlying architecture family, e.g. `base`, `polaris`, `aura-2`.
    pub architecture: String,
    /// BCP-47 language tags supported by the model.
    #[serde(default)]
    pub languages: Vec<String>,
    /// Model version (e.g. `2025-04-07.0`).
    pub version: String,
    /// Deepgram-assigned model UUID.
    pub uuid: String,

    // STT-only fields:
    /// Whether the model is available for batch (pre-recorded) requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch: Option<bool>,
    /// Whether the model is available for streaming requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    /// Whether the model returns formatted output.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formatted_output: Option<bool>,

    // TTS-only fields:
    /// Voice metadata (TTS only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<TtsMetadata>,
}

/// Voice metadata returned for TTS models.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TtsMetadata {
    /// Voice accent (e.g. `American`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    /// Voice age category (e.g. `Adult`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<String>,
    /// Hex color associated with the voice avatar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// URL of the avatar image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// URL of an audio sample of the voice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample: Option<String>,
    /// Descriptive tags (e.g. `masculine`, `deep`).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Recommended use cases (e.g. `IVR`).
    #[serde(default)]
    pub use_cases: Vec<String>,
}

/// Response from `GET /v1/models` and `GET /v1/projects/{id}/models`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct ListModelsResponse {
    /// Speech-to-text models.
    #[serde(default)]
    pub stt: Vec<ModelInfo>,
    /// Text-to-speech models.
    #[serde(default)]
    pub tts: Vec<ModelInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_list_response() {
        let raw = json!({
            "stt": [{
                "name": "nova-3",
                "canonical_name": "nova-3",
                "architecture": "base",
                "languages": ["en", "en-us"],
                "version": "2021-11-10.1",
                "uuid": "6b28e919-8427-4f32-9847-492e2efd7daf",
                "batch": true,
                "streaming": true,
                "formatted_output": true
            }],
            "tts": [{
                "name": "zeus",
                "canonical_name": "aura-2-zeus-en",
                "architecture": "aura-2",
                "languages": ["en"],
                "version": "2025-04-07.0",
                "uuid": "2baf189d-91ac-481d-b6d1-750888667b31",
                "metadata": {
                    "accent": "American",
                    "tags": ["masculine"],
                    "use_cases": ["IVR"]
                }
            }]
        });
        let resp: ListModelsResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.stt.len(), 1);
        assert_eq!(resp.stt[0].batch, Some(true));
        assert!(resp.stt[0].metadata.is_none());
        assert_eq!(resp.tts.len(), 1);
        assert!(resp.tts[0].batch.is_none());
        let tts_meta = resp.tts[0].metadata.as_ref().unwrap();
        assert_eq!(tts_meta.accent.as_deref(), Some("American"));
        assert_eq!(tts_meta.tags, vec!["masculine"]);
    }

    #[test]
    fn deserialize_get_stt() {
        let raw = json!({
            "name": "general",
            "canonical_name": "enhanced-general",
            "architecture": "polaris",
            "languages": ["en"],
            "version": "2022-05-18.1",
            "uuid": "c7226e9e-ae1c-4057-ae2a-a71a6b0dc588",
            "batch": true,
            "streaming": true,
            "formatted_output": false
        });
        let m: ModelInfo = serde_json::from_value(raw).unwrap();
        assert_eq!(m.batch, Some(true));
        assert_eq!(m.formatted_output, Some(false));
        assert!(m.metadata.is_none());
    }
}
