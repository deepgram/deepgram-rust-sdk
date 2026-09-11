//! Manage self-hosted (on-prem) container distribution credentials.
//!
//! These credentials let a self-hosted Deepgram deployment pull container
//! images from the distribution registry. Construct a [`SelfHosted`] client
//! with [`Deepgram::self_hosted`].
//!
//! See the [Deepgram Self-Hosted API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/list

use reqwest::RequestBuilder;
use serde::Serialize;

use crate::{send_and_translate_response, Deepgram};

use response::{
    CreatedDistributionCredentials, DistributionCredentialsEntry, DistributionCredentialsList,
    Message,
};

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

/// The default distribution provider.
pub const DEFAULT_PROVIDER: &str = "quay";

/// The default permission scope granted to new distribution credentials.
pub const DEFAULT_SCOPE: &str = "self-hosted:products";

/// A request to create a set of distribution credentials.
///
/// The API requires a `comment`, so construct the request with
/// [`CreateDistributionCredentials::new`] and adjust the optional fields with
/// the builder methods. All fields are sent in the JSON request body.
///
/// ```
/// use deepgram::manage::self_hosted::CreateDistributionCredentials;
///
/// let request = CreateDistributionCredentials::new("us-east-1 cluster")
///     .scopes(["self-hosted:product:api", "self-hosted:product:engine"])
///     .tags(["prod"]);
/// assert_eq!(request.provider, "quay");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct CreateDistributionCredentials {
    /// The provider of the distribution service. Defaults to
    /// [`DEFAULT_PROVIDER`] (`quay`), currently the only supported value.
    pub provider: String,

    /// A comment describing the credentials (required by the API). Use it to
    /// record which deployment the credentials are for.
    pub comment: String,

    /// The permission scopes to grant. Defaults to [`DEFAULT_SCOPE`]
    /// (`self-hosted:products`), which covers every product image.
    ///
    /// The API accepts these values:
    ///
    /// | Scope | Grants access to |
    /// |---|---|
    /// | `self-hosted:products` | all self-hosted product images |
    /// | `self-hosted:product:api` | the API container |
    /// | `self-hosted:product:engine` | the Engine container |
    /// | `self-hosted:product:license-proxy` | the License Proxy container |
    /// | `self-hosted:product:dgtools` | the `dgtools` container |
    /// | `self-hosted:product:billing` | the Billing container |
    /// | `self-hosted:product:hotpepper` | the Hotpepper container |
    /// | `self-hosted:product:metrics-server` | the Metrics Server container |
    ///
    /// See the [Deepgram API Reference][api] for the current list.
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/create
    pub scopes: Vec<String>,

    /// Optional tags to attach to the credentials. Omitted from the request
    /// when `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl CreateDistributionCredentials {
    /// Construct a request with the given comment, the default provider
    /// ([`DEFAULT_PROVIDER`]) and the default scopes ([`DEFAULT_SCOPE`]).
    pub fn new(comment: impl Into<String>) -> Self {
        Self {
            provider: DEFAULT_PROVIDER.to_string(),
            comment: comment.into(),
            scopes: vec![DEFAULT_SCOPE.to_string()],
            tags: None,
        }
    }

    /// Set the comment.
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = comment.into();
        self
    }

    /// Replace the permission scopes (the default is [`DEFAULT_SCOPE`]). See
    /// [`scopes`](Self::scopes) for the accepted values.
    pub fn scopes<S: Into<String>>(mut self, scopes: impl IntoIterator<Item = S>) -> Self {
        self.scopes = scopes.into_iter().map(Into::into).collect();
        self
    }

    /// Set the provider (the default is [`DEFAULT_PROVIDER`]).
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = provider.into();
        self
    }

    /// Set the tags to attach to the credentials.
    pub fn tags<S: Into<String>>(mut self, tags: impl IntoIterator<Item = S>) -> Self {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
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
    /// The `provider`, `comment`, `scopes`, and optional `tags` are sent in the
    /// JSON request body. The response is the new credentials object including
    /// the registry `username` and `secret`; **the secret is returned exactly
    /// once** and cannot be retrieved again, so store it as soon as this call
    /// returns. See [`CreatedDistributionCredentials`].
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/create
    pub async fn create_distribution_credentials(
        &self,
        project_id: &str,
        request: &CreateDistributionCredentials,
    ) -> crate::Result<CreatedDistributionCredentials> {
        send_and_translate_response(
            self.create_distribution_credentials_request(project_id, request),
        )
        .await
    }

    fn create_distribution_credentials_request(
        &self,
        project_id: &str,
        request: &CreateDistributionCredentials,
    ) -> RequestBuilder {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{project_id}/self-hosted/distribution/credentials"
        );
        self.0.client.post(url).json(request)
    }

    /// Delete a set of distribution credentials by its UUID.
    ///
    /// On success the API returns a confirmation [`Message`] (for example
    /// `"Successfully deleted the distribution credentials!"`).
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

