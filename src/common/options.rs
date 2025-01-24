//! Set various Deepgram features to control how the audio is transcribed.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/documentation/features/

use std::{collections::HashMap, fmt};

use serde::{ser::SerializeSeq, Deserialize, Serialize};

/// Used as a parameter for [`Transcription::prerecorded`](crate::Transcription::prerecorded) and similar functions.
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    model: Option<Model>,
    version: Option<String>,
    language: Option<Language>,
    punctuate: Option<bool>,
    profanity_filter: Option<bool>,
    redact: Vec<Redact>,
    diarize: Option<bool>,
    diarize_version: Option<String>,
    ner: Option<bool>,
    multichannel: Option<Multichannel>,
    alternatives: Option<usize>,
    numerals: Option<bool>,
    search: Vec<String>,
    replace: Vec<Replace>,
    keywords: Vec<Keyword>,
    keyword_boost_legacy: Option<bool>,
    utterances: Option<Utterances>,
    tags: Vec<String>,
    detect_language: Option<DetectLanguage>,
    query_params: Vec<(String, String)>,
    encoding: Option<Encoding>,
    smart_format: Option<bool>,
    filler_words: Option<bool>,
    paragraphs: Option<bool>,
    detect_entities: Option<bool>,
    intents: Option<bool>,
    custom_intent_mode: Option<CustomIntentMode>,
    custom_intents: Vec<String>,
    sentiment: Option<bool>,
    topics: Option<bool>,
    custom_topic_mode: Option<CustomTopicMode>,
    custom_topics: Vec<String>,
    summarize: Option<bool>,
    dictation: Option<bool>,
    measurements: Option<bool>,
    extra: Option<HashMap<String, String>>,
    callback_method: Option<CallbackMethod>,
}

impl Default for Options {
    fn default() -> Self {
        Options::builder().build()
    }
}
/// Detect Language value
///
/// See the [Deepgram Detect Language feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/language-detection
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum DetectLanguage {
    #[allow(missing_docs)]
    Enabled,

    #[allow(missing_docs)]
    Disabled,

    #[allow(missing_docs)]
    Restricted(Vec<Language>),
}

/// DetectLanguage Impl
impl DetectLanguage {
    pub(crate) fn to_key_value_pairs(&self) -> Vec<(&str, String)> {
        match self {
            DetectLanguage::Enabled => vec![("detect_language", "true".to_string())],
            DetectLanguage::Disabled => vec![("detect_language", "false".to_string())],
            DetectLanguage::Restricted(languages) => languages
                .iter()
                .map(|lang| ("detect_language", lang.as_ref().to_string()))
                .collect(),
        }
    }
}

/// Callback Method value
///
/// See the [Deepgram Callback Method feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/callback#pre-recorded-audio
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum CallbackMethod {
    /// POST Callback Method
    POST,
    /// PUT Callback Method
    PUT,
}

/// Encoding Impl
impl CallbackMethod {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            CallbackMethod::POST => "post",
            CallbackMethod::PUT => "put",
        }
    }
}

/// Encoding value
///
/// See the [Deepgram Encoding feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/encoding
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Encoding {
    /// 16-bit, little endian, signed PCM WAV data
    Linear16,
    /// Free Lossless Audio Codec (FLAC) encoded data
    Flac,
    /// Mu-law encoded WAV data
    Mulaw,
    /// Adaptive Multi-Rate (AMR) narrowband codec
    AmrNb,
    /// Adaptive Multi-Rate (AMR) wideband codec
    AmrWb,
    /// Ogg Opus
    Opus,
    /// Speex
    Speex,
    /// G729 low-bandwidth (required for both raw and containerized audio)
    G729,

    #[allow(missing_docs)]
    CustomEncoding(String),
}

/// Encoding Impl
impl Encoding {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Encoding::Linear16 => "linear16",
            Encoding::Flac => "flac",
            Encoding::Mulaw => "mulaw",
            Encoding::AmrNb => "amr-nb",
            Encoding::AmrWb => "amr-wb",
            Encoding::Opus => "opus",
            Encoding::Speex => "speex",
            Encoding::G729 => "g729",
            Encoding::CustomEncoding(encoding) => encoding,
        }
    }
}

/// Endpointing value
///
/// See the [Deepgram Endpointing feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/endpointing
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum Endpointing {
    #[allow(missing_docs)]
    Enabled,

    #[allow(missing_docs)]
    Disabled,

    #[allow(missing_docs)]
    CustomDurationMs(u32),
}

impl fmt::Display for Endpointing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Endpointing::Enabled => f.write_str("true"),
            Endpointing::Disabled => f.write_str("false"),
            Endpointing::CustomDurationMs(value) => f.write_fmt(format_args!("{value}")),
        }
    }
}

/// Used as a parameter for [`OptionsBuilder::model`] and [`OptionsBuilder::multichannel_with_models`].
///
/// See the [Deepgram Model feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/model/
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Model {
    /// Recommended for readability and Deepgram's lowest word error rates.
    /// Recommended for most use cases.
    ///
    /// Nova-2 expands on Nova-1's advancements with speech-specific
    /// optimizations to the underlying Transformer architecture, advanced
    /// data curation techniques, and a multi-stage training methodology.
    /// These changes yield reduced word error rate (WER) and enhancements
    /// to entity recognition (i.e. proper nouns, alphanumerics, etc.),
    /// punctuation, and capitalization.
    Nova2,

    /// Recommended for readability and low word error rates.
    ///
    /// Nova is the predecessor to Nova-2. Training on this model spans over
    /// 100 domains and 47 billion tokens, making it the deepest-trained
    /// automatic speech-to-text model to date. Nova doesn't just excel in one
    /// specific domain — it is ideal for a wide array of voice applications
    /// that require high accuracy in diverse contexts. See the benchmarks.
    Nova,

    /// Recommended for lower word error rates than Base, high accuracy
    /// timestamps, and use cases that require keyword boosting.
    Enhanced,

    /// Recommended for large transcription volumes and high accuracy
    /// timestamps.
    Base,

    #[allow(missing_docs)]
    Nova2Meeting,

    #[allow(missing_docs)]
    Nova2Phonecall,

    #[allow(missing_docs)]
    Nova2Finance,

    #[allow(missing_docs)]
    Nova2Conversationalai,

    #[allow(missing_docs)]
    Nova2Voicemail,

    #[allow(missing_docs)]
    Nova2Video,

    #[allow(missing_docs)]
    Nova2Medical,

    #[allow(missing_docs)]
    Nova2Drivethru,

    #[allow(missing_docs)]
    Nova2Automotive,

    #[allow(missing_docs)]
    NovaPhonecall,

    #[allow(missing_docs)]
    NovaMedical,

    #[allow(missing_docs)]
    EnhancedMeeting,

    #[allow(missing_docs)]
    EnhancedPhonecall,

    #[allow(missing_docs)]
    EnhancedFinance,

    #[allow(missing_docs)]
    BaseMeeting,

    #[allow(missing_docs)]
    BasePhonecall,

    #[allow(missing_docs)]
    BaseVoicemail,

    #[allow(missing_docs)]
    BaseFinance,

    #[allow(missing_docs)]
    BaseConversationalai,

    #[allow(missing_docs)]
    BaseVideo,

    #[deprecated(
        since = "0.5.0",
        note = "use one of the general-purpose models like Model::Nova2 instead"
    )]
    #[allow(missing_docs)]
    General,

    #[deprecated(
        since = "0.5.0",
        note = "use one of the qualified models like Model::Nova2Meeting instead"
    )]
    #[allow(missing_docs)]
    Meeting,

    #[deprecated(
        since = "0.5.0",
        note = "use one of the qualified models like Model::Nova2Phonecall instead"
    )]
    #[allow(missing_docs)]
    Phonecall,

    #[deprecated(
        since = "0.5.0",
        note = "use one of the qualified models like Model::Nova2Voicemail instead"
    )]
    #[allow(missing_docs)]
    Voicemail,

    #[deprecated(
        since = "0.5.0",
        note = "use one of the qualified models like Model::Nova2Finance instead"
    )]
    #[allow(missing_docs)]
    Finance,

    #[deprecated(
        since = "0.5.0",
        note = "use one of the qualified models like Model::Nova2Conversationalai instead"
    )]
    #[allow(missing_docs)]
    Conversationalai,

    #[deprecated(
        since = "0.5.0",
        note = "use one of the qualified models like Model::Nova2Video instead"
    )]
    #[allow(missing_docs)]
    Video,

    #[allow(missing_docs)]
    CustomId(String),
}

