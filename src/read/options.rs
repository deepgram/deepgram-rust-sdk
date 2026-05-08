//! Query parameters for `POST /v1/read`.
//!
//! Mirrors `parameters.shared.yml` (subset) + `parameters.read.v1.yml`
//! in `deepgram-docs/api/specs/openapi/`.

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};
use url::Url;

use crate::common::options::{CallbackMethod, CustomIntentMode, CustomTopicMode};

/// Read API request options. Construct via [`Options::builder`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Options {
    language: Option<String>,
    callback: Option<Url>,
    callback_method: Option<CallbackMethod>,
    sentiment: Option<bool>,
    /// For Read this is a plain boolean per
    /// `parameters.shared.yml#/SharedSummarize`. The Listen API
    /// repurposes the same parameter as a versioned string (`v2`); Read
    /// does not.
    summarize: Option<bool>,
    topics: Option<bool>,
    custom_topic: Vec<String>,
    custom_topic_mode: Option<CustomTopicMode>,
    intents: Option<bool>,
    custom_intent: Vec<String>,
    custom_intent_mode: Option<CustomIntentMode>,
    tag: Vec<String>,
}

impl Options {
    /// Begin building a fresh [`Options`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::new()
    }

    /// URL-encoded query string (without leading `?`). Useful for
    /// debugging the request that will be sent.
    ///
    /// ```
    /// use deepgram::read::options::Options;
    /// let q = Options::builder()
    ///     .language("en")
    ///     .sentiment(true)
    ///     .summarize(true)
    ///     .build()
    ///     .urlencoded()
    ///     .unwrap();
    /// assert_eq!(q, "language=en&sentiment=true&summarize=true");
    /// ```
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(self)
    }
}

/// Builder for [`Options`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OptionsBuilder(Options);

impl OptionsBuilder {
    /// Construct a fresh empty builder.
    pub fn new() -> Self {
        Self(Options::default())
    }

    /// BCP-47 language tag for the input text. Defaults to `en` server-side.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.0.language = Some(language.into());
        self
    }

    /// Callback URL for asynchronous results.
    pub fn callback(mut self, callback: Url) -> Self {
        self.0.callback = Some(callback);
        self
    }

    /// HTTP method used for the callback.
    pub fn callback_method(mut self, method: CallbackMethod) -> Self {
        self.0.callback_method = Some(method);
        self
    }

    /// Enable sentiment analysis on the input.
    pub fn sentiment(mut self, sentiment: bool) -> Self {
        self.0.sentiment = Some(sentiment);
        self
    }

    /// Enable summarization. Plain boolean for the Read API (no `v2` form).
    pub fn summarize(mut self, summarize: bool) -> Self {
        self.0.summarize = Some(summarize);
        self
    }

    /// Enable topic detection.
    pub fn topics(mut self, topics: bool) -> Self {
        self.0.topics = Some(topics);
        self
    }

    /// Replace the custom-topic list. Up to 100 per spec.
    pub fn custom_topics<I, S>(mut self, topics: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.0.custom_topic = topics.into_iter().map(Into::into).collect();
        self
    }

    /// Set how custom topics interact with the model's own detections.
    pub fn custom_topic_mode(mut self, mode: CustomTopicMode) -> Self {
        self.0.custom_topic_mode = Some(mode);
        self
    }

    /// Enable intent detection.
    pub fn intents(mut self, intents: bool) -> Self {
        self.0.intents = Some(intents);
        self
    }

    /// Replace the custom-intent list. Up to 100 per spec.
    pub fn custom_intents<I, S>(mut self, intents: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.0.custom_intent = intents.into_iter().map(Into::into).collect();
        self
    }

    /// Set how custom intents interact with the model's own detections.
    pub fn custom_intent_mode(mut self, mode: CustomIntentMode) -> Self {
        self.0.custom_intent_mode = Some(mode);
        self
    }

    /// Replace the tag list (used for usage reporting).
    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.0.tag = tags.into_iter().map(Into::into).collect();
        self
    }

    /// Finish building.
    pub fn build(self) -> Options {
        self.0
    }
}

