//! Set various Deepgram features to control how Flux TTS speech is
//! generated.
//!
//! See the [Deepgram Flux TTS feature docs][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/docs/text-to-speech/flux/feature-overview

use serde::{ser::SerializeSeq, Serialize};

pub use super::super::options::{Container, Encoding};

/// Used as a parameter for [`OptionsBuilder::new`].
///
/// Flux TTS model strings follow the format `flux-{voice}-{language}`
/// (e.g. `flux-haley-en`). Aura model strings are rejected on `/v2/speak`;
/// use `/v1/speak` for Aura voices.
///
/// See the [Deepgram Flux TTS voices docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/text-to-speech/flux/voices
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Model {
    #[allow(missing_docs)]
    FluxAlexisEn,

    #[allow(missing_docs)]
    FluxBreeEn,

    #[allow(missing_docs)]
    FluxBrittanyEn,

    #[allow(missing_docs)]
    FluxBrookeEn,

    #[allow(missing_docs)]
    FluxBruceEn,

    #[allow(missing_docs)]
    FluxCliffEn,

    #[allow(missing_docs)]
    FluxColeEn,

    #[allow(missing_docs)]
    FluxColinEn,

    #[allow(missing_docs)]
    FluxConorEn,

    #[allow(missing_docs)]
    FluxDonovanEn,

    #[allow(missing_docs)]
    FluxDrewEn,

    #[allow(missing_docs)]
    FluxEliseEn,

    #[allow(missing_docs)]
    FluxGemmaEn,

    #[allow(missing_docs)]
    FluxHaleyEn,

    #[allow(missing_docs)]
    FluxHannahEn,

    #[allow(missing_docs)]
    FluxHeatherEn,

    #[allow(missing_docs)]
    FluxJackEn,

    #[allow(missing_docs)]
    FluxKaiEn,

    #[allow(missing_docs)]
    FluxKelseyEn,

    #[allow(missing_docs)]
    FluxKitEn,

    #[allow(missing_docs)]
    FluxMaeveEn,

    #[allow(missing_docs)]
    FluxMarceloEn,

    #[allow(missing_docs)]
    FluxMarcusEn,

    #[allow(missing_docs)]
    FluxMeenaEn,

    #[allow(missing_docs)]
    FluxMeghanEn,

    #[allow(missing_docs)]
    FluxMilesEn,

    #[allow(missing_docs)]
    FluxNaveenEn,

    #[allow(missing_docs)]
    FluxPaigeEn,

    #[allow(missing_docs)]
    FluxPriyaEn,

    #[allow(missing_docs)]
    FluxRufusEn,

    #[allow(missing_docs)]
    FluxSeanEn,

    #[allow(missing_docs)]
    FluxSharonEn,

    #[allow(missing_docs)]
    FluxSiennaEn,

    #[allow(missing_docs)]
    FluxTannerEn,

    #[allow(missing_docs)]
    FluxWadeEn,

    #[allow(missing_docs)]
    FluxWesEn,

    #[allow(missing_docs)]
    CustomId(String),
}

impl AsRef<str> for Model {
    fn as_ref(&self) -> &str {
        match self {
            Self::FluxAlexisEn => "flux-alexis-en",
            Self::FluxBreeEn => "flux-bree-en",
            Self::FluxBrittanyEn => "flux-brittany-en",
            Self::FluxBrookeEn => "flux-brooke-en",
            Self::FluxBruceEn => "flux-bruce-en",
            Self::FluxCliffEn => "flux-cliff-en",
            Self::FluxColeEn => "flux-cole-en",
            Self::FluxColinEn => "flux-colin-en",
            Self::FluxConorEn => "flux-conor-en",
            Self::FluxDonovanEn => "flux-donovan-en",
            Self::FluxDrewEn => "flux-drew-en",
            Self::FluxEliseEn => "flux-elise-en",
            Self::FluxGemmaEn => "flux-gemma-en",
            Self::FluxHaleyEn => "flux-haley-en",
            Self::FluxHannahEn => "flux-hannah-en",
            Self::FluxHeatherEn => "flux-heather-en",
            Self::FluxJackEn => "flux-jack-en",
            Self::FluxKaiEn => "flux-kai-en",
            Self::FluxKelseyEn => "flux-kelsey-en",
            Self::FluxKitEn => "flux-kit-en",
            Self::FluxMaeveEn => "flux-maeve-en",
            Self::FluxMarceloEn => "flux-marcelo-en",
            Self::FluxMarcusEn => "flux-marcus-en",
            Self::FluxMeenaEn => "flux-meena-en",
            Self::FluxMeghanEn => "flux-meghan-en",
            Self::FluxMilesEn => "flux-miles-en",
            Self::FluxNaveenEn => "flux-naveen-en",
            Self::FluxPaigeEn => "flux-paige-en",
            Self::FluxPriyaEn => "flux-priya-en",
            Self::FluxRufusEn => "flux-rufus-en",
            Self::FluxSeanEn => "flux-sean-en",
            Self::FluxSharonEn => "flux-sharon-en",
            Self::FluxSiennaEn => "flux-sienna-en",
            Self::FluxTannerEn => "flux-tanner-en",
            Self::FluxWadeEn => "flux-wade-en",
            Self::FluxWesEn => "flux-wes-en",
            Self::CustomId(id) => id,
        }
    }
}

