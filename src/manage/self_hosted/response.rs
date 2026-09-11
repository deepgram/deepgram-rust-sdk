//! Response types for the self-hosted distribution credentials endpoints.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Success message.
///
/// Returned by
/// [`SelfHosted::delete_distribution_credentials`](super::SelfHosted::delete_distribution_credentials).
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Message {
    #[allow(missing_docs)]
    pub message: String,
}

/// A list of distribution credentials for a project.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DistributionCredentialsList {
    /// The distribution credentials, each with its associated member.
    #[serde(default)]
    pub distribution_credentials: Vec<DistributionCredentialsEntry>,
}

/// A single set of distribution credentials together with the member that owns
/// them.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DistributionCredentialsEntry {
    #[allow(missing_docs)]
    pub member: Member,

    #[allow(missing_docs)]
    pub distribution_credentials: DistributionCredentials,
}

/// The member associated with a set of distribution credentials.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Member {
    #[allow(missing_docs)]
    pub member_id: Uuid,

    #[allow(missing_docs)]
    pub email: String,
}

/// A set of self-hosted distribution credentials, as returned by the list and
/// get endpoints.
///
/// The registry `username` and `secret` are only ever returned by the create
/// endpoint (see [`CreatedDistributionCredentials`]); they cannot be retrieved
/// again afterwards.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DistributionCredentials {
    #[allow(missing_docs)]
    pub distribution_credentials_id: Uuid,

    /// The provider of the distribution service (e.g. `quay`).
    pub provider: String,

    /// A comment describing the credentials.
    pub comment: Option<String>,

    /// The permission scopes granted to the credentials.
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Tags attached to the credentials. The server omits this field when it
    /// is empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// The timestamp when the credentials were created.
    pub created: String,
}

/// A freshly created set of self-hosted distribution credentials.
///
/// Returned by
/// [`SelfHosted::create_distribution_credentials`](super::SelfHosted::create_distribution_credentials).
/// Unlike the list and get responses, this is a bare credentials object (no
/// `member` wrapper) and it carries the registry `username` and `secret`.
///
/// **The `secret` is shown exactly once.** It cannot be retrieved again through
/// the API or the Console, so store it securely as soon as it is returned. The
/// secret is wrapped in a [`SecretString`], whose `Debug` output is redacted;
/// read it with [`SecretString::expose_secret`].
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreatedDistributionCredentials {
    #[allow(missing_docs)]
    pub distribution_credentials_id: Uuid,

    /// The provider of the distribution service (e.g. `quay`).
    pub provider: String,

    /// The registry username to log in with. Only returned on create.
    pub username: Option<String>,

    /// The registry password to log in with. Only returned on create, and
    /// never retrievable again.
    pub secret: Option<SecretString>,

    /// A comment describing the credentials.
    pub comment: Option<String>,

    /// The permission scopes granted to the credentials.
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Tags attached to the credentials. The server omits this field when it
    /// is empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// The timestamp when the credentials were created.
    pub created: String,
}

