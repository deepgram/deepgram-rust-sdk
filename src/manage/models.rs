//! List and look up Deepgram models, both public and project-scoped.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/manage-api/get-models

use crate::{
    manage::models::response::{ListModelsResponse, ModelInfo},
    send_and_translate_response, Deepgram,
};

pub mod list_options;
pub mod response;

/// Sub-client for the model-listing endpoints. Constructed via
/// [`Deepgram::models`].
#[derive(Debug, Clone)]
pub struct Models<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Models`] sub-client.
    pub fn models(&self) -> Models<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Models<'a> {
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Models<'_> {
    /// `GET /v1/models` — metadata for all latest public models.
    ///
    /// To list project-scoped (including non-public) models, use
    /// [`Models::list_for_project`].
    pub async fn list(&self, options: &list_options::Options) -> crate::Result<ListModelsResponse> {
        let url = "https://api.deepgram.com/v1/models";
        let mut request = self.0.client.get(url);
        let pairs = options.query_pairs();
        if !pairs.is_empty() {
            request = request.query(&pairs);
        }
        send_and_translate_response(request).await
    }

    /// `GET /v1/models/{model_id}` — metadata for one public model.
    pub async fn get(&self, model_id: &str) -> crate::Result<ModelInfo> {
        let url = format!("https://api.deepgram.com/v1/models/{model_id}");
        send_and_translate_response(self.0.client.get(url)).await
    }

    /// `GET /v1/projects/{project_id}/models` — metadata for all
    /// models the project has access to, including non-public ones.
    pub async fn list_for_project(
        &self,
        project_id: &str,
        options: &list_options::Options,
    ) -> crate::Result<ListModelsResponse> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/models");
        let mut request = self.0.client.get(url);
        let pairs = options.query_pairs();
        if !pairs.is_empty() {
            request = request.query(&pairs);
        }
        send_and_translate_response(request).await
    }

    /// `GET /v1/projects/{project_id}/models/{model_id}` — metadata
    /// for a single project-scoped model.
    pub async fn get_for_project(
        &self,
        project_id: &str,
        model_id: &str,
    ) -> crate::Result<ModelInfo> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/models/{model_id}");
        send_and_translate_response(self.0.client.get(url)).await
    }
}
