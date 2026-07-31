//! Client-to-server message types for the Voice Agent WebSocket.
//!
//! Mirrors `AgentV1*Message` schemas in
//! `asyncapi/schemas/schemas.agent.v1.yml`. Six are dynamic-control
//! messages sent during a session (`UpdateSpeak`, `UpdateThink`,
//! `UpdatePrompt`, `InjectUserMessage`, `InjectAgentMessage`,
//! `FunctionCallResponse`); one is a connection keep-alive
//! (`KeepAlive`); the eighth (`Settings`) is defined in
//! [`crate::agent::settings`] and is the only message that can carry the
//! agent's full configuration.
//!
//! [`ClientMessage`] wraps all eight as a discriminated union. Wire
//! discrimination is structural via each variant's own `type` field
//! (modeled as a single-variant marker enum); on the Rust side serde
//! [`#[serde(untagged)]`](serde::Deserialize) tries each variant in
//! turn and accepts whichever matches the JSON's `type`.
//!
//! Note: `FunctionCallResponseMessage` is bidirectional in the spec —
//! the same wire shape appears as both a client→server message and a
//! server→client event. The struct defined here is reused by the
//! server-emitted event when that surface lands.

use serde::{Deserialize, Serialize};

use crate::agent::settings::SettingsMessage;
use crate::agent::speak::SpeakSettings;
use crate::agent::think::ThinkSettings;

/// Discriminated union of all eight client-to-server JSON messages.
///
/// Audio frames sent by the client are binary WebSocket frames and are
/// therefore not part of this enum — they're handled by the connection
/// layer (separate from JSON dispatch).
//
// `Settings` carries the full agent configuration (~312 bytes) while
// `KeepAlive` is essentially empty. The disparity is intentional: each
// variant is constructed and immediately serialized (or deserialized
// and immediately matched), never stored long-term, so boxing the
// larger variants would add a heap allocation per WebSocket send for no
// observable benefit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum ClientMessage {
    /// Initial / re-issued configuration for the session.
    Settings(SettingsMessage),
    /// Swap the Speak provider mid-session.
    UpdateSpeak(UpdateSpeakMessage),
    /// Swap the Think provider mid-session.
    UpdateThink(UpdateThinkMessage),
    /// Replace the system prompt mid-session.
    UpdatePrompt(UpdatePromptMessage),
    /// Inject a synthetic user utterance into the conversation.
    InjectUserMessage(InjectUserMessageMessage),
    /// Trigger an agent utterance immediately.
    InjectAgentMessage(InjectAgentMessageMessage),
    /// Reply to a server `FunctionCallRequest` (client-side function execution).
    FunctionCallResponse(FunctionCallResponseMessage),
    /// Keep the WebSocket alive between user turns.
    KeepAlive(KeepAliveMessage),
}

impl ClientMessage {
    /// Convenience: wrap a `SettingsMessage`.
    pub fn settings(message: SettingsMessage) -> Self {
        Self::Settings(message)
    }

    /// Convenience: build a one-variant `UpdateSpeak` message.
    pub fn update_speak_one(speak: SpeakSettings) -> Self {
        Self::UpdateSpeak(UpdateSpeakMessage::one(speak))
    }

    /// Convenience: build a multi-variant `UpdateSpeak` message.
    pub fn update_speak_many(speak: impl IntoIterator<Item = SpeakSettings>) -> Self {
        Self::UpdateSpeak(UpdateSpeakMessage::many(speak))
    }

    /// Convenience: build a one-variant `UpdateThink` message.
    pub fn update_think_one(think: ThinkSettings) -> Self {
        Self::UpdateThink(UpdateThinkMessage::one(think))
    }

    /// Convenience: build a multi-variant `UpdateThink` message.
    pub fn update_think_many(think: impl IntoIterator<Item = ThinkSettings>) -> Self {
        Self::UpdateThink(UpdateThinkMessage::many(think))
    }

    /// Convenience: build an `UpdatePrompt` message.
    pub fn update_prompt(prompt: impl Into<String>) -> Self {
        Self::UpdatePrompt(UpdatePromptMessage::new(prompt))
    }

    /// Convenience: build an `InjectUserMessage`.
    pub fn inject_user_message(content: impl Into<String>) -> Self {
        Self::InjectUserMessage(InjectUserMessageMessage::new(content))
    }

    /// Convenience: build an `InjectAgentMessage` with the default behavior.
    pub fn inject_agent_message(message: impl Into<String>) -> Self {
        Self::InjectAgentMessage(InjectAgentMessageMessage::new(message))
    }

    /// Convenience: build a `FunctionCallResponse`.
    pub fn function_call_response(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self::FunctionCallResponse(FunctionCallResponseMessage::new(name, content))
    }

    /// Convenience: a `KeepAlive` message.
    pub fn keep_alive() -> Self {
        Self::KeepAlive(KeepAliveMessage::default())
    }
}

