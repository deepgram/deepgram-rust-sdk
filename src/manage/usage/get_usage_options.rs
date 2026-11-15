//! Set options for [`Usage::get_usage`](super::Usage::get_usage).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#usage-summary

use serde::{ser::SerializeSeq, Serialize};

/// Used as a parameter for [`Usage::get_usage`](super::Usage::get_usage).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#usage-summary
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
    accessor: Option<String>,
    tags: Vec<String>,
    methods: Vec<Method>,
    models: Vec<String>,
    multichannel: Option<bool>,
    interim_results: Option<bool>,
    punctuate: Option<bool>,
    ner: Option<bool>,
    utterances: Option<bool>,
    replace: Option<bool>,
    profanity_filter: Option<bool>,
    keywords: Option<bool>,
    diarize: Option<bool>,
    search: Option<bool>,
    redact: Option<bool>,
    alternatives: Option<bool>,
    numerals: Option<bool>,
    callback_method: Option<bool>,
    callback: Option<bool>,
    channels: Option<bool>,
    custom_intent_mode: Option<bool>,
    custom_intent: Option<bool>,
    custom_topic_mode: Option<bool>,
    custom_topic: Option<bool>,
    deployment: Option<String>,
    detect_entities: Option<bool>,
    detect_language: Option<bool>,
    dictation: Option<bool>,
    encoding: Option<bool>,
    endpoint: Option<String>,
    extra: Option<bool>,
    filler_words: Option<bool>,
    intents: Option<bool>,
    keyterm: Option<bool>,
    language: Option<bool>,
    measurements: Option<bool>,
    paragraphs: Option<bool>,
    sample_rate: Option<bool>,
    sentiment: Option<bool>,
    smart_format: Option<bool>,
    summarize: Option<bool>,
    topics: Option<bool>,
    utt_split: Option<bool>,
    version: Option<bool>,
}

/// Used as a parameter for [`OptionsBuilder::method`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Method {
    #[allow(missing_docs)]
    Sync,

    #[allow(missing_docs)]
    Async,

    #[allow(missing_docs)]
    Streaming,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

