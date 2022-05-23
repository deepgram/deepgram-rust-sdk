use serde::{ser::SerializeSeq, Serialize};

#[derive(Debug)]
pub struct OptionsBuilder<'a> {
    model: Option<Model<'a>>,
    version: Option<&'a str>,
    language: Option<Language<'a>>,
    punctuate: Option<bool>,
    profanity_filter: Option<bool>,
    redact: Vec<Redact<'a>>,
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
pub enum Language<'a> {
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
    /// Avoid using the `Other` variant where possible.
    /// It exists so that you can use new languages that Deepgram supports without being forced to update your version of the SDK.
    /// Please consult the [Deepgram Language Documentation](https://developers.deepgram.com/documentation/features/language/) for the most up-to-date list of supported languages.
    Other(&'a str),
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Redact<'a> {
    Pci,
    Numbers,
    Ssn,
    /// Avoid using the `Other` variant where possible.
    /// It exists so that you can use new redactable items that Deepgram supports without being forced to update your version of the SDK.
    /// Please consult the [Deepgram Redact Documentation](https://developers.deepgram.com/documentation/features/redact/) for the most up-to-date list of redactable items.
    Other(&'a str),
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

    pub fn language(mut self, language: Language<'a>) -> Self {
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

    pub fn redact(mut self, redact: impl IntoIterator<Item = Redact<'a>>) -> Self {
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
                Language::Other(bcp_47_tag) => bcp_47_tag,
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
                Redact::Other(id) => id,
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
mod serialize_options_tests {
    use super::*;

    fn check_serialization(options: &OptionsBuilder, expected: &str) {
        let actual = {
            let mut serializer = form_urlencoded::Serializer::new(String::new());

            options
                .serialize(serde_urlencoded::Serializer::new(&mut serializer))
                .unwrap();

            serializer.finish()
        };

        assert_eq!(actual, expected);
    }

    fn generate_alphabet_test(key: &str) -> ([&str; 25], String) {
        let letters = [
            "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q",
            "R", "S", "T", "U", "V", "W", "X", "Y",
        ];

        let expected = {
            let mut expected = String::new();
            for letter in letters {
                expected.push_str(key);
                expected.push_str("=");
                expected.push_str(letter);
                expected.push_str("&");
            }
            expected.pop(); // Pop the extra & off the end
            expected
        };

        (letters, expected)
    }

    #[test]
    fn all_options() {
        let options = OptionsBuilder::new()
            .model(Model::General)
            .version("1.2.3")
            .language(Language::en_US)
            .punctuate(true)
            .profanity_filter(true)
            .redact([Redact::Pci, Redact::Ssn])
            .diarize(true)
            .ner(true)
            .multichannel(true)
            .alternatives(4)
            .numerals(true)
            .search(["Rust", "Deepgram"])
            .callback("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
            .keywords(["Ferris", "Cargo"])
            .utterances(Utterances::Enabled {
                utt_split: Some(0.9),
            })
            .tag("SDK Test");

        check_serialization(&options, "model=general&version=1.2.3&language=en-US&punctuate=true&profanity_filter=true&redact=pci&redact=ssn&diarize=true&ner=true&multichannel=true&alternatives=4&numerals=true&search=Rust&search=Deepgram&callback=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ&keywords=Ferris&keywords=Cargo&utterances=true&utt_split=0.9&tag=SDK+Test");
    }

    #[test]
    fn model() {
        check_serialization(
            &OptionsBuilder::new().model(Model::General),
            "model=general",
        );

        check_serialization(
            &OptionsBuilder::new().model(Model::CustomId("extra_crispy")),
            "model=extra_crispy",
        );
    }

    #[test]
    fn version() {
        check_serialization(&OptionsBuilder::new().version("1.2.3"), "version=1.2.3");
    }

    #[test]
    fn language() {
        check_serialization(
            &OptionsBuilder::new().language(Language::en_US),
            "language=en-US",
        );

        check_serialization(&OptionsBuilder::new().language(Language::ja), "language=ja");
    }

    #[test]
    fn punctuate() {
        check_serialization(&OptionsBuilder::new().punctuate(true), "punctuate=true");

        check_serialization(&OptionsBuilder::new().punctuate(false), "punctuate=false");
    }

    #[test]
    fn profanity_filter() {
        check_serialization(
            &OptionsBuilder::new().profanity_filter(true),
            "profanity_filter=true",
        );

        check_serialization(
            &OptionsBuilder::new().profanity_filter(false),
            "profanity_filter=false",
        );
    }

    #[test]
    fn redact() {
        check_serialization(&OptionsBuilder::new().redact([]), "");

        check_serialization(
            &OptionsBuilder::new().redact([Redact::Numbers]),
            "redact=numbers",
        );

        check_serialization(
            &OptionsBuilder::new().redact([Redact::Ssn, Redact::Pci]),
            "redact=ssn&redact=pci",
        );

        check_serialization(
            &OptionsBuilder::new().redact([
                Redact::Numbers,
                Redact::Ssn,
                Redact::Pci,
                Redact::Ssn,
                Redact::Numbers,
                Redact::Pci,
            ]),
            "redact=numbers&redact=ssn&redact=pci&redact=ssn&redact=numbers&redact=pci",
        );
    }

    #[test]
    fn diarize() {
        check_serialization(&OptionsBuilder::new().diarize(true), "diarize=true");

        check_serialization(&OptionsBuilder::new().diarize(false), "diarize=false");
    }

    #[test]
    fn ner() {
        check_serialization(&OptionsBuilder::new().ner(true), "ner=true");

        check_serialization(&OptionsBuilder::new().ner(false), "ner=false");
    }

    #[test]
    fn multichannel() {
        check_serialization(
            &OptionsBuilder::new().multichannel(true),
            "multichannel=true",
        );

        check_serialization(
            &OptionsBuilder::new().multichannel(false),
            "multichannel=false",
        );
    }

    #[test]
    fn alternatives() {
        check_serialization(&OptionsBuilder::new().alternatives(4), "alternatives=4");
    }

    #[test]
    fn numerals() {
        check_serialization(&OptionsBuilder::new().numerals(true), "numerals=true");

        check_serialization(&OptionsBuilder::new().numerals(false), "numerals=false");
    }

    #[test]
    fn search() {
        check_serialization(&OptionsBuilder::new().search([]), "");

        check_serialization(&OptionsBuilder::new().search(["Rust"]), "search=Rust");

        check_serialization(
            &OptionsBuilder::new().search(["Rust", "Deepgram"]),
            "search=Rust&search=Deepgram",
        );

        {
            let (input, expected) = generate_alphabet_test("search");
            check_serialization(&OptionsBuilder::new().search(input), &expected);
        }
    }

    #[test]
    fn callback() {
        check_serialization(
            &OptionsBuilder::new().callback("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            "callback=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ",
        );
    }

    #[test]
    fn keywords() {
        check_serialization(&OptionsBuilder::new().keywords([]), "");

        check_serialization(
            &OptionsBuilder::new().keywords(["Ferris"]),
            "keywords=Ferris",
        );

        check_serialization(
            &OptionsBuilder::new().keywords(["Ferris", "Cargo"]),
            "keywords=Ferris&keywords=Cargo",
        );

        {
            let (input, expected) = generate_alphabet_test("keywords");
            check_serialization(&OptionsBuilder::new().keywords(input), &expected);
        }
    }

    #[test]
    fn utterances() {
        check_serialization(
            &OptionsBuilder::new().utterances(Utterances::Disabled),
            "utterances=false",
        );

        check_serialization(
            &OptionsBuilder::new().utterances(Utterances::Enabled { utt_split: None }),
            "utterances=true",
        );

        check_serialization(
            &OptionsBuilder::new().utterances(Utterances::Enabled {
                utt_split: Some(0.9),
            }),
            "utterances=true&utt_split=0.9",
        );
    }

    #[test]
    fn tag() {
        check_serialization(&OptionsBuilder::new().tag("SDK Test"), "tag=SDK+Test");
    }
}