/// HTTP method for asynchronous callback delivery, used with
/// [`OptionsBuilder::callback`]. REST (batch) transport only.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum CallbackMethod {
    /// Deliver the callback with an HTTP POST (the default).
    Post,

    /// Deliver the callback with an HTTP PUT.
    Put,
}

impl CallbackMethod {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            CallbackMethod::Post => "POST",
            CallbackMethod::Put => "PUT",
        }
    }
}

/// Used as a parameter for
/// [`Speak::flux_speak_to_file`](crate::Speak::flux_speak_to_file),
/// [`Speak::flux_speak_to_stream`](crate::Speak::flux_speak_to_stream),
/// and [`Speak::flux_request`](crate::Speak::flux_request).
///
/// The `model` is required — Flux TTS has no default model. All other
/// fields are optional.
///
/// The `container`, `bit_rate`, `callback`, `callback_method`, and
/// `priority` options apply to the REST (batch) transport only; the
/// `/v2/speak` WebSocket rejects a builder with any of them set. The
/// WebSocket also emits raw (non-containerized) audio only, so of the
/// encodings only `linear16`, `mulaw`, and `alaw` are valid there;
/// compressed encodings (`mp3`, `opus`, `flac`, `aac`) are REST-only.
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    pub(super) model: Model,
    pub(super) encoding: Option<Encoding>,
    pub(super) sample_rate: Option<u32>,
    pub(super) speed: Option<f64>,
    pub(super) expressivity: Option<i32>,
    pub(super) mip_opt_out: Option<bool>,
    pub(super) tags: Vec<String>,
    pub(super) container: Option<Container>,
    pub(super) bit_rate: Option<u32>,
    pub(super) callback: Option<String>,
    pub(super) callback_method: Option<CallbackMethod>,
    pub(super) priority_low: bool,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Debug, PartialEq, Clone)]
pub(super) struct SerializableOptions<'a>(pub(super) &'a Options);

impl Options {
    /// Construct a new [`OptionsBuilder`] for the given Flux TTS model.
    pub fn builder(model: Model) -> OptionsBuilder {
        OptionsBuilder::new(model)
    }

