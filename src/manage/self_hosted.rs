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
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

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

/// The default permission scope granted to new distribution credentials:
/// [`Scope::Products`] (`self-hosted:products`).
pub const DEFAULT_SCOPE: Scope = Scope::Products;

/// A permission scope granted to a set of distribution credentials. It
/// decides which container images the credentials may pull.
///
/// This is an open enum: the API may add scopes over time, so a value this
/// version of the SDK does not name deserializes as [`Scope::Unknown`], which
/// preserves the wire string and re-serializes to it exactly. `Unknown` is
/// also the escape hatch for *sending* such a scope — build one with
/// `Scope::from("self-hosted:product:new-thing")` rather than waiting for an
/// SDK release.
///
/// See the [Deepgram API Reference][api] for the current list.
///
/// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/create
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scope {
    /// `self-hosted:products`: every self-hosted product image. This is the
    /// scope the API grants when a create request names none.
    Products,

    /// `self-hosted:product:api`: the API container image only.
    Api,

    /// `self-hosted:product:engine`: the Engine container image only.
    Engine,

    /// `self-hosted:product:license-proxy`: the License Proxy container
    /// image only.
    LicenseProxy,

    /// `self-hosted:product:dgtools`: the `dgtools` container image only.
    Dgtools,

    /// `self-hosted:product:billing`: the Billing container image only.
    Billing,

    /// `self-hosted:product:hotpepper`: the Hotpepper container image only.
    Hotpepper,

    /// `self-hosted:product:metrics-server`: the Metrics Server container
    /// image only.
    MetricsServer,

    /// A scope this version of the SDK does not name, held verbatim.
    ///
    /// An unrecognized scope from the server round-trips through this
    /// variant, and it is how a caller sends a scope the SDK has not caught
    /// up with yet. An `Unknown` holding the wire string of a named variant
    /// is *not* equal to that variant even though both serialize
    /// identically, so normalize through [`Scope::from`] (or compare
    /// [`as_str`](Self::as_str) values) rather than comparing constructed
    /// values directly.
    Unknown(String),
}

impl Scope {
    /// The wire representation of this scope.
    pub fn as_str(&self) -> &str {
        match self {
            Scope::Products => "self-hosted:products",
            Scope::Api => "self-hosted:product:api",
            Scope::Engine => "self-hosted:product:engine",
            Scope::LicenseProxy => "self-hosted:product:license-proxy",
            Scope::Dgtools => "self-hosted:product:dgtools",
            Scope::Billing => "self-hosted:product:billing",
            Scope::Hotpepper => "self-hosted:product:hotpepper",
            Scope::MetricsServer => "self-hosted:product:metrics-server",
            Scope::Unknown(scope) => scope,
        }
    }
}

impl AsRef<str> for Scope {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for Scope {
    /// Resolve a wire string (for example `"self-hosted:product:api"`) to its
    /// named variant, or [`Scope::Unknown`] if it is not recognized.
    fn from(value: &str) -> Self {
        match value {
            "self-hosted:products" => Scope::Products,
            "self-hosted:product:api" => Scope::Api,
            "self-hosted:product:engine" => Scope::Engine,
            "self-hosted:product:license-proxy" => Scope::LicenseProxy,
            "self-hosted:product:dgtools" => Scope::Dgtools,
            "self-hosted:product:billing" => Scope::Billing,
            "self-hosted:product:hotpepper" => Scope::Hotpepper,
            "self-hosted:product:metrics-server" => Scope::MetricsServer,
            other => Scope::Unknown(other.to_string()),
        }
    }
}

impl From<String> for Scope {
    fn from(value: String) -> Self {
        Scope::from(value.as_str())
    }
}

// Manual scalar (de)serialization: a scope is a plain string on the wire, and
// an unrecognized value must survive an exact round trip — a derived
// `#[serde(other)]` unit variant would collapse them all to one value.
impl Serialize for Scope {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Scope {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Scope::from(String::deserialize(deserializer)?))
    }
}

