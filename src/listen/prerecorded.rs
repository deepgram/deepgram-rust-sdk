//! Types used for pre-recorded audio transcription.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded

use reqwest::RequestBuilder;
use url::Url;

use crate::common::audio_source::AudioSource;
use crate::send_and_translate_response;

use crate::common::batch_response::{CallbackResponse, Response};
use crate::common::options::{Options, SerializableOptions};

use super::speech_to_text::Transcription;

static DEEPGRAM_API_URL_LISTEN: &str = "v1/listen";

impl Transcription<'_> {
    /// Sends a request to Deepgram to transcribe pre-recorded audio.
    /// If you wish to use the Callback feature, you should use [`Transcription::prerecorded_callback`] instead.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{
    /// #     common::{
    /// #         audio_source::AudioSource,
    /// #         options::{Language, Options},
    /// #     },
    /// #     Deepgram, DeepgramError,
    /// # };
    /// #
    /// # static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key);
    ///
    /// let source = AudioSource::from_url(AUDIO_URL);
    ///
    /// let options = Options::builder()
    ///     .punctuate(true)
    ///     .language(Language::en_US)
    ///     .build();
    ///
    /// let response = dg_client
    ///     .transcription()
    ///     .prerecorded(source, &options)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn prerecorded(
        &self,
        source: AudioSource,
        options: &Options,
    ) -> crate::Result<Response> {
        let request_builder = self.make_prerecorded_request_builder(source, options);

        send_and_translate_response(request_builder).await
    }

    /// Sends a request to Deepgram to transcribe pre-recorded audio using the Callback feature.
    /// Otherwise behaves similarly to [`Transcription::prerecorded`].
    ///
    /// See the [Deepgram Callback feature docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/documentation/features/callback/
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{
    /// #     common::{
    /// #         audio_source::AudioSource,
    /// #         options::{Language, Options},
    /// #     },
    /// #     Deepgram, DeepgramError,
    /// # };
    /// #
    /// # static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key);
    ///
    /// let source = AudioSource::from_url(AUDIO_URL);
    ///
    /// let options = Options::builder()
    ///     .punctuate(true)
    ///     .language(Language::en_US)
    ///     .build();
    ///
    /// # let callback_url =
    /// #     env::var("DEEPGRAM_CALLBACK_URL").expect("DEEPGRAM_CALLBACK_URL environmental variable");
    /// #
    /// let response = dg_client
    ///     .transcription()
    ///     .prerecorded_callback(source, &options, &callback_url)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn prerecorded_callback(
        &self,
        source: AudioSource,
        options: &Options,
        callback: &str,
    ) -> crate::Result<CallbackResponse> {
        let request_builder =
            self.make_prerecorded_callback_request_builder(source, options, callback);

        send_and_translate_response(request_builder).await
    }

    /// Makes a [`reqwest::RequestBuilder`] without actually sending the request.
    /// This allows you to modify the request before it is sent.
    ///
    /// Avoid using this where possible.
    /// By customizing the request, there is less of a guarantee that it will conform to the Deepgram API.
    /// Prefer using [`Transcription::prerecorded`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{
    /// #     common::{
    /// #         audio_source::AudioSource,
    /// #         options::{Language, Options},
    /// #         batch_response::{response::Response},
    /// #     },
    /// #     Deepgram, DeepgramError,
    /// # };
    /// #
    /// # static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> reqwest::Result<()> {
    /// #
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let dg_client = Deepgram::new(&deepgram_api_key);
    /// #
    /// # let source = AudioSource::from_url(AUDIO_URL);
    /// #
    /// # let options = Options::builder()
    /// #     .punctuate(true)
    /// #     .language(Language::en_US)
    /// #     .build();
    /// #
    /// let request_builder = dg_client
    ///     .transcription()
    ///     .make_prerecorded_request_builder(source, &options);
    ///
    /// // Customize the RequestBuilder here
    /// let customized_request_builder = request_builder
    ///     .query(&[("custom_query_key", "custom_query_value")])
    ///     .header("custom_header_key", "custom_header_value");
    ///
    /// // It is necessary to annotate the type of response here
    /// // That way it knows what type to deserialize the JSON into
    /// let response: Response = customized_request_builder.send().await?.json().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_prerecorded_request_builder(
        &self,
        source: AudioSource,
        options: &Options,
    ) -> RequestBuilder {
        let request_builder = self
            .0
            .client
            .post(self.listen_url())
            .query(&SerializableOptions(options));

        source.fill_body(request_builder)
    }

    /// Similar to [`Transcription::make_prerecorded_request_builder`],
    /// but for the purposes of a [callback request][callback].
    ///
    /// You should avoid using this where possible too, preferring [`Transcription::prerecorded_callback`].
    ///
    /// [callback]: https://developers.deepgram.com/documentation/features/callback/
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{
    /// #     common::{
    /// #         audio_source::AudioSource,
    /// #         options::{Language, Options},
    /// #         batch_response::{Response, CallbackResponse},
    /// #     },
    /// #     Deepgram, DeepgramError,
    /// # };
    /// #
    /// # static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> reqwest::Result<()> {
    /// #
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let dg_client = Deepgram::new(&deepgram_api_key);
    /// #
    /// # let source = AudioSource::from_url(AUDIO_URL);
    /// #
    /// # let options = Options::builder()
    /// #     .punctuate(true)
    /// #     .language(Language::en_US)
    /// #     .build();
    /// #
    /// # let callback_url =
    /// #     env::var("DEEPGRAM_CALLBACK_URL").expect("DEEPGRAM_CALLBACK_URL environmental variable");
    /// #
    /// let request_builder = dg_client
    ///     .transcription()
    ///     .make_prerecorded_callback_request_builder(source, &options, &callback_url);
    ///
    /// // Customize the RequestBuilder here
    /// let customized_request_builder = request_builder
    ///     .query(&[("custom_query_key", "custom_query_value")])
    ///     .header("custom_header_key", "custom_header_value");
    ///
    /// // It is necessary to annotate the type of response here
    /// // That way it knows what type to deserialize the JSON into
    /// let response: CallbackResponse = customized_request_builder.send().await?.json().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn make_prerecorded_callback_request_builder(
        &self,
        source: AudioSource,
        options: &Options,
        callback: &str,
    ) -> RequestBuilder {
        self.make_prerecorded_request_builder(source, options)
            .query(&[("callback", callback)])
    }

    fn listen_url(&self) -> Url {
        self.0.base_url.join(DEEPGRAM_API_URL_LISTEN).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn listen_url() {
        let dg = Deepgram::new("token");
        assert_eq!(
            &dg.transcription().listen_url().to_string(),
            "https://api.deepgram.com/v1/listen"
        );
    }

    #[test]
    fn listen_url_custom_host() {
        let dg = Deepgram::with_base_url("http://localhost:8888/abc/");
        assert_eq!(
            &dg.transcription().listen_url().to_string(),
            "http://localhost:8888/abc/v1/listen"
        );
    }
}