// ---------- UpdateSpeak ----------

/// Marker for the `"UpdateSpeak"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UpdateSpeakType {
    /// Always serializes as `"UpdateSpeak"`.
    #[default]
    UpdateSpeak,
}

/// Mirrors `AgentV1UpdateSpeakMessage` — change the Speak provider mid-session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpdateSpeakMessage {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: UpdateSpeakType,

    /// New Speak configuration. Single value or array on the wire; modeled
    /// as `Vec<SpeakSettings>` with custom serde so a one-element vec
    /// serializes as a scalar object.
    #[serde(
        serialize_with = "serialize_speak_one_or_many",
        deserialize_with = "deserialize_speak_one_or_many"
    )]
    pub speak: Vec<SpeakSettings>,
}

impl UpdateSpeakMessage {
    /// Construct with a single Speak provider.
    pub fn one(speak: SpeakSettings) -> Self {
        Self {
            message_type: UpdateSpeakType::default(),
            speak: vec![speak],
        }
    }

    /// Construct with an array of Speak providers.
    pub fn many(speak: impl IntoIterator<Item = SpeakSettings>) -> Self {
        Self {
            message_type: UpdateSpeakType::default(),
            speak: speak.into_iter().collect(),
        }
    }
}

// ---------- UpdateThink ----------

/// Marker for the `"UpdateThink"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UpdateThinkType {
    /// Always serializes as `"UpdateThink"`.
    #[default]
    UpdateThink,
}

/// Mirrors `AgentV1UpdateThinkMessage` — change the Think provider mid-session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpdateThinkMessage {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: UpdateThinkType,

    /// New Think configuration. Single value or array on the wire; modeled
    /// as `Vec<ThinkSettings>` with custom serde so a one-element vec
    /// serializes as a scalar object.
    #[serde(
        serialize_with = "serialize_think_one_or_many",
        deserialize_with = "deserialize_think_one_or_many"
    )]
    pub think: Vec<ThinkSettings>,
}

impl UpdateThinkMessage {
    /// Construct with a single Think provider.
    pub fn one(think: ThinkSettings) -> Self {
        Self {
            message_type: UpdateThinkType::default(),
            think: vec![think],
        }
    }

    /// Construct with an array of Think providers.
    pub fn many(think: impl IntoIterator<Item = ThinkSettings>) -> Self {
        Self {
            message_type: UpdateThinkType::default(),
            think: think.into_iter().collect(),
        }
    }
}

// ---------- UpdatePrompt ----------

/// Marker for the `"UpdatePrompt"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UpdatePromptType {
    /// Always serializes as `"UpdatePrompt"`.
    #[default]
    UpdatePrompt,
}

/// Mirrors `AgentV1UpdatePromptMessage` — replace the system prompt mid-session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpdatePromptMessage {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: UpdatePromptType,

    /// New system prompt.
    pub prompt: String,
}

impl UpdatePromptMessage {
    /// Construct with the given prompt.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            message_type: UpdatePromptType::default(),
            prompt: prompt.into(),
        }
    }
}

// ---------- InjectUserMessage ----------

/// Marker for the `"InjectUserMessage"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InjectUserMessageType {
    /// Always serializes as `"InjectUserMessage"`.
    #[default]
    InjectUserMessage,
}

/// Mirrors `AgentV1InjectUserMessageMessage` — inject a synthetic user
/// utterance into the conversation. The agent will respond as if the
/// user had spoken `content`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InjectUserMessageMessage {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: InjectUserMessageType,

    /// The phrase the agent should respond to.
    pub content: String,
}

impl InjectUserMessageMessage {
    /// Construct with the given content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            message_type: InjectUserMessageType::default(),
            content: content.into(),
        }
    }
}

// ---------- InjectAgentMessage ----------

/// Marker for the `"InjectAgentMessage"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InjectAgentMessageType {
    /// Always serializes as `"InjectAgentMessage"`.
    #[default]
    InjectAgentMessage,
}

/// Behavior knob on `InjectAgentMessage` — controls how injection
/// interacts with any in-progress turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum InjectAgentBehavior {
    /// Speak only if neither the user nor the agent is mid-turn. If a
    /// turn is in progress, the server replies with `InjectionRefused`.
    #[default]
    Default,
    /// Append the message after any queued `ConversationText` without
    /// interrupting the current turn or think response.
    Queue,
}

/// Mirrors `AgentV1InjectAgentMessageMessage` — make the agent speak an
/// arbitrary line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InjectAgentMessageMessage {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: InjectAgentMessageType,

    /// What the agent should say.
    pub message: String,

    /// How the injection interacts with any in-progress turn.
    /// Defaults to [`InjectAgentBehavior::Default`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub behavior: Option<InjectAgentBehavior>,
}

