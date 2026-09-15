//! Server-to-client event types for the Voice Agent WebSocket.
//!
//! Mirrors the `AgentV1*Event` schemas in
//! `asyncapi/schemas/schemas.agent.v1.yml`. [`AgentResponse`] is the
//! discriminated union over every JSON event the server can emit during
//! a session, plus an [`AgentResponse::Unknown`] catch-all for
//! forward-compatibility with future event types.
//!
//! Wire dispatch is structural via each variant's own `type` field —
//! same approach as [`crate::agent::messages::ClientMessage`]. The
//! [`AgentResponse::FunctionCallResponse`] variant reuses
//! [`crate::agent::messages::FunctionCallResponseMessage`] since the
//! spec defines that shape as bidirectional.
//!
//! Audio frames sent by the server are binary WebSocket frames and are
//! not part of this enum — they're delivered out-of-band by the
//! connection layer.

use serde::{Deserialize, Serialize};

use crate::agent::history::{ConversationRole, HistoryMessage};
use crate::agent::messages::FunctionCallResponseMessage;

/// Discriminated union of every server-emitted JSON event.
///
/// Variants are tried in order during deserialization; the
/// [`AgentResponse::Unknown`] tail variant matches anything that didn't
/// fit a typed shape and exposes the raw JSON for inspection or logging.
//
// Same `large_enum_variant` rationale as `ClientMessage`: events are
// constructed once during deserialization and immediately consumed by a
// match — boxing the largest variants would only add a heap allocation
// per received WebSocket frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
#[allow(clippy::large_enum_variant)]
pub enum AgentResponse {
    /// Connection successfully opened.
    Welcome(WelcomeEvent),
    /// `Settings` message was received and applied.
    SettingsApplied(SettingsAppliedEvent),
    /// User or assistant utterance text.
    ConversationText(ConversationTextEvent),
    /// User has begun speaking (VAD trigger).
    UserStartedSpeaking(UserStartedSpeakingEvent),
    /// Agent is processing a response.
    AgentThinking(AgentThinkingEvent),
    /// Server requests one or more function calls.
    FunctionCallRequest(FunctionCallRequestEvent),
    /// Server cancelled client-side function calls it already sent
    /// (the user started speaking again). Do not respond to these IDs.
    FunctionCallCancelled(FunctionCallCancelledEvent),
    /// Server has begun streaming the agent's audio response (only with
    /// `experimental` flag enabled).
    AgentStartedSpeaking(AgentStartedSpeakingEvent),
    /// Server has finished streaming the agent's audio response.
    AgentAudioDone(AgentAudioDoneEvent),
    /// Fatal error.
    Error(ErrorEvent),
    /// Non-fatal warning.
    Warning(WarningEvent),
    /// Replay of conversation history. Reuses [`HistoryMessage`] since
    /// the wire shape is identical to `agent.context.messages[]`.
    History(HistoryMessage),
    /// Per-turn latency breakdown across STT, LLM, and TTS.
    LatencyReport(LatencyReportEvent),
    /// Confirms an `UpdateListen` was applied.
    ListenUpdated(ListenUpdatedEvent),
    /// Confirms an `UpdatePrompt` was applied.
    PromptUpdated(PromptUpdatedEvent),
    /// Confirms an `UpdateSpeak` was applied.
    SpeakUpdated(SpeakUpdatedEvent),
    /// Confirms an `UpdateThink` was applied.
    ThinkUpdated(ThinkUpdatedEvent),
    /// `InjectAgentMessage` was rejected (e.g. mid-turn with `Default` behavior).
    InjectionRefused(InjectionRefusedEvent),
    /// Server-side function execution completed (server emits its own result).
    FunctionCallResponse(FunctionCallResponseMessage),
    /// Forward-compatibility escape — any JSON event the SDK does not
    /// yet model lands here with its raw payload preserved.
    Unknown(serde_json::Value),
}

// ---------- Welcome ----------

/// Marker for the `"Welcome"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WelcomeType {
    /// Always serializes as `"Welcome"`.
    #[default]
    Welcome,
}

/// Mirrors `AgentV1WelcomeMessage` — sent immediately after connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WelcomeEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: WelcomeType,

    /// Unique identifier for the session.
    pub request_id: String,
}

