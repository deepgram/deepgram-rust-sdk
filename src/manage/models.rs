//! List the STT and TTS models available to you or to a project.
//!
//! See the [Deepgram Model Metadata guide][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/guides/fundamentals/model-metadata

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
        let url = "https://api.deepgram.com/v1/models";
        let request = self
            .0
            .client
            .get(url)
            .query(&[("include_outdated", include_outdated)]);
        send_and_translate_response(request).await
    }

    /// Get metadata on a specific public model by its UUID.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/manage/models/get
    pub async fn get_model(&self, model_id: &str) -> crate::Result<Model> {
        let url = format!("https://api.deepgram.com/v1/models/{model_id}");
        send_and_translate_response(self.0.client.get(url)).await
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
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/models");
        let request = self
            .0
            .client
            .get(url)
            .query(&[("include_outdated", include_outdated)]);
        send_and_translate_response(request).await
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
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/models/{model_id}");
        send_and_translate_response(self.0.client.get(url)).await
    }
}
