//! Read API REST endpoint — `POST /v1/read`.

use url::Url;

use crate::read::options::Options;
use crate::read::request::ReadRequest;
use crate::read::response::ReadResponse;
use crate::{DeepgramError, Read};

static READ_PATH: &str = "v1/read";

impl Read<'_> {
    /// Analyze the given text and return a [`ReadResponse`].
    ///
    /// The request body is either a URL or inline text — see
    /// [`ReadRequest::url`] / [`ReadRequest::text`]. The query string
    /// is built from `options`.
    ///
    /// # Errors
    ///
    /// - [`DeepgramError::DeepgramApiError`] if the API returns a
    ///   non-2xx status (the body is included verbatim in `body`).
    /// - [`DeepgramError::ReqwestError`] for transport-level failures.
    /// - [`DeepgramError::JsonError`] if the response body cannot be
    ///   parsed as a [`ReadResponse`].
    pub async fn analyze(
        &self,
        request: &ReadRequest,
        options: &Options,
    ) -> Result<ReadResponse, DeepgramError> {
        let response = self
            .0
            .client
            .post(self.read_url())
            .query(options)
            .json(request)
            .send()
            .await?;

        if let Err(err) = response.error_for_status_ref() {
            let body = response.text().await.unwrap_or_default();
            return Err(DeepgramError::DeepgramApiError { body, err });
        }

        let body = response.json::<ReadResponse>().await?;
        Ok(body)
    }

    fn read_url(&self) -> Url {
        self.0
            .base_url
            .join(READ_PATH)
            .expect("base_url is validated to be a valid base URL when constructing Deepgram")
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn read_url_resolves_against_base() {
        let dg = Deepgram::new("test-key").unwrap();
        let read = dg.read();
        let url = read.read_url();
        assert_eq!(url.as_str(), "https://api.deepgram.com/v1/read");
    }

    #[test]
    fn read_url_resolves_against_self_hosted_base() {
        let dg = Deepgram::with_base_url_and_api_key("http://localhost:8080", "test-key").unwrap();
        let read = dg.read();
        let url = read.read_url();
        assert_eq!(url.as_str(), "http://localhost:8080/v1/read");
    }
}
