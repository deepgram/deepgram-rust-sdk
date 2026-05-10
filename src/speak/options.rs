//! Set various Deepgram features to control how the speech is generated.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/docs/tts-feature-overview

use std::borrow::Cow;

use serde::{ser::SerializeSeq, Deserialize, Serialize};

/// Voice model used by the Speak (TTS) API.
///
/// Construct a named voice via one of the associated functions
/// (e.g. [`Model::aura_2_thalia_en`]) or pass an arbitrary or
/// self-hosted voice id via [`Model::custom`].
///
/// See the [Deepgram TTS Models docs][docs] for the up-to-date voice
/// catalog and locale support.
///
/// [docs]: https://developers.deepgram.com/docs/tts-models
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Model(Cow<'static, str>);

impl Model {
    /// Construct a [`Model`] from an arbitrary voice id — useful for
    /// new voices not yet listed as named constructors, or for
    /// self-hosted voice deployments.
    pub fn custom(id: impl Into<String>) -> Self {
        Self(Cow::Owned(id.into()))
    }

    /// The voice id as sent to the API, e.g. `aura-2-thalia-en`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Model {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<String> for Model {
    fn from(value: String) -> Self {
        Self::custom(value)
    }
}

macro_rules! aura_voices {
    ($( $fn:ident => $id:literal ),+ $(,)?) => {
        impl Model {
            $(
                #[doc = concat!("`", $id, "` voice.")]
                pub const fn $fn() -> Self {
                    Self(Cow::Borrowed($id))
                }
            )+
        }
    };
}

aura_voices! {
    // Aura-1 (English)
    aura_asteria_en   => "aura-asteria-en",
    aura_luna_en      => "aura-luna-en",
    aura_stella_en    => "aura-stella-en",
    aura_athena_en    => "aura-athena-en",
    aura_hera_en      => "aura-hera-en",
    aura_orion_en     => "aura-orion-en",
    aura_arcas_en     => "aura-arcas-en",
    aura_perseus_en   => "aura-perseus-en",
    aura_angus_en     => "aura-angus-en",
    aura_orpheus_en   => "aura-orpheus-en",
    aura_helios_en    => "aura-helios-en",
    aura_zeus_en      => "aura-zeus-en",
    // Aura-2 (English)
    aura_2_amalthea_en   => "aura-2-amalthea-en",
    aura_2_andromeda_en  => "aura-2-andromeda-en",
    aura_2_apollo_en     => "aura-2-apollo-en",
    aura_2_arcas_en      => "aura-2-arcas-en",
    aura_2_aries_en      => "aura-2-aries-en",
    aura_2_asteria_en    => "aura-2-asteria-en",
    aura_2_athena_en     => "aura-2-athena-en",
    aura_2_atlas_en      => "aura-2-atlas-en",
    aura_2_aurora_en     => "aura-2-aurora-en",
    aura_2_callista_en   => "aura-2-callista-en",
    aura_2_cordelia_en   => "aura-2-cordelia-en",
    aura_2_cora_en       => "aura-2-cora-en",
    aura_2_delia_en      => "aura-2-delia-en",
    aura_2_draco_en      => "aura-2-draco-en",
    aura_2_electra_en    => "aura-2-electra-en",
    aura_2_harmonia_en   => "aura-2-harmonia-en",
    aura_2_helena_en     => "aura-2-helena-en",
    aura_2_hera_en       => "aura-2-hera-en",
    aura_2_hermes_en     => "aura-2-hermes-en",
    aura_2_hyperion_en   => "aura-2-hyperion-en",
    aura_2_iris_en       => "aura-2-iris-en",
    aura_2_janus_en      => "aura-2-janus-en",
    aura_2_juno_en       => "aura-2-juno-en",
    aura_2_jupiter_en    => "aura-2-jupiter-en",
    aura_2_luna_en       => "aura-2-luna-en",
    aura_2_mars_en       => "aura-2-mars-en",
    aura_2_minerva_en    => "aura-2-minerva-en",
    aura_2_neptune_en    => "aura-2-neptune-en",
    aura_2_odysseus_en   => "aura-2-odysseus-en",
    aura_2_ophelia_en    => "aura-2-ophelia-en",
    aura_2_orion_en      => "aura-2-orion-en",
    aura_2_orpheus_en    => "aura-2-orpheus-en",
    aura_2_pandora_en    => "aura-2-pandora-en",
    aura_2_phoebe_en     => "aura-2-phoebe-en",
    aura_2_pluto_en      => "aura-2-pluto-en",
    aura_2_saturn_en     => "aura-2-saturn-en",
    aura_2_selene_en     => "aura-2-selene-en",
    aura_2_thalia_en     => "aura-2-thalia-en",
    aura_2_theia_en      => "aura-2-theia-en",
    aura_2_vesta_en      => "aura-2-vesta-en",
    aura_2_zeus_en       => "aura-2-zeus-en",
    // Aura-2 (Spanish)
    aura_2_sirio_es      => "aura-2-sirio-es",
    aura_2_nestor_es     => "aura-2-nestor-es",
    aura_2_carina_es     => "aura-2-carina-es",
    aura_2_celeste_es    => "aura-2-celeste-es",
    aura_2_alvaro_es     => "aura-2-alvaro-es",
    aura_2_diana_es      => "aura-2-diana-es",
    aura_2_aquila_es     => "aura-2-aquila-es",
    aura_2_selena_es     => "aura-2-selena-es",
    aura_2_estrella_es   => "aura-2-estrella-es",
    aura_2_javier_es     => "aura-2-javier-es",
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
            Container::None => "none",
            Container::CustomContainer(container) => container,
        }
    }
}