// ---------- SettingsApplied ----------

/// Marker for the `"SettingsApplied"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SettingsAppliedType {
    /// Always serializes as `"SettingsApplied"`.
    #[default]
    SettingsApplied,
}

/// Mirrors `AgentV1SettingsAppliedEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SettingsAppliedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: SettingsAppliedType,
}

// ---------- ConversationText ----------

/// Marker for the `"ConversationText"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConversationTextType {
    /// Always serializes as `"ConversationText"`.
    #[default]
    ConversationText,
}

/// Mirrors `AgentV1ConversationTextEvent` — relays user/assistant
/// utterances in real time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConversationTextEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: ConversationTextType,

    /// Who spoke.
    pub role: ConversationRole,

    /// What they said.
    pub content: String,

    /// Active language hints at the time of the turn. Only populated on
    /// user-role messages when the listen model is `flux-general-multi`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages_hinted: Vec<String>,

    /// Languages detected in the user's speech (descending by word count).
    /// Only populated on user-role messages when the listen model is
    /// `flux-general-multi`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
}

// ---------- UserStartedSpeaking ----------

/// Marker for the `"UserStartedSpeaking"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UserStartedSpeakingType {
    /// Always serializes as `"UserStartedSpeaking"`.
    #[default]
    UserStartedSpeaking,
}

/// Mirrors `AgentV1UserStartedSpeakingEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UserStartedSpeakingEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: UserStartedSpeakingType,
}

// ---------- AgentThinking ----------

/// Marker for the `"AgentThinking"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AgentThinkingType {
    /// Always serializes as `"AgentThinking"`.
    #[default]
    AgentThinking,
}

/// Mirrors `AgentV1AgentThinkingEvent` — the agent's thought process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentThinkingEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: AgentThinkingType,

    /// The text of the agent's thought.
    pub content: String,
}

// ---------- FunctionCallRequest ----------

/// Marker for the `"FunctionCallRequest"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FunctionCallRequestType {
    /// Always serializes as `"FunctionCallRequest"`.
    #[default]
    FunctionCallRequest,
}

/// Mirrors `AgentV1FunctionCallRequestEvent` — server requests one or
/// more functions to be executed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FunctionCallRequestEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: FunctionCallRequestType,

    /// Functions to be called.
    pub functions: Vec<AgentFunctionCall>,
}

/// Single function call entry in a `FunctionCallRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentFunctionCall {
    /// Unique identifier — used to correlate with the matching `FunctionCallResponse`.
    pub id: String,

    /// Function name.
    pub name: String,

    /// JSON-encoded arguments (opaque string per spec — do not auto-parse).
    pub arguments: String,

    /// Whether the call should be executed client-side. If `false`, the
    /// server will execute it and emit a `FunctionCallResponse` event.
    pub client_side: bool,

    /// Some Gemini models require this as an additional call identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

// ---------- FunctionCallCancelled ----------

/// Marker for the `"FunctionCallCancelled"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FunctionCallCancelledType {
    /// Always serializes as `"FunctionCallCancelled"`.
    #[default]
    FunctionCallCancelled,
}

/// Mirrors `AgentV1FunctionCallCancelledEvent` — the server cancelled
/// one or more client-side function calls it had already sent in a
/// `FunctionCallRequest`, because the user started speaking again
/// (either during the speculative window before the turn was confirmed,
/// or on barge-in after it). The client must abandon these calls and
/// must **not** send a `FunctionCallResponse` for them.
///
/// Only calls the client already received are announced; calls still
/// held by `defer_until_eot` are dropped silently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FunctionCallCancelledEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: FunctionCallCancelledType,

    /// The cancelled calls.
    pub functions: Vec<CancelledFunctionCall>,
}

impl FunctionCallCancelledEvent {
    /// IDs of every cancelled call — convenient for pruning a map of
    /// in-flight function executions keyed by `FunctionCallRequest` id.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.functions.iter().map(|f| f.id.as_str())
    }
}

/// Single entry in a `FunctionCallCancelled` event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CancelledFunctionCall {
    /// Identifier from the cancelled `FunctionCallRequest`.
    pub id: String,

    /// Function name.
    pub name: String,
}

// ---------- AgentStartedSpeaking ----------

