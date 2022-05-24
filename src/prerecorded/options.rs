use serde::{ser::SerializeSeq, Serialize};

#[derive(Debug, PartialEq, Clone)]
pub struct Options<'a> {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum Language<'a> {
    zh,
    zh_CN,
    zh_TW,
    nl,
    en,
    en_AU,
    en_GB,
    en_IN,
    en_NZ,
    en_US,
    fr,
    fr_CA,
    de,
    hi,
    id,
    it,
    ja,
    ko,
    pt,
    pt_BR,
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Utterances {
    Disabled,
    Enabled { utt_split: Option<f64> },
}

pub struct OptionsBuilder<'a>(Options<'a>);

pub(super) struct SerializableOptions<'a>(pub &'a Options<'a>);

impl<'a> Options<'a> {
    pub fn builder() -> OptionsBuilder<'a> {
        OptionsBuilder::new()
    }
}

impl<'a> OptionsBuilder<'a> {
    pub fn new() -> Self {
        Self(Options {
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
        })
    }

    pub fn model(mut self, model: Model<'a>) -> Self {
        self.0.model = Some(model);
        self
    }

    pub fn version(mut self, version: &'a str) -> Self {
        self.0.version = Some(version);
        self
    }

    pub fn language(mut self, language: Language<'a>) -> Self {
        self.0.language = Some(language);
        self
    }

    pub fn punctuate(mut self, punctuate: bool) -> Self {
        self.0.punctuate = Some(punctuate);
        self
    }

    pub fn profanity_filter(mut self, profanity_filter: bool) -> Self {
        self.0.profanity_filter = Some(profanity_filter);
        self
    }

    pub fn redact(mut self, redact: impl IntoIterator<Item = Redact<'a>>) -> Self {
        self.0.redact.extend(redact);
        self
    }

    pub fn diarize(mut self, diarize: bool) -> Self {
        self.0.diarize = Some(diarize);
        self
    }

    pub fn ner(mut self, ner: bool) -> Self {
        self.0.ner = Some(ner);
        self
    }

    pub fn multichannel(mut self, multichannel: bool) -> Self {
        self.0.multichannel = Some(multichannel);
        self
    }

    pub fn alternatives(mut self, alternatives: usize) -> Self {
        self.0.alternatives = Some(alternatives);
        self
    }

    pub fn numerals(mut self, numerals: bool) -> Self {
        self.0.numerals = Some(numerals);
        self
    }

    pub fn search(mut self, search: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.search.extend(search);
        self
    }

    pub fn callback(mut self, callback: &'a str) -> Self {
        self.0.callback = Some(callback);
        self
    }

    pub fn keywords(mut self, keywords: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.keywords.extend(keywords);
        self
    }

    pub fn utterances(mut self, utterances: Utterances) -> Self {
        self.0.utterances = Some(utterances);
        self
    }

    pub fn tag(mut self, tag: &'a str) -> Self {
        self.0.tag = Some(tag);
        self
    }

    pub fn build(self) -> Options<'a> {
        self.0
    }
}

impl<'a> Default for OptionsBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for SerializableOptions<'_> {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
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
            ner,
            multichannel,
            alternatives,
            numerals,
            search,
            callback,
            keywords,
            utterances,
            tag,
        } = self.0;

        if let Some(model) = model {
            seq.serialize_element(&("model", model.as_ref()))?;
        }

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
            Some(Utterances::Enabled { utt_split }) => {
                seq.serialize_element(&("utterances", true))?;

                if let Some(utt_split) = utt_split {
                    seq.serialize_element(&("utt_split", utt_split))?;
                }
            }
            None => (),
        };

        if let Some(tag) = tag {
            seq.serialize_element(&("tag", tag))?;
        }

        seq.end()
    }
}

impl AsRef<str> for Model<'_> {
    fn as_ref(&self) -> &str {
        use Model::*;

        match self {
            General => "general",
            Meeting => "meeting",
            Phonecall => "phonecall",
            Voicemail => "voicemail",
            Finance => "finance",
            Conversational => "conversational",
            Video => "video",
            CustomId(id) => id,
        }
    }
}

impl AsRef<str> for Language<'_> {
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

impl AsRef<str> for Redact<'_> {
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

#[cfg(test)]
mod serialize_options_tests {
    use super::*;

    fn check_serialization(options: &Options, expected: &str) {
        let actual = {
            let mut serializer = form_urlencoded::Serializer::new(String::new());

            SerializableOptions(options)
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
        let options = Options::builder()
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
            .tag("SDK Test")
            .build();

        check_serialization(&options, "model=general&version=1.2.3&language=en-US&punctuate=true&profanity_filter=true&redact=pci&redact=ssn&diarize=true&ner=true&multichannel=true&alternatives=4&numerals=true&search=Rust&search=Deepgram&callback=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ&keywords=Ferris&keywords=Cargo&utterances=true&utt_split=0.9&tag=SDK+Test");
    }

    #[test]
    fn model() {
        check_serialization(
            &Options::builder().model(Model::General).build(),
            "model=general",
        );

        check_serialization(
            &Options::builder()
                .model(Model::CustomId("extra_crispy"))
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
            let (input, expected) = generate_alphabet_test("search");
            check_serialization(&Options::builder().search(input).build(), &expected);
        }
    }

    #[test]
    fn callback() {
        check_serialization(
            &Options::builder()
                .callback("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                .build(),
            "callback=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DdQw4w9WgXcQ",
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
            let (input, expected) = generate_alphabet_test("keywords");
            check_serialization(&Options::builder().keywords(input).build(), &expected);
        }
    }

    #[test]
    fn utterances() {
        check_serialization(
            &Options::builder().utterances(Utterances::Disabled).build(),
            "utterances=false",
        );

        check_serialization(
            &Options::builder()
                .utterances(Utterances::Enabled { utt_split: None })
                .build(),
            "utterances=true",
        );

        check_serialization(
            &Options::builder()
                .utterances(Utterances::Enabled {
                    utt_split: Some(0.9),
                })
                .build(),
            "utterances=true&utt_split=0.9",
        );
    }

    #[test]
    fn tag() {
        check_serialization(&Options::builder().tag("SDK Test").build(), "tag=SDK+Test");
    }
}