/// Used as a parameter for [`OptionsBuilder::language`].
///
/// See the [Deepgram Language feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/language/
#[allow(non_camel_case_types)] // Variants should look like their BCP-47 tag
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Language {
    #[allow(missing_docs)]
    bg,

    #[allow(missing_docs)]
    ca,

    #[allow(missing_docs)]
    cs,

    #[allow(missing_docs)]
    da,

    #[allow(missing_docs)]
    de,

    #[allow(missing_docs)]
    de_CH,

    #[allow(missing_docs)]
    el,

    #[allow(missing_docs)]
    en,

    #[allow(missing_docs)]
    en_AU,

    #[allow(missing_docs)]
    en_GB,

    #[allow(missing_docs)]
    en_IN,

    #[allow(missing_docs)]
    en_NZ,

    #[allow(missing_docs)]
    en_US,

    #[allow(missing_docs)]
    es,

    #[allow(missing_docs)]
    es_419,

    #[allow(missing_docs)]
    es_LATAM,

    #[allow(missing_docs)]
    et,

    #[allow(missing_docs)]
    fi,

    #[allow(missing_docs)]
    fr,

    #[allow(missing_docs)]
    fr_CA,

    #[allow(missing_docs)]
    hi,

    #[allow(missing_docs)]
    hi_Latn,

    #[allow(missing_docs)]
    hu,

    #[allow(missing_docs)]
    id,

    #[allow(missing_docs)]
    it,

    #[allow(missing_docs)]
    ja,

    #[allow(missing_docs)]
    ko,

    #[allow(missing_docs)]
    ko_KR,

    #[allow(missing_docs)]
    lv,

    #[allow(missing_docs)]
    lt,

    #[allow(missing_docs)]
    ms,

    #[allow(missing_docs)]
    multi,

    #[allow(missing_docs)]
    nl,

    #[allow(missing_docs)]
    nl_BE,

    #[allow(missing_docs)]
    no,

    #[allow(missing_docs)]
    pl,

    #[allow(missing_docs)]
    pt,

    #[allow(missing_docs)]
    pt_BR,

    #[allow(missing_docs)]
    ro,

    #[allow(missing_docs)]
    ru,

    #[allow(missing_docs)]
    sk,

    #[allow(missing_docs)]
    sv,

    #[allow(missing_docs)]
    sv_SE,

    #[allow(missing_docs)]
    ta,

    #[allow(missing_docs)]
    taq,

    #[allow(missing_docs)]
    th,

    #[allow(missing_docs)]
    th_TH,

    #[allow(missing_docs)]
    tr,

    #[allow(missing_docs)]
    uk,

    #[allow(missing_docs)]
    vi,

    #[allow(missing_docs)]
    zh,

    #[allow(missing_docs)]
    zh_CN,

    #[allow(missing_docs)]
    zh_Hans,

    #[allow(missing_docs)]
    zh_Hant,

    #[allow(missing_docs)]
    zh_TW,

    /// Avoid using the `Other` variant where possible.
    /// It exists so that you can use new languages that Deepgram supports without being forced to update your version of the SDK.
    /// See the [Deepgram Language feature docs][docs] for the most up-to-date list of supported languages.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/language/
    Other(String),
}

/// Used as a parameter for [`OptionsBuilder::redact`].
///
/// See the [Deepgram Redaction feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/redact/
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Redact {
    #[allow(missing_docs)]
    Pci,

    #[allow(missing_docs)]
    Numbers,

    #[allow(missing_docs)]
    Ssn,

    /// Avoid using the `Other` variant where possible.
    /// It exists so that you can use new redactable items that Deepgram supports without being forced to update your version of the SDK.
    /// See the [Deepgram Redact feature docs][docs] for the most up-to-date list of redactable items.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/redact/
    Other(String),
}

/// Used as a parameter for [`OptionsBuilder::custom_intent_mode`].
///
/// See the [Deepgram Intent Detection feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/intent-recognition#query-parameters
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum CustomIntentMode {
    #[allow(missing_docs)]
    Extended,

    #[allow(missing_docs)]
    Strict,
}

/// Used as a parameter for [`OptionsBuilder::custom_topic_mode`].
///
/// See the [Deepgram Topic Detection feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/topic-detection#query-parameters
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum CustomTopicMode {
    #[allow(missing_docs)]
    Extended,

    #[allow(missing_docs)]
    Strict,
}

/// Used as a parameter for [`OptionsBuilder::replace`].
///
/// See the [Deepgram Find and Replace feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/replace/
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Replace {
    /// The term or phrase to find.
    pub find: String,

    /// The term or phrase to replace [`find`](Replace::find) with.
    /// If set to [`None`], [`find`](Replace::find) will be removed from the transcript without being replaced by anything.
    pub replace: Option<String>,
}

/// Used as a parameter for [`OptionsBuilder::keywords_with_intensifiers`].
///
/// See the [Deepgram Keywords feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/keywords/
#[derive(Debug, PartialEq, Clone)]
pub struct Keyword {
    /// The keyword to boost.
    pub keyword: String,

    /// Optionally specify how much to boost it.
    pub intensifier: Option<f64>,
}

/// Used as a parameter for [`OptionsBuilder::utterances`].
///
/// See the [Deepgram Utterances feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/utterances
#[derive(Debug, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum Utterances {
    #[allow(missing_docs)]
    Enabled,

    #[allow(missing_docs)]
    Disabled,

    #[allow(missing_docs)]
    CustomSplit {
        #[allow(missing_docs)]
        utt_split: Option<f64>,
    },
}

