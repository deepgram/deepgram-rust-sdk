//! AWS credentials shared by AWS Bedrock (Think) and AWS Polly (Speak) providers.
//!
//! Two flavors supported by the Voice Agent: short-lived STS credentials
//! (require `session_token`) and long-lived IAM credentials.

use serde::{Deserialize, Serialize};

/// AWS credentials block.
///
/// All fields are required when used with AWS Polly. AWS Bedrock accepts
/// any subset, so consumers building Bedrock configs may want to wrap the
/// whole struct in `Option<>` rather than supplying partial credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AwsCredentials {
    /// Credential type — STS (short-lived) or IAM (long-lived).
    #[serde(rename = "type")]
    pub credentials_type: AwsCredentialsType,

    /// AWS region (e.g. `us-east-1`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// AWS access key ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,

    /// AWS secret access key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,

    /// AWS session token. Required for STS credentials.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
}

/// Distinguishes short-lived STS credentials from long-lived IAM credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum AwsCredentialsType {
    /// Short-lived AWS Security Token Service credentials. `session_token` is required.
    Sts,
    /// Long-lived IAM credentials.
    Iam,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn iam_round_trip() {
        let raw = json!({
            "type": "iam",
            "region": "us-east-1",
            "access_key_id": "AKIA000",
            "secret_access_key": "secret",
        });
        let creds: AwsCredentials = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(creds.credentials_type, AwsCredentialsType::Iam);
        assert_eq!(creds.session_token, None);
        let back = serde_json::to_value(&creds).unwrap();
        assert_eq!(back, raw);
    }

    #[test]
    fn sts_round_trip() {
        let raw = json!({
            "type": "sts",
            "region": "us-west-2",
            "access_key_id": "AKIA111",
            "secret_access_key": "secret",
            "session_token": "FwoG...",
        });
        let creds: AwsCredentials = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(creds.credentials_type, AwsCredentialsType::Sts);
        assert_eq!(creds.session_token.as_deref(), Some("FwoG..."));
        let back = serde_json::to_value(&creds).unwrap();
        assert_eq!(back, raw);
    }

    #[test]
    fn bedrock_minimal_credentials() {
        // Bedrock allows just the type field with the rest absent.
        let raw = json!({ "type": "iam" });
        let creds: AwsCredentials = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(creds.credentials_type, AwsCredentialsType::Iam);
        assert!(creds.region.is_none());
        let back = serde_json::to_value(&creds).unwrap();
        assert_eq!(back, raw);
    }
}
