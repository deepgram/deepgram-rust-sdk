//! Set various Deepgram features to control how the audio is transcribed.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/documentation/features/

use serde::{ser::SerializeSeq, Serialize};

/// Used as a parameter for [`Transcription::prerecorded`](crate::transcription::Transcription::prerecorded) and similar functions.
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    tier: Option<Tier>,
    model: Option<Model>,
    version: Option<String>,
    language: Option<Language>,
    punctuate: Option<bool>,
    profanity_filter: Option<bool>,
    redact: Vec<Redact>,
    diarize: Option<bool>,
    ner: Option<bool>,
    multichannel: Option<Multichannel>,
    alternatives: Option<usize>,
    numerals: Option<bool>,
    search: Vec<String>,
    replace: Vec<Replace>,
    keywords: Vec<Keyword>,
    filler_words: bool,
    smart_format: bool,
    paragraphs: Option<bool>,
    keyword_boost_legacy: bool,
    utterances: Option<Utterances>,
    tags: Vec<String>,
    detect_language: Option<bool>,
}

/// Used as a parameter for [`OptionsBuilder::tier`].
///
/// See the [Deepgram Tier feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/tier/
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Tier {
    #[allow(missing_docs)]
    Enhanced,

    #[allow(missing_docs)]
    Base,
}

