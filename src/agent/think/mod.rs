//! Voice Agent Think (LLM) settings.
//!
//! Mirrors `asyncapi/schemas/agent/think-settings.v1.yml` and the five
//! provider sub-schemas under `asyncapi/schemas/agent/think-providers/`.

use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::agent::Endpoint;

pub mod anthropic;
pub mod aws_bedrock;
pub mod google;
pub mod groq;
pub mod open_ai;

pub use anthropic::{AnthropicModel, AnthropicThinkProvider, AnthropicVersion};
pub use aws_bedrock::{AwsBedrockModel, AwsBedrockThinkProvider};
pub use google::{GoogleModel, GoogleThinkProvider, GoogleVersion};
pub use groq::{GroqModel, GroqThinkProvider, GroqVersion};
pub use open_ai::{OpenAiModel, OpenAiReasoningMode, OpenAiThinkProvider, OpenAiVersion};

/// Top-level Think configuration on `agent.think` in the Voice Agent
/// `Settings` message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkSettings {
    /// LLM provider configuration.
    pub provider: ThinkProvider,

    /// Custom LLM endpoint. Optional for non-Deepgram providers; ignored otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<Endpoint>,

    /// Function definitions that the agent may call during a conversation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<ThinkFunction>,

    /// System prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Context retention setting. Only configurable when a custom think
    /// endpoint is in use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_length: Option<ContextLength>,
}

impl ThinkSettings {
    /// Construct with just a provider and defaults for all other fields.
    pub fn new(provider: ThinkProvider) -> Self {
        Self {
            provider,
            endpoint: None,
            functions: Vec::new(),
            prompt: None,
            context_length: None,
        }
    }

    /// Set a custom LLM endpoint.
    pub fn with_endpoint(mut self, endpoint: Endpoint) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Replace the function list.
    pub fn with_functions(mut self, functions: impl IntoIterator<Item = ThinkFunction>) -> Self {
        self.functions = functions.into_iter().collect();
        self
    }

    /// Append a single function definition.
    pub fn with_function(mut self, function: ThinkFunction) -> Self {
        self.functions.push(function);
        self
    }

    /// Set the system prompt.
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Set the context length policy.
    pub fn with_context_length(mut self, context_length: ContextLength) -> Self {
        self.context_length = Some(context_length);
        self
    }
}

/// LLM provider variants supported by the Voice Agent.
///
/// Wire format is internally tagged on `type` with snake_case discriminator
/// values: `open_ai`, `anthropic`, `aws_bedrock`, `google`, `groq`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ThinkProvider {
    /// OpenAI chat completions.
    OpenAi(OpenAiThinkProvider),
    /// Anthropic Messages API.
    Anthropic(AnthropicThinkProvider),
    /// AWS Bedrock.
    AwsBedrock(AwsBedrockThinkProvider),
    /// Google generative language API (Gemini).
    Google(GoogleThinkProvider),
    /// Groq chat completions (OpenAI-compatible).
    Groq(GroqThinkProvider),
}

/// A function the agent may call during a conversation.
///
/// When `endpoint` is `None`, the function executes client-side: the
/// server emits a `FunctionCallRequest` and waits for a
/// `FunctionCallResponse` from the client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkFunction {
    /// Function name.
    pub name: String,

    /// Function description.
    pub description: String,

    /// JSON Schema describing the function's parameters. Stored as opaque JSON.
    pub parameters: serde_json::Value,

    /// HTTP endpoint to call for server-side function execution.
    /// When omitted, the function is executed client-side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<FunctionEndpoint>,
}

impl ThinkFunction {
    /// Construct a client-side function (no `endpoint`).
    ///
    /// `parameters` is the JSON Schema describing the function's args.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            endpoint: None,
        }
    }

    /// Attach a server-side execution endpoint.
    pub fn with_endpoint(mut self, endpoint: FunctionEndpoint) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

/// HTTP endpoint for server-side function execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FunctionEndpoint {
    /// Endpoint URL.
    pub url: String,

    /// HTTP method.
    pub method: String,

    /// Custom headers.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

/// Context retention setting for `agent.think.context_length`.
///
/// Wire format is `oneOf [string("max"), number]`:
/// `ContextLength::Max` serializes as the literal string `"max"`;
/// `ContextLength::Tokens(n)` serializes as the number `n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ContextLength {
    /// Retain the full context regardless of length.
    Max,
    /// Retain at most `n` tokens of context. Spec minimum is 2.
    Tokens(u32),
}

impl Serialize for ContextLength {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Max => ser.serialize_str("max"),
            Self::Tokens(n) => ser.serialize_u32(*n),
        }
    }
}

