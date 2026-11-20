//! Query options for [`Usage::get_breakdown`](super::Usage::get_breakdown).

use serde::{ser::SerializeSeq, Serialize};

use super::get_usage_options::{Deployment, Endpoint, Method};

/// Query options for a project usage breakdown.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
    grouping: Option<Grouping>,
    accessor: Option<String>,
    deployment: Option<Deployment>,
    endpoint: Option<Endpoint>,
    method: Option<Method>,
    model: Option<String>,
    tags: Vec<String>,
    alternatives: Option<bool>,
    callback_method: Option<bool>,
    callback: Option<bool>,
    channels: Option<bool>,
    custom_intent_mode: Option<bool>,
    custom_intent: Option<bool>,
    custom_topic_mode: Option<bool>,
    custom_topic: Option<bool>,
    detect_entities: Option<bool>,
    detect_language: Option<bool>,
    diarize: Option<bool>,
    dictation: Option<bool>,
    encoding: Option<bool>,
    extra: Option<bool>,
    filler_words: Option<bool>,
    intents: Option<bool>,
    keyterm: Option<bool>,
    keywords: Option<bool>,
    language: Option<bool>,
    measurements: Option<bool>,
    multichannel: Option<bool>,
    numerals: Option<bool>,
    paragraphs: Option<bool>,
    profanity_filter: Option<bool>,
    punctuate: Option<bool>,
    redact: Option<bool>,
    replace: Option<bool>,
    sample_rate: Option<bool>,
    search: Option<bool>,
    sentiment: Option<bool>,
    smart_format: Option<bool>,
    summarize: Option<bool>,
    topics: Option<bool>,
    utt_split: Option<bool>,
    utterances: Option<bool>,
    version: Option<bool>,
}

/// Builds [`Options`] for a project usage breakdown.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct OptionsBuilder(Options);

pub(crate) struct SerializableOptions<'a>(pub(crate) &'a Options);

/// Dimension used to group a usage breakdown.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Grouping {
    #[allow(missing_docs)]
    Accessor,

    #[allow(missing_docs)]
    Endpoint,

    #[allow(missing_docs)]
    FeatureSet,

    #[allow(missing_docs)]
    Models,

    #[allow(missing_docs)]
    Method,

    #[allow(missing_docs)]
    Tags,

    #[allow(missing_docs)]
    Deployment,

    /// A server-supported grouping dimension not yet represented by this SDK.
    Other(String),
}

impl AsRef<str> for Grouping {
    fn as_ref(&self) -> &str {
        match self {
            Self::Accessor => "accessor",
            Self::Endpoint => "endpoint",
            Self::FeatureSet => "feature_set",
            Self::Models => "models",
            Self::Method => "method",
            Self::Tags => "tags",
            Self::Deployment => "deployment",
            Self::Other(value) => value,
        }
    }
}

impl Serialize for Grouping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

macro_rules! boolean_filter {
    ($(($method:ident, $field:ident, $docs:literal)),+ $(,)?) => {
        $(
            #[doc = $docs]
            pub fn $method(mut self, value: bool) -> Self {
                self.0.$field = Some(value);
                self
            }
        )+
    };
}

impl Options {
    /// Start building query options.
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    /// Serialize these options as a URL query string.
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions(self))
    }
}

impl OptionsBuilder {
    /// Set the inclusive start date (`YYYY-MM-DD`).
    pub fn start(mut self, value: impl Into<String>) -> Self {
        self.0.start = Some(value.into());
        self
    }

    /// Set the inclusive end date (`YYYY-MM-DD`).
    pub fn end(mut self, value: impl Into<String>) -> Self {
        self.0.end = Some(value.into());
        self
    }

    /// Set the result grouping dimension.
    pub fn grouping(mut self, value: Grouping) -> Self {
        self.0.grouping = Some(value);
        self
    }

    /// Filter by accessor ID.
    pub fn accessor(mut self, value: impl Into<String>) -> Self {
        self.0.accessor = Some(value.into());
        self
    }

    /// Filter by deployment.
    pub fn deployment(mut self, value: Deployment) -> Self {
        self.0.deployment = Some(value);
        self
    }

    /// Filter by API endpoint.
    pub fn endpoint(mut self, value: Endpoint) -> Self {
        self.0.endpoint = Some(value);
        self
    }

