//! Set various Deepgram features to control how the speech is generated.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/docs/tts-feature-overview

use serde::{ser::SerializeSeq, Deserialize, Serialize};

/// Used as a parameter for [`OptionsBuilder::model`].
///
/// See the [Deepgram Model feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/tts-models
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Model {
    #[allow(missing_docs)]
    AuraAsteriaEn,

    #[allow(missing_docs)]
    AuraLunaEn,

    #[allow(missing_docs)]
    AuraStellaEn,

    #[allow(missing_docs)]
    AuraAthenaEn,

    #[allow(missing_docs)]
    AuraHeraEn,

    #[allow(missing_docs)]
    AuraOrionEn,

    #[allow(missing_docs)]
    AuraArcasEn,

    #[allow(missing_docs)]
    AuraPerseusEn,

    #[allow(missing_docs)]
    AuraAngusEn,

    #[allow(missing_docs)]
    AuraOrpheusEn,

    #[allow(missing_docs)]
    AuraHeliosEn,

    #[allow(missing_docs)]
    AuraZeusEn,

    #[allow(missing_docs)]
    CustomId(String),
}

impl AsRef<str> for Model {
    fn as_ref(&self) -> &str {
        match self {
            Self::AuraAsteriaEn => "aura-asteria-en",
            Self::AuraLunaEn => "aura-luna-en",
            Self::AuraStellaEn => "aura-stella-en",
            Self::AuraAthenaEn => "aura-athena-en",
            Self::AuraHeraEn => "aura-hera-en",
            Self::AuraOrionEn => "aura-orion-en",
            Self::AuraArcasEn => "aura-arcas-en",
            Self::AuraPerseusEn => "aura-perseus-en",
            Self::AuraAngusEn => "aura-angus-en",
            Self::AuraOrpheusEn => "aura-orpheus-en",
            Self::AuraHeliosEn => "aura-helios-en",
            Self::AuraZeusEn => "aura-zeus-en",
            Self::CustomId(id) => id,
        }
    }
}

/// Encoding value
///
/// See the [Deepgram Encoding feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/tts-encoding
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Encoding {
    /// 16-bit, little endian, signed PCM WAV data
    Linear16,
    /// Mu-law encoded WAV data
    Mulaw,
    /// Alaw
    Alaw,
    /// Mp3
    Mp3,
    /// Ogg Opus
    Opus,
    /// Free Lossless Audio Codec (FLAC) encoded data
    Flac,
    /// Aac
    Aac,

    #[allow(missing_docs)]
    CustomEncoding(String),
}

/// TTSEncoding Impl
impl Encoding {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Encoding::Linear16 => "linear16",
            Encoding::Mulaw => "mulaw",
            Encoding::Alaw => "alaw",
            Encoding::Mp3 => "mp3",
            Encoding::Opus => "opus",
            Encoding::Flac => "flac",
            Encoding::Aac => "aac",
            Encoding::CustomEncoding(encoding) => encoding,
        }
    }
}

/// Container value
///
/// See the [Deepgram Container feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/tts-container
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Container {
    #[allow(missing_docs)]
    Wav,
    #[allow(missing_docs)]
    Ogg,
    #[allow(missing_docs)]
    None,

    #[allow(missing_docs)]
    CustomContainer(String),
}

/// Encoding Impl
impl Container {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Container::Wav => "wav",
            Container::Ogg => "ogg",
            Container::None => "nonne",
            Container::CustomContainer(container) => container,
        }
    }
}

/// Used as a parameter for [`Speak::speak_to_file`](crate::Speak::speak_to_file) and similar functions.
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    model: Option<Model>,
    encoding: Option<Encoding>,
    sample_rate: Option<u32>,
    container: Option<Container>,
    bit_rate: Option<u32>,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// Use it to set any of Deepgram's features except the Callback feature.
/// The Callback feature can be set when making the request by calling [`Transcription::prerecorded_callback`](crate::Speak::speak_to_file).
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Debug, PartialEq, Clone)]
pub(super) struct SerializableOptions<'a>(pub(super) &'a Options);

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::new()
    }

    /// Return the Options in urlencoded format. If serialization would
    /// fail, this will also return an error.
    ///
    /// This is intended primarily to help with debugging API requests.
    ///
    /// ```
    /// use deepgram::speak::options::{Encoding, Model, Options};
    /// let options = Options::builder()
    ///     .model(Model::AuraArcasEn)
    ///     .encoding(Encoding::Flac)
    ///     .build();
    /// assert_eq!(&options.urlencoded().unwrap(), "model=aura-arcas-en&encoding=flac")
    /// ```
    ///
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions(self))
    }

}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new() -> Self {
        Self(Options {
            model: None,
            encoding: None,
            sample_rate: None,
            container: None,
            bit_rate: None,
        })
    }

    /// Set the Model feature.
    ///
    /// See the [Deepgram Model feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-models
    pub fn model(mut self, model: Model) -> Self {
        self.0.model = Some(model);
        self
    }

    /// Set the Encoding feature.
    ///
    /// See the [Deepgram Encoding feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-encoding
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.0.encoding = Some(encoding);
        self
    }

    /// Set the Sample Rate feature.
    ///
    /// See the [Deepgram Sample Rate feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-sample-rate
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.0.sample_rate = Some(sample_rate);
        self
    }

    /// Set the Container feature.
    ///
    /// See the [Deepgram Container docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-container
    pub fn container(mut self, container: Container) -> Self {
        self.0.container = Some(container);
        self
    }

    /// Set the Bit Rate feature.
    ///
    /// See the [Deepgram Bit Rate feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-bit-rate
    pub fn bit_rate(mut self, bit_rate: u32) -> Self {
        self.0.bit_rate = Some(bit_rate);
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options {
        self.0
    }
}

impl Default for OptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for SerializableOptions<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        // Destructuring it makes sure that we don't forget to use any of it
        let Options {
            model,
            encoding,
            sample_rate,
            container,
            bit_rate,
        } = self.0;

        if let Some(model) = model {
            seq.serialize_element(&("model", model.as_ref()))?;
        }

        if let Some(encoding) = encoding {
            seq.serialize_element(&("encoding", encoding.as_str()))?;
        }

        if let Some(sample_rate) = sample_rate {
            seq.serialize_element(&("sample_rate", sample_rate))?;
        }

        if let Some(container) = container {
            seq.serialize_element(&("container", container.as_str()))?;
        }

        if let Some(bit_rate) = bit_rate {
            seq.serialize_element(&("bit_rate", bit_rate))?;
        }

        seq.end()
    }
}