/// Used as a parameter for [`OptionsBuilder::multichannel`].
///
/// See the [Deepgram multichannel feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/multichannel
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Multichannel {
    #[allow(missing_docs)]
    Enabled,

    #[allow(missing_docs)]
    Disabled,

    #[allow(missing_docs)]
    ModelPerChannel {
        #[allow(missing_docs)]
        models: Option<Vec<Model>>,
    },
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// Use it to set of Deepgram's features, excluding the Callback feature.
/// The Callback feature can be set when making the request by calling [`Transcription::prerecorded_callback`](crate::Transcription::prerecorded_callback).
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

/// SerializableOptions
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SerializableOptions<'a>(pub(crate) &'a Options);

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
    /// use deepgram::common::options::{DetectLanguage, Model, Options};
    /// let options = Options::builder()
    ///     .model(Model::Nova2)
    ///     .detect_language(DetectLanguage::Enabled)
    ///     .build();
    /// assert_eq!(&options.urlencoded().unwrap(), "model=nova-2&detect_language=true")
    /// ```
    ///
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions::from(self))
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new() -> Self {
        Self(Options {
            model: None,
            version: None,
            language: None,
            punctuate: None,
            profanity_filter: None,
            redact: Vec::new(),
            diarize: None,
            diarize_version: None,
            ner: None,
            multichannel: None,
            alternatives: None,
            numerals: None,
            search: Vec::new(),
            replace: Vec::new(),
            keywords: Vec::new(),
            keyword_boost_legacy: None,
            utterances: None,
            tags: Vec::new(),
            detect_language: None,
            query_params: Vec::new(),
            encoding: None,
            smart_format: None,
            filler_words: None,
            paragraphs: None,
            detect_entities: None,
            intents: None,
            custom_intent_mode: None,
            custom_intents: Vec::new(),
            sentiment: None,
            topics: None,
            custom_topic_mode: None,
            custom_topics: Vec::new(),
            summarize: None,
            dictation: None,
            measurements: None,
            extra: None,
            callback_method: None,
        })
    }

    /// Set the Model feature.
    ///
    /// Not all models are supported for all languages. For a list of languages and their supported models, see
    /// the [Deepgram Language feature][language] docs.
    ///
    /// If you previously set some models using [`OptionsBuilder::multichannel_with_models`],
    /// calling this will overwrite the models you set there, but won't disable the Multichannel feature.
    ///
    /// See the [Deepgram Model feature docs][docs] for more info.
    ///
    /// [language]: https://developers.deepgram.com/documentation/features/language/
    /// [docs]: https://developers.deepgram.com/documentation/features/model/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Model, Options};
    /// #
    /// let options = Options::builder()
    ///     .model(Model::Nova2)
    ///     .build();
    /// ```
    ///
    pub fn model(mut self, model: Model) -> Self {
        self.0.model = Some(model);

        if let Some(Multichannel::ModelPerChannel { models }) = &mut self.0.multichannel {
            *models = None;
        }

        self
    }

    /// Set the Version feature.
    ///
    /// See the [Deepgram Version feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/version/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .version("12345678-1234-1234-1234-1234567890ab")
    ///     .build();
    /// ```
    pub fn version(mut self, version: &str) -> Self {
        self.0.version = Some(version.into());
        self
    }

    /// Set the Language feature.
    ///
    /// See the [Deepgram Language feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/language/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Language, Options};
    /// #
    /// let options = Options::builder()
    ///     .language(Language::en_US)
    ///     .build();
    /// ```
    pub fn language(mut self, language: Language) -> Self {
        self.0.language = Some(language);
        self
    }

    /// Set the Punctuation feature.
    ///
    /// See the [Deepgram Punctuation feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/punctuate/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .punctuate(true)
    ///     .build();
    /// ```
    pub fn punctuate(mut self, punctuate: bool) -> Self {
        self.0.punctuate = Some(punctuate);
        self
    }

    /// Set the Profanity Filter feature.
    ///
    /// Not necessarily available for all languages.
    ///
    /// See the [Deepgram Profanity Filter feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/profanity-filter/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .profanity_filter(true)
    ///     .build();
    /// ```
    pub fn profanity_filter(mut self, profanity_filter: bool) -> Self {
        self.0.profanity_filter = Some(profanity_filter);
        self
    }

    /// Set the Redaction feature.
    ///
    /// Not necessarily available for all languages.
    ///
    /// Calling this when already set will append to the existing redact items, not overwrite them.
    ///
    /// See the [Deepgram Redaction feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/redact/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Options, Redact};
    /// #
    /// let options = Options::builder()
    ///     .redact([Redact::Pci, Redact::Ssn])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::{Options, Redact};
    /// #
    /// let options1 = Options::builder()
    ///     .redact([Redact::Pci])
    ///     .redact([Redact::Ssn])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .redact([Redact::Pci, Redact::Ssn])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn redact(mut self, redact: impl IntoIterator<Item = Redact>) -> Self {
        self.0.redact.extend(redact);
        self
    }

    /// Set the Diarization feature.
    ///
    /// See the [Deepgram Diarization feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/diarize/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .diarize(true)
    ///     .build();
    /// ```
    pub fn diarize(mut self, diarize: bool) -> Self {
        self.0.diarize = Some(diarize);
        self
    }

    /// Set the Diarization Version feature.
    ///
    /// See the [Deepgram Diarization Version feature docs][docs] for more info.
    ///
    /// [docs]: https://deepgram.com/changelog/improved-speaker-diarization
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .diarize_version("2021-07-14.0")
    ///     .build();
    /// ```
    pub fn diarize_version(mut self, diarize_version: &str) -> Self {
        self.0.diarize_version = Some(diarize_version.into());
        self
    }

    /// Set the Named-Entity Recognition feature.
    ///
    /// Not necessarily available for all languages.
    ///
    /// See the [Deepgram Named-Entity Recognition feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/named-entity-recognition/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .ner(true)
    ///     .build();
    /// ```
    pub fn ner(mut self, ner: bool) -> Self {
        self.0.ner = Some(ner);
        self
    }

    /// Set the Multichannel feature.
    ///
    /// To specify which model should process each channel, use [`OptionsBuilder::multichannel_with_models`] instead.
    /// If [`OptionsBuilder::multichannel_with_models`] is currently set, calling [`OptionsBuilder::multichannel`]
    /// will reset the model to the last call to [`OptionsBuilder::model`].
    ///
    /// See the [Deepgram Multichannel feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/multichannel/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .multichannel(true)
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::{Model, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .model(Model::Nova2)
    ///     .multichannel_with_models([Model::Nova2Meeting, Model::Nova2Phonecall])
    ///     .multichannel(true)
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .model(Model::Nova2)
    ///     .multichannel(true)
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn multichannel(mut self, multichannel: bool) -> Self {
        self.0.multichannel = Some(if multichannel {
            Multichannel::Enabled
        } else {
            Multichannel::Disabled
        });

        self
    }

    /// Set the Multichannel feature to true, specifying which model should process each channel.
    ///
    /// If you previously set a model using [`OptionsBuilder::model`],
    /// calling this will overshadow the model you set there, but won't erase it.
    /// It can be recovered by calling [`OptionsBuilder::multichannel`].
    ///
    /// Calling this when multichannel models are already set will append to the existing models, not overwrite them.
    ///
    /// See the [Deepgram Multichannel feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/multichannel/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Model, Options};
    /// #
    /// let options = Options::builder()
    ///     .multichannel_with_models([Model::Meeting, Model::Phonecall])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use std::env;
    /// #
    /// # use deepgram::{
    /// #     common::{
    /// #         audio_source::AudioSource,
    /// #         options::{Model, Options},
    /// #     },
    /// #     Deepgram,
    /// # };
    /// #
    /// #
    /// # static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";
    /// #
    /// # fn main() -> Result<(), deepgram::DeepgramError> {
    /// # let deepgram_api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    /// let dg_transcription = dg_client.transcription();
    ///
    /// let options1 = Options::builder()
    ///     .model(Model::Nova2)
    ///     .multichannel_with_models([Model::Nova2Meeting, Model::Nova2Phonecall])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .multichannel_with_models([Model::Nova2Meeting, Model::Nova2Phonecall])
    ///     .build();
    ///
    /// let request1 = dg_transcription
    ///     .make_prerecorded_request_builder(AudioSource::from_url(AUDIO_URL), &options1)
    ///     .build()?;
    ///
    /// let request2 = dg_transcription
    ///     .make_prerecorded_request_builder(AudioSource::from_url(AUDIO_URL), &options2)
    ///     .build()?;
    ///
    /// // Both make the same request to Deepgram with the same features
    /// assert_eq!(request1.url(), request2.url());
    ///
    /// // However, they technically aren't "equal"
    /// // This is because `options1` still remembers the model you set previously
    /// assert_ne!(options1, options2);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::{Model, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .model(Model::Nova2)
    ///     .multichannel_with_models([Model::Nova2Meeting, Model::Nova2Phonecall])
    ///     .multichannel(true)
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .model(Model::Nova2)
    ///     .multichannel(true)
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::{Model, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .multichannel_with_models([Model::Nova2Meeting])
    ///     .multichannel_with_models([Model::Nova2Phonecall])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .multichannel_with_models([Model::Nova2Meeting, Model::Nova2Phonecall])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn multichannel_with_models(mut self, models: impl IntoIterator<Item = Model>) -> Self {
        if let Some(Multichannel::ModelPerChannel {
            models: Some(old_models),
        }) = &mut self.0.multichannel
        {
            // Multichannel with models already enabled
            // Don't overwrite existing models
            old_models.extend(models);
        } else {
            // Multichannel with models already enabled
            self.0.multichannel = Some(Multichannel::ModelPerChannel {
                models: Some(models.into_iter().collect()),
            });
        }

        self
    }

    /// Set the Alternatives feature.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#alternatives-pr
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .alternatives(3)
    ///     .build();
    /// ```
    pub fn alternatives(mut self, alternatives: usize) -> Self {
        self.0.alternatives = Some(alternatives);
        self
    }

    /// Set the Numerals feature.
    ///
    /// Not necessarily available for all languages.
    ///
    /// See the [Deepgram Numerals feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/numerals/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .numerals(true)
    ///     .build();
    /// ```
    pub fn numerals(mut self, numerals: bool) -> Self {
        self.0.numerals = Some(numerals);
        self
    }

    /// Set the Search feature.
    ///
    /// Calling this when already set will append to the existing items, not overwrite them.
    ///
    /// See the [Deepgram Search feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/search/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .search(["hello", "world"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .search(["hello"])
    ///     .search(["world"])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .search(["hello", "world"])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn search<'a>(mut self, search: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.search.extend(search.into_iter().map(String::from));
        self
    }

    /// Set the Find and Replace feature.
    ///
    /// Calling this when already set will append to the existing replacements, not overwrite them.
    ///
    /// See the [Deepgram Find and Replace feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/replace/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Options, Replace};
    /// #
    /// let options = Options::builder()
    ///     .replace([
    ///         Replace {
    ///             find: String::from("Aaron"),
    ///             replace: Some(String::from("Erin")),
    ///         },
    ///         Replace {
    ///             find: String::from("Voldemort"),
    ///             replace: None,
    ///         },
    ///     ])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::{Options, Replace};
    /// #
    /// let options1 = Options::builder()
    ///     .replace([Replace {
    ///         find: String::from("Aaron"),
    ///         replace: Some(String::from("Erin")),
    ///     }])
    ///     .replace([Replace {
    ///         find: String::from("Voldemort"),
    ///         replace: None,
    ///     }])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .replace([
    ///         Replace {
    ///             find: String::from("Aaron"),
    ///             replace: Some(String::from("Erin")),
    ///         },
    ///         Replace {
    ///             find: String::from("Voldemort"),
    ///             replace: None,
    ///         },
    ///     ])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn replace(mut self, replace: impl IntoIterator<Item = Replace>) -> Self {
        self.0.replace.extend(replace);
        self
    }

    /// Set the Keywords feature.
    ///
    /// To specify intensifiers, use [`OptionsBuilder::keywords_with_intensifiers`] instead.
    ///
    /// Calling this when already set will append to the existing keywords, not overwrite them.
    /// This includes keywords set by [`OptionsBuilder::keywords_with_intensifiers`].
    ///
    /// See the [Deepgram Keywords feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/keywords/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .keywords(["hello", "world"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .keywords(["hello"])
    ///     .keywords(["world"])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .keywords(["hello", "world"])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn keywords<'a>(mut self, keywords: impl IntoIterator<Item = &'a str>) -> Self {
        let iter = keywords.into_iter().map(|keyword| Keyword {
            keyword: keyword.into(),
            intensifier: None,
        });

        self.0.keywords.extend(iter);
        self
    }

    /// Set the Keywords feature, specifying intensifiers.
    ///
    /// If you do not need to specify intensifiers, you can use [`OptionsBuilder::keywords`] instead.
    ///
    /// Calling this when already set will append to the existing keywords, not overwrite them.
    ///
    /// See the [Deepgram Keywords feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/keywords/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Keyword, Options};
    /// #
    /// let options = Options::builder()
    ///     .keywords_with_intensifiers([
    ///         Keyword {
    ///             keyword: String::from("hello"),
    ///             intensifier: Some(-1.5),
    ///         },
    ///         Keyword {
    ///             keyword: String::from("world"),
    ///             intensifier: None,
    ///         },
    ///     ])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::{Keyword, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .keywords_with_intensifiers([
    ///         Keyword {
    ///             keyword: String::from("hello"),
    ///             intensifier: Some(-1.5),
    ///         },
    ///     ])
    ///     .keywords_with_intensifiers([
    ///         Keyword {
    ///             keyword: String::from("world"),
    ///             intensifier: None,
    ///         },
    ///     ])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .keywords_with_intensifiers([
    ///         Keyword {
    ///             keyword: String::from("hello"),
    ///             intensifier: Some(-1.5),
    ///         },
    ///         Keyword {
    ///             keyword: String::from("world"),
    ///             intensifier: None,
    ///         },
    ///     ])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn keywords_with_intensifiers(
        mut self,
        keywords: impl IntoIterator<Item = Keyword>,
    ) -> Self {
        self.0.keywords.extend(keywords);
        self
    }

    /// Use legacy keyword boosting.
    ///
    /// See the [Deepgram Keywords feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/keywords/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .keywords(["hello", "world"])
    ///     .keyword_boost_legacy()
    ///     .build();
    /// ```
    pub fn keyword_boost_legacy(mut self) -> Self {
        self.0.keyword_boost_legacy = Some(true);
        self
    }

    /// Set the Utterances feature.
    ///
    /// To set the Utterance Split feature, use [`OptionsBuilder::utterances_with_utt_split`] instead.
    ///
    /// See the [Deepgram Utterances feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/utterances/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .utterances(true)
    ///     .build();
    /// ```
    pub fn utterances(mut self, utterances: bool) -> Self {
        self.0.utterances = Some(if utterances {
            Utterances::Enabled
        } else {
            Utterances::Disabled
        });

        self
    }

    /// Set the Utterances feature and the Utterance Split feature.
    ///
    /// If you do not want to set the Utterance Split feature, use [`OptionsBuilder::utterances`] instead.
    ///
    /// See the [Deepgram Utterances feature docs][utterances-docs]
    /// and the [Deepgram Utterance Split feature docs][utt_split-docs] for more info.
    ///
    /// [utterances-docs]: https://developers.deepgram.com/documentation/features/utterances/
    /// [utt_split-docs]: https://developers.deepgram.com/documentation/features/utterance-split/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .utterances_with_utt_split(0.9)
    ///     .build();
    /// ```
    pub fn utterances_with_utt_split(mut self, utt_split: f64) -> Self {
        self.0.utterances = Some(Utterances::CustomSplit {
            utt_split: Some(utt_split),
        });
        self
    }

    /// Set the Tag feature.
    ///
    /// Calling this when already set will append to the existing tags, not overwrite them.
    ///
    /// See the [Deepgram Tag feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/tag/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .tag(["Tag 1", "Tag 2"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .tag(["Tag 1"])
    ///     .tag(["Tag 2"])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .tag(["Tag 1", "Tag 2"])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn tag<'a>(mut self, tag: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.tags.extend(tag.into_iter().map(String::from));
        self
    }

    /// Set the Language Detection feature.
    ///
    /// See the [Deepgram Language Detection feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/language-detection/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{DetectLanguage, Options};
    /// #
    /// let options = Options::builder()
    ///     .detect_language(DetectLanguage::Enabled)
    ///     .build();
    /// ```
    pub fn detect_language(mut self, detect_language: DetectLanguage) -> Self {
        self.0.detect_language = Some(detect_language);

        self
    }

    /// Append extra query parameters to the end of the transcription request.
    /// Users should prefer using the other builder methods over this one. This
    /// exists as an escape hatch for using features before they have been added
    /// to the SDK.
    ///
    /// Calling this twice will add both sets of parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    ///
    /// use std::collections::HashMap;
    /// let mut params = HashMap::new(); // Could also be a Vec<(String, String)>
    /// params.insert("extra".to_string(), "parameter".to_string());
    /// let more_params = vec![("final".to_string(), "option".to_string())];
    /// let options = Options::builder()
    ///     .query_params(params)
    ///     .query_params(more_params)
    ///     .build();
    ///
    /// ```
    pub fn query_params(mut self, params: impl IntoIterator<Item = (String, String)>) -> Self {
        self.0.query_params.extend(params);
        self
    }

    /// Encoding is required when raw, headerless audio packets are sent to the
    /// streaming service. If containerized audio packets are sent to the
    /// streaming service, this feature should not be used.
    ///
    /// See the [Deepgram Encoding feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/encoding
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Options, Encoding};
    /// #
    /// let options = Options::builder()
    ///     .encoding(Encoding::Linear16)
    ///     .build();
    /// ```
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.0.encoding = Some(encoding);
        self
    }

    /// Set the Smart Format feature.
    ///
    /// See the [Deepgram Smart Formatting feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/smart-format
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .smart_format(true)
    ///     .build();
    /// ```
    pub fn smart_format(mut self, smart_format: bool) -> Self {
        self.0.smart_format = Some(smart_format);
        self
    }

    /// Set the Filler Words feature.
    ///
    /// See the [Deepgram Filler Words feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/filler-words
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .filler_words(true)
    ///     .build();
    /// ```
    pub fn filler_words(mut self, filler_words: bool) -> Self {
        self.0.filler_words = Some(filler_words);

        self
    }

    /// Set the Paragraphs feature.
    ///
    /// See the [Deepgram Paragraphs feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/paragraphs
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .paragraphs(true)
    ///     .build();
    /// ```
    pub fn paragraphs(mut self, paragraphs: bool) -> Self {
        self.0.paragraphs = Some(paragraphs);

        self
    }

    /// Set the Detect Entities feature.
    ///
    /// See the [Deepgram Detect Entities feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/detect-entities
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .detect_entities(true)
    ///     .build();
    /// ```
    pub fn detect_entities(mut self, detect_entities: bool) -> Self {
        self.0.detect_entities = Some(detect_entities);

        self
    }

    /// Set the Intent Recognition feature.
    ///
    /// See the [Deepgram Intent Recognition feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/intent-recognition
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .intents(true)
    ///     .build();
    /// ```
    pub fn intents(mut self, intents: bool) -> Self {
        self.0.intents = Some(intents);

        self
    }

    /// Set the Custom Intent Recognition Mode.
    ///
    /// See the [Deepgram Intent Recognition feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/intent-recognition#query-parameters
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Options, CustomIntentMode};
    /// #
    /// let options = Options::builder()
    ///     .custom_intent_mode(CustomIntentMode::Extended)
    ///     .build();
    /// ```
    pub fn custom_intent_mode(mut self, custom_intent_mode: CustomIntentMode) -> Self {
        self.0.custom_intent_mode = Some(custom_intent_mode);

        self
    }

    /// Set the Custom Intents feature.
    ///
    /// Not necessarily available for all languages.
    ///
    /// Calling this when already set will append to the existing custom intents, not overwrite them.
    ///
    /// See the [Deepgram Custom Intents feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/intent-recognition#query-parameters
    ///
    /// # Examples
    /// ```
    /// # use deepgram::common::options::{Options};
    /// #
    /// let options1 = Options::builder()
    ///     .custom_intents(["Intent 1"])
    ///     .custom_intents(["Intent 2"])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .custom_intents(["Intent 1", "Intent 2"])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn custom_intents(
        mut self,
        custom_intent: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.0
            .custom_intents
            .extend(custom_intent.into_iter().map(Into::into));
        self
    }

    /// Set the Sentiment Analysis feature.
    ///
    /// See the [Deepgram Sentiment Analysis feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/sentiment-analysis
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .sentiment(true)
    ///     .build();
    /// ```
    pub fn sentiment(mut self, sentiment: bool) -> Self {
        self.0.sentiment = Some(sentiment);

        self
    }

    /// Set the Topic Detection feature.
    ///
    /// See the [Deepgram Topic Detection feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/topic-detection
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .topics(true)
    ///     .build();
    /// ```
    pub fn topics(mut self, topics: bool) -> Self {
        self.0.topics = Some(topics);

        self
    }

    /// Set the Custom Topics feature.
    ///
    /// Not necessarily available for all languages.
    ///
    /// Calling this when already set will append to the existing custom topics, not overwrite them.
    ///
    /// See the [Deepgram Custom Topics feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/topic-detection#query-parameters
    ///
    /// # Examples
    /// ```
    /// # use deepgram::common::options::{Options};
    /// #
    /// let options1 = Options::builder()
    ///     .custom_topics(["Topic 1"])
    ///     .custom_topics(["Topic 2"])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .custom_topics(["Topic 1", "Topic 2"])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn custom_topics(
        mut self,
        custom_topic: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.0
            .custom_topics
            .extend(custom_topic.into_iter().map(Into::into));
        self
    }

    /// Set the Custom Topics Recognition Mode.
    ///
    /// See the [Deepgram Topics Recognition feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/topic-detection#query-parameters
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Options, CustomTopicMode};
    /// #
    /// let options = Options::builder()
    ///     .custom_topic_mode(CustomTopicMode::Extended)
    ///     .build();
    /// ```
    pub fn custom_topic_mode(mut self, custom_topic_mode: CustomTopicMode) -> Self {
        self.0.custom_topic_mode = Some(custom_topic_mode);

        self
    }

    /// Set the Summarize feature.
    ///
    /// See the [Deepgram Summarize feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/summarization
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .summarize(true)
    ///     .build();
    /// ```
    pub fn summarize(mut self, summarize: bool) -> Self {
        self.0.summarize = Some(summarize);
        self
    }

    /// Set the Language Detection feature.
    ///
    /// See the [Deepgram Language Detection feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/dictation
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .dictation(true)
    ///     .build();
    /// ```
    pub fn dictation(mut self, dictation: bool) -> Self {
        self.0.dictation = Some(dictation);

        self
    }

    /// Set the Measurements feature.
    ///
    /// See the [Deepgram Measurements feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/measurements
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .measurements(true)
    ///     .build();
    /// ```
    pub fn measurements(mut self, measurements: bool) -> Self {
        self.0.measurements = Some(measurements);

        self
    }

    /// Deepgrams Extra Metadata feature
    ///
    /// See the [Deepgram Extra Metadata feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/extra-metadata
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::Options;
    /// # use std::collections::HashMap;
    /// #
    /// let options = Options::builder()
    ///     .extra(HashMap::from([("key".to_string(), "value".to_string())]))
    ///     .build();
    /// ```
    pub fn extra(mut self, extra: HashMap<String, String>) -> Self {
        self.0.extra = Some(extra);
        self
    }

    /// Deepgrams Callback Method feature
    ///
    /// See the [Deepgram Callback Method feature docs][docs] for more info.
    ///
    /// Note that modifying the callback method is only available for pre-recorded audio.
    /// See the [Deepgram Callback feature docs for streaming][streaming-docs] for details
    /// on streaming callbacks.
    ///
    /// [docs]: https://developers.deepgram.com/docs/callback#pre-recorded-audio
    /// [streaming-docs]: https://developers.deepgram.com/docs/callback#streaming-audio
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::common::options::{Options, CallbackMethod};
    /// #
    /// let options = Options::builder()
    ///     .callback_method(CallbackMethod::PUT)
    ///     .build();
    /// ```
    pub fn callback_method(mut self, callback_method: CallbackMethod) -> Self {
        self.0.callback_method = Some(callback_method);
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

impl<'a> SerializableOptions<'a> {
    /// Used as a parameter for [`OptionsBuilder::keywords_with_intensifiers`].
    ///
    /// See the [Deepgram Keywords feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/keywords/
    pub fn from(options: &'a Options) -> Self {
        SerializableOptions(options)
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
            version,
            language,
            punctuate,
            profanity_filter,
            redact,
            diarize,
            diarize_version,
            ner,
            multichannel,
            alternatives,
            numerals,
            search,
            replace,
            keywords,
            keyword_boost_legacy,
            utterances,
            tags,
            detect_language,
            query_params,
            encoding,
            smart_format,
            filler_words,
            paragraphs,
            detect_entities,
            intents,
            custom_intent_mode,
            custom_intents,
            sentiment,
            topics,
            custom_topic_mode,
            custom_topics,
            summarize,
            dictation,
            measurements,
            extra,
            callback_method,
        } = self.0;

        match multichannel {
            // Multichannels with models is enabled
            // Ignore self.model field
            Some(Multichannel::ModelPerChannel {
                models: Some(models),
            }) => {
                seq.serialize_element(&("model", models_to_string(models)))?;
            }

            // Multichannel with models is not enabled
            // Use self.model field
            Some(
                Multichannel::ModelPerChannel { models: None }
                | Multichannel::Enabled
                | Multichannel::Disabled,
            )
            | None => {
                if let Some(model) = model {
                    seq.serialize_element(&("model", model.as_ref()))?;
                }
            }
        };

        if let Some(version) = version {
            seq.serialize_element(&("version", version))?;
        }

        if let Some(language) = language {
            seq.serialize_element(&("language", language.as_ref()))?;
        }

        if let Some(detect_language) = detect_language {
            for (_key, value) in detect_language.to_key_value_pairs() {
                seq.serialize_element(&("detect_language", value))?;
            }
        }

        if let Some(punctuate) = punctuate {
            seq.serialize_element(&("punctuate", punctuate))?;
        }

        if let Some(profanity_filter) = profanity_filter {
            seq.serialize_element(&("profanity_filter", profanity_filter))?;
        }

        for element in redact {
            seq.serialize_element(&("redact", element.as_ref()))?;
        }

        if let Some(diarize) = diarize {
            seq.serialize_element(&("diarize", diarize))?;
        }

        if let Some(diarize_version) = diarize_version {
            seq.serialize_element(&("diarize_version", diarize_version))?;
        }

        if let Some(ner) = ner {
            seq.serialize_element(&("ner", ner))?;
        }

        match multichannel {
            Some(Multichannel::Disabled) => seq.serialize_element(&("multichannel", false))?,
            Some(Multichannel::Enabled) => seq.serialize_element(&("multichannel", true))?,
            Some(Multichannel::ModelPerChannel { models: _ }) => {
                // Multichannel models are serialized above if they exist
                // This is done instead of serializing the self.model field
                seq.serialize_element(&("multichannel", true))?;
            }
            None => (),
        };

        if let Some(alternatives) = alternatives {
            seq.serialize_element(&("alternatives", alternatives))?;
        }

        if let Some(numerals) = numerals {
            seq.serialize_element(&("numerals", numerals))?;
        }

        for element in search {
            seq.serialize_element(&("search", element))?;
        }

        for element in replace {
            if let Some(replace) = &element.replace {
                seq.serialize_element(&("replace", format!("{}:{}", element.find, replace)))?;
            } else {
                seq.serialize_element(&("replace", &element.find))?;
            }
        }

        for element in keywords {
            if let Some(intensifier) = element.intensifier {
                seq.serialize_element(&(
                    "keywords",
                    format!("{}:{}", element.keyword, intensifier),
                ))?;
            } else {
                seq.serialize_element(&("keywords", &element.keyword))?;
            }
        }

        if keyword_boost_legacy.unwrap_or(false) {
            seq.serialize_element(&("keyword_boost", "legacy"))?;
        }

        match utterances {
            Some(Utterances::Disabled) => seq.serialize_element(&("utterances", false))?,
            Some(Utterances::Enabled) => seq.serialize_element(&("utterances", true))?,
            Some(Utterances::CustomSplit { utt_split }) => {
                seq.serialize_element(&("utterances", true))?;

                if let Some(utt_split) = utt_split {
                    seq.serialize_element(&("utt_split", utt_split))?;
                }
            }
            None => (),
        };

        for element in tags {
            seq.serialize_element(&("tag", element))?;
        }

        for (param, value) in query_params {
            seq.serialize_element(&(param, value))?;
        }

        if let Some(encoding) = encoding {
            seq.serialize_element(&("encoding", encoding.as_str()))?;
        }

        if let Some(smart_format) = smart_format {
            seq.serialize_element(&("smart_format", smart_format))?;
        }

        if let Some(filler_words) = filler_words {
            seq.serialize_element(&("filler_words", filler_words))?;
        }

        if let Some(paragraphs) = paragraphs {
            seq.serialize_element(&("paragraphs", paragraphs))?;
        }

        if let Some(detect_entities) = detect_entities {
            seq.serialize_element(&("detect_entities", detect_entities))?;
        }

        if let Some(intents) = intents {
            seq.serialize_element(&("intents", intents))?;
        }

        if let Some(custom_intent_mode) = custom_intent_mode {
            seq.serialize_element(&("custom_intent_mode", custom_intent_mode))?;
        }

        for custom_intent in custom_intents {
            seq.serialize_element(&("custom_intent", &custom_intent))?;
        }

        if let Some(sentiment) = sentiment {
            seq.serialize_element(&("sentiment", sentiment))?;
        }

        if let Some(topics) = topics {
            seq.serialize_element(&("topics", topics))?;
        }

        if let Some(custom_topic_mode) = custom_topic_mode {
            seq.serialize_element(&("custom_topic_mode", custom_topic_mode))?;
        }

        for custom_topic in custom_topics {
            seq.serialize_element(&("custom_topic", &custom_topic))?;
        }

        if let Some(summarize) = summarize {
            if *summarize {
                seq.serialize_element(&("summarize", "v2"))?;
            }
        }

        if let Some(dictation) = dictation {
            seq.serialize_element(&("dictation", dictation))?;
        }

        if let Some(measurements) = measurements {
            seq.serialize_element(&("measurements", measurements))?;
        }

        if let Some(extra) = extra {
            for (key, value) in extra.iter() {
                seq.serialize_element(&("extra", format!("{}:{}", key, value)))?;
            }
        }

        if let Some(callback_method) = callback_method {
            seq.serialize_element(&("callback_method", callback_method.as_str()))?;
        }

        seq.end()
    }
}