/// Marker for the `"AgentStartedSpeaking"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AgentStartedSpeakingType {
    /// Always serializes as `"AgentStartedSpeaking"`.
    #[default]
    AgentStartedSpeaking,
}

/// Mirrors `AgentV1AgentStartedSpeakingEvent` — emitted only when the
/// `experimental` flag is enabled in `Settings`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentStartedSpeakingEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: AgentStartedSpeakingType,

    /// Total seconds from receiving the user utterance to the start of
    /// the agent's reply.
    pub total_latency: f64,

    /// Portion of total latency attributable to text-to-speech.
    pub tts_latency: f64,

    /// Portion of total latency attributable to text-to-text (typically the LLM).
    pub ttt_latency: f64,
}

// ---------- AgentAudioDone ----------

/// Marker for the `"AgentAudioDone"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AgentAudioDoneType {
    /// Always serializes as `"AgentAudioDone"`.
    #[default]
    AgentAudioDone,
}

/// Mirrors `AgentV1AgentAudioDoneEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AgentAudioDoneEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: AgentAudioDoneType,
}

// ---------- Error ----------

/// Marker for the `"Error"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ErrorType {
    /// Always serializes as `"Error"`.
    #[default]
    Error,
}

/// Mirrors `AgentV1ErrorEvent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ErrorEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: ErrorType,

    /// Human-readable description of what went wrong.
    pub description: String,

    /// Error code identifying the failure.
    pub code: String,
}

// ---------- Warning ----------

/// Marker for the `"Warning"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WarningType {
    /// Always serializes as `"Warning"`.
    #[default]
    Warning,
}

/// Mirrors `AgentV1WarningEvent` — non-fatal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WarningEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: WarningType,

    /// Human-readable description.
    pub description: String,

    /// Warning code identifying the issue.
    pub code: String,
}

// ---------- LatencyReport ----------

/// Marker for the `"LatencyReport"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LatencyReportType {
    /// Always serializes as `"LatencyReport"`.
    #[default]
    LatencyReport,
}

/// Mirrors `AgentV1LatencyReportEvent` — detailed latency breakdown
/// across the STT, LLM, and TTS pipeline, emitted after a turn.
///
/// All latency fields are seconds and optional; the server omits any that
/// don't apply to the turn (e.g. `ttt_tool_latency` when no tool was
/// called).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct LatencyReportEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: LatencyReportType,

    /// Speech-to-text: audio received → transcript produced.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stt_latency: Option<f64>,

    /// Time to first token of any type (text, tool call, or thinking).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttt_token_latency: Option<f64>,

    /// Time to first text token from the LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttt_text_latency: Option<f64>,

    /// Time to first tool-call token from the LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttt_tool_latency: Option<f64>,

    /// Time to first thinking token from the LLM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttt_thinking_latency: Option<f64>,

    /// Text-to-speech: first text token → first audio byte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tts_latency: Option<f64>,

    /// End-to-end: user utterance end → first audio byte.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_latency: Option<f64>,
}

// ---------- ListenUpdated / PromptUpdated / SpeakUpdated / ThinkUpdated ----------

/// Marker for the `"ListenUpdated"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ListenUpdatedType {
    /// Always serializes as `"ListenUpdated"`.
    #[default]
    ListenUpdated,
}

/// Mirrors `AgentV1ListenUpdatedEvent` — confirms `UpdateListen` applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ListenUpdatedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: ListenUpdatedType,
}

/// Marker for the `"PromptUpdated"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PromptUpdatedType {
    /// Always serializes as `"PromptUpdated"`.
    #[default]
    PromptUpdated,
}

/// Mirrors `AgentV1PromptUpdatedEvent` — confirms `UpdatePrompt` applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PromptUpdatedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: PromptUpdatedType,
}

/// Marker for the `"SpeakUpdated"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SpeakUpdatedType {
    /// Always serializes as `"SpeakUpdated"`.
    #[default]
    SpeakUpdated,
}

/// Mirrors `AgentV1SpeakUpdatedEvent` — confirms `UpdateSpeak` applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SpeakUpdatedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: SpeakUpdatedType,
}

