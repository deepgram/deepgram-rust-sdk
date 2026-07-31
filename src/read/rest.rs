//! REST Text Intelligence (`/v1/read`) requests.

use reqwest::RequestBuilder;
use serde_json::{json, Value};
use url::Url;

use crate::read::Read;
use crate::send_and_translate_response;

use super::options::{Options, SerializableOptions};
use super::response::Response;

static DEEPGRAM_API_URL_READ: &str = "v1/read";

impl Read<'_> {
    /// Analyze a block of plain text.
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
    /// See the [Deepgram Text Intelligence docs][docs] for more info.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-intelligence
    pub async fn analyze_url(&self, url: &str, options: &Options) -> crate::Result<Response> {
        let request_builder = self.make_read_request_builder(json!({ "url": url }), options);
        send_and_translate_response(request_builder).await
    }

    /// Build the `/v1/read` [`reqwest::RequestBuilder`] without sending it.
    ///
    /// Prefer [`Read::analyze_text`] or [`Read::analyze_url`]; this is exposed
    /// for callers that need to customize the request.
    pub fn make_read_request_builder(&self, body: Value, options: &Options) -> RequestBuilder {
        self.0
            .client
            .post(self.read_url())
            .query(&SerializableOptions(options))
            .json(&body)
    }

    fn read_url(&self) -> Url {
        self.0.base_url.join(DEEPGRAM_API_URL_READ).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn read_url() {
        let dg = Deepgram::new("token").unwrap();
        assert_eq!(
            &dg.text_intelligence().read_url().to_string(),
            "https://api.deepgram.com/v1/read"
        );
    }
}
