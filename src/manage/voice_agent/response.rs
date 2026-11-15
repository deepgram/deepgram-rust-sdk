//! Voice Agent REST response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Available Voice Agent Think models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkModels {
    /// Models available to the authenticated project.
    pub models: Vec<ThinkModel>,
}

/// One Voice Agent Think model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkModel {
    /// Model identifier used in Voice Agent settings.
    pub id: String,
    /// Human-readable model name.
    pub name: String,
    /// Model provider.
    pub provider: String,
}

/// A reusable Voice Agent configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentConfiguration {
    /// Saved configuration identifier.
    pub agent_id: String,
    /// Parsed Voice Agent configuration.
    pub config: Value,
    /// Optional labels associated with the configuration.
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
    /// Configuration creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Configuration metadata update timestamp.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Saved Voice Agent configurations for a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Configurations {
    /// Saved configurations returned by the API.
    #[serde(default)]
    pub agents: Option<Vec<AgentConfiguration>>,
}

/// Successful deletion response for a saved Voice Agent configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeletedAgentConfiguration {}

/// A Voice Agent template variable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentVariable {
    /// Template variable identifier.
    pub variable_id: String,
    /// Template variable name.
    pub key: String,
    /// JSON value substituted into saved configurations.
    pub value: Value,
    /// Variable creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Variable update timestamp.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Voice Agent template variables for a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Variables {
    /// Variables returned by the API.
    #[serde(default)]
    pub variables: Option<Vec<AgentVariable>>,
}

/// Successful deletion response for a Voice Agent template variable.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DeletedAgentVariable {}

#[cfg(test)]
mod tests {
    use super::{AgentConfiguration, AgentVariable, Configurations, ThinkModels, Variables};

    #[test]
    fn deserializes_models_from_multiple_providers() {
        let models: ThinkModels = serde_json::from_str(
            r#"{
                "models": [
                    {"id": "gpt-5-mini", "name": "GPT-5 mini", "provider": "open_ai"},
                    {"id": "claude-3-5-haiku-latest", "name": "Claude Haiku", "provider": "anthropic"}
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(models.models.len(), 2);
        assert_eq!(models.models[1].provider, "anthropic");
    }

    #[test]
    fn deserializes_configurations_and_variables() {
        let configuration: AgentConfiguration = serde_json::from_str(
            r#"{
                "agent_id": "agent",
                "config": {"greeting": "Hello"},
                "metadata": {"environment": "test"}
            }"#,
        )
        .unwrap();
        assert_eq!(configuration.config["greeting"], "Hello");

        let variable: AgentVariable = serde_json::from_str(
            r#"{"variable_id":"variable","key":"DG_ACCOUNT","value":{"tier":"gold"}}"#,
        )
        .unwrap();
        assert_eq!(variable.value["tier"], "gold");
    }

    #[test]
    fn accepts_empty_configuration_and_variable_lists() {
        let configurations: Configurations = serde_json::from_str("{}").unwrap();
        assert!(configurations.agents.is_none());

        let variables: Variables = serde_json::from_str("{}").unwrap();
        assert!(variables.variables.is_none());
    }
}
