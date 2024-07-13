//! Set various Deepgram features to control how the speech is generated.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/docs/tts-feature-overview

use serde::{ser::SerializeSeq, Serialize};

/// Used as a parameter for [`Speak::speak`](crate::speak::Speak::speak) and similar functions.
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    model: Option<String>,
    encoding: Option<String>,
    sample_rate: Option<i32>,
    container: Option<String>,
    bit_rate: Option<i32>,
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
            model: None,
            encoding: None,
            sample_rate: None,
            container: None,
            bit_rate: None,
        })
    }

    /// Set the Model feature.
    ///
    /// See the [Deepgram Model feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-models
    pub fn model(mut self, model: &str) -> Self {
        self.0.model = Some(model.into());
        self
    }

    /// Set the Encoding feature.
    ///
    /// See the [Deepgram Encoding feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-encoding
    pub fn encoding(mut self, encoding: &str) -> Self {
        self.0.encoding = Some(encoding.into());
        self
    }

    /// Set the Sample Rate feature.
    ///
    /// See the [Deepgram Sample Rate feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-sample-rate
    pub fn sample_rate(mut self, sample_rate: i32) -> Self {
        self.0.sample_rate = Some(sample_rate);
        self
    }

    /// Set the Container feature.
    ///
    /// See the [Deepgram Container docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-container
    pub fn container(mut self, container: &str) -> Self {
        self.0.container = Some(container.into());
        self
    }

    /// Set the Bit Rate feature.
    ///
    /// See the [Deepgram Bit Rate feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/tts-bit-rate
    pub fn bit_rate(mut self, bit_rate: i32) -> Self {
        self.0.bit_rate = Some(bit_rate);
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
            model,
            encoding,
            sample_rate,
            container,
            bit_rate,
        } = self.0;

        if let Some(model) = model {
            seq.serialize_element(&("model", model))?;
        }

        if let Some(encoding) = encoding {
            seq.serialize_element(&("encoding", encoding))?;
        }

        if let Some(sample_rate) = sample_rate {
            seq.serialize_element(&("sample_rate", sample_rate))?;
        }

        if let Some(container) = container {
            seq.serialize_element(&("container", container))?;
        }

        if let Some(bit_rate) = bit_rate {
            seq.serialize_element(&("bit_rate", bit_rate))?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod serialize_options_tests {
    use std::cmp;
    use std::env;

    use crate::Deepgram;

    fn check_serialization(options: &Options, expected: &str) {
        let deepgram_api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();

        let dg_client = Deepgram::new(deepgram_api_key);

        let request = dg_client
            .speak(&options)
            .build()
            .unwrap();

        let actual = request.url().query().unwrap_or("");

        assert_eq!(actual, expected);
    }

    #[test]
    fn all_options() {
        let options = Options::builder()
            .model("aura-asteria-en")
            .encoding("linear16")
            .sample_rate(16000)
            .container("wav")
            .bit_rate(32000)
            .build();

        check_serialization(&options, "model=aura-asteria-en&encoding=linear16&sample_rate=16000&container=wav&bit_rate=32000");
    }

    #[test]
    fn model() {

        check_serialization(
            &Options::builder()
                .model("aura-asteria-en")
                .build(),
            "model=aura-asteria-en",
        );
    }
}
