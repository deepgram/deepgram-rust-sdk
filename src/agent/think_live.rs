//! Voice Agent Think Live settings.
//!
//! Think Live runs alongside the reasoning LLM to produce backchannels,
//! presence checks, function-call fillers, and frustration-detection events.

use serde::{Deserialize, Serialize};

/// Think Live configuration on `agent.think_live`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkLiveSettings {
    /// Deepgram Think Live provider settings.
    pub provider: ThinkLiveProvider,
}

impl ThinkLiveSettings {
    /// Construct Think Live settings for the given provider.
    pub fn new(provider: ThinkLiveProvider) -> Self {
        Self { provider }
    }
}

/// Wire-level provider discriminator for Think Live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ThinkLiveProviderType {
    /// Always serializes as `"deepgram"`.
    #[default]
    #[serde(rename = "deepgram")]
    Deepgram,
}

/// Wire-level Think Live API version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ThinkLiveVersion {
    /// Version 1.
    #[default]
    #[serde(rename = "v1")]
    V1,
}

/// Deepgram Think Live provider settings.
///
/// Think Live is enabled only when this provider is included in
/// [`ThinkLiveSettings`]. It runs independently from the reasoning LLM in
/// `agent.think`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkLiveProvider {
    /// Always [`ThinkLiveProviderType::Deepgram`].
    #[serde(rename = "type", default)]
    pub provider_type: ThinkLiveProviderType,

    /// Always [`ThinkLiveVersion::V1`].
    #[serde(default)]
    pub version: ThinkLiveVersion,

    /// Probability of a backchannel while the user is speaking. The service
    /// accepts values from 0 to 1; set 0 to disable backchannels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_backchannel_sensitivity: Option<f64>,

    /// Maximum presence-check prompts after extended silence. Set 0 to
    /// disable presence checks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_check_max_count: Option<u32>,

    /// Milliseconds between presence-check prompts. The service accepts 1 to
    /// 60,000 and floors the first prompt at 3,000 milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence_check_interval_ms: Option<u32>,

    /// Number of frustration events before the server marks one as escalated.
    /// Set 0 to suppress frustration events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frustration_max_count: Option<u32>,

    /// Terms Think Live must not say. A multi-word phrase is one array item.
    /// These restrictions do not affect the reasoning LLM.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub restricted_terms: Vec<String>,
}

impl ThinkLiveProvider {
    /// Construct the default Deepgram Think Live provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set backchannel sensitivity.
    pub fn with_agent_backchannel_sensitivity(mut self, value: f64) -> Self {
        self.agent_backchannel_sensitivity = Some(value);
        self
    }

    /// Set the maximum number of presence-check prompts.
    pub fn with_presence_check_max_count(mut self, value: u32) -> Self {
        self.presence_check_max_count = Some(value);
        self
    }

    /// Set the presence-check interval in milliseconds.
    pub fn with_presence_check_interval_ms(mut self, value: u32) -> Self {
        self.presence_check_interval_ms = Some(value);
        self
    }

    /// Set the frustration escalation threshold.
    pub fn with_frustration_max_count(mut self, value: u32) -> Self {
        self.frustration_max_count = Some(value);
        self
    }

    /// Replace the list of restricted terms.
    pub fn with_restricted_terms(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.restricted_terms = values.into_iter().map(Into::into).collect();
        self
    }
}

impl Default for ThinkLiveProvider {
    fn default() -> Self {
        Self {
            provider_type: ThinkLiveProviderType::Deepgram,
            version: ThinkLiveVersion::V1,
            agent_backchannel_sensitivity: None,
            presence_check_max_count: None,
            presence_check_interval_ms: None,
            frustration_max_count: None,
            restricted_terms: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn provider_round_trips_full_configuration() {
        let raw = json!({
            "type": "deepgram",
            "version": "v1",
            "agent_backchannel_sensitivity": 0.73,
            "presence_check_max_count": 3,
            "presence_check_interval_ms": 4500,
            "frustration_max_count": 2,
            "restricted_terms": ["human", "transfer"]
        });
        let provider: ThinkLiveProvider = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(provider.provider_type, ThinkLiveProviderType::Deepgram);
        assert_eq!(provider.version, ThinkLiveVersion::V1);
        assert_eq!(provider.restricted_terms, ["human", "transfer"]);
        assert_eq!(serde_json::to_value(provider).unwrap(), raw);
    }

    #[test]
    fn default_provider_has_the_required_wire_fields() {
        assert_eq!(
            serde_json::to_value(ThinkLiveProvider::new()).unwrap(),
            json!({ "type": "deepgram", "version": "v1" })
        );
    }
}
