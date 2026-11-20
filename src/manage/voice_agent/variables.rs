//! Voice Agent template variable management.
//!
//! See the [reusable Voice Agent configurations guide][api].
//!
//! [api]: https://developers.deepgram.com/docs/voice-agent/reusable-agent-configurations

use std::fmt;

use serde::Serialize;
use serde_json::Value;
use url::Url;

use crate::{
    manage::voice_agent::response, send_and_check_response, send_and_translate_response, Deepgram,
};

/// Manage template variables for reusable Voice Agent configurations.
#[derive(Debug, Clone)]
pub struct Variables<'a>(&'a Deepgram);

impl<'a> From<&'a Deepgram> for Variables<'a> {
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

/// Request options for creating a Voice Agent template variable.
#[derive(Clone, PartialEq)]
pub struct CreateOptions {
    key: String,
    value: Value,
    api_version: Option<u32>,
    is_sensitive: Option<bool>,
}

impl fmt::Debug for CreateOptions {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = formatter.debug_struct("CreateOptions");
        debug
            .field("key", &self.key)
            .field("api_version", &self.api_version)
            .field("is_sensitive", &self.is_sensitive);
        if self.is_sensitive == Some(true) {
            debug.field("value", &"<redacted>");
        } else {
            debug.field("value", &self.value);
        }
        debug.finish()
    }
}

/// Builds [`CreateOptions`].
#[derive(Debug, Clone, PartialEq)]
pub struct CreateOptionsBuilder(CreateOptions);

/// Request options for replacing a Voice Agent template variable value.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateOptions {
    value: Value,
}

#[derive(Serialize)]
struct CreateRequest<'a> {
    key: &'a str,
    value: &'a Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_sensitive: Option<bool>,
}

#[derive(Serialize)]
struct UpdateRequest<'a> {
    value: &'a Value,
}

impl CreateOptions {
    /// Start building a template-variable creation request.
    pub fn builder(key: impl Into<String>, value: Value) -> CreateOptionsBuilder {
        CreateOptionsBuilder(CreateOptions {
            key: key.into(),
            value,
            api_version: None,
            is_sensitive: None,
        })
    }

    /// Serialize this request body as JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&CreateRequest {
            key: &self.key,
            value: &self.value,
            api_version: self.api_version,
            is_sensitive: self.is_sensitive,
        })
    }
}

impl CreateOptionsBuilder {
    /// Set the Voice Agent variable API version.
    pub fn api_version(mut self, api_version: u32) -> Self {
        self.0.api_version = Some(api_version);
        self
    }

    /// Mark whether the variable value contains sensitive data.
    ///
    /// The Voice Agent REST API currently rejects `true` in production and
    /// staging with `400 BAD_REQUEST: Sensitive template variables are not
    /// supported yet`. Do not enable this option until the service supports
    /// sensitive template variables.
    pub fn is_sensitive(mut self, is_sensitive: bool) -> Self {
        self.0.is_sensitive = Some(is_sensitive);
        self
    }

    /// Finish building the request.
    pub fn build(self) -> CreateOptions {
        self.0
    }
}

impl UpdateOptions {
    /// Construct a variable-value replacement request.
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    /// Serialize this request body as JSON.
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&UpdateRequest { value: &self.value })
    }
}

impl Variables<'_> {
    /// List template variables for a project.
    pub async fn list(&self, project_id: &str) -> crate::Result<response::Variables> {
        send_and_translate_response(self.0.client.get(self.variables_url(project_id)?)).await
    }

    /// Create a Voice Agent template variable.
    pub async fn create(
        &self,
        project_id: &str,
        options: &CreateOptions,
    ) -> crate::Result<response::CreatedAgentVariable> {
        let request = self
            .0
            .client
            .post(self.variables_url(project_id)?)
            .json(&CreateRequest {
                key: &options.key,
                value: &options.value,
                api_version: options.api_version,
                is_sensitive: options.is_sensitive,
            });
        send_and_translate_response(request).await
    }

    /// Get a Voice Agent template variable.
    pub async fn get(
        &self,
        project_id: &str,
        variable_id: &str,
    ) -> crate::Result<response::AgentVariable> {
        send_and_translate_response(
            self.0
                .client
                .get(self.variable_url(project_id, variable_id)?),
        )
        .await
    }

    /// Replace a Voice Agent template variable value.
    pub async fn update(
        &self,
        project_id: &str,
        variable_id: &str,
        options: &UpdateOptions,
    ) -> crate::Result<()> {
        let request = self
            .0
            .client
            .patch(self.variable_url(project_id, variable_id)?)
            .json(&UpdateRequest {
                value: &options.value,
            });
        send_and_check_response(request).await
    }

    /// Delete a Voice Agent template variable.
    pub async fn delete(&self, project_id: &str, variable_id: &str) -> crate::Result<()> {
        send_and_check_response(
            self.0
                .client
                .delete(self.variable_url(project_id, variable_id)?),
        )
        .await
    }

    fn variables_url(&self, project_id: &str) -> crate::Result<Url> {
        self.0
            .api_url(&format!("v1/projects/{project_id}/agent-variables"))
    }

    fn variable_url(&self, project_id: &str, variable_id: &str) -> crate::Result<Url> {
        self.0.api_url(&format!(
            "v1/projects/{project_id}/agent-variables/{variable_id}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CreateOptions, UpdateOptions};
    use crate::Deepgram;

    #[test]
    fn serializes_json_values_verbatim() {
        let create = CreateOptions::builder("DG_CUSTOMER", json!({"tier": "gold"}))
            .api_version(1)
            .is_sensitive(true)
            .build();
        assert_eq!(
            create.json().unwrap(),
            r#"{"key":"DG_CUSTOMER","value":{"tier":"gold"},"api_version":1,"is_sensitive":true}"#
        );

        let update = UpdateOptions::new(json!(false));
        assert_eq!(update.json().unwrap(), r#"{"value":false}"#);
    }

    #[test]
    fn debug_redacts_sensitive_creation_values() {
        let options = CreateOptions::builder("DG_SECRET", json!("do-not-log-me"))
            .is_sensitive(true)
            .build();

        let debug = format!("{options:?}");
        assert!(debug.contains("<redacted>"));
        assert!(!debug.contains("do-not-log-me"));
    }

    #[test]
    fn uses_the_standard_api_base_url() {
        let dg = Deepgram::with_base_url("http://gateway.internal/deepgram/").unwrap();
        let voice_agent = dg.voice_agent();
        let variables = voice_agent.variables();

        assert_eq!(
            variables.variables_url("proj").unwrap().as_str(),
            "http://gateway.internal/deepgram/v1/projects/proj/agent-variables"
        );
        assert_eq!(
            variables.variable_url("proj", "variable").unwrap().as_str(),
            "http://gateway.internal/deepgram/v1/projects/proj/agent-variables/variable"
        );
    }
}
