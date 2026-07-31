//! AWS Polly Speak provider settings.
//!
//! Mirrors `asyncapi/schemas/agent/speak-providers/aws-polly.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::agent::AwsCredentials;

/// AWS Polly as a Speak provider for the Voice Agent.
///
/// Unlike most providers, Polly requires an explicit `voice`, `language`,
/// `engine`, and `credentials` block — there are no defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AwsPollySpeakProvider {
    /// AWS Polly voice name.
    pub voice: AwsPollyVoice,

    /// Language code, e.g. `en-US`.
    pub language: String,

    /// Deprecated alias for `language`.
    #[deprecated(
        since = "0.10.0",
        note = "Use the `language` field instead. Mirrors deprecation in the AsyncAPI spec."
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,

    /// Polly synthesis engine.
    pub engine: AwsPollyEngine,

    /// AWS credentials.
    pub credentials: AwsCredentials,
}

#[allow(deprecated)]
impl AwsPollySpeakProvider {
    /// Construct with all required fields.
    pub fn new(
        voice: AwsPollyVoice,
        language: impl Into<String>,
        engine: AwsPollyEngine,
        credentials: AwsCredentials,
    ) -> Self {
        Self {
            voice,
            language: language.into(),
            language_code: None,
            engine,
            credentials,
        }
    }
}

/// AWS Polly voice name. Use [`AwsPollyVoice::Other`] for unrecognized values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[allow(missing_docs)]
pub enum AwsPollyVoice {
    Matthew,
    Joanna,
    Amy,
    Emma,
    Brian,
    Arthur,
    Aria,
    Ayanda,
    /// Forward-compatibility escape.
    Other(String),
}

impl AwsPollyVoice {
    /// Wire string representation.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Matthew => "Matthew",
            Self::Joanna => "Joanna",
            Self::Amy => "Amy",
            Self::Emma => "Emma",
            Self::Brian => "Brian",
            Self::Arthur => "Arthur",
            Self::Aria => "Aria",
            Self::Ayanda => "Ayanda",
            Self::Other(s) => s,
        }
    }
}

impl From<String> for AwsPollyVoice {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Matthew" => Self::Matthew,
            "Joanna" => Self::Joanna,
            "Amy" => Self::Amy,
            "Emma" => Self::Emma,
            "Brian" => Self::Brian,
            "Arthur" => Self::Arthur,
            "Aria" => Self::Aria,
            "Ayanda" => Self::Ayanda,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for AwsPollyVoice {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AwsPollyVoice {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(de)?))
    }
}

/// AWS Polly synthesis engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum AwsPollyEngine {
    /// `generative`
    Generative,
    /// `long-form`
    LongForm,
    /// `standard`
    Standard,
    /// `neural`
    Neural,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AwsCredentialsType;
    use serde_json::json;

    #[test]
    fn round_trip_iam_credentials() {
        let raw = json!({
            "voice": "Joanna",
            "language": "en-US",
            "engine": "neural",
            "credentials": {
                "type": "iam",
                "region": "us-east-1",
                "access_key_id": "AKIA000",
                "secret_access_key": "secret"
            }
        });
        let p: AwsPollySpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.voice, AwsPollyVoice::Joanna);
        assert_eq!(p.engine, AwsPollyEngine::Neural);
        assert_eq!(p.credentials.credentials_type, AwsCredentialsType::Iam);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }

    #[test]
    fn long_form_engine_uses_kebab_case() {
        let raw = json!({
            "voice": "Brian",
            "language": "en-GB",
            "engine": "long-form",
            "credentials": {
                "type": "iam",
                "region": "eu-west-1",
                "access_key_id": "AKIA001",
                "secret_access_key": "s"
            }
        });
        let p: AwsPollySpeakProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(p.engine, AwsPollyEngine::LongForm);
        assert_eq!(serde_json::to_value(&p).unwrap(), raw);
    }
}
