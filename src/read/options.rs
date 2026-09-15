//! Set various Deepgram features for a Text Intelligence (`/v1/read`) request.
//!
//! See the [Deepgram Text Intelligence docs][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/docs/text-intelligence

use serde::{ser::SerializeSeq, Serialize};

// Reuse the shared analysis mode + language enums from the transcription
// options rather than duplicating them.
pub use crate::common::options::{CallbackMethod, CustomIntentMode, CustomTopicMode, Language};

/// Features for a Text Intelligence request.
///
/// Construct with [`Options::builder`].
///
/// The Text Intelligence API supports English only, and requires the
/// `language` query parameter on every request. The builder therefore sends
/// `language=en` by default; you do not need to call
/// [`OptionsBuilder::language`] unless the API starts accepting other
/// languages.
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    sentiment: Option<bool>,
    summarize: Option<bool>,
    topics: Option<bool>,
    custom_topics: Vec<String>,
    custom_topic_mode: Option<CustomTopicMode>,
    intents: Option<bool>,
    custom_intents: Vec<String>,
    custom_intent_mode: Option<CustomIntentMode>,
    language: Option<Language>,
    tags: Vec<String>,
    callback_method: Option<CallbackMethod>,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
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

    /// Return the options in urlencoded format. If serialization would fail,
    /// this will also return an error.
    ///
    /// ```
    /// use deepgram::read::options::Options;
    /// let options = Options::builder().sentiment(true).topics(true).build();
    /// assert_eq!(&options.urlencoded().unwrap(), "sentiment=true&topics=true&language=en");
    /// ```
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions(self))
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    ///
    /// `language` defaults to [`Language::en`], the only language the Text
    /// Intelligence API accepts; the API rejects requests that omit it with
    /// HTTP 400.
    pub fn new() -> Self {
        Self(Options {
            sentiment: None,
            summarize: None,
            topics: None,
            custom_topics: Vec::new(),
            custom_topic_mode: None,
            intents: None,
            custom_intents: Vec::new(),
            custom_intent_mode: None,
            language: Some(Language::en),
            tags: Vec::new(),
            callback_method: None,
        })
    }

    /// Enable sentiment analysis.
    ///
    /// See the [Deepgram Text Sentiment docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-sentiment-analysis
    pub fn sentiment(mut self, sentiment: bool) -> Self {
        self.0.sentiment = Some(sentiment);
        self
    }

    /// Enable summarization.
    ///
    /// Unlike the transcription API, the Text Intelligence API accepts a
    /// boolean only.
    ///
    /// See the [Deepgram Text Summarization docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-summarization
    pub fn summarize(mut self, summarize: bool) -> Self {
        self.0.summarize = Some(summarize);
        self
    }

    /// Enable topic detection.
    ///
    /// See the [Deepgram Text Topic Detection docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-topic-detection
    pub fn topics(mut self, topics: bool) -> Self {
        self.0.topics = Some(topics);
        self
    }

    /// Set the custom topic detection mode.
    pub fn custom_topic_mode(mut self, mode: CustomTopicMode) -> Self {
        self.0.custom_topic_mode = Some(mode);
        self
    }

    /// Add custom topics for the model to detect.
    ///
    /// Calling this repeatedly appends to the existing custom topics.
    pub fn custom_topics<'a>(mut self, topics: impl IntoIterator<Item = &'a str>) -> Self {
        self.0
            .custom_topics
            .extend(topics.into_iter().map(String::from));
        self
    }

    /// Enable intent recognition.
    ///
    /// See the [Deepgram Text Intent Recognition docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-intention-recognition
    pub fn intents(mut self, intents: bool) -> Self {
        self.0.intents = Some(intents);
        self
    }

    /// Set the custom intent detection mode.
    pub fn custom_intent_mode(mut self, mode: CustomIntentMode) -> Self {
        self.0.custom_intent_mode = Some(mode);
        self
    }

    /// Add custom intents for the model to detect.
    ///
    /// Calling this repeatedly appends to the existing custom intents.
    pub fn custom_intents<'a>(mut self, intents: impl IntoIterator<Item = &'a str>) -> Self {
        self.0
            .custom_intents
            .extend(intents.into_iter().map(String::from));
        self
    }

    /// Set the language of the input text.
    ///
    /// Defaults to [`Language::en`], which is sent on every request because
    /// the Text Intelligence API requires the parameter. Only English is
    /// supported by the Text Intelligence API at this time, so there is
    /// normally no reason to call this.
    pub fn language(mut self, language: Language) -> Self {
        self.0.language = Some(language);
        self
    }

    /// Add tags to label the request for usage reporting.
    ///
    /// Calling this repeatedly appends to the existing tags.
    pub fn tag<'a>(mut self, tags: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.tags.extend(tags.into_iter().map(String::from));
        self
    }

    /// Set the HTTP method Deepgram uses to deliver the callback.
    ///
    /// Only meaningful with [`TextIntelligence::analyze_text_callback`] or
    /// [`TextIntelligence::analyze_url_callback`]. Defaults to `POST` on the server when
    /// omitted.
    ///
    /// See the [Deepgram Text Intelligence Callback docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-intelligence-callback
    /// [`TextIntelligence::analyze_text_callback`]: crate::read::TextIntelligence::analyze_text_callback
    /// [`TextIntelligence::analyze_url_callback`]: crate::read::TextIntelligence::analyze_url_callback
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