impl AsRef<str> for Model {
    fn as_ref(&self) -> &str {
        match self {
            Self::Nova2 => "nova-2",
            Self::Nova => "nova",
            Self::Enhanced => "enhanced",
            Self::Base => "base",
            Self::Nova2Meeting => "nova-2-meeting",
            Self::Nova2Phonecall => "nova-2-phonecall",
            Self::Nova2Finance => "nova-2-finance",
            Self::Nova2Conversationalai => "nova-2-conversationalai",
            Self::Nova2Voicemail => "nova-2-voicemail",
            Self::Nova2Video => "nova-2-video",
            Self::Nova2Medical => "nova-2-medical",
            Self::Nova2Drivethru => "nova-2-drivethru",
            Self::Nova2Automotive => "nova-2-automotive",
            Self::NovaPhonecall => "nova-phonecall",
            Self::NovaMedical => "nova-medical",
            Self::EnhancedMeeting => "enhanced-meeting",
            Self::EnhancedPhonecall => "enhanced-phonecall",
            Self::EnhancedFinance => "enhanced-finance",
            Self::BaseMeeting => "base-meeting",
            Self::BasePhonecall => "base-phonecall",
            Self::BaseVoicemail => "base-voicemail",
            Self::BaseFinance => "base-finance",
            Self::BaseConversationalai => "base-conversationalai",
            Self::BaseVideo => "base-video",
            #[allow(deprecated)]
            Self::General => "general",
            #[allow(deprecated)]
            Self::Phonecall => "phonecall",
            #[allow(deprecated)]
            Self::Voicemail => "voicemail",
            #[allow(deprecated)]
            Self::Finance => "finance",
            #[allow(deprecated)]
            Self::Meeting => "meeting",
            #[allow(deprecated)]
            Self::Conversationalai => "conversationalai",
            #[allow(deprecated)]
            Self::Video => "video",
            Self::CustomId(id) => id,
        }
    }
}

