//! Saved Voice-Agent configurations CRUD.
//!
//! Mirrors `/v1/projects/{project_id}/agents` and
//! `/v1/projects/{project_id}/agents/{agent_id}` in
//! `openapi/paths/agent.v1.yml`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::agent::Agent;
use crate::{Deepgram, DeepgramError};

/// Sub-client for the saved-configurations endpoints.
///
/// Construct via [`Agent::configurations`].
#[derive(Debug, Clone)]
pub struct AgentConfigurations<'a>(&'a Deepgram);

impl<'a> Agent<'a> {
    /// Sub-client for the saved-configurations CRUD endpoints.
    pub fn configurations(&self) -> AgentConfigurations<'a> {
        AgentConfigurations(self.0)
    }
}

impl AgentConfigurations<'_> {
    /// `GET /v1/projects/{project_id}/agents` — list every saved
    /// configuration for the project.
    pub async fn list(
        &self,
        project_id: &str,
    ) -> Result<ListAgentConfigurationsResponse, DeepgramError> {
        let url = self.collection_url(project_id)?;
        let response = self.0.client.get(url).send().await?;
        finish_json(response).await
    }

    /// `POST /v1/projects/{project_id}/agents` — create a new saved
    /// configuration. The request's `config` field must already be a
    /// JSON-encoded string of an agent settings block.
    pub async fn create(
        &self,
        project_id: &str,
        request: &CreateAgentConfigurationRequest,
    ) -> Result<CreateAgentConfigurationResponse, DeepgramError> {
        let url = self.collection_url(project_id)?;
        let response = self.0.client.post(url).json(request).send().await?;
        finish_json(response).await
    }

    /// `GET /v1/projects/{project_id}/agents/{agent_id}` — fetch a
    /// single configuration.
    pub async fn get(
        &self,
        project_id: &str,
        agent_id: &str,
    ) -> Result<AgentConfiguration, DeepgramError> {
        let url = self.item_url(project_id, agent_id)?;
        let response = self.0.client.get(url).send().await?;
        finish_json(response).await
    }

    /// `PUT /v1/projects/{project_id}/agents/{agent_id}` — update the
    /// metadata on a configuration.
    ///
    /// The configuration itself (the `config` JSON) is **immutable** per
    /// spec: only the metadata can be updated. To change the config
    /// itself, delete this entry and create a new one.
    pub async fn update_metadata(
        &self,
        project_id: &str,
        agent_id: &str,
        request: &UpdateAgentMetadataRequest,
    ) -> Result<AgentConfiguration, DeepgramError> {
        let url = self.item_url(project_id, agent_id)?;
        let response = self.0.client.put(url).json(request).send().await?;
        finish_json(response).await
    }

    /// `DELETE /v1/projects/{project_id}/agents/{agent_id}` — delete a
    /// configuration. **Caution:** the spec warns that deleting an
    /// agent UUID referenced by an active session can cause an outage
    /// — migrate sessions first.
    pub async fn delete(&self, project_id: &str, agent_id: &str) -> Result<(), DeepgramError> {
        let url = self.item_url(project_id, agent_id)?;
        let response = self.0.client.delete(url).send().await?;
        if let Err(err) = response.error_for_status_ref() {
            let body = response.text().await.unwrap_or_default();
            return Err(DeepgramError::DeepgramApiError { body, err });
        }
        Ok(())
    }

    fn collection_url(&self, project_id: &str) -> Result<Url, DeepgramError> {
        self.0
            .base_url
            .join(&format!("v1/projects/{project_id}/agents"))
            .map_err(|_| DeepgramError::InvalidUrl)
    }

    fn item_url(&self, project_id: &str, agent_id: &str) -> Result<Url, DeepgramError> {
        self.0
            .base_url
            .join(&format!("v1/projects/{project_id}/agents/{agent_id}"))
            .map_err(|_| DeepgramError::InvalidUrl)
    }
}

async fn finish_json<T>(response: reqwest::Response) -> Result<T, DeepgramError>
where
    T: serde::de::DeserializeOwned,
{
    if let Err(err) = response.error_for_status_ref() {
        let body = response.text().await.unwrap_or_default();
        return Err(DeepgramError::DeepgramApiError { body, err });
    }
    Ok(response.json::<T>().await?)
}

