//! List the STT and TTS models available to you or to a project.
//!
//! See the [Deepgram Model Metadata guide][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/guides/fundamentals/model-metadata

use reqwest::RequestBuilder;
use url::Url;

use crate::{send_and_translate_response, Deepgram};

use response::{Model, ModelsResponse};

pub mod response;

/// List the models available to you or to a project.
///
/// Constructed using [`Deepgram::models`].
///
/// See the [Deepgram Model Metadata guide][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/guides/fundamentals/model-metadata
#[derive(Debug, Clone)]
pub struct Models<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Models`] from a [`Deepgram`].
    pub fn models(&self) -> Models<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Models<'a> {
    /// Construct a new [`Models`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Models<'_> {
    /// List metadata on all the latest public models.
    ///
    /// To also include non-latest (outdated) versions, use
    /// [`Models::get_models_including_outdated`].
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/models/list
    pub async fn get_models(&self) -> crate::Result<ModelsResponse> {
        send_and_translate_response(self.get_models_request(false)).await
    }

    /// List metadata on all public models, including non-latest (outdated)
    /// versions.
    ///
    /// Sends `include_outdated=true`. For the latest versions only, use
    /// [`Models::get_models`].
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/models/list
    pub async fn get_models_including_outdated(&self) -> crate::Result<ModelsResponse> {
        send_and_translate_response(self.get_models_request(true)).await
    }

    /// Get metadata on a specific public model by its UUID.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/models/get
    pub async fn get_model(&self, model_id: &str) -> crate::Result<Model> {
        send_and_translate_response(self.get_model_request(model_id)).await
    }

    /// List metadata on all the latest models a project has access to,
    /// including non-public (custom) models.
    ///
    /// To also include non-latest (outdated) versions, use
    /// [`Models::get_project_models_including_outdated`].
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/projects/models/list
    pub async fn get_project_models(&self, project_id: &str) -> crate::Result<ModelsResponse> {
        send_and_translate_response(self.get_project_models_request(project_id, false)).await
    }

    /// List metadata on all the models a project has access to, including
    /// non-public (custom) models and non-latest (outdated) versions.
    ///
    /// Sends `include_outdated=true`. For the latest versions only, use
    /// [`Models::get_project_models`].
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/projects/models/list
    pub async fn get_project_models_including_outdated(
        &self,
        project_id: &str,
    ) -> crate::Result<ModelsResponse> {
        send_and_translate_response(self.get_project_models_request(project_id, true)).await
    }

    /// Get metadata for a specific model a project has access to.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/projects/models/get
    pub async fn get_project_model(
        &self,
        project_id: &str,
        model_id: &str,
    ) -> crate::Result<Model> {
        send_and_translate_response(self.get_project_model_request(project_id, model_id)).await
    }

    /// `include_outdated` is optional on the endpoint and defaults to the
    /// latest versions only, so it is sent only when it is `true`.
    fn get_models_request(&self, include_outdated: bool) -> RequestBuilder {
        let request = self.0.client.get(self.models_url());

        if include_outdated {
            request.query(&[("include_outdated", "true")])
        } else {
            request
        }
    }

    fn get_model_request(&self, model_id: &str) -> RequestBuilder {
        self.0.client.get(self.model_url(model_id))
    }

    /// `include_outdated` is sent only when `true`, as in
    /// `get_models_request`.
    fn get_project_models_request(
        &self,
        project_id: &str,
        include_outdated: bool,
    ) -> RequestBuilder {
        let request = self.0.client.get(self.project_models_url(project_id));

        if include_outdated {
            request.query(&[("include_outdated", "true")])
        } else {
            request
        }
    }

    fn get_project_model_request(&self, project_id: &str, model_id: &str) -> RequestBuilder {
        self.0
            .client
            .get(self.project_model_url(project_id, model_id))
    }

    fn models_url(&self) -> Url {
        self.join("v1/models")
    }

    fn model_url(&self, model_id: &str) -> Url {
        self.join(&format!("v1/models/{model_id}"))
    }

    fn project_models_url(&self, project_id: &str) -> Url {
        self.join(&format!("v1/projects/{project_id}/models"))
    }

    fn project_model_url(&self, project_id: &str, model_id: &str) -> Url {
        self.join(&format!("v1/projects/{project_id}/models/{model_id}"))
    }

    fn join(&self, path: &str) -> Url {
        self.0
            .base_url
            .join(path)
            .expect("base_url is checked to be a valid base_url when constructing Deepgram client")
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    /// Pin each of the four model request URLs. A path typo would otherwise
    /// only surface against the live API.
    #[test]
    fn request_urls() {
        let dg = Deepgram::new("token").unwrap();
        let models = dg.models();

        let url = |request: reqwest::RequestBuilder| request.build().unwrap().url().to_string();

        // `include_outdated` is optional on both listing endpoints and the
        // endpoint default is the latest versions only, so `get_models` and
        // `get_project_models` omit the parameter entirely and only the
        // `_including_outdated` variants send it.
        assert_eq!(
            url(models.get_models_request(false)),
            "https://api.deepgram.com/v1/models"
        );
        assert_eq!(
            url(models.get_models_request(true)),
            "https://api.deepgram.com/v1/models?include_outdated=true"
        );
        assert_eq!(
            url(models.get_model_request("4899aa60-f731-4d2b-b1fd-fa5d6a2dc98b")),
            "https://api.deepgram.com/v1/models/4899aa60-f731-4d2b-b1fd-fa5d6a2dc98b"
        );
        assert_eq!(
            url(models.get_project_models_request("proj-1", false)),
            "https://api.deepgram.com/v1/projects/proj-1/models"
        );
        assert_eq!(
            url(models.get_project_models_request("proj-1", true)),
            "https://api.deepgram.com/v1/projects/proj-1/models?include_outdated=true"
        );
        assert_eq!(
            url(models.get_project_model_request("proj-1", "model-2")),
            "https://api.deepgram.com/v1/projects/proj-1/models/model-2"
        );
    }

    #[test]
    fn request_methods_are_get() {
        let dg = Deepgram::new("token").unwrap();
        let models = dg.models();

        for request in [
            models.get_models_request(false),
            models.get_model_request("model-1"),
            models.get_project_models_request("proj-1", false),
            models.get_project_model_request("proj-1", "model-1"),
        ] {
            assert_eq!(request.build().unwrap().method(), reqwest::Method::GET);
        }
    }

    /// A custom base URL must reach the management endpoints too: before this,
    /// every `manage` module hardcoded `https://api.deepgram.com` and silently
    /// ignored `with_base_url`.
    #[test]
    fn request_urls_honor_a_custom_base_url() {
        let dg =
            Deepgram::with_base_url_and_api_key("http://localhost:8888/abc/", "token").unwrap();
        let models = dg.models();

        let url = |request: reqwest::RequestBuilder| request.build().unwrap().url().to_string();

        assert_eq!(
            url(models.get_models_request(false)),
            "http://localhost:8888/abc/v1/models"
        );
        assert_eq!(
            url(models.get_model_request("model-2")),
            "http://localhost:8888/abc/v1/models/model-2"
        );
        assert_eq!(
            url(models.get_project_models_request("proj-1", true)),
            "http://localhost:8888/abc/v1/projects/proj-1/models?include_outdated=true"
        );
        assert_eq!(
            url(models.get_project_model_request("proj-1", "model-2")),
            "http://localhost:8888/abc/v1/projects/proj-1/models/model-2"
        );
    }

    /// A base URL that carries a path prefix has to end in `/` for the prefix
    /// to survive: `Url::join` treats a base whose path does not end in `/` as
    /// naming a file and replaces its last segment, so `.../abc` drops `abc`.
    /// That is standard `Url::join` behavior and is how every surface in this
    /// crate builds its URLs, not something specific to `models`. A bare host
    /// (`http://localhost:8888`) is unaffected, because its path is already
    /// `/`.
    #[test]
    fn a_base_url_path_prefix_needs_a_trailing_slash() {
        let url = |request: reqwest::RequestBuilder| request.build().unwrap().url().to_string();

        let no_prefix =
            Deepgram::with_base_url_and_api_key("http://localhost:8888", "token").unwrap();
        assert_eq!(
            url(no_prefix.models().get_models_request(false)),
            "http://localhost:8888/v1/models"
        );

        let unterminated_prefix =
            Deepgram::with_base_url_and_api_key("http://localhost:8888/abc", "token").unwrap();
        assert_eq!(
            url(unterminated_prefix.models().get_models_request(false)),
            "http://localhost:8888/v1/models"
        );
    }
}