    /// Return the Options in urlencoded format. If serialization would
    /// fail, this will also return an error.
    ///
    /// This is intended primarily to help with debugging API requests.
    ///
    /// ```
    /// use deepgram::speak::flux::options::{Encoding, Model, Options};
    /// let options = Options::builder(Model::FluxHaleyEn)
    ///     .encoding(Encoding::Linear16)
    ///     .sample_rate(24000)
    ///     .build();
    /// assert_eq!(
    ///     &options.urlencoded().unwrap(),
    ///     "model=flux-haley-en&encoding=linear16&sample_rate=24000"
    /// )
    /// ```
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions(self))
    }

    /// Whether any REST-only option (`container`, `bit_rate`, `callback`,
    /// `callback_method`, `priority`) is set. The `/v2/speak` WebSocket
    /// does not accept these.
    pub(super) fn rest_only_options_set(&self) -> Option<&'static str> {
        if self.container.is_some() {
            Some("container")
        } else if self.bit_rate.is_some() {
            Some("bit_rate")
        } else if self.callback.is_some() {
            Some("callback")
        } else if self.callback_method.is_some() {
            Some("callback_method")
        } else if self.priority_low {
            Some("priority")
        } else {
            None
        }
    }

    /// Whether the configured encoding is one of the known REST-only
    /// compressed encodings. The `/v2/speak` WebSocket emits raw audio
    /// only (`linear16`, `mulaw`, `alaw`). [`Encoding::CustomEncoding`]
    /// is not checked — unknown values are left for the server to
    /// validate.
    pub(super) fn rest_only_encoding_set(&self) -> Option<&'static str> {
        match self.encoding {
            Some(Encoding::Mp3) => Some("mp3"),
            Some(Encoding::Opus) => Some("opus"),
            Some(Encoding::Flac) => Some("flac"),
            Some(Encoding::Aac) => Some("aac"),
            _ => None,
        }
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`] for the given Flux TTS model.
    pub fn new(model: Model) -> Self {
        Self(Options {
            model,
            encoding: None,
            sample_rate: None,
            speed: None,
            expressivity: None,
            mip_opt_out: None,
            tags: Vec::new(),
            container: None,
            bit_rate: None,
            callback: None,
            callback_method: None,
            priority_low: false,
        })
    }

    /// Set the Encoding feature.
    ///
    /// The WebSocket transport emits raw audio only, so only `linear16`,
    /// `mulaw`, and `alaw` are valid there. The REST (batch) transport
    /// additionally accepts `mp3` (its default), `opus`, `flac`, and `aac`.
    pub fn encoding(mut self, encoding: Encoding) -> Self {
        self.0.encoding = Some(encoding);
        self
    }

    /// Set the Sample Rate feature, in Hz.
    ///
    /// Supported values depend on the encoding. With `linear16`: `8000`,
    /// `16000`, `24000`, `32000`, `44100`, or `48000`. With `mulaw` or
    /// `alaw`: `8000` or `16000`. Defaults to the model's native sample
    /// rate.
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.0.sample_rate = Some(sample_rate);
        self
    }

    /// Set the speech-rate multiplier. `1.0` is the model's nominal rate;
    /// lower is slower. Accepted values run `0.5` to `1.5` in `0.05`
    /// increments. Not supported by every model or language.
    ///
    /// On the WebSocket transport this can also be changed mid-session
    /// with [`FluxSpeakHandle::configure_speed`](super::websocket::FluxSpeakHandle::configure_speed).
    pub fn speed(mut self, speed: f64) -> Self {
        self.0.speed = Some(speed);
        self
    }

    /// Set the expressive range of the generated speech, on a
    /// calm-to-animated axis. Accepted values: `-2`, `-1`, `0`, `1`, `2`.
    /// `0` (the default) is the voice's tuned delivery, with `-2` the calm
    /// end of the range and `2` the animated end.
    ///
    /// Beta: behavior may change in future model versions, and
    /// non-default values increase the risk of hallucinations and
    /// pronunciation errors; audition before shipping. Fixed for the
    /// connection — not settable mid-session.
    pub fn expressivity(mut self, expressivity: i32) -> Self {
        self.0.expressivity = Some(expressivity);
        self
    }

    /// Opt out of the Deepgram Model Improvement Program. Refer to the
    /// [MIP docs](https://dpgr.am/deepgram-mip) for pricing impacts
    /// before setting this to true.
    pub fn mip_opt_out(mut self, mip_opt_out: bool) -> Self {
        self.0.mip_opt_out = Some(mip_opt_out);
        self
    }

    /// Set the Tag feature. Labels requests for identification during
    /// usage reporting. Repeatable — each tag is sent as a separate
    /// `tag` query parameter.
    pub fn tag<'a>(mut self, tags: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.tags.extend(tags.into_iter().map(String::from));
        self
    }

    /// Set the Container feature — the file format wrapper for the
    /// output audio. REST (batch) transport only.
    pub fn container(mut self, container: Container) -> Self {
        self.0.container = Some(container);
        self
    }

    /// Set the Bit Rate feature, in bits per second, for compressed
    /// encodings. REST (batch) transport only.
    pub fn bit_rate(mut self, bit_rate: u32) -> Self {
        self.0.bit_rate = Some(bit_rate);
        self
    }

    /// Set a callback URL for asynchronous delivery. The request is
    /// processed asynchronously and the audio is delivered to this URL;
    /// the immediate response body is instead a JSON acknowledgement of
    /// the form `{"request_id": "..."}`. REST (batch) transport only.
    pub fn callback(mut self, callback: impl Into<String>) -> Self {
        self.0.callback = Some(callback.into());
        self
    }

    /// Set the HTTP method for the callback request (default: POST).
    /// REST (batch) transport only.
    pub fn callback_method(mut self, callback_method: CallbackMethod) -> Self {
        self.0.callback_method = Some(callback_method);
        self
    }

    /// Request low processing priority. Applies only to asynchronous
    /// (callback) requests; `low` is the only supported value. REST
    /// (batch) transport only.
    pub fn priority_low(mut self) -> Self {
        self.0.priority_low = true;
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options {
        self.0
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
            speed,
            expressivity,
            mip_opt_out,
            tags,
            container,
            bit_rate,
            callback,
            callback_method,
            priority_low,
        } = self.0;

        seq.serialize_element(&("model", model.as_ref()))?;

        if let Some(encoding) = encoding {
            seq.serialize_element(&("encoding", encoding.as_str()))?;
        }

        if let Some(sample_rate) = sample_rate {
            seq.serialize_element(&("sample_rate", sample_rate))?;
        }

        if let Some(speed) = speed {
            seq.serialize_element(&("speed", speed))?;
        }

        if let Some(expressivity) = expressivity {
            seq.serialize_element(&("expressivity", expressivity))?;
        }

        if let Some(mip_opt_out) = mip_opt_out {
            seq.serialize_element(&("mip_opt_out", mip_opt_out))?;
        }

        for tag in tags {
            seq.serialize_element(&("tag", tag))?;
        }

        if let Some(container) = container {
            seq.serialize_element(&("container", container.as_str()))?;
        }

        if let Some(bit_rate) = bit_rate {
            seq.serialize_element(&("bit_rate", bit_rate))?;
        }

        if let Some(callback) = callback {
            seq.serialize_element(&("callback", callback))?;
        }

        if let Some(callback_method) = callback_method {
            seq.serialize_element(&("callback_method", callback_method.as_str()))?;
        }

        if *priority_low {
            seq.serialize_element(&("priority", "low"))?;
        }

        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_required_always_serialized() {
        let options = Options::builder(Model::FluxHaleyEn).build();
        assert_eq!(options.urlencoded().unwrap(), "model=flux-haley-en");
    }

    #[test]
    fn all_options_serialized() {
        let options = Options::builder(Model::CustomId("flux-custom-en".to_string()))
            .encoding(Encoding::Mp3)
            .sample_rate(48000)
            .speed(1.05)
            .expressivity(-1)
            .mip_opt_out(true)
            .tag(["prod", "client-xyz"])
            .container(Container::None)
            .bit_rate(48000)
            .callback("https://example.com/hook")
            .callback_method(CallbackMethod::Put)
            .priority_low()
            .build();
        assert_eq!(
            options.urlencoded().unwrap(),
            "model=flux-custom-en&encoding=mp3&sample_rate=48000&speed=1.05&expressivity=-1&mip_opt_out=true&tag=prod&tag=client-xyz&container=none&bit_rate=48000&callback=https%3A%2F%2Fexample.com%2Fhook&callback_method=PUT&priority=low"
        );
    }

    #[test]
    fn rest_only_detection() {
        let ws_ok = Options::builder(Model::FluxHaleyEn)
            .encoding(Encoding::Linear16)
            .sample_rate(24000)
            .speed(0.95)
            .expressivity(1)
            .mip_opt_out(false)
            .tag(["a"])
            .build();
        assert_eq!(ws_ok.rest_only_options_set(), None);

        let with_container = Options::builder(Model::FluxHaleyEn)
            .container(Container::Wav)
            .build();
        assert_eq!(with_container.rest_only_options_set(), Some("container"));

        let with_callback = Options::builder(Model::FluxHaleyEn)
            .callback("https://example.com")
            .build();
        assert_eq!(with_callback.rest_only_options_set(), Some("callback"));

        let with_priority = Options::builder(Model::FluxHaleyEn).priority_low().build();
        assert_eq!(with_priority.rest_only_options_set(), Some("priority"));
    }

    #[test]
    fn rest_only_encoding_detection() {
        for (encoding, name) in [
            (Encoding::Mp3, "mp3"),
            (Encoding::Opus, "opus"),
            (Encoding::Flac, "flac"),
            (Encoding::Aac, "aac"),
        ] {
            let options = Options::builder(Model::FluxHaleyEn)
                .encoding(encoding)
                .build();
            assert_eq!(options.rest_only_encoding_set(), Some(name));
        }

        // Raw encodings pass, no encoding passes, and custom encodings
        // are left for the server to validate.
        for options in [
            Options::builder(Model::FluxHaleyEn).build(),
            Options::builder(Model::FluxHaleyEn)
                .encoding(Encoding::Linear16)
                .build(),
            Options::builder(Model::FluxHaleyEn)
                .encoding(Encoding::Mulaw)
                .build(),
            Options::builder(Model::FluxHaleyEn)
                .encoding(Encoding::Alaw)
                .build(),
            Options::builder(Model::FluxHaleyEn)
                .encoding(Encoding::CustomEncoding("linear32".to_string()))
                .build(),
        ] {
            assert_eq!(options.rest_only_encoding_set(), None);
        }
    }
}
