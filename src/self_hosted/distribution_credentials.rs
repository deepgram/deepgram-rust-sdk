//! Distribution credentials for self-hosted Deepgram deployments.
//!
//! Wraps `/v1/projects/{project_id}/self-hosted/distribution/credentials`.
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/self-hosted-api/list-credentials

use crate::{
    self_hosted::distribution_credentials::response::{CredentialEntry, ListCredentialsResponse},
    send_and_translate_response, Deepgram,
};

pub mod create_options;
pub mod response;

/// Sub-client for the self-hosted distribution credentials endpoints.
/// Constructed via [`Deepgram::distribution_credentials`].
#[derive(Debug, Clone)]
pub struct DistributionCredentials<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`DistributionCredentials`] sub-client.
    pub fn distribution_credentials(&self) -> DistributionCredentials<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for DistributionCredentials<'a> {
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl DistributionCredentials<'_> {
    /// `GET /v1/projects/{project_id}/self-hosted/distribution/credentials`.
    pub async fn list(&self, project_id: &str) -> crate::Result<ListCredentialsResponse> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials"
        );
        send_and_translate_response(self.0.client.get(url)).await
    }

    /// `POST /v1/projects/{project_id}/self-hosted/distribution/credentials`.
    ///
    /// `scopes` and `provider` from [`create_options::Options`] are
    /// sent as query params; `comment` is sent as the JSON body.
    pub async fn create(
        &self,
        project_id: &str,
        options: &create_options::Options,
    ) -> crate::Result<CredentialEntry> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials"
        );
        let mut request = self.0.client.post(url).json(&options.body());
        let pairs = options.query_pairs();
        if !pairs.is_empty() {
            request = request.query(&pairs);
        }
        send_and_translate_response(request).await
    }

    /// `GET /v1/projects/{project_id}/self-hosted/distribution/credentials/{credentials_id}`.
    pub async fn get(
        &self,
        project_id: &str,
        credentials_id: &str,
    ) -> crate::Result<CredentialEntry> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials/{credentials_id}"
        );
        send_and_translate_response(self.0.client.get(url)).await
    }

    /// `DELETE /v1/projects/{project_id}/self-hosted/distribution/credentials/{credentials_id}`.
    ///
    /// On success the API echoes back the deleted credential set.
    pub async fn delete(
        &self,
        project_id: &str,
        credentials_id: &str,
    ) -> crate::Result<CredentialEntry> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials/{credentials_id}"
        );
        send_and_translate_response(self.0.client.delete(url)).await
    }
}
