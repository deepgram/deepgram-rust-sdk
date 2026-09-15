//! REST Text Intelligence (`/v1/read`) requests.

use reqwest::RequestBuilder;
use serde_json::{json, Value};
use url::Url;

use crate::common::batch_response::CallbackResponse;
use crate::read::TextIntelligence;
use crate::send_and_translate_response;

use super::options::{Options, SerializableOptions};
use super::response::Response;

static DEEPGRAM_API_URL_READ: &str = "v1/read";

impl TextIntelligence<'_> {
    /// Analyze a block of plain text.
    ///
    /// To have Deepgram deliver the result to a webhook instead of returning
    /// it inline, use [`TextIntelligence::analyze_text_callback`].
    ///
    /// See the [Deepgram Text Intelligence docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-intelligence
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// # use deepgram::{read::options::Options, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let options = Options::builder().sentiment(true).topics(true).build();
    ///
    /// let response = dg_client
    ///     .text_intelligence()
    ///     .analyze_text("The weather today is lovely.", &options)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_text(&self, text: &str, options: &Options) -> crate::Result<Response> {
        let request_builder = self.make_read_request_builder(json!({ "text": text }), options);
        send_and_translate_response(request_builder).await
    }

    /// Analyze the text at a hosted URL (a plain-text document).
    ///
    /// To have Deepgram deliver the result to a webhook instead of returning
    /// it inline, use [`TextIntelligence::analyze_url_callback`].
    ///
    /// See the [Deepgram Text Intelligence docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-intelligence
    pub async fn analyze_url(&self, url: &str, options: &Options) -> crate::Result<Response> {
        let request_builder = self.make_read_request_builder(json!({ "url": url }), options);
        send_and_translate_response(request_builder).await
    }

    /// Analyze a block of plain text, delivering the result to `callback`.
    ///
    /// Deepgram acknowledges the request immediately with a
    /// [`CallbackResponse`] carrying the `request_id`, then sends the full
    /// [`Response`] to the callback URL when the analysis finishes. Set the
    /// delivery method with
    /// [`OptionsBuilder::callback_method`](super::options::OptionsBuilder::callback_method).
    ///
    /// See the [Deepgram Text Intelligence Callback docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-intelligence-callback
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// # use deepgram::{read::options::{CallbackMethod, Options}, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key = env::var("DEEPGRAM_API_KEY").unwrap_or_default();
    /// # let callback_url = env::var("DEEPGRAM_CALLBACK_URL").unwrap_or_default();
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let options = Options::builder()
    ///     .sentiment(true)
    ///     .callback_method(CallbackMethod::POST)
    ///     .build();
    ///
    /// let ack = dg_client
    ///     .text_intelligence()
    ///     .analyze_text_callback("The weather today is lovely.", &options, &callback_url)
    ///     .await?;
    /// println!("request id: {}", ack.request_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_text_callback(
        &self,
        text: &str,
        options: &Options,
        callback: &str,
    ) -> crate::Result<CallbackResponse> {
        let request_builder =
            self.make_read_callback_request_builder(json!({ "text": text }), options, callback);
        send_and_translate_response(request_builder).await
    }

    /// Analyze the text at a hosted URL, delivering the result to `callback`.
    ///
    /// Behaves like [`TextIntelligence::analyze_text_callback`] but for a hosted
    /// plain-text document; see that method for details.
    ///
    /// See the [Deepgram Text Intelligence Callback docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-intelligence-callback
    pub async fn analyze_url_callback(
        &self,
        url: &str,
        options: &Options,
        callback: &str,
    ) -> crate::Result<CallbackResponse> {
        let request_builder =
            self.make_read_callback_request_builder(json!({ "url": url }), options, callback);
        send_and_translate_response(request_builder).await
    }

    /// Build the `/v1/read` [`reqwest::RequestBuilder`] without sending it.
    ///
    /// Prefer [`TextIntelligence::analyze_text`] or [`TextIntelligence::analyze_url`]; this is exposed
    /// for callers that need to customize the request, for example to append
    /// extra query parameters with [`RequestBuilder::query`].
    pub fn make_read_request_builder(&self, body: Value, options: &Options) -> RequestBuilder {
        self.0
            .client
            .post(self.read_url())
            .query(&SerializableOptions(options))
            .json(&body)
    }

    /// Like [`TextIntelligence::make_read_request_builder`], but appends the `callback`
    /// query parameter for a [callback request][callback].
    ///
    /// Prefer [`TextIntelligence::analyze_text_callback`] or [`TextIntelligence::analyze_url_callback`].
    ///
    /// [callback]: https://developers.deepgram.com/docs/text-intelligence-callback
    pub fn make_read_callback_request_builder(
        &self,
        body: Value,
        options: &Options,
        callback: &str,
    ) -> RequestBuilder {
        self.make_read_request_builder(body, options)
            .query(&[("callback", callback)])
    }

    fn read_url(&self) -> Url {
        self.0.base_url.join(DEEPGRAM_API_URL_READ).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::read::options::{CallbackMethod, Options};
    use crate::Deepgram;

    #[test]
    fn read_url() {
        let dg = Deepgram::new("token").unwrap();
        assert_eq!(
            &dg.text_intelligence().read_url().to_string(),
            "https://api.deepgram.com/v1/read"
        );
    }

    #[test]
    fn callback_request_carries_callback_and_method_on_the_wire() {
        let dg = Deepgram::new("token").unwrap();
        let options = Options::builder()
            .sentiment(true)
            .callback_method(CallbackMethod::PUT)
            .build();

        let request = dg
            .text_intelligence()
            .make_read_callback_request_builder(
                json!({ "text": "hello" }),
                &options,
                "https://example.com/hook",
            )
            .build()
            .unwrap();

        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(request.url().path(), "/v1/read");
        let query = request.url().query().unwrap();
        assert!(
            query.contains("callback=https%3A%2F%2Fexample.com%2Fhook"),
            "query was: {query}"
        );
        assert!(query.contains("callback_method=put"), "query was: {query}");
        assert!(query.contains("language=en"), "query was: {query}");
        assert!(query.contains("sentiment=true"), "query was: {query}");
    }
}