/// A string that holds a secret and redacts itself when formatted with
/// `Debug`, so that a `{:?}` of a response (for example in a `tracing` field)
/// never writes the secret to a log.
///
/// Read the value explicitly with [`expose_secret`](Self::expose_secret).
/// `Serialize` writes the real value, so the secret survives a round-trip
/// through `serde_json` for callers who persist it deliberately.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    /// Wrap a secret.
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    /// Return the wrapped secret.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }

    /// Consume the wrapper and return the wrapped secret.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl From<String> for SecretString {
    fn from(secret: String) -> Self {
        Self(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CreatedDistributionCredentials, DistributionCredentialsEntry, DistributionCredentialsList,
        Message, SecretString,
    };

    #[test]
    fn deserializes_documented_response() {
        // Shape from the Deepgram Self-Hosted API docs.
        let json = serde_json::json!({
            "distribution_credentials": [{
                "member": {
                    "member_id": "3376abcd-8e5e-49d3-92d4-876d3a4f0363",
                    "email": "email@example.com"
                },
                "distribution_credentials": {
                    "distribution_credentials_id": "8b36cfd0-472f-4a21-833f-2d6343c3a2f3",
                    "provider": "quay",
                    "scopes": ["self-hosted:product:api", "self-hosted:product:engine"],
                    "created": "2023-06-28T15:36:59.609841Z",
                    "comment": "My Self-Hosted Distribution Credentials"
                }
            }]
        });

        let list: DistributionCredentialsList = serde_json::from_value(json).unwrap();
        assert_eq!(list.distribution_credentials.len(), 1);
        let entry = &list.distribution_credentials[0];
        assert_eq!(entry.member.email, "email@example.com");
        assert_eq!(entry.distribution_credentials.provider, "quay");
        assert_eq!(entry.distribution_credentials.scopes.len(), 2);
        assert_eq!(
            entry.distribution_credentials.comment.as_deref(),
            Some("My Self-Hosted Distribution Credentials")
        );
        // `tags` is omitted by the server when empty.
        assert!(entry.distribution_credentials.tags.is_empty());
    }

    #[test]
    fn deserializes_single_entry_response() {
        // `get` returns one `{ member, distribution_credentials }` entry.
        let json = serde_json::json!({
            "member": {
                "member_id": "3376abcd-8e5e-49d3-92d4-876d3a4f0363",
                "email": "email@example.com"
            },
            "distribution_credentials": {
                "distribution_credentials_id": "8b36cfd0-472f-4a21-833f-2d6343c3a2f3",
                "provider": "quay",
                "scopes": ["self-hosted:product:api"],
                "tags": ["prod", "us-east"],
                "created": "2023-06-28T15:36:59.609841Z",
                "comment": "primary"
            }
        });

        let entry: DistributionCredentialsEntry = serde_json::from_value(json).unwrap();
        assert_eq!(entry.member.email, "email@example.com");
        assert_eq!(
            entry
                .distribution_credentials
                .distribution_credentials_id
                .to_string(),
            "8b36cfd0-472f-4a21-833f-2d6343c3a2f3"
        );
        assert_eq!(
            entry.distribution_credentials.comment.as_deref(),
            Some("primary")
        );
        assert_eq!(entry.distribution_credentials.tags, ["prod", "us-east"]);
    }

    #[test]
    fn deserializes_response_without_comment() {
        // `comment` is optional.
        let json = serde_json::json!({
            "distribution_credentials": [{
                "member": { "member_id": "3376abcd-8e5e-49d3-92d4-876d3a4f0363", "email": "e@x.com" },
                "distribution_credentials": {
                    "distribution_credentials_id": "8b36cfd0-472f-4a21-833f-2d6343c3a2f3",
                    "provider": "quay",
                    "scopes": [],
                    "created": "2023-06-28T15:36:59.609841Z"
                }
            }]
        });
        let list: DistributionCredentialsList = serde_json::from_value(json).unwrap();
        assert!(list.distribution_credentials[0]
            .distribution_credentials
            .comment
            .is_none());
    }

    #[test]
    fn deserializes_create_response_with_one_time_secret() {
        // The API server answers `create` with a bare credentials object (no
        // `member` wrapper) that carries the one-time `username` / `secret`.
        let json = serde_json::json!({
            "distribution_credentials_id": "8b36cfd0-472f-4a21-833f-2d6343c3a2f3",
            "provider": "quay",
            "username": "deepgram+8b36cfd0",
            "secret": "s3cr3t-registry-password",
            "comment": "created from the rust sdk",
            "scopes": ["self-hosted:products"],
            "created": "2023-06-28T15:36:59.609841Z"
        });

        let created: CreatedDistributionCredentials = serde_json::from_value(json).unwrap();
        assert_eq!(
            created.distribution_credentials_id.to_string(),
            "8b36cfd0-472f-4a21-833f-2d6343c3a2f3"
        );
        assert_eq!(created.provider, "quay");
        assert_eq!(created.username.as_deref(), Some("deepgram+8b36cfd0"));
        assert_eq!(
            created.secret.as_ref().map(SecretString::expose_secret),
            Some("s3cr3t-registry-password")
        );
        assert_eq!(
            created.comment.as_deref(),
            Some("created from the rust sdk")
        );
        assert_eq!(created.scopes, ["self-hosted:products"]);
        assert!(created.tags.is_empty());

        // The secret must never reach Debug output.
        let debug = format!("{created:?}");
        assert!(
            !debug.contains("s3cr3t-registry-password"),
            "secret leaked into Debug: {debug}"
        );
        assert!(debug.contains("***"), "{debug}");

        // ...but it does survive a deliberate serialization round-trip.
        let json = serde_json::to_value(&created).unwrap();
        assert_eq!(json["secret"], "s3cr3t-registry-password");
        assert!(json.get("tags").is_none(), "empty tags are omitted");
    }

    #[test]
    fn deserializes_create_response_with_tags() {
        let json = serde_json::json!({
            "distribution_credentials_id": "8b36cfd0-472f-4a21-833f-2d6343c3a2f3",
            "provider": "quay",
            "username": "u",
            "secret": "s",
            "comment": "",
            "scopes": ["self-hosted:product:api", "self-hosted:product:engine"],
            "tags": ["staging"],
            "created": "2023-06-28T15:36:59.609841Z"
        });
        let created: CreatedDistributionCredentials = serde_json::from_value(json).unwrap();
        assert_eq!(created.tags, ["staging"]);
        assert_eq!(created.scopes.len(), 2);
    }

    #[test]
    fn deserializes_delete_message() {
        // The API server answers `delete` with a message, not the entry.
        let json = serde_json::json!({
            "message": "Successfully deleted the distribution credentials!"
        });
        let message: Message = serde_json::from_value(json).unwrap();
        assert_eq!(
            message.message,
            "Successfully deleted the distribution credentials!"
        );
    }

    #[test]
    fn secret_string_redacts_debug() {
        let secret = SecretString::new("hunter2");
        assert_eq!(format!("{secret:?}"), "***");
        assert_eq!(secret.expose_secret(), "hunter2");
        assert_eq!(secret.clone().into_inner(), "hunter2");
        assert_eq!(SecretString::from("hunter2".to_string()), secret);
    }
}
