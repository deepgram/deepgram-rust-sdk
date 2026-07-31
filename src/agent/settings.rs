//! Voice Agent `Settings` message — the full client-to-server payload
//! that configures an agent session.
//!
//! Mirrors `AgentV1SettingsMessage` in `asyncapi/schemas/schemas.agent.v1.yml`.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::agent::audio::AudioConfig;
use crate::agent::history::HistoryMessage;
use crate::agent::listen::AgentListenSettings;
use crate::agent::speak::SpeakSettings;
use crate::agent::think::ThinkSettings;

/// Wire discriminator for the Settings message — always `"Settings"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SettingsMessageType {
    /// Always serializes as `"Settings"`.
    #[default]
    Settings,
}

/// Top-level `Settings` message sent from client to server when starting
/// or reconfiguring an agent session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SettingsMessage {
    /// Always [`SettingsMessageType::Settings`]. Round-tripped for fidelity.
    #[serde(rename = "type", default)]
    pub message_type: SettingsMessageType,

    /// Tags to associate with the session for usage reporting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Whether to enable experimental features.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<bool>,

    /// Reporting flags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<SettingsFlags>,

    /// Whether to opt out of the Deepgram Model Improvement Program.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mip_opt_out: Option<bool>,

    /// Audio I/O configuration.
    pub audio: AudioConfig,

    /// Agent configuration — either an inline config or a saved-config UUID.
    pub agent: AgentConfig,
}

impl SettingsMessage {
    /// Construct with the given audio config and agent config; all flag
    /// fields default to absent.
    pub fn new(audio: AudioConfig, agent: AgentConfig) -> Self {
        Self {
            message_type: SettingsMessageType::Settings,
            tags: Vec::new(),
            experimental: None,
            flags: None,
            mip_opt_out: None,
            audio,
            agent,
        }
    }

    #[allow(missing_docs)]
    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    #[allow(missing_docs)]
    pub fn with_experimental(mut self, experimental: bool) -> Self {
        self.experimental = Some(experimental);
        self
    }

    #[allow(missing_docs)]
    pub fn with_flags(mut self, flags: SettingsFlags) -> Self {
        self.flags = Some(flags);
        self
    }

    #[allow(missing_docs)]
    pub fn with_mip_opt_out(mut self, mip_opt_out: bool) -> Self {
        self.mip_opt_out = Some(mip_opt_out);
        self
    }
}

/// Reporting flags on the `Settings` message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SettingsFlags {
    /// Whether to enable history message reporting. Spec default: `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<bool>,
}

impl SettingsFlags {
    /// Construct with the given history flag.
    pub fn with_history(history: bool) -> Self {
        Self {
            history: Some(history),
        }
    }
}

/// Agent configuration on `SettingsMessage.agent` — either an inline
/// config or a UUID referencing a previously-saved configuration.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AgentConfig {
    /// Full inline configuration.
    Inline(InlineAgentConfig),
    /// Reference to a saved agent configuration by UUID.
    Saved(Uuid),
}

impl AgentConfig {
    /// Construct from an inline config.
    pub fn inline(config: InlineAgentConfig) -> Self {
        Self::Inline(config)
    }

    /// Construct from a saved-agent UUID.
    pub fn saved(id: Uuid) -> Self {
        Self::Saved(id)
    }
}

impl Serialize for AgentConfig {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Inline(cfg) => cfg.serialize(ser),
            Self::Saved(id) => ser.serialize_str(&id.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for AgentConfig {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(de)?;
        match value {
            serde_json::Value::String(s) => Uuid::parse_str(&s).map(Self::Saved).map_err(|e| {
                D::Error::custom(format!(
                    "agent field expected an inline config object or a UUID string; got string {s:?} which failed to parse as UUID: {e}"
                ))
            }),
            serde_json::Value::Object(_) => serde_json::from_value::<InlineAgentConfig>(value)
                .map(Self::Inline)
                .map_err(D::Error::custom),
            other => Err(D::Error::custom(format!(
                "agent field expected an inline config object or a UUID string; got {other:?}"
            ))),
        }
    }
}

/// Inline agent configuration on `SettingsMessage.agent`.
///
/// All fields are optional individually, but at least `listen`, `think`,
/// and `speak` are required for a working agent. The Rust types let you
/// build incrementally; the API will reject incomplete configs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InlineAgentConfig {
    /// Deprecated — set `language` on `listen.provider` and `speak.provider` instead.
    #[deprecated(
        since = "0.10.0",
        note = "Set `language` on listen.provider and speak.provider instead. Mirrors deprecation in the AsyncAPI spec."
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Conversation context — replayed messages and function-call history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<AgentContext>,

    /// Speech-to-text configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listen: Option<AgentListenSettings>,

    /// LLM configuration. Single value or array on the wire.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_think_one_or_many",
        deserialize_with = "deserialize_think_one_or_many"
    )]
    pub think: Vec<ThinkSettings>,

    /// TTS configuration. Single value or array on the wire.
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_speak_one_or_many",
        deserialize_with = "deserialize_speak_one_or_many"
    )]
    pub speak: Vec<SpeakSettings>,

    /// Optional message the agent speaks at the start of the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub greeting: Option<String>,
}