    /// Filter by processing method.
    pub fn method(mut self, value: Method) -> Self {
        self.0.method = Some(value);
        self
    }

    /// Filter by model ID.
    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.0.model = Some(value.into());
        self
    }

    /// Add tags to filter by.
    pub fn tag<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.0.tags.extend(values.into_iter().map(Into::into));
        self
    }

    boolean_filter!(
        (alternatives, alternatives, "Filter by alternatives use."),
        (
            callback_method,
            callback_method,
            "Filter by callback method use."
        ),
        (callback, callback, "Filter by callback use."),
        (channels, channels, "Filter by channel-count configuration."),
        (
            custom_intent_mode,
            custom_intent_mode,
            "Filter by custom intent mode use."
        ),
        (custom_intent, custom_intent, "Filter by custom intent use."),
        (
            custom_topic_mode,
            custom_topic_mode,
            "Filter by custom topic mode use."
        ),
        (custom_topic, custom_topic, "Filter by custom topic use."),
        (
            detect_entities,
            detect_entities,
            "Filter by named-entity recognition use."
        ),
        (
            detect_language,
            detect_language,
            "Filter by language detection use."
        ),
        (diarize, diarize, "Filter by diarization use."),
        (dictation, dictation, "Filter by dictation use."),
        (encoding, encoding, "Filter by encoding use."),
        (extra, extra, "Filter by extra metadata use."),
        (
            filler_words,
            filler_words,
            "Filter by filler-word reporting use."
        ),
        (intents, intents, "Filter by intent recognition use."),
        (keyterm, keyterm, "Filter by keyterm use."),
        (keywords, keywords, "Filter by keyword use."),
        (language, language, "Filter by explicit language use."),
        (
            measurements,
            measurements,
            "Filter by measurement formatting use."
        ),
        (multichannel, multichannel, "Filter by multichannel use."),
        (numerals, numerals, "Filter by numerals use."),
        (
            paragraphs,
            paragraphs,
            "Filter by paragraph formatting use."
        ),
        (
            profanity_filter,
            profanity_filter,
            "Filter by profanity filtering use."
        ),
        (punctuate, punctuate, "Filter by punctuation use."),
        (redact, redact, "Filter by redaction use."),
        (replace, replace, "Filter by replacement use."),
        (sample_rate, sample_rate, "Filter by sample-rate use."),
        (search, search, "Filter by search use."),
        (sentiment, sentiment, "Filter by sentiment analysis use."),
        (
            smart_format,
            smart_format,
            "Filter by smart formatting use."
        ),
        (summarize, summarize, "Filter by summarization use."),
        (topics, topics, "Filter by topic detection use."),
        (utt_split, utt_split, "Filter by utterance splitting use."),
        (utterances, utterances, "Filter by utterance reporting use."),
        (version, version, "Filter by model version use."),
    );

    /// Finish building the options.
    pub fn build(self) -> Options {
        self.0
    }
}

