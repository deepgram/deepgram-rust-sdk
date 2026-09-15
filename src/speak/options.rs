//! Set various Deepgram features to control how the speech is generated.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/docs/tts-feature-overview

use serde::{ser::SerializeSeq, Deserialize, Serialize};

/// Defines the [`Model`] enum together with its wire-string mapping in both
/// directions, so a voice can never be added to one without the other.
///
/// Voices are grouped by language; each variant's doc comment is generated
/// from its wire string and that group's language name, so all 100+ variants
/// stay documented from this one place.
macro_rules! speak_models {
    ($($language:literal { $($variant:ident => $wire:literal,)+ })+) => {
        /// Used as a parameter for [`OptionsBuilder::model`] and
        /// [`SpeakStreamBuilder::model`](crate::speak::SpeakStreamBuilder::model).
        ///
        /// The named variants cover the Aura-1 (`aura-*-en`) and Aura-2
        /// (`aura-2-*`) voices listed in the Deepgram API specification; the
        /// server default is [`Model::AuraAsteriaEn`]. Any voice not yet
        /// modeled can be passed as [`Model::CustomId`], and
        /// `Model::from("aura-2-thalia-en")` resolves a wire string to its
        /// named variant (or `CustomId` for an unrecognized one).
        ///
        /// See the [Deepgram Model feature docs][docs] for more info.
        ///
        /// [docs]: https://developers.deepgram.com/docs/tts-models
        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        #[non_exhaustive]
        pub enum Model {
            $($(
                #[doc = concat!("The `", $wire, "` ", $language, " voice.")]
                $variant,
            )+)+

            /// A voice identified by its raw wire string, for voices that do
            /// not yet have a named variant.
            ///
            /// A `CustomId` holding the wire string of a named variant is
            /// *not* equal to that variant, even though both serialize
            /// identically: `Model::CustomId("aura-2-thalia-en".to_string())
            /// != Model::Aura2ThaliaEn`, while
            /// `Model::from("aura-2-thalia-en") == Model::Aura2ThaliaEn`.
            /// When comparing `Model` values, normalize through
            /// [`Model::from`] (or compare
            /// [`as_ref`](AsRef::as_ref) strings) rather than comparing
            /// constructed values directly.
            CustomId(String),
        }

        impl AsRef<str> for Model {
            fn as_ref(&self) -> &str {
                match self {
                    $($(Self::$variant => $wire,)+)+
                    Self::CustomId(id) => id,
                }
            }
        }

        impl From<&str> for Model {
            /// Resolve a wire string (for example `"aura-2-thalia-en"`) to its
            /// named variant, or [`Model::CustomId`] if it is not recognized.
            fn from(value: &str) -> Self {
                match value {
                    $($($wire => Self::$variant,)+)+
                    other => Self::CustomId(other.to_string()),
                }
            }
        }

        impl From<String> for Model {
            fn from(value: String) -> Self {
                Self::from(value.as_str())
            }
        }

        impl Model {
            /// Every named variant, in specification order (excludes
            /// [`Model::CustomId`]).
            #[cfg(test)]
            pub(crate) fn named_variants() -> Vec<Model> {
                vec![$($(Self::$variant,)+)+]
            }
        }
    };
}

