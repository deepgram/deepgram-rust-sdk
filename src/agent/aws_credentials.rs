//! AWS credentials shared by AWS Bedrock (Think) and AWS Polly (Speak) providers.
//!
//! Two flavors supported by the Voice Agent: short-lived STS credentials
//! (require `session_token`) and long-lived IAM credentials.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::agent::endpoint::REDACTED;

/// AWS credentials block.
///
/// All fields are required when used with AWS Polly. AWS Bedrock accepts
/// any subset, so consumers building Bedrock configs may want to wrap the
/// whole struct in `Option<>` rather than supplying partial credentials.
///
/// The `Debug` output redacts `access_key_id`, `secret_access_key`, and
/// `session_token` (printing `"<redacted>"` when set) so that logging a
/// `Settings`, `ThinkSettings`, or `SpeakSettings` never leaks AWS
/// credentials. `region` and the credential type are shown. Serialization
/// is unaffected.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl fmt::Debug for AwsCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AwsCredentials")
            .field("credentials_type", &self.credentials_type)
            .field("region", &self.region)
            .field("access_key_id", &redacted(&self.access_key_id))
            .field("secret_access_key", &redacted(&self.secret_access_key))
            .field("session_token", &redacted(&self.session_token))
            .finish()
    }
}

/// `None` stays `None`; `Some(secret)` becomes `Some("<redacted>")`.
fn redacted(value: &Option<String>) -> Option<&'static str> {
    value.as_ref().map(|_| REDACTED)
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

    #[test]
    fn debug_redacts_secrets_but_keeps_region_and_type() {
        let creds: AwsCredentials = serde_json::from_value(json!({
            "type": "sts",
            "region": "us-west-2",
            "access_key_id": "AKIAIOSFODNN7EXAMPLE",
            "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "session_token": "FwoGZXIvYXdzEBYaDExampleSessionToken",
        }))
        .unwrap();
        let debug = format!("{creds:?}");
        assert!(!debug.contains("AKIAIOSFODNN7EXAMPLE"), "got: {debug}");
        assert!(
            !debug.contains("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"),
            "got: {debug}"
        );
        assert!(
            !debug.contains("FwoGZXIvYXdzEBYaDExampleSessionToken"),
            "got: {debug}"
        );
        assert!(debug.contains("<redacted>"), "got: {debug}");
        assert!(debug.contains("us-west-2"), "got: {debug}");
        assert!(debug.contains("Sts"), "got: {debug}");
        // Field names remain visible so a log line is still diagnosable.
        assert!(debug.contains("secret_access_key"), "got: {debug}");
    }

    #[test]
    fn debug_shows_none_for_absent_secrets() {
        let creds: AwsCredentials = serde_json::from_value(json!({ "type": "iam" })).unwrap();
        let debug = format!("{creds:?}");
        assert!(!debug.contains("<redacted>"), "got: {debug}");
        assert!(debug.contains("access_key_id: None"), "got: {debug}");
    }

    #[test]
    fn debug_redaction_does_not_affect_serialization() {
        let creds: AwsCredentials = serde_json::from_value(json!({
            "type": "iam",
            "access_key_id": "AKIA000",
            "secret_access_key": "secret",
        }))
        .unwrap();
        let json = serde_json::to_value(&creds).unwrap();
        assert_eq!(json["access_key_id"], "AKIA000");
        assert_eq!(json["secret_access_key"], "secret");
    }
}
