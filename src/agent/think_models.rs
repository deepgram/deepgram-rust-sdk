//! Think-models catalog endpoint.
//!
//! Mirrors `GET /v1/agent/settings/think/models` in
//! `openapi/paths/agent.v1.yml`. Note the host is
//! `agent.deepgram.com` (the same one the Voice Agent WebSocket
//! uses), not the Deepgram API host.

use serde::{Deserialize, Serialize};

use crate::agent::Agent;
use crate::{Deepgram, DeepgramError};

const AGENT_API_BASE_URL: &str = "https://agent.deepgram.com";
const THINK_MODELS_PATH: &str = "v1/agent/settings/think/models";

/// Sub-client for the think-models catalog. Construct via
/// [`Agent::think_models`].
#[derive(Debug, Clone)]
pub struct AgentThinkModels<'a>(&'a Deepgram);

impl<'a> Agent<'a> {
    /// Sub-client for the think-models catalog endpoint.
    pub fn think_models(&self) -> AgentThinkModels<'a> {
        AgentThinkModels(self.0)
    }
}

impl AgentThinkModels<'_> {
    /// `GET /v1/agent/settings/think/models` — list all think models
    /// available to the project.
    pub async fn list(&self) -> Result<ListAgentThinkModelsResponse, DeepgramError> {
        let url = format!("{AGENT_API_BASE_URL}/{THINK_MODELS_PATH}");
        let response = self.0.client.get(&url).send().await?;

        if let Err(err) = response.error_for_status_ref() {
            let body = response.text().await.unwrap_or_default();
            return Err(DeepgramError::DeepgramApiError { body, err });
        }

        Ok(response.json::<ListAgentThinkModelsResponse>().await?)
    }
}

/// Response shape for the think-models catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ListAgentThinkModelsResponse {
    /// Available think models.
    pub models: Vec<ThinkModel>,
}

/// A single think model entry from the catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkModel {
    /// Model identifier (e.g. `gpt-4o-mini`, `claude-3-5-haiku-latest`).
    pub id: String,

    /// Display name.
    pub name: String,

    /// Provider this model belongs to.
    pub provider: ThinkModelProvider,
}

/// Provider tag on a [`ThinkModel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ThinkModelProvider {
    /// OpenAI.
    OpenAi,
    /// Anthropic.
    Anthropic,
    /// Google (Gemini).
    Google,
    /// Groq.
    Groq,
    /// AWS Bedrock — accepts any custom model identifier.
    AwsBedrock,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_mixed_providers() {
        let raw = json!({
            "models": [
                {"id": "gpt-4o", "name": "GPT-4o", "provider": "open_ai"},
                {"id": "claude-3-5-haiku-latest", "name": "Claude 3.5 Haiku", "provider": "anthropic"},
                {"id": "gemini-2.5-flash", "name": "Gemini 2.5 Flash", "provider": "google"},
                {"id": "openai/gpt-oss-20b", "name": "GPT-OSS 20B", "provider": "groq"},
                {"id": "anthropic/claude-3-5-sonnet-20240620-v1:0", "name": "Claude 3.5 Sonnet (Bedrock)", "provider": "aws_bedrock"},
            ]
        });
        let resp: ListAgentThinkModelsResponse = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(resp.models.len(), 5);
        assert_eq!(resp.models[0].provider, ThinkModelProvider::OpenAi);
        assert_eq!(resp.models[4].provider, ThinkModelProvider::AwsBedrock);
        assert_eq!(serde_json::to_value(&resp).unwrap(), raw);
    }
}
