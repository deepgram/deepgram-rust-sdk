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

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(in super::super) enum Multichannel {
    Disabled,
    Enabled { models: Option<Vec<Model>> },
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

pub(in super::super) fn models_to_string(models: &[Model]) -> String {
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