impl From<String> for Model {
    fn from(value: String) -> Self {
        match &*value {
            "nova-2" | "nova-2-general" => Self::Nova2,
            "nova" | "nova-general" => Self::Nova,
            "enhanced" | "enhanced-general" => Self::Enhanced,
            "base" | "base-general" => Self::Base,
            "nova-2-meeting" => Self::Nova2Meeting,
            "nova-2-phonecall" => Self::Nova2Phonecall,
            "nova-2-finance" => Self::Nova2Finance,
            "nova-2-conversationalai" => Self::Nova2Conversationalai,
            "nova-2-voicemail" => Self::Nova2Voicemail,
            "nova-2-video" => Self::Nova2Video,
            "nova-2-medical" => Self::Nova2Medical,
            "nova-2-drivethru" => Self::Nova2Drivethru,
            "nova-2-automotive" => Self::Nova2Automotive,
            "nova-phonecall" => Self::NovaPhonecall,
            "nova-medical" => Self::NovaMedical,
            "enhanced-meeting" => Self::EnhancedMeeting,
            "enhanced-phonecall" => Self::EnhancedPhonecall,
            "enhanced-finance" => Self::EnhancedFinance,
            "base-meeting" => Self::BaseMeeting,
            "base-phonecall" => Self::BasePhonecall,
            "base-voicemail" => Self::BaseVoicemail,
            "base-finance" => Self::BaseFinance,
            "base-conversationalai" => Self::BaseConversationalai,
            "base-video" => Self::BaseVideo,
            #[allow(deprecated)]
            "general" => Self::General,
            #[allow(deprecated)]
            "phonecall" => Self::Phonecall,
            #[allow(deprecated)]
            "voicemail" => Self::Voicemail,
            #[allow(deprecated)]
            "finance" => Self::Finance,
            #[allow(deprecated)]
            "meeting" => Self::Meeting,
            #[allow(deprecated)]
            "conversationalai" => Self::Conversationalai,
            #[allow(deprecated)]
            "video" => Self::Video,
            _ => Self::CustomId(value),
        }
    }
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        match self {
            Self::bg => "bg",
            Self::ca => "ca",
            Self::cs => "cs",
            Self::da => "da",
            Self::de => "de",
            Self::de_CH => "de-CH",
            Self::el => "el",
            Self::en => "en",
            Self::en_AU => "en-AU",
            Self::en_GB => "en-GB",
            Self::en_IN => "en-IN",
            Self::en_NZ => "en-NZ",
            Self::en_US => "en-US",
            Self::es => "es",
            Self::es_419 => "es-419",
            Self::es_LATAM => "es-LATAM",
            Self::et => "et",
            Self::fi => "fi",
            Self::fr => "fr",
            Self::fr_CA => "fr-CA",
            Self::hi => "hi",
            Self::hi_Latn => "hi-Latn",
            Self::hu => "hu",
            Self::id => "id",
            Self::it => "it",
            Self::ja => "ja",
            Self::ko => "ko",
            Self::ko_KR => "ko-KR",
            Self::lv => "lv",
            Self::lt => "lt",
            Self::ms => "ms",
            Self::nl => "nl",
            Self::nl_BE => "nl-BE",
            Self::no => "no",
            Self::pl => "pl",
            Self::pt => "pt",
            Self::pt_BR => "pt-BR",
            Self::ro => "ro",
            Self::ru => "ru",
            Self::sk => "sk",
            Self::sv => "sv",
            Self::sv_SE => "sv-SE",
            Self::ta => "ta",
            Self::taq => "taq",
            Self::th => "th",
            Self::th_TH => "th-TH",
            Self::tr => "tr",
            Self::uk => "uk",
            Self::vi => "vi",
            Self::zh => "zh",
            Self::zh_CN => "zh-CN",
            Self::zh_Hans => "zh-Hans",
            Self::zh_Hant => "zh-Hant",
            Self::zh_TW => "zh-TW",
            Self::Other(bcp_47_tag) => bcp_47_tag,
        }
    }
}