impl InjectAgentMessageMessage {
    /// Construct with the given utterance and the default behavior.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message_type: InjectAgentMessageType::default(),
            message: message.into(),
            behavior: None,
        }
    }

    /// Override the behavior.
    pub fn with_behavior(mut self, behavior: InjectAgentBehavior) -> Self {
        self.behavior = Some(behavior);
        self
    }
}

// ---------- FunctionCallResponse ----------

/// Marker for the `"FunctionCallResponse"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FunctionCallResponseType {
    /// Always serializes as `"FunctionCallResponse"`.
    #[default]
    FunctionCallResponse,
}

/// Mirrors `AgentV1FunctionCallResponseMessage` — bidirectional
/// function-call result.
///
/// In the **client → server** direction this responds to a server-emitted
/// `FunctionCallRequest` for a client-side function: `id` should match
/// the request's `id`. In the **server → client** direction it reports
/// the result of a server-side function execution; the server may omit
/// `id` when it doesn't track an internal request ID.
///
/// `name` and `content` are required in both directions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FunctionCallResponseMessage {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: FunctionCallResponseType,

    /// Identifier matching the originating `FunctionCallRequest`. Required
    /// for client responses; the server may omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Function name.
    pub name: String,

    /// Function result payload (opaque string per spec — typically a
    /// JSON-encoded blob).
    pub content: String,
}

impl FunctionCallResponseMessage {
    /// Construct without an `id` (suitable for server-emitted responses).
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            message_type: FunctionCallResponseType::default(),
            id: None,
            name: name.into(),
            content: content.into(),
        }
    }

    /// Construct with an explicit `id` (required for client → server use).
    pub fn with_id(
        id: impl Into<String>,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            message_type: FunctionCallResponseType::default(),
            id: Some(id.into()),
            name: name.into(),
            content: content.into(),
        }
    }
}

// ---------- KeepAlive ----------

/// Marker for the `"KeepAlive"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum KeepAliveType {
    /// Always serializes as `"KeepAlive"`.
    #[default]
    KeepAlive,
}

/// Mirrors `AgentV1ControlMessage` (`type: "KeepAlive"`).
///
/// The Voice Agent WebSocket can sit idle between turns; sending a
/// `KeepAlive` message every ~10s keeps proxies and load balancers
/// from closing the connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeepAliveMessage {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: KeepAliveType,
}

// ---------- one-or-many serde helpers ----------

fn serialize_speak_one_or_many<S>(values: &[SpeakSettings], ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if values.len() == 1 {
        values[0].serialize(ser)
    } else {
        values.serialize(ser)
    }
}

fn deserialize_speak_one_or_many<'de, D>(de: D) -> Result<Vec<SpeakSettings>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // See note in `crate::agent::settings` re: `large_enum_variant`.
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

fn serialize_think_one_or_many<S>(values: &[ThinkSettings], ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if values.len() == 1 {
        values[0].serialize(ser)
    } else {
        values.serialize(ser)
    }
}