/// HTTP method used for callback delivery on the Speak REST API.
///
/// See the [Deepgram Callback docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/callback
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
pub enum CallbackMethod {
    /// POST callback (default).
    POST,
    /// PUT callback.
    PUT,
}

impl CallbackMethod {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            CallbackMethod::POST => "post",
            CallbackMethod::PUT => "put",
        }
    }
}

/// Used as a parameter for [`Speak::speak_to_file`](crate::Speak::speak_to_file) and similar functions.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Options {
    model: Option<Model>,
    encoding: Option<Encoding>,
    sample_rate: Option<u32>,
    container: Option<Container>,
    bit_rate: Option<u32>,
    speed: Option<f32>,
    callback: Option<String>,
    callback_method: Option<CallbackMethod>,
    tags: Vec<String>,
    mip_opt_out: Option<bool>,
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
    ///     .model(Model::aura_arcas_en())
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
        Self(Options::default())
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

    /// Speaking rate multiplier. Valid range per spec is `0.7..=1.5`,
    /// with a default of `1.0` server-side. Not supported in every
    /// language.
    pub fn speed(mut self, speed: f32) -> Self {
        self.0.speed = Some(speed);
        self
    }

    /// URL to receive the generated audio via callback rather than in
    /// the response body.
    ///
    /// See the [Deepgram Callback docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/callback
    pub fn callback(mut self, callback: impl Into<String>) -> Self {
        self.0.callback = Some(callback.into());
        self
    }

    /// HTTP method used for the callback request. Defaults to `POST`
    /// server-side when unset.
    pub fn callback_method(mut self, callback_method: CallbackMethod) -> Self {
        self.0.callback_method = Some(callback_method);
        self
    }

    /// Replace the request tags. Tags are repeated query params in
    /// usage reporting.
    pub fn tag<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.0.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    /// Opt this request out of the Deepgram Model Improvement Program.
    /// Refer to the [Deepgram MIP docs][docs] for pricing implications.
    ///
    /// [docs]: https://dpgr.am/deepgram-mip
    pub fn mip_opt_out(mut self, mip_opt_out: bool) -> Self {
        self.0.mip_opt_out = Some(mip_opt_out);
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
            speed,
            callback,
            callback_method,
            tags,
            mip_opt_out,
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

        if let Some(speed) = speed {
            seq.serialize_element(&("speed", speed))?;
        }

        if let Some(callback) = callback {
            seq.serialize_element(&("callback", callback))?;
        }

        if let Some(callback_method) = callback_method {
            seq.serialize_element(&("callback_method", callback_method.as_str()))?;
        }

        for tag in tags {
            seq.serialize_element(&("tag", tag))?;
        }

        if let Some(mip_opt_out) = mip_opt_out {
            seq.serialize_element(&("mip_opt_out", mip_opt_out))?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_params_round_trip() {
        let q = Options::builder()
            .model(Model::aura_asteria_en())
            .speed(1.25)
            .callback("https://example.com/cb")
            .callback_method(CallbackMethod::PUT)
            .tag(["prod", "us-east"])
            .mip_opt_out(true)
            .build()
            .urlencoded()
            .unwrap();
        assert_eq!(
            q,
            "model=aura-asteria-en\
             &speed=1.25\
             &callback=https%3A%2F%2Fexample.com%2Fcb\
             &callback_method=put\
             &tag=prod&tag=us-east\
             &mip_opt_out=true"
        );
    }
}