/// Marker for the `"ThinkUpdated"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ThinkUpdatedType {
    /// Always serializes as `"ThinkUpdated"`.
    #[default]
    ThinkUpdated,
}

/// Mirrors `AgentV1ThinkUpdatedEvent` — confirms `UpdateThink` applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ThinkUpdatedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: ThinkUpdatedType,
}

// ---------- InjectionRefused ----------

/// Marker for the `"InjectionRefused"` discriminator value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InjectionRefusedType {
    /// Always serializes as `"InjectionRefused"`.
    #[default]
    InjectionRefused,
}

/// Mirrors `AgentV1InjectionRefusedEvent` — server refused an
/// `InjectAgentMessage`, typically because a turn was in progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct InjectionRefusedEvent {
    #[serde(rename = "type", default)]
    #[allow(missing_docs)]
    pub message_type: InjectionRefusedType,

    /// Reason the injection was refused.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn welcome_round_trip() {
        let raw = json!({
            "type": "Welcome",
            "request_id": "550e8400-e29b-41d4-a716-446655440000"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::Welcome(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn settings_applied_round_trip() {
        let raw = json!({ "type": "SettingsApplied" });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::SettingsApplied(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn conversation_text_user_basic() {
        let raw = json!({
            "type": "ConversationText",
            "role": "user",
            "content": "Hello"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::ConversationText(e) => {
                assert_eq!(e.role, ConversationRole::User);
                assert_eq!(e.content, "Hello");
                assert!(e.languages_hinted.is_empty());
                assert!(e.languages.is_empty());
            }
            _ => panic!("expected ConversationText"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn conversation_text_with_multi_language_fields() {
        let raw = json!({
            "type": "ConversationText",
            "role": "user",
            "content": "Hola",
            "languages_hinted": ["en", "es"],
            "languages": ["es"]
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::ConversationText(e) => {
                assert_eq!(e.languages_hinted, vec!["en", "es"]);
                assert_eq!(e.languages, vec!["es"]);
            }
            _ => panic!("expected ConversationText"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn user_started_speaking_round_trip() {
        let raw = json!({ "type": "UserStartedSpeaking" });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::UserStartedSpeaking(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn agent_thinking_round_trip() {
        let raw = json!({
            "type": "AgentThinking",
            "content": "Looking up the weather…"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::AgentThinking(e) => assert_eq!(e.content, "Looking up the weather…"),
            _ => panic!("expected AgentThinking"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn function_call_request_round_trip() {
        let raw = json!({
            "type": "FunctionCallRequest",
            "functions": [{
                "id": "fc_1",
                "name": "get_weather",
                "arguments": "{\"city\":\"NYC\"}",
                "client_side": true
            }]
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::FunctionCallRequest(e) => {
                assert_eq!(e.functions.len(), 1);
                assert_eq!(e.functions[0].id, "fc_1");
                assert!(e.functions[0].client_side);
                assert!(e.functions[0].thought_signature.is_none());
            }
            _ => panic!("expected FunctionCallRequest"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn function_call_request_with_thought_signature() {
        let raw = json!({
            "type": "FunctionCallRequest",
            "functions": [{
                "id": "fc_2",
                "name": "lookup",
                "arguments": "{}",
                "client_side": false,
                "thought_signature": "sig-abc"
            }]
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::FunctionCallRequest(e) => {
                assert_eq!(e.functions[0].thought_signature.as_deref(), Some("sig-abc"));
            }
            _ => panic!("expected FunctionCallRequest"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn function_call_cancelled_round_trip() {
        // Shape from the Voice Agent reference: `functions[]` of `{id, name}`.
        let raw = json!({
            "type": "FunctionCallCancelled",
            "functions": [
                { "id": "fc_12345678-90ab-cdef-1234-567890abcdef", "name": "book_appointment" },
                { "id": "fc_2", "name": "end_call" }
            ]
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::FunctionCallCancelled(e) => {
                assert_eq!(e.functions.len(), 2);
                assert_eq!(e.functions[0].id, "fc_12345678-90ab-cdef-1234-567890abcdef");
                assert_eq!(e.functions[0].name, "book_appointment");
                assert_eq!(
                    e.ids().collect::<Vec<_>>(),
                    vec!["fc_12345678-90ab-cdef-1234-567890abcdef", "fc_2"]
                );
            }
            other => panic!("expected FunctionCallCancelled, got {other:?}"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn function_call_cancelled_is_not_confused_with_request() {
        // Same `functions` key as FunctionCallRequest but a different
        // `type`; the marker enums keep the two apart in both directions.
        let cancelled = json!({
            "type": "FunctionCallCancelled",
            "functions": [{ "id": "fc_1", "name": "get_weather" }]
        });
        assert!(matches!(
            serde_json::from_value::<AgentResponse>(cancelled).unwrap(),
            AgentResponse::FunctionCallCancelled(_)
        ));
        let request = json!({
            "type": "FunctionCallRequest",
            "functions": [{
                "id": "fc_1",
                "name": "get_weather",
                "arguments": "{}",
                "client_side": true
            }]
        });
        assert!(matches!(
            serde_json::from_value::<AgentResponse>(request).unwrap(),
            AgentResponse::FunctionCallRequest(_)
        ));
    }

    #[test]
    fn latency_report_full_round_trip() {
        let raw = json!({
            "type": "LatencyReport",
            "stt_latency": 0.21,
            "ttt_token_latency": 0.35,
            "ttt_text_latency": 0.4,
            "ttt_tool_latency": 0.38,
            "ttt_thinking_latency": 0.36,
            "tts_latency": 0.12,
            "total_latency": 0.9
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::LatencyReport(e) => {
                assert_eq!(e.stt_latency, Some(0.21));
                assert_eq!(e.ttt_token_latency, Some(0.35));
                assert_eq!(e.ttt_text_latency, Some(0.4));
                assert_eq!(e.ttt_tool_latency, Some(0.38));
                assert_eq!(e.ttt_thinking_latency, Some(0.36));
                assert_eq!(e.tts_latency, Some(0.12));
                assert_eq!(e.total_latency, Some(0.9));
            }
            other => panic!("expected LatencyReport, got {other:?}"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn latency_report_partial_fields_round_trip() {
        // Every latency is optional; a turn with no tool call omits the
        // tool/thinking fields and they must not be emitted as null.
        let raw = json!({
            "type": "LatencyReport",
            "stt_latency": 0.2,
            "ttt_text_latency": 0.5,
            "tts_latency": 0.1,
            "total_latency": 0.8
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::LatencyReport(e) => {
                assert!(e.ttt_tool_latency.is_none());
                assert!(e.ttt_thinking_latency.is_none());
                assert!(e.ttt_token_latency.is_none());
            }
            other => panic!("expected LatencyReport, got {other:?}"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
        assert_eq!(
            serde_json::to_string(&AgentResponse::LatencyReport(LatencyReportEvent::default()))
                .unwrap(),
            r#"{"type":"LatencyReport"}"#
        );
    }

    #[test]
    fn listen_updated_round_trip() {
        let raw = json!({ "type": "ListenUpdated" });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::ListenUpdated(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
        assert_eq!(
            serde_json::to_string(&ListenUpdatedEvent::default()).unwrap(),
            r#"{"type":"ListenUpdated"}"#
        );
    }

    #[test]
    fn agent_started_speaking_round_trip() {
        let raw = json!({
            "type": "AgentStartedSpeaking",
            "total_latency": 1.23,
            "tts_latency": 0.4,
            "ttt_latency": 0.83
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::AgentStartedSpeaking(e) => {
                assert!((e.total_latency - 1.23).abs() < 1e-9);
                assert!((e.tts_latency - 0.4).abs() < 1e-9);
                assert!((e.ttt_latency - 0.83).abs() < 1e-9);
            }
            _ => panic!("expected AgentStartedSpeaking"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn agent_audio_done_round_trip() {
        let raw = json!({ "type": "AgentAudioDone" });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::AgentAudioDone(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn error_round_trip() {
        let raw = json!({
            "type": "Error",
            "description": "rate limit exceeded",
            "code": "RATE_LIMITED"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::Error(e) => {
                assert_eq!(e.code, "RATE_LIMITED");
                assert_eq!(e.description, "rate limit exceeded");
            }
            _ => panic!("expected Error"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn warning_round_trip() {
        let raw = json!({
            "type": "Warning",
            "description": "audio buffer running low",
            "code": "BUFFER_LOW"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::Warning(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn history_event_round_trip_conversation() {
        // History events carry a HistoryMessage payload, which itself
        // discriminates between conversation and function-call shapes.
        let raw = json!({
            "type": "History",
            "role": "assistant",
            "content": "Hello!"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::History(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn history_event_round_trip_function_call() {
        let raw = json!({
            "type": "History",
            "function_calls": [{
                "id": "f1",
                "name": "fn",
                "client_side": false,
                "arguments": "{}",
                "response": "{}"
            }]
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::History(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn prompt_updated_round_trip() {
        let raw = json!({ "type": "PromptUpdated" });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::PromptUpdated(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn speak_and_think_updated_round_trip() {
        for (raw, is_speak) in [
            (json!({ "type": "SpeakUpdated" }), true),
            (json!({ "type": "ThinkUpdated" }), false),
        ] {
            let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
            if is_speak {
                assert!(matches!(event, AgentResponse::SpeakUpdated(_)));
            } else {
                assert!(matches!(event, AgentResponse::ThinkUpdated(_)));
            }
            assert_eq!(serde_json::to_value(&event).unwrap(), raw);
        }
    }

    #[test]
    fn injection_refused_round_trip() {
        let raw = json!({
            "type": "InjectionRefused",
            "message": "agent is mid-turn"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::InjectionRefused(e) => assert_eq!(e.message, "agent is mid-turn"),
            _ => panic!("expected InjectionRefused"),
        }
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn function_call_response_round_trip() {
        let raw = json!({
            "type": "FunctionCallResponse",
            "name": "internal_lookup",
            "content": "[{\"id\":1}]"
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::FunctionCallResponse(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn unknown_type_lands_in_unknown() {
        let raw = json!({
            "type": "FutureEvent",
            "some_field": 42,
            "data": [1, 2, 3]
        });
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        match &event {
            AgentResponse::Unknown(value) => {
                assert_eq!(value["type"], "FutureEvent");
                assert_eq!(value["some_field"], 42);
            }
            _ => panic!("expected Unknown"),
        }
        // Unknown round-trips its raw value verbatim.
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    /// `EndOfTurn` is emitted in production by Flux STT (V2) listen
    /// providers but is absent from the published AsyncAPI spec, so the
    /// SDK does not model it. It must reach the consumer through
    /// `Unknown` with its payload intact — both documented triggers —
    /// rather than failing the stream. `AgentListenSettings`'
    /// `eager_eot_threshold` docs and the changelog promise exactly this.
    #[test]
    fn off_spec_end_of_turn_lands_in_unknown_with_its_payload() {
        for trigger in ["model", "timeout"] {
            let raw = json!({ "type": "EndOfTurn", "trigger": trigger });
            let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
            match &event {
                AgentResponse::Unknown(value) => {
                    assert_eq!(value["type"], "EndOfTurn");
                    assert_eq!(value["trigger"], trigger);
                }
                other => panic!("expected Unknown for EndOfTurn, got {other:?}"),
            }
            // Re-serializes byte-for-byte, so a consumer can forward it.
            assert_eq!(serde_json::to_value(&event).unwrap(), raw);
        }
    }

    #[test]
    fn unknown_round_trips_arbitrary_shapes() {
        // Even a non-object falls through to Unknown — useful if the
        // server ever emits a primitive or array. We don't expect this
        // in practice but it ensures graceful degradation.
        let raw = json!([1, 2, 3]);
        let event: AgentResponse = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(event, AgentResponse::Unknown(_)));
        assert_eq!(serde_json::to_value(&event).unwrap(), raw);
    }

    #[test]
    fn dispatch_does_not_misroute_history_to_other_variants() {
        // Sanity: a History event must dispatch to AgentResponse::History
        // and not get swallowed by an earlier variant.
        let raw = json!({
            "type": "History",
            "role": "user",
            "content": "Hi"
        });
        let event: AgentResponse = serde_json::from_value(raw).unwrap();
        match event {
            AgentResponse::History(HistoryMessage::Conversation(c)) => {
                assert_eq!(c.role, ConversationRole::User);
                assert_eq!(c.content, "Hi");
            }
            other => panic!("expected History/Conversation, got {other:?}"),
        }
    }
}
