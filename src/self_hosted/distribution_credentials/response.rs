//! Response shapes for the self-hosted distribution credentials
//! endpoints.
//!
//! Mirrors `schemas.selfHosted.v1.yml`. The List, Create, and Get
//! responses share the same `member` + `distribution_credentials`
//! shape, so they're modeled as a single [`CredentialEntry`].

use serde::{Deserialize, Serialize};

/// Member who created/owns a credential set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Member {
    /// Member UUID.
    pub member_id: String,
    /// Member email.
    pub email: String,
}

/// A single set of distribution credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DistributionCredentialsInfo {
    /// Credentials UUID.
    pub distribution_credentials_id: String,
    /// Distribution provider — `quay` for the public catalog.
    pub provider: String,
    /// Optional comment supplied at creation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Permission scopes attached to the credentials.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// ISO 8601 creation timestamp.
    pub created: String,
}

/// `member` + `distribution_credentials` pair returned by every
/// endpoint in this module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CredentialEntry {
    /// Member who owns this credential set.
    pub member: Member,
    /// The credential set itself.
    pub distribution_credentials: DistributionCredentialsInfo,
}

/// Response from the list endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct ListCredentialsResponse {
    /// Credential entries, one per `member`/`distribution_credentials`
    /// pair.
    #[serde(default)]
    pub distribution_credentials: Vec<CredentialEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn list_response_round_trip() {
        let raw = json!({
            "distribution_credentials": [{
                "member": {
                    "member_id": "3376abcd-8e5e-49d3-92d4-876d3a4f0363",
                    "email": "user@example.com"
                },
                "distribution_credentials": {
                    "distribution_credentials_id": "8b36cfd0-472f-4a21-833f-2d6343c3a2f3",
                    "provider": "quay",
                    "comment": "ops",
                    "scopes": ["self-hosted:product:api"],
                    "created": "2023-06-28T15:36:59.609841Z"
                }
            }]
        });
        let resp: ListCredentialsResponse = serde_json::from_value(raw).unwrap();
        assert_eq!(resp.distribution_credentials.len(), 1);
        let entry = &resp.distribution_credentials[0];
        assert_eq!(entry.member.email, "user@example.com");
        assert_eq!(entry.distribution_credentials.provider, "quay");
        assert_eq!(entry.distribution_credentials.scopes.len(), 1);
    }

    #[test]
    fn entry_without_comment_round_trip() {
        let raw = json!({
            "member": {
                "member_id": "c7b9b131-73f3-11d9-8665-0b00d2e44b83",
                "email": "u@example.com"
            },
            "distribution_credentials": {
                "distribution_credentials_id": "82c32c10-53b2-4d23-993f-864b3d44502a",
                "provider": "quay",
                "scopes": ["self-hosted:products"],
                "created": "2023-06-28T15:36:59.609841Z"
            }
        });
        let entry: CredentialEntry = serde_json::from_value(raw).unwrap();
        assert!(entry.distribution_credentials.comment.is_none());
    }
}
