//! Response types for the model-listing endpoints.

use serde::{Deserialize, Serialize};

/// A list of the STT and TTS models available.
///
/// Returned by [`Models::get_models`](crate::manage::models::Models::get_models)
/// and
/// [`Models::get_project_models`](crate::manage::models::Models::get_project_models).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelsResponse {
    /// The available speech-to-text models.
    #[serde(default)]
    pub stt: Vec<Model>,

    /// The available text-to-speech models.
    #[serde(default)]
    pub tts: Vec<Model>,
}

/// Metadata describing a single Deepgram model.
///
/// The same shape is returned for STT and TTS models; STT models populate
/// `batch` / `streaming` / `formatted_output`, while TTS models populate
/// `metadata`.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Model {
    #[allow(missing_docs)]
    pub name: String,

    #[allow(missing_docs)]
    pub canonical_name: String,

    #[allow(missing_docs)]
    pub architecture: String,

    #[allow(missing_docs)]
    #[serde(default)]
    pub languages: Vec<String>,

    #[allow(missing_docs)]
    pub version: String,

    #[allow(missing_docs)]
    pub uuid: String,

    /// Whether the model supports batch (pre-recorded) transcription. STT only.
    pub batch: Option<bool>,

    /// Whether the model supports streaming transcription. STT only.
    pub streaming: Option<bool>,

    /// Whether the model applies formatted output. STT only.
    pub formatted_output: Option<bool>,

    /// Voice metadata (accent, tags, sample audio, …). TTS only.
    pub metadata: Option<ModelMetadata>,
}

/// Voice metadata for a TTS model.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ModelMetadata {
    #[allow(missing_docs)]
    pub accent: Option<String>,

    #[allow(missing_docs)]
    pub age: Option<String>,

    #[allow(missing_docs)]
    pub color: Option<String>,

    #[allow(missing_docs)]
    pub image: Option<String>,

    #[allow(missing_docs)]
    pub sample: Option<String>,

    #[allow(missing_docs)]
    #[serde(default)]
    pub tags: Vec<String>,

    #[allow(missing_docs)]
    #[serde(default)]
    pub use_cases: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::ModelsResponse;

    #[test]
    fn deserializes_documented_response() {
        // Shape from the Deepgram Model Metadata docs.
        let json = serde_json::json!({
            "stt": [{
                "name": "general",
                "canonical_name": "nova-3-general",
                "architecture": "nova3",
                "languages": ["en", "en-US"],
                "version": "2025-01-09.0",
                "uuid": "bf05427e-a1f1-4ced-a976-38b2f3533d8d",
                "batch": false,
                "streaming": true,
                "formatted_output": false
            }],
            "tts": [{
                "name": "angus",
                "canonical_name": "aura-angus-en",
                "architecture": "aura",
                "languages": ["en", "en-IE"],
                "version": "2024-11-19.0",
                "uuid": "b50880e3-4e2e-4e53-ba27-ea0472bf2cf4",
                "metadata": {
                    "accent": "Irish",
                    "color": "#BA80F5",
                    "image": "https://static.deepgram.com/examples/avatars/angus.jpg",
                    "sample": "https://static.deepgram.com/examples/voices/angus.wav",
                    "tags": ["masculine"]
                }
            }]
        });

        let response: ModelsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.stt.len(), 1);
        assert_eq!(response.tts.len(), 1);

        let stt = &response.stt[0];
        assert_eq!(stt.canonical_name, "nova-3-general");
        assert_eq!(stt.streaming, Some(true));
        assert!(stt.metadata.is_none());

        let tts = &response.tts[0];
        assert_eq!(tts.name, "angus");
        assert_eq!(
            tts.metadata.as_ref().unwrap().accent.as_deref(),
            Some("Irish")
        );
        assert_eq!(tts.metadata.as_ref().unwrap().tags, vec!["masculine"]);
        // TTS models don't carry the STT flags.
        assert!(tts.batch.is_none());
    }
}
