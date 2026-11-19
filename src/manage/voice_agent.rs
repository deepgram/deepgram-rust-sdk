//! Manage reusable Voice Agent resources and discover available Think models.
//!
//! See the [reusable Voice Agent configurations guide][configurations] and
//! [Voice Agent LLM models reference][models].
//!
//! [configurations]: https://developers.deepgram.com/docs/voice-agent/reusable-agent-configurations
//! [models]: https://developers.deepgram.com/docs/voice-agent/voice-agent-llm-models

use url::Url;

use crate::{send_and_translate_response, Deepgram};

pub mod configurations;
pub mod response;
pub mod variables;

/// Voice Agent REST management client.
///
/// Constructed with [`Deepgram::voice_agent`]. The saved-configuration and
/// variable APIs use the ordinary Deepgram API base URL; Think-model discovery
/// uses the separately configurable Agent REST base URL.
#[derive(Debug, Clone)]
pub struct VoiceAgent<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a Voice Agent REST management client.
    pub fn voice_agent(&self) -> VoiceAgent<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for VoiceAgent<'a> {
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl VoiceAgent<'_> {
    /// Manage reusable Voice Agent configurations.
    pub fn configurations(&self) -> configurations::Configurations<'_> {
        self.0.into()
    }

    /// Manage template variables for reusable Voice Agent configurations.
    pub fn variables(&self) -> variables::Variables<'_> {
        self.0.into()
    }

    /// List Think models available to the authenticated project.
    pub async fn list_think_models(&self) -> crate::Result<response::ThinkModels> {
        send_and_translate_response(self.0.client.get(self.think_models_url()?)).await
    }

    fn think_models_url(&self) -> crate::Result<Url> {
        self.0.agent_rest_api_url("v1/agent/settings/think/models")
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn think_models_uses_the_hosted_agent_rest_url() {
        let dg = Deepgram::new("token").unwrap();

        assert_eq!(
            dg.voice_agent().think_models_url().unwrap().as_str(),
            "https://agent.deepgram.com/v1/agent/settings/think/models"
        );
    }

    #[test]
    fn custom_base_is_inherited_until_agent_rest_is_overridden() {
        let dg = Deepgram::with_base_url("http://gateway.internal/deepgram/")
            .unwrap()
            .with_agent_rest_url("http://agent.internal/agent/")
            .unwrap();

        assert_eq!(
            dg.voice_agent().think_models_url().unwrap().as_str(),
            "http://agent.internal/agent/v1/agent/settings/think/models"
        );
    }
}