fn deserialize_think_one_or_many<'de, D>(de: D) -> Result<Vec<ThinkSettings>, D::Error>
where
    D: serde::Deserializer<'de>,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::audio::{AudioConfig, AudioInput, AudioInputEncoding};
    use crate::agent::listen::{
        AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider,
    };
    use crate::agent::settings::{AgentConfig, InlineAgentConfig};
    use crate::agent::speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider};
    use crate::agent::think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider};
    use serde_json::json;

    fn sample_speak() -> SpeakSettings {
        SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
            DeepgramSpeakModel::Aura2ThaliaEn,
        )))
    }

    fn sample_think() -> ThinkSettings {
        ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
            OpenAiModel::Gpt4oMini,
        )))
    }

    #[test]
    fn update_speak_one_round_trip() {
        let raw = json!({
            "type": "UpdateSpeak",
            "speak": {
                "provider": {
                    "type": "deepgram",
                    "model": "aura-2-thalia-en"
                }
            }
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(msg, ClientMessage::UpdateSpeak(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn update_speak_many_round_trip() {
        let raw = json!({
            "type": "UpdateSpeak",
            "speak": [
                { "provider": { "type": "deepgram", "model": "aura-asteria-en" } },
                { "provider": { "type": "deepgram", "model": "aura-2-luna-en" } }
            ]
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        if let ClientMessage::UpdateSpeak(m) = &msg {
            assert_eq!(m.speak.len(), 2);
        } else {
            panic!("expected UpdateSpeak");
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn update_think_round_trip() {
        let raw = json!({
            "type": "UpdateThink",
            "think": {
                "provider": {
                    "type": "open_ai",
                    "model": "gpt-4o"
                }
            }
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(msg, ClientMessage::UpdateThink(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn update_prompt_round_trip() {
        let raw = json!({
            "type": "UpdatePrompt",
            "prompt": "You are now in customer-support mode."
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        if let ClientMessage::UpdatePrompt(m) = &msg {
            assert_eq!(m.prompt, "You are now in customer-support mode.");
        } else {
            panic!("expected UpdatePrompt");
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn inject_user_message_round_trip() {
        let raw = json!({
            "type": "InjectUserMessage",
            "content": "What's the weather?"
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(msg, ClientMessage::InjectUserMessage(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn inject_agent_message_default_behavior() {
        let raw = json!({
            "type": "InjectAgentMessage",
            "message": "Sorry, I missed that."
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        if let ClientMessage::InjectAgentMessage(m) = &msg {
            assert_eq!(m.message, "Sorry, I missed that.");
            assert!(m.behavior.is_none());
        } else {
            panic!("expected InjectAgentMessage");
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn inject_agent_message_queue_behavior() {
        let raw = json!({
            "type": "InjectAgentMessage",
            "message": "By the way, business hours are 9-5.",
            "behavior": "queue"
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        if let ClientMessage::InjectAgentMessage(m) = &msg {
            assert_eq!(m.behavior, Some(InjectAgentBehavior::Queue));
        } else {
            panic!("expected InjectAgentMessage");
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn function_call_response_with_id_round_trip() {
        let raw = json!({
            "type": "FunctionCallResponse",
            "id": "func_42",
            "name": "get_weather",
            "content": "{\"temp\": 72}"
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        if let ClientMessage::FunctionCallResponse(m) = &msg {
            assert_eq!(m.id.as_deref(), Some("func_42"));
            assert_eq!(m.name, "get_weather");
        } else {
            panic!("expected FunctionCallResponse");
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn function_call_response_without_id_round_trip() {
        let raw = json!({
            "type": "FunctionCallResponse",
            "name": "internal_lookup",
            "content": "[{\"id\": 1}]"
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        if let ClientMessage::FunctionCallResponse(m) = &msg {
            assert!(m.id.is_none());
        } else {
            panic!("expected FunctionCallResponse");
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn keep_alive_round_trip() {
        let raw = json!({ "type": "KeepAlive" });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(msg, ClientMessage::KeepAlive(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn settings_dispatched_through_client_message() {
        // Sanity: the existing SettingsMessage routes through the
        // untagged enum's first arm.
        let raw = json!({
            "type": "Settings",
            "audio": {
                "input": { "encoding": "linear16", "sample_rate": 16000 }
            },
            "agent": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
        });
        let msg: ClientMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(msg, ClientMessage::Settings(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn unknown_type_does_not_match_any_variant() {
        let raw = json!({ "type": "MysteryMessage", "x": 1 });
        let err = serde_json::from_value::<ClientMessage>(raw).unwrap_err();
        // Untagged dispatch produces a generic error pointing at the
        // last attempted variant; exact message shape is serde-internal.
        // We just want to make sure deserialization fails cleanly.
        let s = err.to_string();
        assert!(!s.is_empty());
    }

    #[test]
    fn convenience_constructors() {
        // Exercise the helpers on ClientMessage so they actually compile.
        let _ = ClientMessage::settings(crate::agent::SettingsMessage::new(
            AudioConfig::new(
                Some(AudioInput::new(AudioInputEncoding::Linear16, 16_000)),
                None,
            ),
            AgentConfig::inline(InlineAgentConfig::from_parts(
                AgentListenSettings::new(AgentListenProvider::DeepgramV2(
                    DeepgramListenV2Provider::new("flux-general-en"),
                )),
                sample_think(),
                sample_speak(),
            )),
        ));
        let _ = ClientMessage::update_speak_one(sample_speak());
        let _ = ClientMessage::update_speak_many([sample_speak(), sample_speak()]);
        let _ = ClientMessage::update_think_one(sample_think());
        let _ = ClientMessage::update_think_many([sample_think()]);
        let _ = ClientMessage::update_prompt("hi");
        let _ = ClientMessage::inject_user_message("hi");
        let _ = ClientMessage::inject_agent_message("hi");
        let _ = ClientMessage::function_call_response("fn", "{}");
        let _ = ClientMessage::keep_alive();
    }

    #[test]
    fn inject_agent_with_behavior_builder() {
        let m = InjectAgentMessageMessage::new("hi").with_behavior(InjectAgentBehavior::Queue);
        let v = serde_json::to_value(&m).unwrap();
        assert_eq!(v["behavior"], "queue");
    }

    #[test]
    fn function_call_response_with_id_builder() {
        let m = FunctionCallResponseMessage::with_id("f1", "fn", "{}");
        assert_eq!(m.id.as_deref(), Some("f1"));
    }

    #[test]
    fn behavior_default_is_default() {
        assert_eq!(InjectAgentBehavior::default(), InjectAgentBehavior::Default);
    }
}