impl From<String> for Language {
    fn from(value: String) -> Language {
        match &*value {
            "bg" => Self::bg,
            "ca" => Self::ca,
            "cs" => Self::cs,
            "da" => Self::da,
            "de" => Self::de,
            "de-CH" => Self::de_CH,
            "el" => Self::el,
            "en" => Self::en,
            "en-AU" => Self::en_AU,
            "en-GB" => Self::en_GB,
            "en-IN" => Self::en_IN,
            "en-NZ" => Self::en_NZ,
            "en-US" => Self::en_US,
            "es" => Self::es,
            "es-419" => Self::es_419,
            "es-LATAM" => Self::es_LATAM,
            "et" => Self::et,
            "fi" => Self::fi,
            "fr" => Self::fr,
            "fr-CA" => Self::fr_CA,
            "hi" => Self::hi,
            "hi-Latn" => Self::hi_Latn,
            "hu" => Self::hu,
            "id" => Self::id,
            "it" => Self::it,
            "ja" => Self::ja,
            "ko" => Self::ko,
            "ko-KR" => Self::ko_KR,
            "lv" => Self::lv,
            "lt" => Self::lt,
            "ms" => Self::ms,
            "nl" => Self::nl,
            "nl-BE" => Self::nl_BE,
            "no" => Self::no,
            "pl" => Self::pl,
            "pt" => Self::pt,
            "pt-BR" => Self::pt_BR,
            "ro" => Self::ro,
            "ru" => Self::ru,
            "sk" => Self::sk,
            "sv" => Self::sv,
            "sv-SE" => Self::sv_SE,
            "ta" => Self::ta,
            "taq" => Self::taq,
            "th" => Self::th,
            "th-TH" => Self::th_TH,
            "tr" => Self::tr,
            "uk" => Self::uk,
            "vi" => Self::vi,
            "zh" => Self::zh,
            "zh-CN" => Self::zh_CN,
            "zh-Hans" => Self::zh_Hans,
            "zh-Hant" => Self::zh_Hant,
            "zh-TW" => Self::zh_TW,
            _ => Self::Other(value),
        }
    }
}