/// A saved Voice-Agent configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentConfiguration {
    /// Unique identifier (UUID) of the saved configuration.
    pub agent_id: String,

    /// The parsed agent configuration object. Stored as opaque JSON —
    /// the spec describes it as the `agent` block of a Settings message,
    /// but its detailed shape varies and we don't try to type it here.
    pub config: serde_json::Value,

    /// Arbitrary key-value labels.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,

    /// ISO 8601 creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// ISO 8601 last-update timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Request body for [`AgentConfigurations::create`].
///
/// `config` is a **JSON-encoded string** representing the `agent` block
/// of a Settings message — the spec is explicit about this. Use
/// [`CreateAgentConfigurationRequest::from_inline`] to serialize an
/// [`crate::agent::settings::InlineAgentConfig`] for you.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateAgentConfigurationRequest {
    /// JSON-encoded `agent` block.
    pub config: String,

    /// Optional labels.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,

    /// API version. Defaults to 1 server-side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_version: Option<u32>,
}

impl CreateAgentConfigurationRequest {
    /// Construct from a pre-serialized JSON string.
    pub fn new(config: impl Into<String>) -> Self {
        Self {
            config: config.into(),
            metadata: HashMap::new(),
            api_version: None,
        }
    }

    /// Construct from a typed [`crate::agent::settings::InlineAgentConfig`].
    /// Serializes the inline config to JSON automatically.
    pub fn from_inline(
        inline: &crate::agent::settings::InlineAgentConfig,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new(serde_json::to_string(inline)?))
    }

    /// Replace the metadata map.
    pub fn with_metadata<I, K, V>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata = entries
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }

    /// Override the API version.
    pub fn with_api_version(mut self, api_version: u32) -> Self {
        self.api_version = Some(api_version);
        self
    }
}

/// Response from [`AgentConfigurations::create`]. The `config` field is
/// returned as a parsed object (not a string), per spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateAgentConfigurationResponse {
    /// Unique identifier of the newly-created configuration.
    pub agent_id: String,

    /// Parsed configuration object.
    pub config: serde_json::Value,

    /// Metadata that was attached.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
}

/// Request body for [`AgentConfigurations::update_metadata`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpdateAgentMetadataRequest {
    /// Replacement metadata map.
    pub metadata: HashMap<String, String>,
}

impl UpdateAgentMetadataRequest {
    /// Construct from any iterator of key-value pairs.
    pub fn new<I, K, V>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self {
            metadata: entries
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        }
    }
}

/// Response from [`AgentConfigurations::list`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ListAgentConfigurationsResponse {
    /// Configurations on the project.
    #[serde(default)]
    pub agents: Vec<AgentConfiguration>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_request_serializes() {
        let req = CreateAgentConfigurationRequest::new("{\"language\":\"en\"}")
            .with_metadata([("env", "prod"), ("team", "voice")])
            .with_api_version(1);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["config"], "{\"language\":\"en\"}");
        assert_eq!(v["api_version"], 1);
        assert_eq!(v["metadata"]["env"], "prod");
    }

    #[test]
    fn create_request_from_inline_serializes_inline_to_string() {
        use crate::agent::settings::InlineAgentConfig;
        let inline = InlineAgentConfig::new();
        let req = CreateAgentConfigurationRequest::from_inline(&inline).unwrap();
        // The `config` field is a JSON string — round-trip it.
        let parsed: serde_json::Value = serde_json::from_str(&req.config).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn list_response_round_trips() {
        let raw = json!({
            "agents": [
                {
                    "agent_id": "a1",
                    "config": {"language": "en"},
                    "metadata": {"env": "prod"},
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-02-01T00:00:00Z"
                }
            ]
        });
        let resp: ListAgentConfigurationsResponse = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(resp.agents.len(), 1);
        assert_eq!(resp.agents[0].agent_id, "a1");
        assert_eq!(serde_json::to_value(&resp).unwrap(), raw);
    }

    #[test]
    fn list_response_empty_round_trips() {
        let raw = json!({});
        let resp: ListAgentConfigurationsResponse = serde_json::from_value(raw).unwrap();
        assert!(resp.agents.is_empty());
    }

    #[test]
    fn update_metadata_request_constructs_from_iterator() {
        let req = UpdateAgentMetadataRequest::new([("env", "staging")]);
        assert_eq!(req.metadata.get("env").map(String::as_str), Some("staging"));
    }
}
