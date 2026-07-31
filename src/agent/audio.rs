//! Audio I/O configuration for the Voice Agent `Settings` message.
//!
//! Mirrors the `audio` block on `AgentV1SettingsMessage` in
//! `asyncapi/schemas/schemas.agent.v1.yml`.
//!
//! Note the encoding lists here are agent-specific and intentionally
//! distinct from `crate::common::options::Encoding` (STT) and
//! `crate::speak::options::Encoding` (TTS REST). The agent input
//! encoding adds `linear32`, `alaw`, and `ogg-opus` over what STT
//! exposes today; the agent output encoding includes `aac` like the
//! Speak REST API.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Audio configuration block on `AgentV1SettingsMessage.audio`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AudioConfig {
    /// Inbound audio (client → agent) configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<AudioInput>,

    /// Outbound audio (agent → client) configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<AudioOutput>,
}

impl AudioConfig {
    /// Construct with the given input and output sub-configs.
    pub fn new(input: Option<AudioInput>, output: Option<AudioOutput>) -> Self {
        Self { input, output }
    }
}

/// Inbound audio configuration. Spec defaults to `linear16` at `24000` Hz.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AudioInput {
    /// Audio encoding format.
    pub encoding: AudioInputEncoding,

    /// Sample rate in Hz. Common values: 16000, 24000, 44100, 48000.
    pub sample_rate: u32,
}

impl AudioInput {
    /// Construct with explicit encoding and sample rate.
    pub fn new(encoding: AudioInputEncoding, sample_rate: u32) -> Self {
        Self {
            encoding,
            sample_rate,
        }
    }
}

impl Default for AudioInput {
    /// Spec defaults: `linear16` at 24000 Hz.
    fn default() -> Self {
        Self {
            encoding: AudioInputEncoding::Linear16,
            sample_rate: 24_000,
        }
    }
}

/// Outbound audio configuration. All fields optional per spec.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AudioOutput {
    /// Audio encoding format. Spec default is `linear16`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<AudioOutputEncoding>,

    /// Sample rate in Hz.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,

    /// Bitrate in bits per second.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,

    /// Container format. Spec default is `none`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<AudioContainer>,
}

impl AudioOutput {
    /// Construct an empty output config; all fields default to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(missing_docs)]
    pub fn with_encoding(mut self, encoding: AudioOutputEncoding) -> Self {
        self.encoding = Some(encoding);
        self
    }

    #[allow(missing_docs)]
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    #[allow(missing_docs)]
    pub fn with_bitrate(mut self, bitrate: u32) -> Self {
        self.bitrate = Some(bitrate);
        self
    }

    #[allow(missing_docs)]
    pub fn with_container(mut self, container: AudioContainer) -> Self {
        self.container = Some(container);
        self
    }
}

/// Inbound audio encoding for the Voice Agent. Use [`AudioInputEncoding::Other`] for
/// values not yet enumerated by this SDK.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum AudioInputEncoding {
    Linear16,
    Linear32,
    Flac,
    Alaw,
    Mulaw,
    AmrNb,
    AmrWb,
    Opus,
    OggOpus,
    Speex,
    G729,
    /// Forward-compatibility escape.
    Other(String),
}

