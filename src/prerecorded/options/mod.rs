use serde::{ser::SerializeSeq, Serialize};

#[derive(Debug)]
pub struct OptionsBuilder<'a> {
    model: Option<Model<'a>>,
    version: Option<&'a str>,
    language: Option<Language>,
    punctuate: Option<bool>,
    profanity_filter: Option<bool>,
    redact: Vec<Redact>,
    diarize: Option<bool>,
    ner: Option<bool>,
    multichannel: Option<bool>,
    alternatives: Option<usize>,
    numerals: Option<bool>,
    search: Vec<&'a str>,
    callback: Option<&'a str>,
    keywords: Vec<&'a str>,
    utterances: Option<Utterances>,
    tag: Option<&'a str>,
}

#[derive(Debug)]
pub enum Model<'a> {
    General,
    Meeting,
    Phonecall,
    Voicemail,
    Finance,
    Conversational,
    Video,
    CustomId(&'a str),
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Language {
    zh_CN,
    zh_TW,
    nl,
    en_US,
    en_AU,
    en_GB,
    en_IN,
    en_NZ,
    fr,
    fr_CA,
    de,
    hi,
    id,
    it,
    ja,
    ko,
    pt,
    pr_BR,
    ru,
    es,
    es_419,
    sv,
    tr,
    uk,
}

#[derive(Debug)]
pub enum Redact {
    Pci,
    Numbers,
    Ssn,
}

#[derive(Debug)]
pub enum Utterances {
    Disabled,
    Enabled { utt_split: Option<f64> },
}

impl<'a> OptionsBuilder<'a> {
    pub fn new() -> Self {
        Self {
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
            callback: None,
            keywords: Vec::new(),
            utterances: None,
            tag: None,
        }
    }

    pub fn model(mut self, model: Model<'a>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    pub fn punctuate(mut self, punctuate: bool) -> Self {
        self.punctuate = Some(punctuate);
        self
    }

    pub fn profanity_filter(mut self, profanity_filter: bool) -> Self {
        self.profanity_filter = Some(profanity_filter);
        self
    }

    pub fn redact(mut self, redact: impl IntoIterator<Item = Redact>) -> Self {
        self.redact.extend(redact);
        self
    }

    pub fn diarize(mut self, diarize: bool) -> Self {
        self.diarize = Some(diarize);
        self
    }

    pub fn ner(mut self, ner: bool) -> Self {
        self.ner = Some(ner);
        self
    }

    pub fn multichannel(mut self, multichannel: bool) -> Self {
        self.multichannel = Some(multichannel);
        self
    }

    pub fn alternatives(mut self, alternatives: usize) -> Self {
        self.alternatives = Some(alternatives);
        self
    }

    pub fn numerals(mut self, numerals: bool) -> Self {
        self.numerals = Some(numerals);
        self
    }

    pub fn search(mut self, search: impl IntoIterator<Item = &'a str>) -> Self {
        self.search.extend(search);
        self
    }

    pub fn callback(mut self, callback: &'a str) -> Self {
        self.callback = Some(callback);
        self
    }

    pub fn keywords(mut self, keywords: impl IntoIterator<Item = &'a str>) -> Self {
        self.keywords.extend(keywords);
        self
    }

    pub fn utterances(mut self, utterances: Utterances) -> Self {
        self.utterances = Some(utterances);
        self
    }

    pub fn tag(mut self, tag: &'a str) -> Self {
        self.tag = Some(tag);
        self
    }
}

impl<'a> Default for OptionsBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for OptionsBuilder<'_> {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        // Destructuring it makes sure that we don't forget to use any of it
        let Self {
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
            callback,
            keywords,
            utterances,
            tag,
        } = self;

        if let Some(model) = model {
            let s = match model {
                Model::General => "general",
                Model::Meeting => "meeting",
                Model::Phonecall => "phonecall",
                Model::Voicemail => "voicemail",
                Model::Finance => "finance",
                Model::Conversational => "conversational",
                Model::Video => "video",
                Model::CustomId(id) => id,
            };

            seq.serialize_element(&("model", s))?;
        }

        if let Some(version) = version {
            seq.serialize_element(&("version", version))?;
        }

        if let Some(language) = language {
            let s = match language {
                Language::zh_CN => "zh-CN",
                Language::zh_TW => "zh-TW",
                Language::nl => "nl",
                Language::en_US => "en-US",
                Language::en_AU => "en-AU",
                Language::en_GB => "en-GB",
                Language::en_IN => "en-IN",
                Language::en_NZ => "en-NZ",
                Language::fr => "fr",
                Language::fr_CA => "fr-CA",
                Language::de => "de",
                Language::hi => "hi",
                Language::id => "id",
                Language::it => "it",
                Language::ja => "ja",
                Language::ko => "ko",
                Language::pt => "pt",
                Language::pr_BR => "pr_BR",
                Language::ru => "ru",
                Language::es => "es",
                Language::es_419 => "es-419",
                Language::sv => "sv",
                Language::tr => "tr",
                Language::uk => "uk",
            };

            seq.serialize_element(&("language", s))?;
        }

        if let Some(punctuate) = punctuate {
            seq.serialize_element(&("punctuate", punctuate))?;
        }

        if let Some(profanity_filter) = profanity_filter {
            seq.serialize_element(&("profanity_filter", profanity_filter))?;
        }

        for element in redact {
            let s = match element {
                Redact::Pci => "pci",
                Redact::Numbers => "numbers",
                Redact::Ssn => "ssn",
            };

            seq.serialize_element(&("redact", s))?;
        }

        if let Some(diarize) = diarize {
            seq.serialize_element(&("diarize", diarize))?;
        }

        if let Some(ner) = ner {
            seq.serialize_element(&("ner", ner))?;
        }

        if let Some(multichannel) = multichannel {
            seq.serialize_element(&("multichannel", multichannel))?;
        }

        if let Some(alternatives) = alternatives {
            seq.serialize_element(&("alternatives", alternatives))?;
        }

        if let Some(numerals) = numerals {
            seq.serialize_element(&("numerals", numerals))?;
        }

        for element in search {
            seq.serialize_element(&("search", element))?;
        }

        if let Some(callback) = callback {
            seq.serialize_element(&("callback", callback))?;
        }

        for element in keywords {
            seq.serialize_element(&("keywords", element))?;
        }

        match utterances {
            Some(Utterances::Disabled) => seq.serialize_element(&("utterances", false))?,
            Some(Utterances::Enabled { utt_split: None }) => {
                seq.serialize_element(&("utterances", true))?
            }
            Some(Utterances::Enabled {
                utt_split: Some(utt_split),
            }) => {
                seq.serialize_element(&("utterances", true))?;
                seq.serialize_element(&("utt_split", utt_split))?;
            }
            None => (),
        };

        if let Some(tag) = tag {
            seq.serialize_element(&("tag", tag))?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod serialize_options_tests;