/// A request to create a set of distribution credentials.
///
/// The API requires a `comment`, so construct the request with
/// [`CreateDistributionCredentials::new`] and adjust the optional fields with
/// the builder methods. All fields are sent in the JSON request body.
///
/// ```
/// use deepgram::manage::self_hosted::{CreateDistributionCredentials, Scope};
///
/// let request = CreateDistributionCredentials::new("us-east-1 cluster")
///     .scopes([Scope::Api, Scope::Engine])
///     .tags(["prod"]);
/// assert_eq!(request.provider, "quay");
/// assert_eq!(request.scopes, [Scope::Api, Scope::Engine]);
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
    /// ([`Scope::Products`]), which covers every product image. See
    /// [`Scope`] for the accepted values.
    pub scopes: Vec<Scope>,

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
            scopes: vec![DEFAULT_SCOPE],
            tags: None,
        }
    }

    /// Set the comment.
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = comment.into();
        self
    }

    /// Replace the permission scopes (the default is [`DEFAULT_SCOPE`]).
    ///
    /// Accepts [`Scope`] values, or the wire strings they convert from:
    /// `.scopes(["self-hosted:product:api"])` is the same as
    /// `.scopes([Scope::Api])`.
    pub fn scopes<S: Into<Scope>>(mut self, scopes: impl IntoIterator<Item = S>) -> Self {
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
        send_and_translate_response(self.0.client.get(self.credentials_url(project_id))).await
    }

    /// Get a single set of distribution credentials by its UUID.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/self-hosted/distribution-credentials/get
    pub async fn get_distribution_credentials(
        &self,
        project_id: &str,
        distribution_credentials_id: Uuid,
    ) -> crate::Result<DistributionCredentialsEntry> {
        send_and_translate_response(
            self.0
                .client
                .get(self.credentials_id_url(project_id, distribution_credentials_id)),
        )
        .await
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
        self.0
            .client
            .post(self.credentials_url(project_id))
            .json(request)
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
        distribution_credentials_id: Uuid,
    ) -> crate::Result<Message> {
        send_and_translate_response(
            self.0
                .client
                .delete(self.credentials_id_url(project_id, distribution_credentials_id)),
        )
        .await
    }

    fn credentials_url(&self, project_id: &str) -> Url {
        self.0
            .base_url
            .join(&format!(
                "v1/projects/{project_id}/self-hosted/distribution/credentials"
            ))
            .expect("base_url is checked to be a valid base_url when constructing Deepgram client")
    }

    fn credentials_id_url(&self, project_id: &str, distribution_credentials_id: Uuid) -> Url {
        self.0
            .base_url
            .join(&format!(
                "v1/projects/{project_id}/self-hosted/distribution/credentials/{distribution_credentials_id}"
            ))
            .expect("base_url is checked to be a valid base_url when constructing Deepgram client")
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{CreateDistributionCredentials, Scope, DEFAULT_PROVIDER, DEFAULT_SCOPE};
    use crate::Deepgram;

    const CREDENTIALS_ID: Uuid = Uuid::from_u128(0x8b36cfd0_472f_4a21_833f_2d6343c3a2f3);

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
            .scopes([Scope::Api, Scope::Engine])
            .tags(["prod", "us-east"])
            .provider("quay")
            .comment("renamed");
        assert_eq!(request.scopes, [Scope::Api, Scope::Engine]);
        assert_eq!(
            request.tags,
            Some(vec!["prod".to_string(), "us-east".to_string()])
        );
        assert_eq!(request.comment, "renamed");
    }

    #[test]
    fn scopes_builder_also_accepts_wire_strings() {
        // `Into<Scope>` keeps the string form working, and resolves it to the
        // named variant rather than stashing it in `Unknown`.
        let request = CreateDistributionCredentials::new("c").scopes([
            "self-hosted:product:api",
            "self-hosted:product:not-invented-yet",
        ]);
        assert_eq!(
            request.scopes,
            [
                Scope::Api,
                Scope::Unknown("self-hosted:product:not-invented-yet".to_string())
            ]
        );
    }

    #[test]
    fn every_named_scope_round_trips_through_its_wire_string() {
        let scopes = [
            (Scope::Products, "self-hosted:products"),
            (Scope::Api, "self-hosted:product:api"),
            (Scope::Engine, "self-hosted:product:engine"),
            (Scope::LicenseProxy, "self-hosted:product:license-proxy"),
            (Scope::Dgtools, "self-hosted:product:dgtools"),
            (Scope::Billing, "self-hosted:product:billing"),
            (Scope::Hotpepper, "self-hosted:product:hotpepper"),
            (Scope::MetricsServer, "self-hosted:product:metrics-server"),
        ];
        for (scope, wire) in scopes {
            assert_eq!(scope.as_str(), wire);
            assert_eq!(scope.as_ref(), wire);
            assert_eq!(Scope::from(wire), scope);
            assert_eq!(Scope::from(wire.to_string()), scope);
            assert_eq!(serde_json::to_value(&scope).unwrap(), wire);
            assert_eq!(
                serde_json::from_value::<Scope>(serde_json::json!(wire)).unwrap(),
                scope
            );
        }
    }

    #[test]
    fn unknown_scope_round_trips_verbatim() {
        let scope: Scope =
            serde_json::from_value(serde_json::json!("self-hosted:product:not-invented-yet"))
                .unwrap();
        assert_eq!(
            scope,
            Scope::Unknown("self-hosted:product:not-invented-yet".to_string())
        );
        assert_eq!(scope.as_str(), "self-hosted:product:not-invented-yet");
        assert_eq!(
            serde_json::to_value(&scope).unwrap(),
            "self-hosted:product:not-invented-yet"
        );
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
            .scopes([Scope::Engine])
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

    #[test]
    fn credentials_urls() {
        let dg = Deepgram::new("token").unwrap();
        let self_hosted = dg.self_hosted();
        assert_eq!(
            self_hosted.credentials_url("proj-123").as_str(),
            "https://api.deepgram.com/v1/projects/proj-123/self-hosted/distribution/credentials"
        );
        assert_eq!(
            self_hosted
                .credentials_id_url("proj-123", CREDENTIALS_ID)
                .as_str(),
            "https://api.deepgram.com/v1/projects/proj-123/self-hosted/distribution/credentials/8b36cfd0-472f-4a21-833f-2d6343c3a2f3"
        );
    }

    #[test]
    fn credentials_urls_honor_a_custom_base_url() {
        // Self-hosted deployments point the client at their own host with
        // `Deepgram::with_base_url`; every request has to follow.
        let dg = Deepgram::with_base_url("http://localhost:8888/abc/").unwrap();
        let self_hosted = dg.self_hosted();
        assert_eq!(
            self_hosted.credentials_url("proj-123").as_str(),
            "http://localhost:8888/abc/v1/projects/proj-123/self-hosted/distribution/credentials"
        );
        assert_eq!(
            self_hosted
                .credentials_id_url("proj-123", CREDENTIALS_ID)
                .as_str(),
            "http://localhost:8888/abc/v1/projects/proj-123/self-hosted/distribution/credentials/8b36cfd0-472f-4a21-833f-2d6343c3a2f3"
        );

        let built = self_hosted
            .create_distribution_credentials_request(
                "proj-123",
                &CreateDistributionCredentials::new("c"),
            )
            .build()
            .unwrap();
        assert_eq!(
            built.url().as_str(),
            "http://localhost:8888/abc/v1/projects/proj-123/self-hosted/distribution/credentials"
        );
    }
}