pub(crate) struct SerializableOptions<'a>(&'a Options);

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
    /// use deepgram::manage::usage::get_usage_options::{Options, Method};
    /// let options = Options::builder()
    ///     .method([Method::Sync, Method::Async])
    ///     .ner(true)
    ///     .build();
    /// assert_eq!(&options.urlencoded().unwrap(), "method=sync&method=async&ner=true")
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
            start: None,
            end: None,
            accessor: None,
            tags: Vec::new(),
            methods: Vec::new(),
            models: Vec::new(),
            multichannel: None,
            interim_results: None,
            punctuate: None,
            ner: None,
            utterances: None,
            replace: None,
            profanity_filter: None,
            keywords: None,
            diarize: None,
            search: None,
            redact: None,
            alternatives: None,
            numerals: None,
            callback_method: None,
            callback: None,
            channels: None,
            custom_intent_mode: None,
            custom_intent: None,
            custom_topic_mode: None,
            custom_topic: None,
            deployment: None,
            detect_entities: None,
            detect_language: None,
            dictation: None,
            encoding: None,
            endpoint: None,
            extra: None,
            filler_words: None,
            intents: None,
            keyterm: None,
            language: None,
            measurements: None,
            paragraphs: None,
            sample_rate: None,
            sentiment: None,
            smart_format: None,
            summarize: None,
            topics: None,
            utt_split: None,
            version: None,
        })
    }

    /// Set the time range start date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .start("1970-01-01")
    ///     .build();
    /// ```
    pub fn start(mut self, start: impl Into<String>) -> Self {
        self.0.start = Some(start.into());
        self
    }

    /// Set the time range end date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .end("2038-01-19")
    ///     .build();
    /// ```
    pub fn end(mut self, end: impl Into<String>) -> Self {
        self.0.end = Some(end.into());
        self
    }

    /// Limits results to requests made using the API key corresponding to the given accessor.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .accessor("12345678-1234-1234-1234-1234567890ab")
    ///     .build();
    /// ```
    pub fn accessor(mut self, accessor: impl Into<String>) -> Self {
        self.0.accessor = Some(accessor.into());
        self
    }

    /// Limits results to requests associated with the specified tag.
    ///
    /// Calling this when already set will append to the existing tags, not overwrite them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .tag(["Tag 1", "Tag 2"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
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

    /// Limits results to requests processed using the specified method.
    ///
    /// Calling this when already set will append to the existing methods, not overwrite them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::{Method, Options};
    /// #
    /// let options = Options::builder()
    ///     .method([Method::Sync, Method::Streaming])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::{Method, Options};
    /// #
    /// let options1 = Options::builder()
    ///     .method([Method::Sync])
    ///     .method([Method::Streaming])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .method([Method::Sync, Method::Streaming])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn method(mut self, method: impl IntoIterator<Item = Method>) -> Self {
        self.0.methods.extend(method);
        self
    }

    /// Limits results to requests run with the specified model UUID applied.
    ///
    /// Calling this when already set will append to the models, not overwrite them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .model([
    ///         "4899aa60-f723-4517-9815-2042acc12a82",
    ///         "125125fb-e391-458e-a227-a60d6426f5d6",
    ///     ])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .model(["4899aa60-f723-4517-9815-2042acc12a82"])
    ///     .model(["125125fb-e391-458e-a227-a60d6426f5d6"])
    ///     .build();
    ///
    /// let options2 = Options::builder()
    ///     .model([
    ///         "4899aa60-f723-4517-9815-2042acc12a82",
    ///         "125125fb-e391-458e-a227-a60d6426f5d6",
    ///     ])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn model<'a>(mut self, model: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.models.extend(model.into_iter().map(String::from));
        self
    }

    /// Limits results to requests that include the Multichannel feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .multichannel(true)
    ///     .build();
    /// ```
    pub fn multichannel(mut self, multichannel: bool) -> Self {
        self.0.multichannel = Some(multichannel);
        self
    }

    /// Limits results to requests that include the Interim Results feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .interim_results(true)
    ///     .build();
    /// ```
    pub fn interim_results(mut self, interim_results: bool) -> Self {
        self.0.interim_results = Some(interim_results);
        self
    }

    /// Limits results to requests that include the Punctuation feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .punctuate(true)
    ///     .build();
    /// ```
    pub fn punctuate(mut self, punctuate: bool) -> Self {
        self.0.punctuate = Some(punctuate);
        self
    }

    /// Limits results to requests that include the Named-Entity Recognition feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .ner(true)
    ///     .build();
    /// ```
    pub fn ner(mut self, ner: bool) -> Self {
        self.0.ner = Some(ner);
        self
    }

    /// Limits results to requests that include the Utterances feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .utterances(true)
    ///     .build();
    /// ```
    pub fn utterances(mut self, utterances: bool) -> Self {
        self.0.utterances = Some(utterances);
        self
    }

    /// Limits results to requests that include the Replace feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .replace(true)
    ///     .build();
    /// ```
    pub fn replace(mut self, replace: bool) -> Self {
        self.0.replace = Some(replace);
        self
    }

    /// Limits results to requests that include the Profanity Filter feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .profanity_filter(true)
    ///     .build();
    /// ```
    pub fn profanity_filter(mut self, profanity_filter: bool) -> Self {
        self.0.profanity_filter = Some(profanity_filter);
        self
    }

    /// Limits results to requests that include the Keywords feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .keywords(true)
    ///     .build();
    /// ```
    pub fn keywords(mut self, keywords: bool) -> Self {
        self.0.keywords = Some(keywords);
        self
    }

    /// Limits results to requests that include the Diarization feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .diarize(true)
    ///     .build();
    /// ```
    pub fn diarize(mut self, diarize: bool) -> Self {
        self.0.diarize = Some(diarize);
        self
    }

    /// Limits results to requests that include the Search feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .search(true)
    ///     .build();
    /// ```
    pub fn search(mut self, search: bool) -> Self {
        self.0.search = Some(search);
        self
    }

    /// Limits results to requests that include the Redaction feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .redact(true)
    ///     .build();
    /// ```
    pub fn redact(mut self, redact: bool) -> Self {
        self.0.redact = Some(redact);
        self
    }

    /// Limits results to requests that include the Alternatives feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .alternatives(true)
    ///     .build();
    /// ```
    pub fn alternatives(mut self, alternatives: bool) -> Self {
        self.0.alternatives = Some(alternatives);
        self
    }

    /// Limits results to requests that include the Numerals feature.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::get_usage_options::Options;
    /// #
    /// let options = Options::builder()
    ///     .numerals(true)
    ///     .build();
    /// ```
    pub fn numerals(mut self, numerals: bool) -> Self {
        self.0.numerals = Some(numerals);
        self
    }

    /// Filter by callback method use.
    pub fn callback_method(mut self, value: bool) -> Self {
        self.0.callback_method = Some(value);
        self
    }

    /// Filter by callback use.
    pub fn callback(mut self, value: bool) -> Self {
        self.0.callback = Some(value);
        self
    }

    /// Filter by channel-count configuration.
    pub fn channels(mut self, value: bool) -> Self {
        self.0.channels = Some(value);
        self
    }

    /// Filter by custom intent mode use.
    pub fn custom_intent_mode(mut self, value: bool) -> Self {
        self.0.custom_intent_mode = Some(value);
        self
    }

    /// Filter by custom intent use.
    pub fn custom_intent(mut self, value: bool) -> Self {
        self.0.custom_intent = Some(value);
        self
    }

    /// Filter by custom topic mode use.
    pub fn custom_topic_mode(mut self, value: bool) -> Self {
        self.0.custom_topic_mode = Some(value);
        self
    }

    /// Filter by custom topic use.
    pub fn custom_topic(mut self, value: bool) -> Self {
        self.0.custom_topic = Some(value);
        self
    }

    /// Filter by deployment.
    pub fn deployment(mut self, value: impl Into<String>) -> Self {
        self.0.deployment = Some(value.into());
        self
    }

    /// Filter by named-entity recognition use.
    pub fn detect_entities(mut self, value: bool) -> Self {
        self.0.detect_entities = Some(value);
        self
    }

    /// Filter by language detection use.
    pub fn detect_language(mut self, value: bool) -> Self {
        self.0.detect_language = Some(value);
        self
    }

    /// Filter by dictation use.
    pub fn dictation(mut self, value: bool) -> Self {
        self.0.dictation = Some(value);
        self
    }

    /// Filter by encoding use.
    pub fn encoding(mut self, value: bool) -> Self {
        self.0.encoding = Some(value);
        self
    }

    /// Filter by API endpoint.
    pub fn endpoint(mut self, value: impl Into<String>) -> Self {
        self.0.endpoint = Some(value.into());
        self
    }

    /// Filter by extra metadata use.
    pub fn extra(mut self, value: bool) -> Self {
        self.0.extra = Some(value);
        self
    }

    /// Filter by filler-word reporting use.
    pub fn filler_words(mut self, value: bool) -> Self {
        self.0.filler_words = Some(value);
        self
    }

    /// Filter by intent recognition use.
    pub fn intents(mut self, value: bool) -> Self {
        self.0.intents = Some(value);
        self
    }

    /// Filter by keyterm use.
    pub fn keyterm(mut self, value: bool) -> Self {
        self.0.keyterm = Some(value);
        self
    }

    /// Filter by explicit language use.
    pub fn language(mut self, value: bool) -> Self {
        self.0.language = Some(value);
        self
    }

    /// Filter by measurement formatting use.
    pub fn measurements(mut self, value: bool) -> Self {
        self.0.measurements = Some(value);
        self
    }

    /// Filter by paragraph formatting use.
    pub fn paragraphs(mut self, value: bool) -> Self {
        self.0.paragraphs = Some(value);
        self
    }

    /// Filter by sample-rate use.
    pub fn sample_rate(mut self, value: bool) -> Self {
        self.0.sample_rate = Some(value);
        self
    }

    /// Filter by sentiment analysis use.
    pub fn sentiment(mut self, value: bool) -> Self {
        self.0.sentiment = Some(value);
        self
    }

    /// Filter by smart formatting use.
    pub fn smart_format(mut self, value: bool) -> Self {
        self.0.smart_format = Some(value);
        self
    }

    /// Filter by summarization use.
    pub fn summarize(mut self, value: bool) -> Self {
        self.0.summarize = Some(value);
        self
    }

    /// Filter by topic detection use.
    pub fn topics(mut self, value: bool) -> Self {
        self.0.topics = Some(value);
        self
    }

    /// Filter by utterance splitting use.
    pub fn utt_split(mut self, value: bool) -> Self {
        self.0.utt_split = Some(value);
        self
    }

    /// Filter by model version use.
    pub fn version(mut self, value: bool) -> Self {
        self.0.version = Some(value);
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

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        Self(options)
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
            start,
            end,
            accessor,
            tags,
            methods,
            models,
            multichannel,
            interim_results,
            punctuate,
            ner,
            utterances,
            replace,
            profanity_filter,
            keywords,
            diarize,
            search,
            redact,
            alternatives,
            numerals,
            callback_method,
            callback,
            channels,
            custom_intent_mode,
            custom_intent,
            custom_topic_mode,
            custom_topic,
            deployment,
            detect_entities,
            detect_language,
            dictation,
            encoding,
            endpoint,
            extra,
            filler_words,
            intents,
            keyterm,
            language,
            measurements,
            paragraphs,
            sample_rate,
            sentiment,
            smart_format,
            summarize,
            topics,
            utt_split,
            version,
        } = self.0;

        if let Some(start) = start {
            seq.serialize_element(&("start", start))?;
        }

        if let Some(end) = end {
            seq.serialize_element(&("end", end))?;
        }

        if let Some(accessor) = accessor {
            seq.serialize_element(&("accessor", accessor))?;
        }

        for element in tags {
            seq.serialize_element(&("tag", element))?;
        }

        for element in methods {
            seq.serialize_element(&("method", AsRef::<str>::as_ref(element)))?;
        }

        for element in models {
            seq.serialize_element(&("model", element))?;
        }

        if let Some(multichannel) = multichannel {
            seq.serialize_element(&("multichannel", multichannel))?;
        }

        if let Some(interim_results) = interim_results {
            seq.serialize_element(&("interim_results", interim_results))?;
        }

        if let Some(punctuate) = punctuate {
            seq.serialize_element(&("punctuate", punctuate))?;
        }

        if let Some(ner) = ner {
            seq.serialize_element(&("ner", ner))?;
        }

        if let Some(utterances) = utterances {
            seq.serialize_element(&("utterances", utterances))?;
        }

        if let Some(replace) = replace {
            seq.serialize_element(&("replace", replace))?;
        }

        if let Some(profanity_filter) = profanity_filter {
            seq.serialize_element(&("profanity_filter", profanity_filter))?;
        }

        if let Some(keywords) = keywords {
            seq.serialize_element(&("keywords", keywords))?;
        }

        if let Some(diarize) = diarize {
            seq.serialize_element(&("diarize", diarize))?;
        }

        if let Some(search) = search {
            seq.serialize_element(&("search", search))?;
        }

        if let Some(redact) = redact {
            seq.serialize_element(&("redact", redact))?;
        }

        if let Some(alternatives) = alternatives {
            seq.serialize_element(&("alternatives", alternatives))?;
        }

        if let Some(numerals) = numerals {
            seq.serialize_element(&("numerals", numerals))?;
        }

        macro_rules! option {
            ($name:literal, $value:expr) => {
                if let Some(value) = $value {
                    seq.serialize_element(&($name, value))?;
                }
            };
        }

        option!("callback_method", callback_method);
        option!("callback", callback);
        option!("channels", channels);
        option!("custom_intent_mode", custom_intent_mode);
        option!("custom_intent", custom_intent);
        option!("custom_topic_mode", custom_topic_mode);
        option!("custom_topic", custom_topic);
        option!("deployment", deployment);
        option!("detect_entities", detect_entities);
        option!("detect_language", detect_language);
        option!("dictation", dictation);
        option!("encoding", encoding);
        option!("endpoint", endpoint);
        option!("extra", extra);
        option!("filler_words", filler_words);
        option!("intents", intents);
        option!("keyterm", keyterm);
        option!("language", language);
        option!("measurements", measurements);
        option!("paragraphs", paragraphs);
        option!("sample_rate", sample_rate);
        option!("sentiment", sentiment);
        option!("smart_format", smart_format);
        option!("summarize", summarize);
        option!("topics", topics);
        option!("utt_split", utt_split);
        option!("version", version);

        seq.end()
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        use Method::*;

        match self {
            Sync => "sync",
            Async => "async",
            Streaming => "streaming",
        }
    }
}

#[cfg(test)]
mod serialize_options_tests {
    use super::Options;

    #[test]
    fn serializes_current_documented_filters() {
        let options = Options::builder()
            .deployment("hosted")
            .detect_entities(true)
            .callback(true)
            .custom_topic_mode(false)
            .smart_format(true)
            .version(false)
            .build();

        assert_eq!(
            options.urlencoded().unwrap(),
            "callback=true&custom_topic_mode=false&deployment=hosted&detect_entities=true&smart_format=true&version=false"
        );
    }
}
