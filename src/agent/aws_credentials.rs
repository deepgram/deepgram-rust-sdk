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
/// Build one with [`AwsCredentials::iam`] or [`AwsCredentials::sts`], or
/// with [`AwsCredentials::new`] plus the `with_*` setters for a partial
/// Bedrock block. The struct is `#[non_exhaustive]`, so a struct literal
/// does not compile outside this crate and a constructor is the only way
/// in.
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

impl AwsCredentials {
    /// Long-lived IAM credentials.
    ///
    /// ```
    /// use deepgram::agent::aws_credentials::AwsCredentials;
    ///
    /// let credentials = AwsCredentials::iam("us-east-1", "AKIAEXAMPLE", "secret");
    /// ```
    pub fn iam(
        region: impl Into<String>,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
    ) -> Self {
        Self {
            credentials_type: AwsCredentialsType::Iam,
            region: Some(region.into()),
            access_key_id: Some(access_key_id.into()),
            secret_access_key: Some(secret_access_key.into()),
            session_token: None,
        }
    }

    /// Short-lived AWS Security Token Service credentials, which carry a
    /// `session_token`.
    pub fn sts(
        region: impl Into<String>,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        session_token: impl Into<String>,
    ) -> Self {
        Self {
            credentials_type: AwsCredentialsType::Sts,
            region: Some(region.into()),
            access_key_id: Some(access_key_id.into()),
            secret_access_key: Some(secret_access_key.into()),
            session_token: Some(session_token.into()),
        }
    }

    /// An empty credentials block of the given type, for AWS Bedrock, which
    /// accepts any subset of the fields. Pair with the `with_*` setters.
    pub fn new(credentials_type: AwsCredentialsType) -> Self {
        Self {
            credentials_type,
            region: None,
            access_key_id: None,
            secret_access_key: None,
            session_token: None,
        }
    }

    /// Set the AWS region.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set the access key ID.
    pub fn with_access_key_id(mut self, access_key_id: impl Into<String>) -> Self {
        self.access_key_id = Some(access_key_id.into());
        self
    }

    /// Set the secret access key.
    pub fn with_secret_access_key(mut self, secret_access_key: impl Into<String>) -> Self {
        self.secret_access_key = Some(secret_access_key.into());
        self
    }

    /// Set the session token, required for STS credentials.
    pub fn with_session_token(mut self, session_token: impl Into<String>) -> Self {
        self.session_token = Some(session_token.into());
        self
    }
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

    /// `AwsCredentials` is `#[non_exhaustive]`, so a downstream crate cannot
    /// use a struct literal. These constructors are the only way for a
    /// consumer to reach `AwsPollySpeakProvider::new`, which takes the
    /// credentials by value — without them the AWS providers are
    /// unconstructible outside this crate. Written with no struct literal
    /// on purpose, so it exercises the same path a consumer has.
    #[test]
    fn constructors_are_the_downstream_path() {
        let iam = AwsCredentials::iam("us-east-1", "AKIAEXAMPLE", "iam-secret");
        assert_eq!(iam.credentials_type, AwsCredentialsType::Iam);
        assert_eq!(iam.region.as_deref(), Some("us-east-1"));
        assert_eq!(iam.session_token, None);
        assert_eq!(
            serde_json::to_value(&iam).unwrap(),
            json!({
                "type": "iam",
                "region": "us-east-1",
                "access_key_id": "AKIAEXAMPLE",
                "secret_access_key": "iam-secret",
            })
        );

        let sts = AwsCredentials::sts("us-west-2", "AKIASTS", "sts-secret", "session");
        assert_eq!(sts.credentials_type, AwsCredentialsType::Sts);
        assert_eq!(sts.session_token.as_deref(), Some("session"));

        // The Bedrock shape: any subset.
        let partial = AwsCredentials::new(AwsCredentialsType::Iam).with_region("eu-west-1");
        assert_eq!(partial.region.as_deref(), Some("eu-west-1"));
        assert_eq!(partial.access_key_id, None);
        assert_eq!(
            serde_json::to_value(&partial).unwrap(),
            json!({ "type": "iam", "region": "eu-west-1" })
        );
    }

    /// The constructors must not defeat the redaction the struct promises.
    #[test]
    fn constructed_credentials_still_redact_in_debug() {
        let debug = format!(
            "{:?}",
            AwsCredentials::sts("us-west-2", "AKIALEAK", "SECRETLEAK", "TOKENLEAK")
        );
        assert!(!debug.contains("AKIALEAK"), "got: {debug}");
        assert!(!debug.contains("SECRETLEAK"), "got: {debug}");
        assert!(!debug.contains("TOKENLEAK"), "got: {debug}");
        assert!(debug.contains("us-west-2"), "got: {debug}");
    }

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
