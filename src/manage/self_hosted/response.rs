//! Response types for the self-hosted distribution credentials endpoints.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// A set of self-hosted distribution credentials.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DistributionCredentials {
    #[allow(missing_docs)]
    pub distribution_credentials_id: Uuid,

    /// The provider of the distribution service (e.g. `quay`).
    pub provider: String,

    /// An optional comment describing the credentials.
    pub comment: Option<String>,

    /// The permission scopes granted to the credentials.
    #[serde(default)]
    pub scopes: Vec<String>,

    /// The timestamp when the credentials were created.
    pub created: String,
}

/// A simple message response, returned when deleting credentials.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Message {
    #[allow(missing_docs)]
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::DistributionCredentialsList;

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
}