impl Serialize for SerializableOptions<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;

        // Destructuring ensures we don't forget to serialize any new field.
        let Options {
            sentiment,
            summarize,
            topics,
            custom_topics,
            custom_topic_mode,
            intents,
            custom_intents,
            custom_intent_mode,
            language,
            tags,
            callback_method,
        } = self.0;

        if let Some(sentiment) = sentiment {
            seq.serialize_element(&("sentiment", sentiment))?;
        }
        if let Some(summarize) = summarize {
            seq.serialize_element(&("summarize", summarize))?;
        }
        if let Some(topics) = topics {
            seq.serialize_element(&("topics", topics))?;
        }
        if let Some(custom_topic_mode) = custom_topic_mode {
            seq.serialize_element(&("custom_topic_mode", custom_topic_mode))?;
        }
        for custom_topic in custom_topics {
            seq.serialize_element(&("custom_topic", custom_topic))?;
        }
        if let Some(intents) = intents {
            seq.serialize_element(&("intents", intents))?;
        }
        if let Some(custom_intent_mode) = custom_intent_mode {
            seq.serialize_element(&("custom_intent_mode", custom_intent_mode))?;
        }
        for custom_intent in custom_intents {
            seq.serialize_element(&("custom_intent", custom_intent))?;
        }
        if let Some(language) = language {
            seq.serialize_element(&("language", language.as_ref()))?;
        }
        for tag in tags {
            seq.serialize_element(&("tag", tag))?;
        }
        if let Some(callback_method) = callback_method {
            seq.serialize_element(&("callback_method", callback_method.as_str()))?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_flags_and_repeats() {
        let options = Options::builder()
            .sentiment(true)
            .summarize(true)
            .topics(true)
            .custom_topics(["refund", "billing"])
            .custom_topic_mode(CustomTopicMode::Strict)
            .intents(true)
            .language(Language::en)
            .tag(["team:cx"])
            .build();
        assert_eq!(
            options.urlencoded().unwrap(),
            "sentiment=true&summarize=true&topics=true&custom_topic_mode=strict\
             &custom_topic=refund&custom_topic=billing&intents=true&language=en&tag=team%3Acx"
        );
    }

    #[test]
    fn summarize_is_boolean_not_v2() {
        // The /v1/read endpoint accepts a boolean, unlike the transcription API
        // which serializes `summarize=v2`.
        let options = Options::builder().summarize(true).build();
        assert_eq!(options.urlencoded().unwrap(), "summarize=true&language=en");
    }

    #[test]
    fn default_options_send_language_en() {
        // /v1/read rejects requests without `language` (HTTP 400), and English
        // is the only value it accepts, so the default builder must send it.
        let options = Options::builder().build();
        assert_eq!(options.urlencoded().unwrap(), "language=en");
    }

    #[test]
    fn callback_method_serializes_lowercase() {
        let options = Options::builder()
            .callback_method(CallbackMethod::PUT)
            .build();
        assert_eq!(
            options.urlencoded().unwrap(),
            "language=en&callback_method=put"
        );
    }
}
