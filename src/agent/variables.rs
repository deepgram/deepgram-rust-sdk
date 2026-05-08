//! Agent template-variables CRUD.
//!
//! Mirrors `/v1/projects/{project_id}/agent-variables` and
//! `/v1/projects/{project_id}/agent-variables/{variable_id}` in
//! `openapi/paths/agent.v1.yml`.
//!
//! Variable keys follow the convention `DG_<NAME>` per spec. The SDK
//! does not enforce this — pass whatever the API expects and let the
//! server validate.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::agent::Agent;
use crate::{Deepgram, DeepgramError};

/// Sub-client for agent template-variable endpoints.
///
/// Construct via [`Agent::variables`].
#[derive(Debug, Clone)]
pub struct AgentVariables<'a>(&'a Deepgram);

impl<'a> Agent<'a> {
    /// Sub-client for the agent template-variables CRUD endpoints.
    pub fn variables(&self) -> AgentVariables<'a> {
        AgentVariables(self.0)
    }
}

impl AgentVariables<'_> {
    /// `GET /v1/projects/{project_id}/agent-variables` — list every
    /// variable defined for the project.
    pub async fn list(
        &self,
        project_id: &str,
    ) -> Result<ListAgentVariablesResponse, DeepgramError> {
        let url = self.collection_url(project_id)?;
        let response = self.0.client.get(url).send().await?;
        finish_json(response).await
    }

    /// `POST /v1/projects/{project_id}/agent-variables` — create a new variable.
    pub async fn create(
        &self,
        project_id: &str,
        request: &CreateAgentVariableRequest,
    ) -> Result<AgentVariable, DeepgramError> {
        let url = self.collection_url(project_id)?;
        let response = self.0.client.post(url).json(request).send().await?;
        finish_json(response).await
    }

    /// `GET /v1/projects/{project_id}/agent-variables/{variable_id}` —
    /// fetch a single variable.
    pub async fn get(
        &self,
        project_id: &str,
        variable_id: &str,
    ) -> Result<AgentVariable, DeepgramError> {
        let url = self.item_url(project_id, variable_id)?;
        let response = self.0.client.get(url).send().await?;
        finish_json(response).await
    }

    /// `PATCH /v1/projects/{project_id}/agent-variables/{variable_id}` —
    /// update a variable's value. The `value` may be any JSON type.
    pub async fn update(
        &self,
        project_id: &str,
        variable_id: &str,
        value: Value,
    ) -> Result<AgentVariable, DeepgramError> {
        let url = self.item_url(project_id, variable_id)?;
        let request = UpdateAgentVariableRequest { value };
        let response = self.0.client.patch(url).json(&request).send().await?;
        finish_json(response).await
    }

    /// `DELETE /v1/projects/{project_id}/agent-variables/{variable_id}` —
    /// delete a variable.
    pub async fn delete(&self, project_id: &str, variable_id: &str) -> Result<(), DeepgramError> {
        let url = self.item_url(project_id, variable_id)?;
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
            .join(&format!("v1/projects/{project_id}/agent-variables"))
            .map_err(|_| DeepgramError::InvalidUrl)
    }

    fn item_url(&self, project_id: &str, variable_id: &str) -> Result<Url, DeepgramError> {
        self.0
            .base_url
            .join(&format!(
                "v1/projects/{project_id}/agent-variables/{variable_id}"
            ))
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

/// A template variable usable by saved agent configurations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentVariable {
    /// Unique identifier of the variable.
    pub variable_id: String,

    /// Variable name. Spec convention is `DG_<NAME>` (uppercase, prefixed).
    pub key: String,

    /// Substitution value. Any JSON type — string, number, boolean,
    /// object, or array.
    pub value: Value,

    /// ISO 8601 creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// ISO 8601 last-update timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Request body for [`AgentVariables::create`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateAgentVariableRequest {
    /// Variable name (spec convention: `DG_<NAME>`).
    pub key: String,

    /// Initial substitution value.
    pub value: Value,

    /// API version. Defaults to 1 server-side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_version: Option<u32>,
}

impl CreateAgentVariableRequest {
    /// Construct with a key and value.
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value,
            api_version: None,
        }
    }

    /// Override the API version.
    pub fn with_api_version(mut self, api_version: u32) -> Self {
        self.api_version = Some(api_version);
        self
    }
}

/// Request body for [`AgentVariables::update`] (PATCH).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpdateAgentVariableRequest {
    /// New substitution value.
    pub value: Value,
}

/// Response from [`AgentVariables::list`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ListAgentVariablesResponse {
    /// Variables defined for the project.
    #[serde(default)]
    pub variables: Vec<AgentVariable>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_request_serializes_with_string_value() {
        let req =
            CreateAgentVariableRequest::new("DG_AGENT_NAME", json!("Alice")).with_api_version(1);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v,
            json!({"key": "DG_AGENT_NAME", "value": "Alice", "api_version": 1})
        );
    }

    #[test]
    fn create_request_serializes_with_object_value() {
        let req = CreateAgentVariableRequest::new(
            "DG_BUSINESS_HOURS",
            json!({"open": "09:00", "close": "17:00"}),
        );
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["value"]["open"], "09:00");
    }

    #[test]
    fn variable_round_trips() {
        let raw = json!({
            "variable_id": "v1",
            "key": "DG_AGENT_NAME",
            "value": "Alice",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-02-01T00:00:00Z"
        });
        let var: AgentVariable = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(var.key, "DG_AGENT_NAME");
        assert_eq!(serde_json::to_value(&var).unwrap(), raw);
    }

    #[test]
    fn update_request_round_trips() {
        let raw = json!({"value": [1, 2, 3]});
        let req: UpdateAgentVariableRequest = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(&req).unwrap(), raw);
    }

    #[test]
    fn list_response_empty_round_trips() {
        let raw = json!({});
        let resp: ListAgentVariablesResponse = serde_json::from_value(raw).unwrap();
        assert!(resp.variables.is_empty());
    }
}