#[allow(deprecated)]
impl InlineAgentConfig {
    /// Construct an empty inline config; populate via the `with_*` builders.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct with the three commonly-required pieces.
    pub fn from_parts(
        listen: AgentListenSettings,
        think: ThinkSettings,
        speak: SpeakSettings,
    ) -> Self {
        Self {
            language: None,
            context: None,
            listen: Some(listen),
            think: vec![think],
            speak: vec![speak],
            greeting: None,
        }
    }

    #[allow(missing_docs)]
    pub fn with_listen(mut self, listen: AgentListenSettings) -> Self {
        self.listen = Some(listen);
        self
    }

    #[allow(missing_docs)]
    pub fn with_think(mut self, think: ThinkSettings) -> Self {
        self.think = vec![think];
        self
    }

    #[allow(missing_docs)]
    pub fn with_thinks(mut self, think: impl IntoIterator<Item = ThinkSettings>) -> Self {
        self.think = think.into_iter().collect();
        self
    }

    #[allow(missing_docs)]
    pub fn with_speak(mut self, speak: SpeakSettings) -> Self {
        self.speak = vec![speak];
        self
    }

    #[allow(missing_docs)]
    pub fn with_speaks(mut self, speak: impl IntoIterator<Item = SpeakSettings>) -> Self {
        self.speak = speak.into_iter().collect();
        self
    }

    #[allow(missing_docs)]
    pub fn with_context(mut self, context: AgentContext) -> Self {
        self.context = Some(context);
        self
    }

    #[allow(missing_docs)]
    pub fn with_greeting(mut self, greeting: impl Into<String>) -> Self {
        self.greeting = Some(greeting.into());
        self
    }
}

/// Conversation context for an inline agent config — currently just message history.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentContext {
    /// Conversation history — user/assistant utterances and function-call records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<HistoryMessage>,
}

impl AgentContext {
    /// Construct from a list of history messages.
    pub fn new(messages: impl IntoIterator<Item = HistoryMessage>) -> Self {
        Self {
            messages: messages.into_iter().collect(),
        }
    }
}

// --- one-or-many serde helpers for `think` and `speak` ---

fn serialize_think_one_or_many<S>(values: &[ThinkSettings], ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if values.len() == 1 {
        values[0].serialize(ser)
    } else {
        values.serialize(ser)
    }
}

fn deserialize_think_one_or_many<'de, D>(de: D) -> Result<Vec<ThinkSettings>, D::Error>
where
    D: Deserializer<'de>,
{
    // The disparity between Many (~24 bytes) and One (~272 bytes) is fine
    // here: this enum is a transient helper used only during deserialization
    // and is dropped immediately after the match below.
    #[derive(Deserialize)]
    #[serde(untagged)]
    #[allow(clippy::large_enum_variant)]
    enum OneOrMany {
        Many(Vec<ThinkSettings>),
        One(ThinkSettings),
    }
    Ok(match OneOrMany::deserialize(de)? {
        OneOrMany::One(x) => vec![x],
        OneOrMany::Many(v) => v,
    })
}

fn serialize_speak_one_or_many<S>(values: &[SpeakSettings], ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if values.len() == 1 {
        values[0].serialize(ser)
    } else {
        values.serialize(ser)
    }
}