#[cfg(test)]
mod tests {
    use super::{CreateDistributionCredentials, DEFAULT_PROVIDER, DEFAULT_SCOPE};
    use crate::Deepgram;

    fn body_json(request: &reqwest::Request) -> serde_json::Value {
        let bytes = request
            .body()
            .expect("request has a body")
            .as_bytes()
            .expect("body is buffered");
        serde_json::from_slice(bytes).expect("body is JSON")
    }

    #[test]
    fn create_request_defaults() {
        let request = CreateDistributionCredentials::new("primary cluster");
        assert_eq!(request.provider, DEFAULT_PROVIDER);
        assert_eq!(request.comment, "primary cluster");
        assert_eq!(request.scopes, [DEFAULT_SCOPE]);
        assert_eq!(request.tags, None);
    }

    #[test]
    fn create_request_builder_replaces_scopes_and_sets_tags() {
        let request = CreateDistributionCredentials::new("c")
            .scopes(["self-hosted:product:api", "self-hosted:product:engine"])
            .tags(["prod", "us-east"])
            .provider("quay")
            .comment("renamed");
        assert_eq!(
            request.scopes,
            ["self-hosted:product:api", "self-hosted:product:engine"]
        );
        assert_eq!(
            request.tags,
            Some(vec!["prod".to_string(), "us-east".to_string()])
        );
        assert_eq!(request.comment, "renamed");
    }

    #[test]
    fn create_sends_provider_comment_scopes_in_json_body_with_no_query() {
        // The API server reads `provider`, `comment`, and `scopes` from the
        // JSON body (all required) and does not read any query parameters.
        let dg = Deepgram::new("token").unwrap();
        let request = CreateDistributionCredentials::new("my deployment");
        let built = dg
            .self_hosted()
            .create_distribution_credentials_request("proj-123", &request)
            .build()
            .unwrap();

        assert_eq!(built.method(), reqwest::Method::POST);
        assert_eq!(
            built.url().as_str(),
            "https://api.deepgram.com/v1/projects/proj-123/self-hosted/distribution/credentials"
        );
        assert_eq!(built.url().query(), None, "no query parameters");
        assert_eq!(
            built.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = body_json(&built);
        assert_eq!(
            body,
            serde_json::json!({
                "provider": "quay",
                "comment": "my deployment",
                "scopes": ["self-hosted:products"],
            })
        );
    }

    #[test]
    fn create_sends_custom_scopes_and_tags_in_json_body() {
        let dg = Deepgram::new("token").unwrap();
        let request = CreateDistributionCredentials::new("engine only")
            .scopes(["self-hosted:product:engine"])
            .tags(["staging"]);
        let built = dg
            .self_hosted()
            .create_distribution_credentials_request("proj-123", &request)
            .build()
            .unwrap();

        assert_eq!(built.url().query(), None);
        assert_eq!(
            body_json(&built),
            serde_json::json!({
                "provider": "quay",
                "comment": "engine only",
                "scopes": ["self-hosted:product:engine"],
                "tags": ["staging"],
            })
        );
    }
}