impl Serialize for SerializableOptions<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Options {
            start,
            end,
            grouping,
            accessor,
            deployment,
            endpoint,
            method,
            model,
            tags,
            alternatives,
            callback_method,
            callback,
            channels,
            custom_intent_mode,
            custom_intent,
            custom_topic_mode,
            custom_topic,
            detect_entities,
            detect_language,
            diarize,
            dictation,
            encoding,
            extra,
            filler_words,
            intents,
            keyterm,
            keywords,
            language,
            measurements,
            multichannel,
            numerals,
            paragraphs,
            profanity_filter,
            punctuate,
            redact,
            replace,
            sample_rate,
            search,
            sentiment,
            smart_format,
            summarize,
            topics,
            utt_split,
            utterances,
            version,
        } = self.0;
        let mut seq = serializer.serialize_seq(None)?;

        for (name, value) in [
            ("start", start),
            ("end", end),
            ("accessor", accessor),
            ("model", model),
        ] {
            if let Some(value) = value {
                seq.serialize_element(&(name, value))?;
            }
        }
        if let Some(grouping) = grouping {
            seq.serialize_element(&("grouping", grouping))?;
        }
        if let Some(deployment) = deployment {
            seq.serialize_element(&("deployment", deployment))?;
        }
        if let Some(endpoint) = endpoint {
            seq.serialize_element(&("endpoint", endpoint))?;
        }
        if let Some(method) = method {
            seq.serialize_element(&("method", method))?;
        }
        for tag in tags {
            seq.serialize_element(&("tag", tag))?;
        }
        macro_rules! option {
            ($name:literal, $value:expr) => {
                if let Some(value) = $value {
                    seq.serialize_element(&($name, value))?;
                }
            };
        }
        option!("alternatives", alternatives);
        option!("callback_method", callback_method);
        option!("callback", callback);
        option!("channels", channels);
        option!("custom_intent_mode", custom_intent_mode);
        option!("custom_intent", custom_intent);
        option!("custom_topic_mode", custom_topic_mode);
        option!("custom_topic", custom_topic);
        option!("detect_entities", detect_entities);
        option!("detect_language", detect_language);
        option!("diarize", diarize);
        option!("dictation", dictation);
        option!("encoding", encoding);
        option!("extra", extra);
        option!("filler_words", filler_words);
        option!("intents", intents);
        option!("keyterm", keyterm);
        option!("keywords", keywords);
        option!("language", language);
        option!("measurements", measurements);
        option!("multichannel", multichannel);
        option!("numerals", numerals);
        option!("paragraphs", paragraphs);
        option!("profanity_filter", profanity_filter);
        option!("punctuate", punctuate);
        option!("redact", redact);
        option!("replace", replace);
        option!("sample_rate", sample_rate);
        option!("search", search);
        option!("sentiment", sentiment);
        option!("smart_format", smart_format);
        option!("summarize", summarize);
        option!("topics", topics);
        option!("utt_split", utt_split);
        option!("utterances", utterances);
        option!("version", version);
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Grouping, Options};

    #[test]
    fn serializes_filters_and_repeated_tags() {
        let options = Options::builder()
            .grouping(Grouping::FeatureSet)
            .tag(["production", "billing"])
            .detect_entities(true)
            .summarize(false)
            .build();

        assert_eq!(
            options.urlencoded().unwrap(),
            "grouping=feature_set&tag=production&tag=billing&detect_entities=true&summarize=false"
        );
    }

    #[test]
    fn serializes_server_groupings_not_yet_modeled_by_the_sdk() {
        let options = Options::builder()
            .grouping(Grouping::Other("future_dimension".to_owned()))
            .build();

        assert_eq!(options.urlencoded().unwrap(), "grouping=future_dimension");
    }

    #[test]
    fn serializes_every_feature_filter() {
        let options = Options::builder()
            .alternatives(true)
            .callback_method(false)
            .callback(true)
            .channels(false)
            .custom_intent_mode(true)
            .custom_intent(false)
            .custom_topic_mode(true)
            .custom_topic(false)
            .detect_entities(true)
            .detect_language(false)
            .diarize(true)
            .dictation(false)
            .encoding(true)
            .extra(false)
            .filler_words(true)
            .intents(false)
            .keyterm(true)
            .keywords(false)
            .language(true)
            .measurements(false)
            .multichannel(true)
            .numerals(false)
            .paragraphs(true)
            .profanity_filter(false)
            .punctuate(true)
            .redact(false)
            .replace(true)
            .sample_rate(false)
            .search(true)
            .sentiment(false)
            .smart_format(true)
            .summarize(false)
            .topics(true)
            .utt_split(false)
            .utterances(true)
            .version(false)
            .build();
        let query = options.urlencoded().unwrap();
        let actual: BTreeSet<_> = query.split('&').collect();
        let expected = BTreeSet::from([
            "alternatives=true",
            "callback_method=false",
            "callback=true",
            "channels=false",
            "custom_intent_mode=true",
            "custom_intent=false",
            "custom_topic_mode=true",
            "custom_topic=false",
            "detect_entities=true",
            "detect_language=false",
            "diarize=true",
            "dictation=false",
            "encoding=true",
            "extra=false",
            "filler_words=true",
            "intents=false",
            "keyterm=true",
            "keywords=false",
            "language=true",
            "measurements=false",
            "multichannel=true",
            "numerals=false",
            "paragraphs=true",
            "profanity_filter=false",
            "punctuate=true",
            "redact=false",
            "replace=true",
            "sample_rate=false",
            "search=true",
            "sentiment=false",
            "smart_format=true",
            "summarize=false",
            "topics=true",
            "utt_split=false",
            "utterances=true",
            "version=false",
        ]);

        assert_eq!(actual, expected);
    }
}
