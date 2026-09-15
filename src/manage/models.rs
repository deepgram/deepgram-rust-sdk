//! List the STT and TTS models available to you or to a project.
//!
//! See the [Deepgram Model Metadata guide][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/guides/fundamentals/model-metadata

use reqwest::RequestBuilder;

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
    /// Pass `include_outdated = true` to also return non-latest versions.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/models/list
    pub async fn get_models(&self, include_outdated: bool) -> crate::Result<ModelsResponse> {
        send_and_translate_response(self.get_models_request(include_outdated)).await
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
    /// Pass `include_outdated = true` to also return non-latest versions.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/projects/models/list
    pub async fn get_project_models(
        &self,
        project_id: &str,
        include_outdated: bool,
    ) -> crate::Result<ModelsResponse> {
        send_and_translate_response(self.get_project_models_request(project_id, include_outdated))
            .await
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

    fn get_models_request(&self, include_outdated: bool) -> RequestBuilder {
        self.0
            .client
            .get("https://api.deepgram.com/v1/models")
            .query(&[("include_outdated", include_outdated)])
    }

    fn get_model_request(&self, model_id: &str) -> RequestBuilder {
        self.0
            .client
            .get(format!("https://api.deepgram.com/v1/models/{model_id}"))
    }

    fn get_project_models_request(
        &self,
        project_id: &str,
        include_outdated: bool,
    ) -> RequestBuilder {
        self.0
            .client
            .get(format!(
                "https://api.deepgram.com/v1/projects/{project_id}/models"
            ))
            .query(&[("include_outdated", include_outdated)])
    }

    fn get_project_model_request(&self, project_id: &str, model_id: &str) -> RequestBuilder {
        self.0.client.get(format!(
            "https://api.deepgram.com/v1/projects/{project_id}/models/{model_id}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    /// The four model endpoints hardcode their request URLs, so pin each one.
    /// A path typo would otherwise only surface against the live API.
    #[test]
    fn request_urls() {
        let dg = Deepgram::new("token").unwrap();
        let models = dg.models();

        let url = |request: reqwest::RequestBuilder| request.build().unwrap().url().to_string();

        assert_eq!(
            url(models.get_models_request(false)),
            "https://api.deepgram.com/v1/models?include_outdated=false"
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
}
