//! Response types for the model-listing endpoints.

use std::collections::HashMap;

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

    /// Every language any listed model supports, as a BCP-47 language tag
    /// mapped to its human-readable display name (`"es-CO"` to
    /// `"Spanish (Colombia)"`). Use it to label the tags in each model's
    /// [`Model::languages`].
    #[serde(default)]
    pub languages: HashMap<String, String>,
}

/// Metadata describing a single Deepgram model.
///
/// This is the management-API model record returned by the `/v1/models`
/// endpoints.
///
// The whole sentence lives in both arms so it reads as one sentence in
// source: `common::options` is gated on `listen`/`read`, so the link is
// emitted only when it will resolve. docs.rs builds all features.
#[cfg_attr(
    any(feature = "listen", feature = "read"),
    doc = "It is distinct from the request-time model selector you pass when transcribing or synthesizing, [`common::options::Model`](crate::common::options::Model)."
)]
#[cfg_attr(
    not(any(feature = "listen", feature = "read")),
    doc = "It is distinct from the request-time model selector you pass when transcribing or synthesizing, `common::options::Model`."
)]
///
/// The same shape is returned for STT and TTS models; STT models populate
/// `batch` / `streaming` / `formatted_output`, while TTS models populate
/// `metadata`.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Model {
    #[allow(missing_docs)]
    pub name: Option<String>,

    #[allow(missing_docs)]
    pub canonical_name: Option<String>,

    #[allow(missing_docs)]
    pub architecture: Option<String>,

    #[allow(missing_docs)]
    #[serde(default)]
    pub languages: Vec<String>,

    #[allow(missing_docs)]
    pub version: Option<String>,

    #[allow(missing_docs)]
    pub uuid: Option<String>,

    /// Whether the model supports batch (pre-recorded) transcription. STT only.
    pub batch: Option<bool>,

    /// Whether the model supports streaming transcription. STT only.
    pub streaming: Option<bool>,

    /// Whether the model applies formatted output. STT only.
    pub formatted_output: Option<bool>,

    /// Whether the model transcribes more than one language within a single
    /// request, rather than the single language given in `language`. STT only.
    pub multilingual: Option<bool>,

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

    /// The voice's human-readable name (`"Agathe"` for `aura-2-agathe-fr`),
    /// for showing in a voice picker; [`Model::name`] carries the wire form
    /// (`"agathe"`). The field is sent for every voice but is `null` for the
    /// ones that have not been given a display name, so fall back to
    /// [`Model::name`] rather than treating `None` as an error.
    pub display_name: Option<String>,

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
        // Shape from the Deepgram Model Metadata docs, plus the three fields
        // a live `GET /v1/models` returns on every record: the top-level
        // `languages` map, `multilingual` on each STT model, and
        // `display_name` in each TTS voice's `metadata`.
        let json = serde_json::json!({
            "languages": {
                "en": "English",
                "en-US": "English (United States)",
                "en-IE": "English (Ireland)"
            },
            "stt": [{
                "name": "general",
                "canonical_name": "nova-3-general",
                "architecture": "nova3",
                "languages": ["en", "en-US"],
                "version": "2025-01-09.0",
                "uuid": "bf05427e-a1f1-4ced-a976-38b2f3533d8d",
                "batch": false,
                "streaming": true,
                "formatted_output": false,
                "multilingual": false
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
                    "display_name": "Angus",
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
        assert_eq!(stt.canonical_name.as_deref(), Some("nova-3-general"));
        assert_eq!(stt.streaming, Some(true));
        assert!(stt.metadata.is_none());

        let tts = &response.tts[0];
        assert_eq!(tts.name.as_deref(), Some("angus"));
        assert_eq!(
            tts.metadata.as_ref().unwrap().accent.as_deref(),
            Some("Irish")
        );
        assert_eq!(
            tts.metadata.as_ref().unwrap().display_name.as_deref(),
            Some("Angus")
        );
        assert_eq!(tts.metadata.as_ref().unwrap().tags, vec!["masculine"]);
        // TTS models don't carry the STT flags.
        assert!(tts.batch.is_none());
        // `multilingual` is STT-only; TTS records omit it.
        assert_eq!(stt.multilingual, Some(false));
        assert!(tts.multilingual.is_none());

        assert_eq!(response.languages.len(), 3);
        assert_eq!(
            response.languages.get("en-US").map(String::as_str),
            Some("English (United States)")
        );
    }

    #[test]
    fn deserializes_model_with_omitted_optional_fields() {
        // The Models API marks `name`, `canonical_name`, `architecture`,
        // `version`, and `uuid` optional; a contract-valid response may omit
        // any of them and must still deserialize.
        let json = serde_json::json!({
            "stt": [{
                "name": "general",
                "languages": ["en"],
                "streaming": true
            }],
            "tts": [{
                "canonical_name": "aura-2-thalia-en",
                // The live endpoint sends an explicit `null` here for voices
                // with no display name, rather than omitting the key.
                "metadata": { "tags": ["feminine"], "display_name": null }
            }]
        });

        let response: ModelsResponse = serde_json::from_value(json).unwrap();

        let stt = &response.stt[0];
        assert_eq!(stt.name.as_deref(), Some("general"));
        assert!(stt.canonical_name.is_none());
        assert!(stt.architecture.is_none());
        assert!(stt.version.is_none());
        assert!(stt.uuid.is_none());
        assert_eq!(stt.streaming, Some(true));

        let tts = &response.tts[0];
        assert!(tts.name.is_none());
        assert_eq!(tts.canonical_name.as_deref(), Some("aura-2-thalia-en"));
        assert!(tts.uuid.is_none());
        assert_eq!(tts.metadata.as_ref().unwrap().tags, vec!["feminine"]);
        assert!(tts.metadata.as_ref().unwrap().display_name.is_none());
        assert!(stt.multilingual.is_none());
        // No `languages` map in the payload; the field defaults to empty
        // rather than failing to deserialize.
        assert!(response.languages.is_empty());
    }
}
