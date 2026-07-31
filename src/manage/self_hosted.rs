//! Manage self-hosted (on-prem) container distribution credentials.
//!
//! These credentials let a self-hosted Deepgram deployment pull container
//! images from the distribution registry. Construct a [`SelfHosted`] client
//! with [`Deepgram::self_hosted`].
//!
//! See the [Deepgram Self-Hosted API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/list

use serde::Serialize;

use crate::{send_and_translate_response, Deepgram};

use response::{DistributionCredentialsEntry, DistributionCredentialsList, Message};

pub mod response;

/// Manage self-hosted distribution credentials for a project.
///
/// Constructed using [`Deepgram::self_hosted`].
///
/// See the [Deepgram Self-Hosted API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/list
#[derive(Debug, Clone)]
pub struct SelfHosted<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`SelfHosted`] from a [`Deepgram`].
    pub fn self_hosted(&self) -> SelfHosted<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for SelfHosted<'a> {
    /// Construct a new [`SelfHosted`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

/// Options for creating a set of distribution credentials.
///
/// All fields are optional; the server defaults the provider to `quay` and the
/// scopes to `["self-hosted:products"]`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CreateDistributionCredentialsOptions {
    /// An optional comment describing the credentials.
    pub comment: Option<String>,

    /// The permission scopes to grant. See the [supported scopes][api].
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/create
    pub scopes: Vec<String>,

    /// The provider of the distribution service. Defaults to `quay`.
    pub provider: Option<String>,
}

impl CreateDistributionCredentialsOptions {
    /// Construct a new, empty set of options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the comment.
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Add permission scopes. Calling this repeatedly appends.
    pub fn scopes<'s>(mut self, scopes: impl IntoIterator<Item = &'s str>) -> Self {
        self.scopes.extend(scopes.into_iter().map(String::from));
        self
    }

    /// Set the provider (defaults to `quay`).
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }
}

impl SelfHosted<'_> {
    /// List all sets of distribution credentials for a project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/list
    pub async fn list_distribution_credentials(
        &self,
        project_id: &str,
    ) -> crate::Result<DistributionCredentialsList> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials"
        );
        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Get a single set of distribution credentials by its UUID.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/get
    pub async fn get_distribution_credentials(
        &self,
        project_id: &str,
        distribution_credentials_id: &str,
    ) -> crate::Result<DistributionCredentialsEntry> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials/{distribution_credentials_id}"
        );
        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Create a set of distribution credentials for a project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/create
    pub async fn create_distribution_credentials(
        &self,
        project_id: &str,
        options: &CreateDistributionCredentialsOptions,
    ) -> crate::Result<DistributionCredentialsEntry> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            comment: Option<&'a str>,
        }

        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials"
        );

        let mut query: Vec<(&str, &str)> = Vec::new();
        for scope in &options.scopes {
            query.push(("scopes", scope));
        }
        if let Some(provider) = &options.provider {
            query.push(("provider", provider));
        }

        let request = self.0.client.post(url).query(&query).json(&Body {
            comment: options.comment.as_deref(),
        });

        send_and_translate_response(request).await
    }

    /// Delete a set of distribution credentials by its UUID.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/delete
    pub async fn delete_distribution_credentials(
        &self,
        project_id: &str,
        distribution_credentials_id: &str,
    ) -> crate::Result<Message> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials/{distribution_credentials_id}"
        );
        send_and_translate_response(self.0.client.delete(url)).await
    }
}