impl AudioInputEncoding {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Linear16 => "linear16",
            Self::Linear32 => "linear32",
            Self::Flac => "flac",
            Self::Alaw => "alaw",
            Self::Mulaw => "mulaw",
            Self::AmrNb => "amr-nb",
            Self::AmrWb => "amr-wb",
            Self::Opus => "opus",
            Self::OggOpus => "ogg-opus",
            Self::Speex => "speex",
            Self::G729 => "g729",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for AudioInputEncoding {
    fn from(value: String) -> Self {
        match value.as_str() {
            "linear16" => Self::Linear16,
            "linear32" => Self::Linear32,
            "flac" => Self::Flac,
            "alaw" => Self::Alaw,
            "mulaw" => Self::Mulaw,
            "amr-nb" => Self::AmrNb,
            "amr-wb" => Self::AmrWb,
            "opus" => Self::Opus,
            "ogg-opus" => Self::OggOpus,
            "speex" => Self::Speex,
            "g729" => Self::G729,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for AudioInputEncoding {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AudioInputEncoding {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

/// Outbound audio encoding for the Voice Agent.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum AudioOutputEncoding {
    Linear16,
    Mulaw,
    Alaw,
    Mp3,
    Opus,
    Flac,
    Aac,
    /// Forward-compatibility escape.
    Other(String),
}

impl AudioOutputEncoding {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Linear16 => "linear16",
            Self::Mulaw => "mulaw",
            Self::Alaw => "alaw",
            Self::Mp3 => "mp3",
            Self::Opus => "opus",
            Self::Flac => "flac",
            Self::Aac => "aac",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for AudioOutputEncoding {
    fn from(value: String) -> Self {
        match value.as_str() {
            "linear16" => Self::Linear16,
            "mulaw" => Self::Mulaw,
            "alaw" => Self::Alaw,
            "mp3" => Self::Mp3,
            "opus" => Self::Opus,
            "flac" => Self::Flac,
            "aac" => Self::Aac,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for AudioOutputEncoding {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AudioOutputEncoding {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

/// Audio container format for outbound audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AudioContainer {
    /// No container (`none`).
    None,
    /// WAV.
    Wav,
    /// Ogg.
    Ogg,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn input_round_trip_default_encoding() {
        let raw = json!({ "encoding": "linear16", "sample_rate": 24000 });
        let input: AudioInput = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(input.encoding, AudioInputEncoding::Linear16);
        assert_eq!(input.sample_rate, 24_000);
        assert_eq!(serde_json::to_value(&input).unwrap(), raw);
    }

    #[test]
    fn input_round_trip_ogg_opus() {
        let raw = json!({ "encoding": "ogg-opus", "sample_rate": 48000 });
        let input: AudioInput = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(input.encoding, AudioInputEncoding::OggOpus);
        assert_eq!(serde_json::to_value(&input).unwrap(), raw);
    }

    #[test]
    fn input_unknown_encoding_falls_back_to_other() {
        let raw = json!({ "encoding": "future-codec", "sample_rate": 16000 });
        let input: AudioInput = serde_json::from_value(raw).unwrap();
        assert_eq!(
            input.encoding,
            AudioInputEncoding::Other("future-codec".into())
        );
    }

    #[test]
    fn output_round_trip_full() {
        let raw = json!({
            "encoding": "mp3",
            "sample_rate": 22050,
            "bitrate": 48000,
            "container": "none"
        });
        let output: AudioOutput = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(output.encoding, Some(AudioOutputEncoding::Mp3));
        assert_eq!(output.container, Some(AudioContainer::None));
        assert_eq!(serde_json::to_value(&output).unwrap(), raw);
    }

    #[test]
    fn output_default_serializes_empty() {
        let output = AudioOutput::default();
        let value = serde_json::to_value(&output).unwrap();
        assert_eq!(value, json!({}));
    }

    #[test]
    fn output_builder_chain() {
        let output = AudioOutput::new()
            .with_encoding(AudioOutputEncoding::Aac)
            .with_sample_rate(48000)
            .with_bitrate(192_000);
        assert_eq!(output.encoding, Some(AudioOutputEncoding::Aac));
        assert_eq!(output.sample_rate, Some(48000));
        assert_eq!(output.bitrate, Some(192_000));
    }

    #[test]
    fn audio_config_round_trip() {
        let raw = json!({
            "input": { "encoding": "alaw", "sample_rate": 8000 },
            "output": {
                "encoding": "opus",
                "sample_rate": 48000,
                "container": "ogg"
            }
        });
        let config: AudioConfig = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(
            config.input.as_ref().unwrap().encoding,
            AudioInputEncoding::Alaw
        ));
        assert_eq!(
            config.output.as_ref().unwrap().container,
            Some(AudioContainer::Ogg)
        );
        assert_eq!(serde_json::to_value(&config).unwrap(), raw);
    }

    #[test]
    fn audio_config_omits_none_fields() {
        let config = AudioConfig::new(Some(AudioInput::default()), None);
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(
            value,
            json!({ "input": { "encoding": "linear16", "sample_rate": 24000 } })
        );
    }

    #[test]
    fn audio_input_default_helper() {
        let input = AudioInput::default();
        assert_eq!(input.encoding, AudioInputEncoding::Linear16);
        assert_eq!(input.sample_rate, 24_000);
    }

    #[test]
    fn output_container_serialization() {
        for (container, wire) in [
            (AudioContainer::None, "none"),
            (AudioContainer::Wav, "wav"),
            (AudioContainer::Ogg, "ogg"),
        ] {
            let serialized = serde_json::to_value(container).unwrap();
            assert_eq!(serialized, json!(wire));
            let back: AudioContainer = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(back, container);
        }
    }
}