speak_models! {
    "English" {
        AuraAngusEn => "aura-angus-en",
        AuraArcasEn => "aura-arcas-en",
        AuraAsteriaEn => "aura-asteria-en",
        AuraAthenaEn => "aura-athena-en",
        AuraHeliosEn => "aura-helios-en",
        AuraHeraEn => "aura-hera-en",
        AuraLunaEn => "aura-luna-en",
        AuraOrionEn => "aura-orion-en",
        AuraOrpheusEn => "aura-orpheus-en",
        AuraPerseusEn => "aura-perseus-en",
        AuraStellaEn => "aura-stella-en",
        AuraZeusEn => "aura-zeus-en",
        Aura2AmaltheaEn => "aura-2-amalthea-en",
        Aura2AndromedaEn => "aura-2-andromeda-en",
        Aura2ApolloEn => "aura-2-apollo-en",
        Aura2ArcasEn => "aura-2-arcas-en",
        Aura2AriesEn => "aura-2-aries-en",
        Aura2AsteriaEn => "aura-2-asteria-en",
        Aura2AthenaEn => "aura-2-athena-en",
        Aura2AtlasEn => "aura-2-atlas-en",
        Aura2AuroraEn => "aura-2-aurora-en",
        Aura2CallistaEn => "aura-2-callista-en",
        Aura2CoraEn => "aura-2-cora-en",
        Aura2CordeliaEn => "aura-2-cordelia-en",
        Aura2DeliaEn => "aura-2-delia-en",
        Aura2DracoEn => "aura-2-draco-en",
        Aura2ElectraEn => "aura-2-electra-en",
        Aura2HarmoniaEn => "aura-2-harmonia-en",
        Aura2HelenaEn => "aura-2-helena-en",
        Aura2HeraEn => "aura-2-hera-en",
        Aura2HermesEn => "aura-2-hermes-en",
        Aura2HyperionEn => "aura-2-hyperion-en",
        Aura2IrisEn => "aura-2-iris-en",
        Aura2JanusEn => "aura-2-janus-en",
        Aura2JunoEn => "aura-2-juno-en",
        Aura2JupiterEn => "aura-2-jupiter-en",
        Aura2LunaEn => "aura-2-luna-en",
        Aura2MarsEn => "aura-2-mars-en",
        Aura2MinervaEn => "aura-2-minerva-en",
        Aura2NeptuneEn => "aura-2-neptune-en",
        Aura2OdysseusEn => "aura-2-odysseus-en",
        Aura2OpheliaEn => "aura-2-ophelia-en",
        Aura2OrionEn => "aura-2-orion-en",
        Aura2OrpheusEn => "aura-2-orpheus-en",
        Aura2PandoraEn => "aura-2-pandora-en",
        Aura2PhoebeEn => "aura-2-phoebe-en",
        Aura2PlutoEn => "aura-2-pluto-en",
        Aura2SaturnEn => "aura-2-saturn-en",
        Aura2SeleneEn => "aura-2-selene-en",
        Aura2ThaliaEn => "aura-2-thalia-en",
        Aura2TheiaEn => "aura-2-theia-en",
        Aura2VestaEn => "aura-2-vesta-en",
        Aura2ZeusEn => "aura-2-zeus-en",
    }

    "Spanish" {
        Aura2AgustinaEs => "aura-2-agustina-es",
        Aura2AlvaroEs => "aura-2-alvaro-es",
        Aura2AntoniaEs => "aura-2-antonia-es",
        Aura2AquilaEs => "aura-2-aquila-es",
        Aura2CarinaEs => "aura-2-carina-es",
        Aura2CelesteEs => "aura-2-celeste-es",
        Aura2DianaEs => "aura-2-diana-es",
        Aura2EstrellaEs => "aura-2-estrella-es",
        Aura2GloriaEs => "aura-2-gloria-es",
        Aura2JavierEs => "aura-2-javier-es",
        Aura2LucianoEs => "aura-2-luciano-es",
        Aura2NestorEs => "aura-2-nestor-es",
        Aura2OliviaEs => "aura-2-olivia-es",
        Aura2SelenaEs => "aura-2-selena-es",
        Aura2SilviaEs => "aura-2-silvia-es",
        Aura2SirioEs => "aura-2-sirio-es",
        Aura2ValerioEs => "aura-2-valerio-es",
    }

    "German" {
        Aura2AureliaDe => "aura-2-aurelia-de",
        Aura2ElaraDe => "aura-2-elara-de",
        Aura2FabianDe => "aura-2-fabian-de",
        Aura2JuliusDe => "aura-2-julius-de",
        Aura2KaraDe => "aura-2-kara-de",
        Aura2LaraDe => "aura-2-lara-de",
        Aura2ViktoriaDe => "aura-2-viktoria-de",
    }

    "Dutch" {
        Aura2BeatrixNl => "aura-2-beatrix-nl",
        Aura2CorneliaNl => "aura-2-cornelia-nl",
        Aura2DaphneNl => "aura-2-daphne-nl",
        Aura2HestiaNl => "aura-2-hestia-nl",
        Aura2LarsNl => "aura-2-lars-nl",
        Aura2LedaNl => "aura-2-leda-nl",
        Aura2RheaNl => "aura-2-rhea-nl",
        Aura2RomanNl => "aura-2-roman-nl",
        Aura2SanderNl => "aura-2-sander-nl",
    }

    "French" {
        Aura2AgatheFr => "aura-2-agathe-fr",
        Aura2HectorFr => "aura-2-hector-fr",
    }

    "Italian" {
        Aura2CesareIt => "aura-2-cesare-it",
        Aura2CinziaIt => "aura-2-cinzia-it",
        Aura2DemetraIt => "aura-2-demetra-it",
        Aura2DionisioIt => "aura-2-dionisio-it",
        Aura2ElioIt => "aura-2-elio-it",
        Aura2FlavioIt => "aura-2-flavio-it",
        Aura2LiviaIt => "aura-2-livia-it",
        Aura2MaiaIt => "aura-2-maia-it",
        Aura2MeliaIt => "aura-2-melia-it",
        Aura2PerseoIt => "aura-2-perseo-it",
    }

    "Japanese" {
        Aura2AmaJa => "aura-2-ama-ja",
        Aura2EbisuJa => "aura-2-ebisu-ja",
        Aura2FujinJa => "aura-2-fujin-ja",
        Aura2IzanamiJa => "aura-2-izanami-ja",
        Aura2UzumeJa => "aura-2-uzume-ja",
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
            Container::None => "none",
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

#[cfg(test)]
mod model_tests {
    use super::Model;

    #[test]
    fn every_named_variant_round_trips_through_its_wire_string() {
        let variants = Model::named_variants();
        // Exact, not a lower bound: 12 Aura-1 voices plus the 91 Aura-2
        // voices in the API specification. When the specification grows,
        // this fails until the new voice is added.
        assert_eq!(
            variants.len(),
            103,
            "expected every voice in the specification, got {}",
            variants.len()
        );
        for model in variants {
            let wire = model.as_ref().to_string();
            let parsed = Model::from(wire.as_str());
            assert_eq!(parsed, model, "`{wire}` must parse back to {model:?}");
            assert!(
                !matches!(parsed, Model::CustomId(_)),
                "`{wire}` must resolve to a named variant"
            );
            assert_eq!(Model::from(wire.clone()), model);
        }
    }

    #[test]
    fn wire_strings_are_unique() {
        let variants = Model::named_variants();
        let mut wires: Vec<&str> = variants.iter().map(AsRef::as_ref).collect();
        let before = wires.len();
        wires.sort_unstable();
        wires.dedup();
        assert_eq!(wires.len(), before, "duplicate wire strings");
    }

    #[test]
    fn aura_2_voices_are_named() {
        assert_eq!(Model::Aura2ThaliaEn.as_ref(), "aura-2-thalia-en");
        assert_eq!(Model::from("aura-2-thalia-en"), Model::Aura2ThaliaEn);
        assert_eq!(Model::Aura2AgustinaEs.as_ref(), "aura-2-agustina-es");
        assert_eq!(Model::Aura2PerseoIt.as_ref(), "aura-2-perseo-it");
        assert_eq!(Model::from("aura-2-perseo-it"), Model::Aura2PerseoIt);
        assert_eq!(Model::Aura2UzumeJa.as_ref(), "aura-2-uzume-ja");
        // Aura-1 mapping is unchanged.
        assert_eq!(Model::AuraAsteriaEn.as_ref(), "aura-asteria-en");
    }

    #[test]
    fn unknown_wire_string_becomes_custom_id() {
        let model = Model::from("aura-3-future-en");
        assert_eq!(model, Model::CustomId("aura-3-future-en".to_string()));
        assert_eq!(model.as_ref(), "aura-3-future-en");
    }
}
