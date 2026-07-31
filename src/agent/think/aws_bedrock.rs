//! AWS Bedrock Think provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/think-providers/aws-bedrock.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::agent::AwsCredentials;

/// AWS Bedrock as a Think provider for the Voice Agent.
///
/// Bedrock providers don't carry a `version` field — credentials live on
/// the provider object directly and govern AWS auth.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AwsBedrockThinkProvider {
    /// Bedrock model identifier.
    pub model: AwsBedrockModel,

    /// Sampling temperature (0–2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// AWS credentials. Bedrock accepts a partial credentials object — any
    /// missing fields fall back to the agent server's default AWS environment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credentials: Option<AwsCredentials>,
}

impl AwsBedrockThinkProvider {
    /// Construct with the given model and no temperature/credentials override.
    pub fn new(model: AwsBedrockModel) -> Self {
        Self {
            model,
            temperature: None,
            credentials: None,
        }
    }
}

/// AWS Bedrock model identifier. Use [`AwsBedrockModel::Other`] for unrecognized values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AwsBedrockModel {
    /// `anthropic/claude-3-5-sonnet-20240620-v1:0`
    AnthropicClaude35Sonnet20240620V1,
    /// `anthropic/claude-3-5-haiku-20240307-v1:0`
    AnthropicClaude35Haiku20240307V1,
    /// Forward-compatibility escape.
    Other(String),
}

impl AwsBedrockModel {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AnthropicClaude35Sonnet20240620V1 => "anthropic/claude-3-5-sonnet-20240620-v1:0",
            Self::AnthropicClaude35Haiku20240307V1 => "anthropic/claude-3-5-haiku-20240307-v1:0",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for AwsBedrockModel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "anthropic/claude-3-5-sonnet-20240620-v1:0" => Self::AnthropicClaude35Sonnet20240620V1,
            "anthropic/claude-3-5-haiku-20240307-v1:0" => Self::AnthropicClaude35Haiku20240307V1,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for AwsBedrockModel {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AwsBedrockModel {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AwsCredentialsType;
    use serde_json::json;

    #[test]
    fn deserialize_with_iam_credentials() {
        let raw = json!({
            "model": "anthropic/claude-3-5-sonnet-20240620-v1:0",
            "temperature": 0.4,
            "credentials": {
                "type": "iam",
                "region": "us-east-1",
                "access_key_id": "AKIA",
                "secret_access_key": "secret"
            }
        });
        let p: AwsBedrockThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, AwsBedrockModel::AnthropicClaude35Sonnet20240620V1);
        let creds = p.credentials.as_ref().unwrap();
        assert_eq!(creds.credentials_type, AwsCredentialsType::Iam);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn deserialize_minimal() {
        let raw = json!({ "model": "anthropic/claude-3-5-haiku-20240307-v1:0" });
        let p: AwsBedrockThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.model, AwsBedrockModel::AnthropicClaude35Haiku20240307V1);
        assert!(p.temperature.is_none());
        assert!(p.credentials.is_none());
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }
}