impl<'de> Deserialize<'de> for ContextLength {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(de)?;
        if let Some(s) = value.as_str() {
            if s == "max" {
                Ok(Self::Max)
            } else {
                Err(D::Error::custom(format!(
                    "expected `\"max\"` or a non-negative integer for context_length, got string {s:?}"
                )))
            }
        } else if let Some(n) = value.as_u64() {
            u32::try_from(n)
                .map(Self::Tokens)
                .map_err(|_| D::Error::custom(format!("context_length {n} does not fit in u32")))
        } else {
            Err(D::Error::custom(
                "expected `\"max\"` or a non-negative integer for context_length",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Endpoint;
    use serde_json::json;

    #[test]
    fn provider_round_trip_openai() {
        let raw = json!({
            "type": "open_ai",
            "model": "gpt-4o",
            "temperature": 0.5,
        });
        let provider: ThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            ThinkProvider::OpenAi(p) => assert_eq!(p.model, OpenAiModel::Gpt4o),
            _ => panic!("expected OpenAi"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn provider_round_trip_anthropic() {
        let raw = json!({
            "type": "anthropic",
            "model": "claude-3-5-haiku-latest",
        });
        let provider: ThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            ThinkProvider::Anthropic(p) => {
                assert_eq!(p.model, AnthropicModel::Claude35HaikuLatest);
            }
            _ => panic!("expected Anthropic"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn provider_round_trip_aws_bedrock() {
        let raw = json!({
            "type": "aws_bedrock",
            "model": "anthropic/claude-3-5-haiku-20240307-v1:0",
        });
        let provider: ThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            ThinkProvider::AwsBedrock(p) => {
                assert_eq!(p.model, AwsBedrockModel::AnthropicClaude35Haiku20240307V1);
            }
            _ => panic!("expected AwsBedrock"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn provider_round_trip_google() {
        let raw = json!({
            "type": "google",
            "model": "gemini-2.0-flash",
        });
        let provider: ThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            ThinkProvider::Google(p) => assert_eq!(p.model, GoogleModel::Gemini20Flash),
            _ => panic!("expected Google"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn provider_round_trip_groq() {
        let raw = json!({
            "type": "groq",
            "model": "openai/gpt-oss-20b",
        });
        let provider: ThinkProvider = serde_json::from_value(raw.clone()).unwrap();
        match &provider {
            ThinkProvider::Groq(p) => assert_eq!(p.model, GroqModel::OpenAiGptOss20b),
            _ => panic!("expected Groq"),
        }
        assert_eq!(serde_json::to_value(&provider).unwrap(), raw);
    }

    #[test]
    fn settings_round_trip_full() {
        let raw = json!({
            "provider": {
                "type": "open_ai",
                "model": "gpt-4o-mini",
            },
            "endpoint": {
                "url": "https://llm.internal/v1/chat",
                "headers": { "Authorization": "Bearer abc" }
            },
            "functions": [
                {
                    "name": "get_weather",
                    "description": "Look up current weather",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "city": { "type": "string" }
                        }
                    }
                }
            ],
            "prompt": "You are a helpful assistant.",
            "context_length": 4096,
        });
        let settings: ThinkSettings = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(settings.context_length, Some(ContextLength::Tokens(4096)));
        assert_eq!(settings.functions.len(), 1);
        assert_eq!(settings.functions[0].name, "get_weather");
        assert_eq!(serde_json::to_value(&settings).unwrap(), raw);
    }

    #[test]
    fn settings_round_trip_minimal() {
        let raw = json!({
            "provider": {
                "type": "anthropic",
                "model": "claude-3-5-haiku-latest"
            }
        });
        let settings: ThinkSettings = serde_json::from_value(raw.clone()).unwrap();
        assert!(settings.endpoint.is_none());
        assert!(settings.functions.is_empty());
        assert_eq!(serde_json::to_value(&settings).unwrap(), raw);
    }

    #[test]
    fn context_length_max_string() {
        let json = json!("max");
        let cl: ContextLength = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(cl, ContextLength::Max);
        assert_eq!(serde_json::to_value(cl).unwrap(), json);
    }

    #[test]
    fn context_length_number() {
        let json = json!(8192);
        let cl: ContextLength = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(cl, ContextLength::Tokens(8192));
        assert_eq!(serde_json::to_value(cl).unwrap(), json);
    }

    #[test]
    fn context_length_rejects_garbage_string() {
        let json = json!("infinite");
        let err = serde_json::from_value::<ContextLength>(json).unwrap_err();
        assert!(err.to_string().contains("max"), "got: {err}");
    }

    #[test]
    fn function_endpoint_round_trip() {
        let raw = json!({
            "url": "https://hooks.internal/fn",
            "method": "POST",
            "headers": { "X-Tenant": "acme" }
        });
        let fe: FunctionEndpoint = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(fe.method, "POST");
        assert_eq!(serde_json::to_value(&fe).unwrap(), raw);
    }

    #[test]
    fn settings_with_context_length_max() {
        let settings = ThinkSettings {
            provider: ThinkProvider::OpenAi(OpenAiThinkProvider::new(OpenAiModel::Gpt5)),
            endpoint: Some(Endpoint::new("https://example.com")),
            functions: Vec::new(),
            prompt: Some("hi".into()),
            context_length: Some(ContextLength::Max),
        };
        let serialized = serde_json::to_value(&settings).unwrap();
        assert_eq!(serialized["context_length"], json!("max"));
    }
}
