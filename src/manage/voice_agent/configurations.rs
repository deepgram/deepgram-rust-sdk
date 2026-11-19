//! Reusable Voice Agent configuration management.
//!
//! See the [reusable Voice Agent configurations guide][api].
//!
//! [api]: https://developers.deepgram.com/docs/voice-agent/reusable-agent-configurations

use std::collections::HashMap;

use serde::Serialize;
use url::Url;

use crate::{
    manage::voice_agent::response, send_and_check_response, send_and_translate_response, Deepgram,
};

/// Manage reusable Voice Agent configurations.
#[derive(Debug, Clone)]
pub struct Configurations<'a>(&'a Deepgram);

impl<'a> From<&'a Deepgram> for Configurations<'a> {
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

/// Request options for creating a reusable Voice Agent configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateOptions {
    config: String,
    metadata: Option<HashMap<String, String>>,
    api_version: Option<u32>,
}

/// Builds [`CreateOptions`].
#[derive(Debug, Clone, PartialEq)]
pub struct CreateOptionsBuilder(CreateOptions);

/// Request options for replacing Voice Agent configuration metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOptions {
    metadata: HashMap<String, String>,
}

#[derive(Serialize)]
struct CreateRequest<'a> {
    config: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: &'a Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_version: Option<u32>,
}

#[derive(Serialize)]
struct UpdateRequest<'a> {
    metadata: &'a HashMap<String, String>,
}

impl CreateOptions {
    /// Start building a configuration creation request.
    pub fn builder(config: impl Into<String>) -> CreateOptionsBuilder {
        CreateOptionsBuilder(CreateOptions {
            config: config.into(),
            metadata: None,
            api_version: None,
        })
    }

    /// Serialize this request body as JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&CreateRequest {
            config: &self.config,
            metadata: &self.metadata,
            api_version: self.api_version,
        })
    }
}

impl CreateOptionsBuilder {
    /// Set labels for organizing the saved configuration.
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.0.metadata = Some(metadata);
        self
    }

    /// Set the Voice Agent configuration API version.
    pub fn api_version(mut self, api_version: u32) -> Self {
        self.0.api_version = Some(api_version);
        self
    }

    /// Finish building the request.
    pub fn build(self) -> CreateOptions {
        self.0
    }
}

impl UpdateOptions {
    /// Construct metadata replacement options.
    pub fn new(metadata: HashMap<String, String>) -> Self {
        Self { metadata }
    }

    /// Serialize this request body as JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&UpdateRequest {
            metadata: &self.metadata,
        })
    }
}

impl Configurations<'_> {
    /// List saved Voice Agent configurations for a project.
    pub async fn list(&self, project_id: &str) -> crate::Result<response::Configurations> {
        send_and_translate_response(self.0.client.get(self.configurations_url(project_id)?)).await
    }

    /// Create a reusable Voice Agent configuration.
    pub async fn create(
        &self,
        project_id: &str,
        options: &CreateOptions,
    ) -> crate::Result<response::CreatedAgentConfiguration> {
        let request = self
            .0
            .client
            .post(self.configurations_url(project_id)?)
            .json(&CreateRequest {
                config: &options.config,
                metadata: &options.metadata,
                api_version: options.api_version,
            });
        send_and_translate_response(request).await
    }

    /// Get a saved Voice Agent configuration.
    pub async fn get(
        &self,
        project_id: &str,
        agent_id: &str,
    ) -> crate::Result<response::AgentConfiguration> {
        send_and_translate_response(
            self.0
                .client
                .get(self.configuration_url(project_id, agent_id)?),
        )
        .await
    }

    /// Replace metadata for a saved Voice Agent configuration.
    pub async fn update(
        &self,
        project_id: &str,
        agent_id: &str,
        options: &UpdateOptions,
    ) -> crate::Result<response::AgentConfiguration> {
        let request = self
            .0
            .client
            .put(self.configuration_url(project_id, agent_id)?)
            .json(&UpdateRequest {
                metadata: &options.metadata,
            });
        send_and_translate_response(request).await
    }

    /// Delete a saved Voice Agent configuration.
    pub async fn delete(&self, project_id: &str, agent_id: &str) -> crate::Result<()> {
        send_and_check_response(
            self.0
                .client
                .delete(self.configuration_url(project_id, agent_id)?),
        )
        .await
    }

    fn configurations_url(&self, project_id: &str) -> crate::Result<Url> {
        self.0.api_url(&format!("v1/projects/{project_id}/agents"))
    }

    fn configuration_url(&self, project_id: &str, agent_id: &str) -> crate::Result<Url> {
        self.0
            .api_url(&format!("v1/projects/{project_id}/agents/{agent_id}"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{CreateOptions, UpdateOptions};
    use crate::Deepgram;

    #[test]
    fn serializes_create_and_update_requests() {
        let create = CreateOptions::builder(r#"{"greeting":"Hello"}"#)
            .metadata(HashMap::from([("environment".into(), "test".into())]))
            .api_version(1)
            .build();
        assert_eq!(
            create.json().unwrap(),
            r#"{"config":"{\"greeting\":\"Hello\"}","metadata":{"environment":"test"},"api_version":1}"#
        );

        let update = UpdateOptions::new(HashMap::from([("environment".into(), "prod".into())]));
        assert_eq!(
            update.json().unwrap(),
            r#"{"metadata":{"environment":"prod"}}"#
        );
    }

    #[test]
    fn uses_the_standard_api_base_url() {
        let dg = Deepgram::with_base_url("http://gateway.internal/deepgram/").unwrap();
        let voice_agent = dg.voice_agent();
        let configurations = voice_agent.configurations();

        assert_eq!(
            configurations.configurations_url("proj").unwrap().as_str(),
            "http://gateway.internal/deepgram/v1/projects/proj/agents"
        );
        assert_eq!(
            configurations
                .configuration_url("proj", "agent")
                .unwrap()
                .as_str(),
            "http://gateway.internal/deepgram/v1/projects/proj/agents/agent"
        );
    }
}