impl AsRef<str> for Redact {
    fn as_ref(&self) -> &str {
        match self {
            Redact::Pci => "pci",
            Redact::Numbers => "numbers",
            Redact::Ssn => "ssn",
            Redact::Other(id) => id,
        }
    }
}

impl From<String> for Redact {
    fn from(value: String) -> Redact {
        match &*value {
            "pci" => Redact::Pci,
            "numbers" => Redact::Numbers,
            "ssn" => Redact::Ssn,
            _ => Redact::Other(value),
        }
    }
}

fn models_to_string(models: &[Model]) -> String {
    models
        .iter()
        .map(AsRef::<str>::as_ref)
        .collect::<Vec<&str>>()
        .join(":")
}

#[cfg(test)]
mod from_string_tests {
    use super::{Language, Model, Redact};

    #[test]
    fn model_from_string() {
        assert_eq!(Model::from("nova-2".to_string()), Model::Nova2);
        assert_eq!(
            Model::from("custom".to_string()),
            Model::CustomId("custom".to_string())
        );
        assert_eq!(Model::from("".to_string()), Model::CustomId("".to_string()));
    }

    #[test]
    fn language_from_string() {
        assert_eq!(Language::from("zh-Hant".to_string()), Language::zh_Hant);
        assert_eq!(
            Language::from("custom".to_string()),
            Language::Other("custom".to_string())
        );
        assert_eq!(
            Language::from("".to_string()),
            Language::Other("".to_string())
        );
    }

    #[test]
    fn redact_from_string() {
        assert_eq!(Redact::from("pci".to_string()), Redact::Pci);
        assert_eq!(
            Redact::from("custom".to_string()),
            Redact::Other("custom".to_string())
        );
        assert_eq!(Redact::from("".to_string()), Redact::Other("".to_string()));
    }
}
#[cfg(test)]
mod models_to_string_tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(models_to_string(&[]), "");
    }

    #[test]
    fn one() {
        assert_eq!(models_to_string(&[Model::Base]), "base");
    }

    #[test]
    fn many() {
        assert_eq!(
            models_to_string(&[
                Model::BasePhonecall,
                Model::BaseMeeting,
                Model::BaseVoicemail
            ]),
            "base-phonecall:base-meeting:base-voicemail"
        );
    }

    #[test]
    fn custom() {
        assert_eq!(
            models_to_string(&[
                Model::EnhancedFinance,
                Model::CustomId(String::from("extra_crispy")),
                Model::Nova2Conversationalai,
            ]),
            "enhanced-finance:extra_crispy:nova-2-conversationalai"
        );
    }
}

#[cfg(test)]
mod serialize_options_tests {
    use std::cmp;
    use std::collections::HashMap;
    use std::env;

    use crate::common::audio_source::AudioSource;
    use crate::Deepgram;

    use super::CallbackMethod;
    use super::CustomIntentMode;
    use super::CustomTopicMode;
    use super::DetectLanguage;
    use super::Encoding;
    use super::Keyword;
    use super::Language;
    use super::Model;
    use super::Options;
    use super::Redact;
    use super::Replace;

    fn check_serialization(options: &Options, expected: &str) {
        let deepgram_api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();

        let dg_client = Deepgram::new(deepgram_api_key).unwrap();

        let request = dg_client
            .transcription()
            .make_prerecorded_request_builder(AudioSource::from_url(""), options)
            .build()
            .unwrap();

        let actual = request.url().query().unwrap_or("");

        assert_eq!(actual, expected);
    }

