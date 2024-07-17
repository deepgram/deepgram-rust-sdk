//! Set options for [`Usage::get_usage`](super::mod_usage::Usage::get_usage).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#usage-summary

use serde::{ser::SerializeSeq, Serialize};

/// Used as a parameter for [`Usage::get_usage`](super::mod_usage::Usage::get_usage).
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

mod serialize_options_tests {
    // TODO
}