/// Used as a parameter for [`OptionsBuilder::model`] and [`OptionsBuilder::multichannel_with_models`].
///
/// See the [Deepgram Model feature docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/documentation/features/model/
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Model {
    #[allow(missing_docs)]
    General,

    #[allow(missing_docs)]
    Meeting,

    #[allow(missing_docs)]
    Phonecall,

    #[allow(missing_docs)]
    Voicemail,

    #[allow(missing_docs)]
    Finance,

    #[allow(missing_docs)]
    Conversationalai,

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
    zh,

    #[allow(missing_docs)]
    zh_CN,

    #[allow(missing_docs)]
    zh_TW,

    #[allow(missing_docs)]
    nl,

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
    fr,

    #[allow(missing_docs)]
    fr_CA,

    #[allow(missing_docs)]
    de,

    #[allow(missing_docs)]
    hi,

    #[allow(missing_docs)]
    hi_Latn,

    #[allow(missing_docs)]
    id,

    #[allow(missing_docs)]
    it,

    #[allow(missing_docs)]
    ja,

    #[allow(missing_docs)]
    ko,

    #[allow(missing_docs)]
    pt,

    #[allow(missing_docs)]
    pt_BR,

    #[allow(missing_docs)]
    ru,

    #[allow(missing_docs)]
    es,

    #[allow(missing_docs)]
    es_419,

    #[allow(missing_docs)]
    sv,

    #[allow(missing_docs)]
    tr,

    #[allow(missing_docs)]
    uk,

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

#[derive(Debug, PartialEq, Clone, Copy)]
enum Utterances {
    Disabled,
    Enabled { utt_split: Option<f64> },
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
enum Multichannel {
    Disabled,
    Enabled { models: Option<Vec<Model>> },
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// Use it to set of Deepgram's features, excluding the Callback feature.
/// The Callback feature can be set when making the request by calling [`Transcription::prerecorded_callback`](crate::transcription::Transcription::prerecorded_callback).
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
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new() -> Self {
        Self(Options {
            tier: None,
            model: None,
            version: None,
            language: None,
            punctuate: None,
            profanity_filter: None,
            redact: Vec::new(),
            diarize: None,
            ner: None,
            multichannel: None,
            alternatives: None,
            numerals: None,
            search: Vec::new(),
            replace: Vec::new(),
            keywords: Vec::new(),
            filler_words: false,
            smart_format: false,
            paragraphs: None,
            keyword_boost_legacy: false,
            utterances: None,
            tags: Vec::new(),
            detect_language: None,
        })
    }

    /// Set the Tier feature.
    ///
    /// Not all tiers are supported for all models and languages.
    /// For a list of models/languages and their supported models,
    /// see the [Deepgram Language feature][language] docs.
    ///
    /// See the [Deepgram Tier feature docs][docs] for more info.
    ///
    /// [language]: https://developers.deepgram.com/documentation/features/language/
    /// [docs]: https://developers.deepgram.com/documentation/features/tier/
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::{Options, Tier};
    /// #
    /// let options = Options::builder()
    ///     .tier(Tier::Enhanced)
    ///     .build();
    /// ```
    pub fn tier(mut self, tier: Tier) -> Self {
        self.0.tier = Some(tier);
        self
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
    /// # use deepgram::transcription::prerecorded::options::{Model, Options};
    /// #
    /// let options = Options::builder()
    ///     .model(Model::General)
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::{Model, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .multichannel_with_models([Model::Meeting, Model::Phonecall])
    ///     .model(Model::General)
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .multichannel(true)
    ///     .model(Model::General)
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn model(mut self, model: Model) -> Self {
        self.0.model = Some(model);

        if let Some(Multichannel::Enabled { models }) = &mut self.0.multichannel {
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
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::{Language, Options};
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
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::{Options, Redact};
    /// #
    /// let options = Options::builder()
    ///     .redact([Redact::Pci, Redact::Ssn])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::{Options, Redact};
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .diarize(true)
    ///     .build();
    /// ```
    pub fn diarize(mut self, diarize: bool) -> Self {
        self.0.diarize = Some(diarize);
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
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .multichannel(true)
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::{Model, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .model(Model::General)
    ///     .multichannel_with_models([Model::Meeting, Model::Phonecall])
    ///     .multichannel(true)
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .model(Model::General)
    ///     .multichannel(true)
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn multichannel(mut self, multichannel: bool) -> Self {
        self.0.multichannel = Some(if multichannel {
            Multichannel::Enabled { models: None }
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
    /// # use deepgram::transcription::prerecorded::options::{Model, Options};
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
    /// #     transcription::prerecorded::{
    /// #         audio_source::AudioSource,
    /// #         options::{Model, Options},
    /// #     },
    /// #     Deepgram,
    /// # };
    /// #
    /// #
    /// # static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";
    /// #
    /// # fn main() -> Result<(), reqwest::Error> {
    /// # let deepgram_api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key);
    /// let dg_transcription = dg_client.transcription();
    ///
    /// let options1 = Options::builder()
    ///     .model(Model::General)
    ///     .multichannel_with_models([Model::Meeting, Model::Phonecall])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .multichannel_with_models([Model::Meeting, Model::Phonecall])
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
    /// # use deepgram::transcription::prerecorded::options::{Model, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .model(Model::General)
    ///     .multichannel_with_models([Model::Meeting, Model::Phonecall])
    ///     .multichannel(true)
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .model(Model::General)
    ///     .multichannel(true)
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::{Model, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .multichannel_with_models([Model::Meeting])
    ///     .multichannel_with_models([Model::Phonecall])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .multichannel_with_models([Model::Meeting, Model::Phonecall])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn multichannel_with_models(mut self, models: impl IntoIterator<Item = Model>) -> Self {
        if let Some(Multichannel::Enabled {
            models: Some(old_models),
        }) = &mut self.0.multichannel
        {
            // Multichannel with models already enabled
            // Don't overwrite existing models
            old_models.extend(models);
        } else {
            // Multichannel with models already enabled
            self.0.multichannel = Some(Multichannel::Enabled {
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
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .search(["hello", "world"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::{Options, Replace};
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
    /// # use deepgram::transcription::prerecorded::options::{Options, Replace};
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .keywords(["hello", "world"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::{Keyword, Options};
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
    /// # use deepgram::transcription::prerecorded::options::{Keyword, Options};
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .keywords(["hello", "world"])
    ///     .keyword_boost_legacy()
    ///     .build();
    /// ```
    pub fn keyword_boost_legacy(mut self) -> Self {
        self.0.keyword_boost_legacy = true;
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .utterances(true)
    ///     .build();
    /// ```
    pub fn utterances(mut self, utterances: bool) -> Self {
        self.0.utterances = Some(if utterances {
            Utterances::Enabled { utt_split: None }
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .utterances_with_utt_split(0.9)
    ///     .build();
    /// ```
    pub fn utterances_with_utt_split(mut self, utt_split: f64) -> Self {
        self.0.utterances = Some(Utterances::Enabled {
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .tag(["Tag 1", "Tag 2"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::transcription::prerecorded::options::Options;
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
    /// # use deepgram::transcription::prerecorded::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .detect_language(true)
    ///     .build();
    /// ```
    pub fn detect_language(mut self, detect_language: bool) -> Self {
        self.0.detect_language = Some(detect_language);

        self
    }

    #[allow(missing_docs)]
    pub fn smart_format(mut self, smart_format: bool) -> Self {
        self.0.smart_format = smart_format;

        self
    }

    #[allow(missing_docs)]
    pub fn filler_words(mut self, filler_words: bool) -> Self {
        self.0.filler_words = filler_words;

        self
    }

    #[allow(missing_docs)]
    pub fn paragraphs(mut self, paragraphs: bool) -> Self {
        self.0.paragraphs = Some(paragraphs);

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
            tier,
            model,
            version,
            language,
            punctuate,
            profanity_filter,
            redact,
            diarize,
            ner,
            multichannel,
            alternatives,
            numerals,
            search,
            replace,
            keywords,
            filler_words,
            smart_format,
            paragraphs,
            keyword_boost_legacy,
            utterances,
            tags,
            detect_language,
        } = self.0;

        if let Some(tier) = tier {
            seq.serialize_element(&("tier", tier.as_ref()))?;
        }

        match multichannel {
            // Multichannels with models is enabled
            // Ignore self.model field
            Some(Multichannel::Enabled {
                models: Some(models),
            }) => {
                seq.serialize_element(&("model", models_to_string(models)))?;
            }

            // Multichannel with models is not enabled
            // Use self.model field
            Some(Multichannel::Enabled { models: None } | Multichannel::Disabled) | None => {
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

        if let Some(ner) = ner {
            seq.serialize_element(&("ner", ner))?;
        }

        match multichannel {
            Some(Multichannel::Disabled) => seq.serialize_element(&("multichannel", false))?,
            Some(Multichannel::Enabled { models: _ }) => {
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

        if *keyword_boost_legacy {
            seq.serialize_element(&("keyword_boost", "legacy"))?;
        }

        seq.serialize_element(&("filler_words", filler_words))?;
        seq.serialize_element(&("smart_format", smart_format))?;

        if let Some(paragraphs) = paragraphs {
            seq.serialize_element(&("paragraphs", paragraphs))?;
        }

        match utterances {
            Some(Utterances::Disabled) => seq.serialize_element(&("utterances", false))?,
            Some(Utterances::Enabled { utt_split }) => {
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

        if let Some(detect_language) = detect_language {
            seq.serialize_element(&("detect_language", detect_language))?;
        }

        seq.end()
    }
}

impl AsRef<str> for Tier {
    fn as_ref(&self) -> &str {
        use Tier::*;

        match self {
            Enhanced => "enhanced",
            Base => "base",
        }
    }
}

impl AsRef<str> for Model {
    fn as_ref(&self) -> &str {
        use Model::*;

        match self {
            General => "general",
            Meeting => "meeting",
            Phonecall => "phonecall",
            Voicemail => "voicemail",
            Finance => "finance",
            Conversationalai => "conversationalai",
            Video => "video",
            CustomId(id) => id,
        }
    }
}

impl AsRef<str> for Language {
    fn as_ref(&self) -> &str {
        use Language::*;

        match self {
            zh => "zh",
            zh_CN => "zh-CN",
            zh_TW => "zh-TW",
            nl => "nl",
            en => "en",
            en_AU => "en-AU",
            en_GB => "en-GB",
            en_IN => "en-IN",
            en_NZ => "en-NZ",
            en_US => "en-US",
            fr => "fr",
            fr_CA => "fr-CA",
            de => "de",
            hi => "hi",
            hi_Latn => "hi-Latn",
            id => "id",
            it => "it",
            ja => "ja",
            ko => "ko",
            pt => "pt",
            pt_BR => "pt-BR",
            ru => "ru",
            es => "es",
            es_419 => "es-419",
            sv => "sv",
            tr => "tr",
            uk => "uk",
            Other(bcp_47_tag) => bcp_47_tag,
        }
    }
}

impl AsRef<str> for Redact {
    fn as_ref(&self) -> &str {
        use Redact::*;

        match self {
            Pci => "pci",
            Numbers => "numbers",
            Ssn => "ssn",
            Other(id) => id,
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
mod models_to_string_tests {
    use super::{Model::*, *};

    #[test]
    fn empty() {
        assert_eq!(models_to_string(&[]), "");
    }

    #[test]
    fn one() {
        assert_eq!(models_to_string(&[General]), "general");
    }

    #[test]
    fn many() {
        assert_eq!(
            models_to_string(&[Phonecall, Meeting, Voicemail]),
            "phonecall:meeting:voicemail"
        );
    }

    #[test]
    fn custom() {
        assert_eq!(
            models_to_string(&[
                Finance,
                CustomId(String::from("extra_crispy")),
                Conversationalai
            ]),
            "finance:extra_crispy:conversationalai"
        );
    }
}

#[cfg(test)]
mod serialize_options_tests {
    use std::cmp;
    use std::env;

    use super::{super::audio_source::AudioSource, *};
    use crate::Deepgram;

    fn check_serialization(options: &Options, expected: &str) {
        let deepgram_api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();

        let dg_client = Deepgram::new(deepgram_api_key);

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
            .tier(Tier::Enhanced)
            .model(Model::General)
            .version("1.2.3")
            .language(Language::en_US)
            .punctuate(true)
            .profanity_filter(true)
            .redact([Redact::Pci, Redact::Ssn])
            .diarize(true)
            .ner(true)
            .multichannel_with_models([
                Model::Finance,
                Model::CustomId(String::from("extra_crispy")),
                Model::Conversationalai,
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
            .build();

        check_serialization(&options, "tier=enhanced&model=finance%3Aextra_crispy%3Aconversationalai&version=1.2.3&language=en-US&punctuate=true&profanity_filter=true&redact=pci&redact=ssn&diarize=true&ner=true&multichannel=true&alternatives=4&numerals=true&search=Rust&search=Deepgram&replace=Aaron%3AErin&keywords=Ferris&keywords=Cargo%3A-1.5&utterances=true&utt_split=0.9&tag=Tag+1");
    }

    #[test]
    fn tier() {
        check_serialization(
            &Options::builder().tier(Tier::Enhanced).build(),
            "tier=enhanced",
        );

        check_serialization(&Options::builder().tier(Tier::Base).build(), "tier=base");
    }

    #[test]
    fn model() {
        check_serialization(
            &Options::builder().model(Model::General).build(),
            "model=general",
        );

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
                    Model::Finance,
                    Model::CustomId(String::from("extra_crispy")),
                    Model::Conversationalai,
                ])
                .build(),
            "model=finance%3Aextra_crispy%3Aconversationalai&multichannel=true",
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
            &Options::builder().detect_language(false).build(),
            "detect_language=false",
        );

        check_serialization(
            &Options::builder().detect_language(true).build(),
            "detect_language=true",
        );
    }
}
