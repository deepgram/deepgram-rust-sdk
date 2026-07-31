//! Voice Agent Speak (TTS) settings.
//!
//! Mirrors `asyncapi/schemas/agent/speak-settings.v1.yml` and the five
//! provider sub-schemas under `asyncapi/schemas/agent/speak-providers/`.

use serde::{Deserialize, Serialize};

use crate::agent::Endpoint;

pub mod aws_polly;
pub mod cartesia;
pub mod deepgram;
pub mod eleven_labs;
pub mod open_ai;

pub use aws_polly::{AwsPollyEngine, AwsPollySpeakProvider, AwsPollyVoice};
pub use cartesia::{CartesiaModelId, CartesiaSpeakProvider, CartesiaVersion, CartesiaVoice};
pub use deepgram::{DeepgramSpeakModel, DeepgramSpeakProvider, DeepgramSpeakVersion};
pub use eleven_labs::{ElevenLabsModelId, ElevenLabsSpeakProvider, ElevenLabsVersion};
pub use open_ai::{OpenAiSpeakModel, OpenAiSpeakProvider, OpenAiSpeakVersion, OpenAiVoice};

/// Top-level Speak configuration on `agent.speak` in the Voice Agent
/// `Settings` message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SpeakSettings {
    /// TTS provider configuration.
    pub provider: SpeakProvider,

    /// Custom TTS endpoint. Optional with the Deepgram provider; required
    /// for non-Deepgram providers per the spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<Endpoint>,
}

impl SpeakSettings {
    /// Construct with just a provider and no custom endpoint.
    pub fn new(provider: SpeakProvider) -> Self {
        Self {
            provider,
            endpoint: None,
        }
    }
}

/// TTS provider variants supported by the Voice Agent.
///
/// Wire format is internally tagged on `type` with snake_case discriminator
/// values: `deepgram`, `eleven_labs`, `cartesia`, `open_ai`, `aws_polly`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum SpeakProvider {
    /// Deepgram Aura TTS.
    Deepgram(DeepgramSpeakProvider),
    /// ElevenLabs.
    ElevenLabs(ElevenLabsSpeakProvider),
    /// Cartesia.
    Cartesia(CartesiaSpeakProvider),
    /// OpenAI TTS.
    OpenAi(OpenAiSpeakProvider),
    /// AWS Polly.
    AwsPolly(AwsPollySpeakProvider),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AwsCredentials, AwsCredentialsType};
    use serde_json::json;

    #[test]
    fn round_trip_deepgram() {
        let raw = json!({
            "type": "deepgram",
            "model": "aura-2-thalia-en",
            "speed": 1.0
        });
        let provider: SpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            SpeakProvider::Deepgram(p) => {
                assert_eq!(p.model, DeepgramSpeakModel::Aura2ThaliaEn);
            }
            _ => panic!("expected Deepgram"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn round_trip_eleven_labs() {
        let raw = json!({
            "type": "eleven_labs",
            "model_id": "eleven_multilingual_v2",
            "language": "en-US"
        });
        let provider: SpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            SpeakProvider::ElevenLabs(p) => {
                assert_eq!(p.model_id, ElevenLabsModelId::ElevenMultilingualV2);
            }
            _ => panic!("expected ElevenLabs"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn round_trip_cartesia() {
        let raw = json!({
            "type": "cartesia",
            "model_id": "sonic-2",
            "voice": { "mode": "id", "id": "voice_42" }
        });
        let provider: SpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            SpeakProvider::Cartesia(p) => {
                assert_eq!(p.model_id, CartesiaModelId::Sonic2);
                assert_eq!(p.voice.id, "voice_42");
            }
            _ => panic!("expected Cartesia"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn round_trip_openai_speak() {
        let raw = json!({
            "type": "open_ai",
            "model": "tts-1",
            "voice": "shimmer"
        });
        let provider: SpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            SpeakProvider::OpenAi(p) => {
                assert_eq!(p.model, OpenAiSpeakModel::Tts1);
                assert_eq!(p.voice, OpenAiVoice::Shimmer);
            }
            _ => panic!("expected OpenAi"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn round_trip_aws_polly() {
        let raw = json!({
            "type": "aws_polly",
            "voice": "Matthew",
            "language": "en-US",
            "engine": "generative",
            "credentials": {
                "type": "sts",
                "region": "us-east-1",
                "access_key_id": "AKIA999",
                "secret_access_key": "s",
                "session_token": "tok"
            }
        });
        let provider: SpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            SpeakProvider::AwsPolly(p) => {
                assert_eq!(p.voice, AwsPollyVoice::Matthew);
                assert_eq!(p.engine, AwsPollyEngine::Generative);
                assert_eq!(p.credentials.credentials_type, AwsCredentialsType::Sts);
            }
            _ => panic!("expected AwsPolly"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn settings_with_endpoint() {
        let raw = json!({
            "provider": {
                "type": "eleven_labs",
                "model_id": "eleven_turbo_v2_5"
            },
            "endpoint": {
                "url": "wss://tts.internal/stream"
            }
        });
        let settings: SpeakSettings = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(settings.provider, SpeakProvider::ElevenLabs(_)));
        assert!(settings.endpoint.is_some());
        assert_eq!(serde_json::to_value(&settings).unwrap(), raw);
    }

    #[test]
    fn settings_minimal() {
        let raw = json!({
            "provider": {
                "type": "deepgram",
                "model": "aura-asteria-en"
            }
        });
        let settings: SpeakSettings = serde_json::from_value(raw.clone()).unwrap();
        assert!(settings.endpoint.is_none());
        assert_eq!(serde_json::to_value(&settings).unwrap(), raw);
    }

    #[test]
    fn settings_with_aws_polly_full_construction() {
        // Smoke test that the typed builders compose without serde gymnastics.
        let creds = AwsCredentials {
            credentials_type: AwsCredentialsType::Iam,
            region: Some("us-east-1".into()),
            access_key_id: Some("AKIA".into()),
            secret_access_key: Some("s".into()),
            session_token: None,
        };
        let polly =
            AwsPollySpeakProvider::new(AwsPollyVoice::Aria, "en-US", AwsPollyEngine::Neural, creds);
        let settings = SpeakSettings::new(SpeakProvider::AwsPolly(polly));
        let value = serde_json::to_value(&settings).unwrap();
        assert_eq!(value["provider"]["type"], "aws_polly");
        assert_eq!(value["provider"]["voice"], "Aria");
    }
}