fn deserialize_speak_one_or_many<'de, D>(de: D) -> Result<Vec<SpeakSettings>, D::Error>
where
    D: Deserializer<'de>,
{
    // See note on `deserialize_think_one_or_many` re: `large_enum_variant`.
    #[derive(Deserialize)]
    #[serde(untagged)]
    #[allow(clippy::large_enum_variant)]
    enum OneOrMany {
        Many(Vec<SpeakSettings>),
        One(SpeakSettings),
    }
    Ok(match OneOrMany::deserialize(de)? {
        OneOrMany::One(x) => vec![x],
        OneOrMany::Many(v) => v,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::audio::{
        AudioConfig, AudioContainer, AudioInput, AudioInputEncoding, AudioOutput,
        AudioOutputEncoding,
    };
    use crate::agent::history::{ConversationRole, HistoryMessage};
    use crate::agent::listen::{
        AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider,
    };
    use crate::agent::speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider};
    use crate::agent::think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider};
    use serde_json::json;

    fn sample_listen() -> AgentListenSettings {
        AgentListenSettings::new(AgentListenProvider::DeepgramV2(
            DeepgramListenV2Provider::new("flux-general-en"),
        ))
    }

    fn sample_think() -> ThinkSettings {
        ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
            OpenAiModel::Gpt4oMini,
        )))
    }

    fn sample_speak() -> SpeakSettings {
        SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
            DeepgramSpeakModel::Aura2ThaliaEn,
        )))
    }

    #[test]
    fn agent_config_serializes_uuid_as_string() {
        let id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        let cfg = AgentConfig::saved(id);
        let value = serde_json::to_value(&cfg).unwrap();
        assert_eq!(value, json!("a1b2c3d4-e5f6-7890-abcd-ef1234567890"));
        // Round-trip back to a UUID variant.
        let back: AgentConfig = serde_json::from_value(value).unwrap();
        assert!(matches!(back, AgentConfig::Saved(parsed) if parsed == id));
    }

    #[test]
    fn agent_config_inline_round_trip() {
        let cfg = AgentConfig::inline(InlineAgentConfig::from_parts(
            sample_listen(),
            sample_think(),
            sample_speak(),
        ));
        let value = serde_json::to_value(&cfg).unwrap();
        // think/speak with a single element collapse to scalar objects, not arrays.
        assert!(value.get("think").unwrap().is_object());
        assert!(value.get("speak").unwrap().is_object());
        let back: AgentConfig = serde_json::from_value(value).unwrap();
        assert_eq!(back, cfg);
    }

    #[test]
    fn agent_config_inline_with_multiple_thinks_uses_array() {
        let mut cfg =
            InlineAgentConfig::from_parts(sample_listen(), sample_think(), sample_speak());
        cfg.think.push(sample_think());
        let value = serde_json::to_value(AgentConfig::inline(cfg)).unwrap();
        assert!(value.get("think").unwrap().is_array());
    }

    #[test]
    fn agent_config_rejects_invalid_uuid_string() {
        let raw = json!("not-a-uuid");
        let err = serde_json::from_value::<AgentConfig>(raw).unwrap_err();
        assert!(err.to_string().contains("UUID"), "got: {err}");
    }

    #[test]
    fn settings_message_full_round_trip() {
        let raw = json!({
            "type": "Settings",
            "tags": ["prod"],
            "experimental": false,
            "flags": { "history": true },
            "mip_opt_out": false,
            "audio": {
                "input": { "encoding": "linear16", "sample_rate": 24000 },
                "output": {
                    "encoding": "mp3",
                    "sample_rate": 22050,
                    "container": "none"
                }
            },
            "agent": {
                "listen": {
                    "provider": {
                        "type": "deepgram",
                        "version": "v2",
                        "model": "flux-general-en"
                    }
                },
                "think": {
                    "provider": {
                        "type": "open_ai",
                        "model": "gpt-4o-mini"
                    }
                },
                "speak": {
                    "provider": {
                        "type": "deepgram",
                        "model": "aura-2-thalia-en"
                    }
                },
                "greeting": "Hello there!"
            }
        });
        let msg: SettingsMessage = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(msg.tags, vec!["prod".to_string()]);
        assert_eq!(msg.experimental, Some(false));
        assert_eq!(msg.flags.unwrap().history, Some(true));
        assert!(matches!(msg.agent, AgentConfig::Inline(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn settings_message_minimal_round_trip() {
        let raw = json!({
            "type": "Settings",
            "audio": {
                "input": { "encoding": "linear16", "sample_rate": 16000 }
            },
            "agent": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
        });
        let msg: SettingsMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(msg.tags.is_empty());
        assert!(matches!(msg.agent, AgentConfig::Saved(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn inline_agent_config_with_history_context() {
        let context =
            AgentContext::new([HistoryMessage::conversation(ConversationRole::User, "Hi")]);
        let inline = InlineAgentConfig::from_parts(sample_listen(), sample_think(), sample_speak())
            .with_context(context)
            .with_greeting("Welcome");
        let value = serde_json::to_value(&inline).unwrap();
        assert_eq!(value["greeting"], "Welcome");
        assert_eq!(value["context"]["messages"][0]["content"], "Hi");
    }

    #[test]
    fn settings_builders_compose() {
        let msg = SettingsMessage::new(
            AudioConfig::new(
                Some(AudioInput::new(AudioInputEncoding::Linear16, 16_000)),
                Some(
                    AudioOutput::new()
                        .with_encoding(AudioOutputEncoding::Linear16)
                        .with_sample_rate(24_000)
                        .with_container(AudioContainer::Wav),
                ),
            ),
            AgentConfig::inline(InlineAgentConfig::from_parts(
                sample_listen(),
                sample_think(),
                sample_speak(),
            )),
        )
        .with_tags(["staging"])
        .with_experimental(true)
        .with_flags(SettingsFlags::with_history(false))
        .with_mip_opt_out(true);
        let value = serde_json::to_value(&msg).unwrap();
        assert_eq!(value["tags"], json!(["staging"]));
        assert_eq!(value["experimental"], json!(true));
        assert_eq!(value["flags"]["history"], json!(false));
        assert_eq!(value["mip_opt_out"], json!(true));
    }

    #[test]
    fn think_one_or_many_array_round_trip() {
        let raw = json!({
            "listen": {
                "provider": {
                    "type": "deepgram",
                    "version": "v2",
                    "model": "flux-general-en"
                }
            },
            "think": [
                { "provider": { "type": "open_ai", "model": "gpt-4o" } },
                { "provider": { "type": "anthropic", "model": "claude-3-5-haiku-latest" } }
            ],
            "speak": {
                "provider": {
                    "type": "deepgram",
                    "model": "aura-asteria-en"
                }
            }
        });
        let cfg: InlineAgentConfig = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(cfg.think.len(), 2);
        assert_eq!(serde_json::to_value(&cfg).unwrap(), raw);
    }
}