// Custom serialization to emit repeated query params for vector fields
// (custom_topic, custom_intent, tag), matching the spec's repeated-param
// convention.
impl Serialize for Options {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let Options {
            language,
            callback,
            callback_method,
            sentiment,
            summarize,
            topics,
            custom_topic,
            custom_topic_mode,
            intents,
            custom_intent,
            custom_intent_mode,
            tag,
        } = self;

        let mut seq = serializer.serialize_seq(None)?;

        if let Some(language) = language {
            seq.serialize_element(&("language", language))?;
        }
        if let Some(callback) = callback {
            seq.serialize_element(&("callback", callback.as_str()))?;
        }
        if let Some(callback_method) = callback_method {
            seq.serialize_element(&("callback_method", callback_method.as_str()))?;
        }
        if let Some(sentiment) = sentiment {
            seq.serialize_element(&("sentiment", sentiment))?;
        }
        if let Some(summarize) = summarize {
            seq.serialize_element(&("summarize", summarize))?;
        }
        if let Some(topics) = topics {
            seq.serialize_element(&("topics", topics))?;
        }
        for entry in custom_topic {
            seq.serialize_element(&("custom_topic", entry))?;
        }
        if let Some(mode) = custom_topic_mode {
            seq.serialize_element(&("custom_topic_mode", custom_mode_str(mode)))?;
        }
        if let Some(intents) = intents {
            seq.serialize_element(&("intents", intents))?;
        }
        for entry in custom_intent {
            seq.serialize_element(&("custom_intent", entry))?;
        }
        if let Some(mode) = custom_intent_mode {
            seq.serialize_element(&("custom_intent_mode", custom_intent_mode_str(mode)))?;
        }
        for entry in tag {
            seq.serialize_element(&("tag", entry))?;
        }

        seq.end()
    }
}

fn custom_mode_str(mode: &CustomTopicMode) -> &'static str {
    // Both enums are `#[non_exhaustive]` for downstream crates, but
    // within this crate we can (and should) match exhaustively — if a
    // new variant is added the compiler surfaces this site so we can
    // decide on its wire mapping.
    match mode {
        CustomTopicMode::Extended => "extended",
        CustomTopicMode::Strict => "strict",
    }
}

fn custom_intent_mode_str(mode: &CustomIntentMode) -> &'static str {
    match mode {
        CustomIntentMode::Extended => "extended",
        CustomIntentMode::Strict => "strict",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_options_serializes_to_empty_query() {
        let q = Options::builder().build().urlencoded().unwrap();
        assert_eq!(q, "");
    }

    #[test]
    fn full_options_serializes() {
        let q = Options::builder()
            .language("en")
            .sentiment(true)
            .summarize(true)
            .topics(true)
            .custom_topics(["weather", "sports"])
            .custom_topic_mode(CustomTopicMode::Strict)
            .intents(true)
            .custom_intents(["question", "command"])
            .custom_intent_mode(CustomIntentMode::Extended)
            .tags(["staging", "team-alpha"])
            .build()
            .urlencoded()
            .unwrap();

        // Order is the order written above.
        assert_eq!(
            q,
            "language=en&sentiment=true&summarize=true&topics=true\
             &custom_topic=weather&custom_topic=sports&custom_topic_mode=strict\
             &intents=true&custom_intent=question&custom_intent=command\
             &custom_intent_mode=extended&tag=staging&tag=team-alpha"
        );
    }

    #[test]
    fn callback_serializes() {
        let q = Options::builder()
            .callback("https://example.com/cb".parse().unwrap())
            .callback_method(CallbackMethod::PUT)
            .build()
            .urlencoded()
            .unwrap();
        assert_eq!(
            q,
            "callback=https%3A%2F%2Fexample.com%2Fcb&callback_method=put"
        );
    }
}