    fn generate_alphabet_test(key: &str, length: usize) -> (Vec<&str>, String) {
        let letters = [
            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q",
            "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        ];

        let limited_letters = Vec::from(&letters[..cmp::min(length, letters.len())]);

        let expected = limited_letters
            .iter()
            .map(|letter| format!("{}={}", key, letter))
            .collect::<Vec<String>>()
            .join("&");

        (limited_letters, expected)
    }

    #[test]
    fn all_options() {
        let options = Options::builder()
            .model(Model::Base)
            .version("1.2.3")
            .language(Language::en)
            .detect_language(DetectLanguage::Restricted(vec![Language::en, Language::es]))
            .punctuate(true)
            .profanity_filter(true)
            .redact([Redact::Pci, Redact::Ssn])
            .diarize(true)
            .diarize_version("2021-07-14.0")
            .ner(true)
            .multichannel_with_models([
                Model::EnhancedFinance,
                Model::CustomId(String::from("extra_crispy")),
                Model::Nova2Conversationalai,
            ])
            .alternatives(4)
            .numerals(true)
            .search(["Rust", "Deepgram"])
            .replace([Replace {
                find: String::from("Aaron"),
                replace: Some(String::from("Erin")),
            }])
            .keywords(["Ferris"])
            .keywords_with_intensifiers([Keyword {
                keyword: String::from("Cargo"),
                intensifier: Some(-1.5),
            }])
            .utterances_with_utt_split(0.9)
            .tag(["Tag 1"])
            .encoding(Encoding::Linear16)
            .smart_format(true)
            .filler_words(true)
            .paragraphs(true)
            .detect_entities(true)
            .intents(true)
            .custom_intent_mode(CustomIntentMode::Extended)
            .custom_intents(["Phone repair", "Phone cancellation"])
            .sentiment(true)
            .topics(true)
            .custom_topic_mode(CustomTopicMode::Strict)
            .custom_topics(["Get support", "Complain"])
            .summarize(true)
            .dictation(true)
            .measurements(true)
            .extra(HashMap::from([("key".to_string(), "value".to_string())]))
            .callback_method(CallbackMethod::PUT)
            .build();

        check_serialization(&options, "model=enhanced-finance%3Aextra_crispy%3Anova-2-conversationalai&version=1.2.3&language=en&detect_language=en&detect_language=es&punctuate=true&profanity_filter=true&redact=pci&redact=ssn&diarize=true&diarize_version=2021-07-14.0&ner=true&multichannel=true&alternatives=4&numerals=true&search=Rust&search=Deepgram&replace=Aaron%3AErin&keywords=Ferris&keywords=Cargo%3A-1.5&utterances=true&utt_split=0.9&tag=Tag+1&encoding=linear16&smart_format=true&filler_words=true&paragraphs=true&detect_entities=true&intents=true&custom_intent_mode=extended&custom_intent=Phone+repair&custom_intent=Phone+cancellation&sentiment=true&topics=true&custom_topic_mode=strict&custom_topic=Get+support&custom_topic=Complain&summarize=v2&dictation=true&measurements=true&extra=key%3Avalue&callback_method=put");
    }

    #[test]
    fn model() {
        check_serialization(&Options::builder().model(Model::Base).build(), "model=base");

        check_serialization(
            &Options::builder()
                .model(Model::CustomId(String::from("extra_crispy")))
                .build(),
            "model=extra_crispy",
        );
    }

    #[test]
    fn version() {
        check_serialization(
            &Options::builder().version("1.2.3").build(),
            "version=1.2.3",
        );
    }

    #[test]
    fn language() {
        check_serialization(
            &Options::builder().language(Language::en_US).build(),
            "language=en-US",
        );

        check_serialization(
            &Options::builder().language(Language::ja).build(),
            "language=ja",
        );
    }

    #[test]
    fn punctuate() {
        check_serialization(
            &Options::builder().punctuate(true).build(),
            "punctuate=true",
        );

        check_serialization(
            &Options::builder().punctuate(false).build(),
            "punctuate=false",
        );
    }

    #[test]
    fn profanity_filter() {
        check_serialization(
            &Options::builder().profanity_filter(true).build(),
            "profanity_filter=true",
        );

        check_serialization(
            &Options::builder().profanity_filter(false).build(),
            "profanity_filter=false",
        );
    }

    #[test]
    fn redact() {
        check_serialization(&Options::builder().redact([]).build(), "");

        check_serialization(
            &Options::builder().redact([Redact::Numbers]).build(),
            "redact=numbers",
        );

        check_serialization(
            &Options::builder()
                .redact([Redact::Ssn, Redact::Pci])
                .build(),
            "redact=ssn&redact=pci",
        );

        check_serialization(
            &Options::builder()
                .redact([
                    Redact::Numbers,
                    Redact::Ssn,
                    Redact::Pci,
                    Redact::Ssn,
                    Redact::Numbers,
                    Redact::Pci,
                ])
                .build(),
            "redact=numbers&redact=ssn&redact=pci&redact=ssn&redact=numbers&redact=pci",
        );
    }

    #[test]
    fn diarize() {
        check_serialization(&Options::builder().diarize(true).build(), "diarize=true");

        check_serialization(&Options::builder().diarize(false).build(), "diarize=false");
    }

    #[test]
    fn ner() {
        check_serialization(&Options::builder().ner(true).build(), "ner=true");

        check_serialization(&Options::builder().ner(false).build(), "ner=false");
    }

    #[test]
    fn multichannel() {
        check_serialization(
            &Options::builder().multichannel(true).build(),
            "multichannel=true",
        );

        check_serialization(
            &Options::builder().multichannel(false).build(),
            "multichannel=false",
        );

        check_serialization(
            &Options::builder()
                .multichannel_with_models([
                    Model::EnhancedFinance,
                    Model::CustomId(String::from("extra_crispy")),
                    Model::Nova2Conversationalai,
                ])
                .build(),
            "model=enhanced-finance%3Aextra_crispy%3Anova-2-conversationalai&multichannel=true",
        );
    }

    #[test]
    fn alternatives() {
        check_serialization(
            &Options::builder().alternatives(4).build(),
            "alternatives=4",
        );
    }

    #[test]
    fn numerals() {
        check_serialization(&Options::builder().numerals(true).build(), "numerals=true");

        check_serialization(
            &Options::builder().numerals(false).build(),
            "numerals=false",
        );
    }

    #[test]
    fn search() {
        check_serialization(&Options::builder().search([]).build(), "");

        check_serialization(&Options::builder().search(["Rust"]).build(), "search=Rust");

        check_serialization(
            &Options::builder().search(["Rust", "Deepgram"]).build(),
            "search=Rust&search=Deepgram",
        );

        {
            let (input, expected) = generate_alphabet_test("search", 25);
            check_serialization(&Options::builder().search(input).build(), &expected);
        }
    }

    #[test]
    fn replace() {
        check_serialization(&Options::builder().replace([]).build(), "");

        check_serialization(
            &Options::builder()
                .replace([Replace {
                    find: String::from("Aaron"),
                    replace: Some(String::from("Erin")),
                }])
                .build(),
            "replace=Aaron%3AErin",
        );

        check_serialization(
            &Options::builder()
                .replace([Replace {
                    find: String::from("Voldemort"),
                    replace: None,
                }])
                .build(),
            "replace=Voldemort",
        );

        check_serialization(
            &Options::builder()
                .replace([
                    Replace {
                        find: String::from("Aaron"),
                        replace: Some(String::from("Erin")),
                    },
                    Replace {
                        find: String::from("Voldemort"),
                        replace: None,
                    },
                ])
                .build(),
            "replace=Aaron%3AErin&replace=Voldemort",
        );

        check_serialization(
            &Options::builder()
                .replace([Replace {
                    find: String::from("this too"),
                    replace: Some(String::from("that too")),
                }])
                .build(),
            "replace=this+too%3Athat+too",
        );
    }

    #[test]
    fn keywords() {
        check_serialization(&Options::builder().keywords([]).build(), "");

        check_serialization(
            &Options::builder().keywords(["Ferris"]).build(),
            "keywords=Ferris",
        );

        check_serialization(
            &Options::builder().keywords(["Ferris", "Cargo"]).build(),
            "keywords=Ferris&keywords=Cargo",
        );

        {
            let (input, expected) = generate_alphabet_test("keywords", 200);

            check_serialization(&Options::builder().keywords(input).build(), &expected);
        }

        {
            let keywords = [Keyword {
                keyword: String::from("Ferris"),
                intensifier: Some(0.5),
            }];

            check_serialization(
                &Options::builder()
                    .keywords_with_intensifiers(keywords)
                    .build(),
                "keywords=Ferris%3A0.5",
            );
        }

        {
            let keywords = [
                Keyword {
                    keyword: String::from("Ferris"),
                    intensifier: Some(0.5),
                },
                Keyword {
                    keyword: String::from("Cargo"),
                    intensifier: Some(-1.5),
                },
            ];

            check_serialization(
                &Options::builder()
                    .keywords_with_intensifiers(keywords)
                    .build(),
                "keywords=Ferris%3A0.5&keywords=Cargo%3A-1.5",
            );
        }

        check_serialization(
            &Options::builder()
                .keywords(["Ferris"])
                .keyword_boost_legacy()
                .build(),
            "keywords=Ferris&keyword_boost=legacy",
        );
    }

    #[test]
    fn utterances() {
        check_serialization(
            &Options::builder().utterances(false).build(),
            "utterances=false",
        );

        check_serialization(
            &Options::builder().utterances(true).build(),
            "utterances=true",
        );

        check_serialization(
            &Options::builder().utterances_with_utt_split(0.9).build(),
            "utterances=true&utt_split=0.9",
        );
    }

    #[test]
    fn tag() {
        check_serialization(&Options::builder().tag(["Tag 1"]).build(), "tag=Tag+1");

        check_serialization(
            &Options::builder().tag(["Tag 1", "Tag 2"]).build(),
            "tag=Tag+1&tag=Tag+2",
        );
    }

    #[test]
    fn detect_language() {
        check_serialization(
            &Options::builder()
                .detect_language(DetectLanguage::Disabled)
                .build(),
            "detect_language=false",
        );

        check_serialization(
            &Options::builder()
                .detect_language(DetectLanguage::Enabled)
                .build(),
            "detect_language=true",
        );
    }

    #[test]
    fn encoding() {
        check_serialization(
            &Options::builder().encoding(Encoding::Linear16).build(),
            "encoding=linear16",
        );
    }

    #[test]
    fn smart_format() {
        check_serialization(
            &Options::builder().smart_format(false).build(),
            "smart_format=false",
        );

        check_serialization(
            &Options::builder().smart_format(true).build(),
            "smart_format=true",
        );
    }

    #[test]
    fn filler_words() {
        check_serialization(
            &Options::builder().filler_words(false).build(),
            "filler_words=false",
        );

        check_serialization(
            &Options::builder().filler_words(true).build(),
            "filler_words=true",
        );
    }

    #[test]
    fn paragraphs() {
        check_serialization(
            &Options::builder().paragraphs(false).build(),
            "paragraphs=false",
        );

        check_serialization(
            &Options::builder().paragraphs(true).build(),
            "paragraphs=true",
        );
    }
}
