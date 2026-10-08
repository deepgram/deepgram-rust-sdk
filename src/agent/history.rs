//! Conversation history messages used by `agent.context.messages[]` on
//! the Voice Agent `Settings` message and emitted by the server as
//! `History` events during a session.
//!
//! Mirrors `asyncapi/schemas/agent/history-message.v1.yml` and its two
//! sub-schemas (`history-message/ConversationHistoryMessage.yml`,
//! `history-message/FunctionCallHistoryMessage.yml`).
//!
//! Both variants share `type: "History"` on the wire — the discriminator
//! is structural (`role`/`content` vs. `function_calls`), so this enum is
//! [serde-untagged](serde::Deserialize) with ordering chosen so the more
//! specific [`FunctionCallHistoryMessage`] is tried first and the
//! [`HistoryMessage::Unknown`] catch-all last.

use serde::{Deserialize, Serialize};

/// A single entry in the agent's conversation history.
///
/// Either a user/assistant utterance or a record of one or more function
/// calls executed during a turn. Both forms share the wire-level
/// `type: "History"` discriminator; structural fields differentiate them.
///
/// This is an open enum: a history shape a future server release adds
/// lands in [`HistoryMessage::Unknown`] with its JSON intact, so it never
/// turns into a deserialization error on the event stream.
//
// `Eq` is deliberately absent: `Unknown` holds a `serde_json::Value`,
// which is `PartialEq` but not `Eq` — the same trade-off
// `AgentResponse` makes for its own `Unknown` variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum HistoryMessage {
    /// Function call request and response. Ordering note: this variant is
    /// listed first so JSON containing `function_calls` is matched here
    /// rather than falling through to [`HistoryMessage::Conversation`].
    FunctionCall(FunctionCallHistoryMessage),
    /// User or assistant utterance.
    Conversation(ConversationHistoryMessage),
    /// Forward-compatibility escape — a history entry matching neither
    /// modeled shape lands here with its raw payload preserved.
    ///
    /// Ordering note: this variant is listed **last** so it is only
    /// reached after [`HistoryMessage::FunctionCall`] and
    /// [`HistoryMessage::Conversation`] both fail to match.
    Unknown(serde_json::Value),
}

impl HistoryMessage {
    /// Construct a conversation entry.
    pub fn conversation(role: ConversationRole, content: impl Into<String>) -> Self {
        Self::Conversation(ConversationHistoryMessage::new(role, content))
    }

    /// Construct a function-call history entry from a list of calls.
    pub fn function_calls(calls: impl IntoIterator<Item = HistoryFunctionCall>) -> Self {
        Self::FunctionCall(FunctionCallHistoryMessage::new(calls))
    }
}

/// Wire discriminator for history messages — always serializes as `"History"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HistoryMessageType {
    /// The only valid value per spec. Default value for this type.
    #[default]
    History,
}

/// User/assistant utterance recorded in conversation history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ConversationHistoryMessage {
    /// Always [`HistoryMessageType::History`] on the wire. Public for
    /// round-trip fidelity; prefer [`ConversationHistoryMessage::new`].
    #[serde(rename = "type", default)]
    pub message_type: HistoryMessageType,

    /// Speaker role.
    pub role: ConversationRole,

    /// What was said.
    pub content: String,
}

impl ConversationHistoryMessage {
    /// Construct a conversation history entry.
    pub fn new(role: ConversationRole, content: impl Into<String>) -> Self {
        Self {
            message_type: HistoryMessageType::History,
            role,
            content: content.into(),
        }
    }
}

/// Speaker role on a [`ConversationHistoryMessage`] or a
/// [`ConversationTextEvent`](crate::agent::response::ConversationTextEvent).
///
/// This is an open enum: new values may be added over time, and
/// unrecognized values deserialize as [`ConversationRole::Unknown`],
/// which preserves the original wire string and re-serializes to it
/// exactly. A role this release does not name therefore reaches the
/// consumer as an event rather than ending the session with a
/// deserialization error.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ConversationRole {
    /// The end user.
    User,
    /// The agent.
    Assistant,
    /// An unrecognized role value from the server, preserved verbatim.
    Unknown(String),
}

impl ConversationRole {
    /// The wire representation of this role.
    pub fn as_str(&self) -> &str {
        match self {
            ConversationRole::User => "user",
            ConversationRole::Assistant => "assistant",
            ConversationRole::Unknown(value) => value,
        }
    }
}

// Manual scalar (de)serialization: `ConversationRole` is a plain string
// on the wire, and unknown values must survive an exact round trip — a
// derived `#[serde(other)]` unit variant would collapse them all to one
// value. Mirrors `TurnTrigger` in `crate::common::flux_response`.
impl Serialize for ConversationRole {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ConversationRole {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "user" => ConversationRole::User,
            "assistant" => ConversationRole::Assistant,
            _ => ConversationRole::Unknown(value),
        })
    }
}

/// Record of one or more function calls executed during a turn.
///
/// Used to seed agent context with prior function-call activity so the
/// agent can reason over past tool use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FunctionCallHistoryMessage {
    /// Always [`HistoryMessageType::History`]. Prefer [`FunctionCallHistoryMessage::new`].
    #[serde(rename = "type", default)]
    pub message_type: HistoryMessageType,

    /// One or more function calls.
    pub function_calls: Vec<HistoryFunctionCall>,
}

impl FunctionCallHistoryMessage {
    /// Construct from a list of calls.
    pub fn new(calls: impl IntoIterator<Item = HistoryFunctionCall>) -> Self {
        Self {
            message_type: HistoryMessageType::History,
            function_calls: calls.into_iter().collect(),
        }
    }
}

/// A single function call recorded in conversation history.
///
/// `arguments` and `response` are wire-string-typed per spec — they are
/// JSON-encoded blobs but the API treats them opaquely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoryFunctionCall {
    /// Unique identifier for the call, matching the original `FunctionCallRequest`.
    pub id: String,

    /// Name of the function called.
    pub name: String,

    /// Whether the call was executed client-side (`true`) or server-side (`false`).
    pub client_side: bool,

    /// Arguments passed to the function (opaque string per spec).
    pub arguments: String,

    /// Response from the function call (opaque string per spec).
    pub response: String,

    /// Some Gemini models require this as an additional call identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

impl HistoryFunctionCall {
    /// Construct a fully-populated history function call entry.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        client_side: bool,
        arguments: impl Into<String>,
        response: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            client_side,
            arguments: arguments.into(),
            response: response.into(),
            thought_signature: None,
        }
    }

    /// Attach a thought signature.
    pub fn with_thought_signature(mut self, signature: impl Into<String>) -> Self {
        self.thought_signature = Some(signature.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn conversation_round_trip() {
        let raw = json!({
            "type": "History",
            "role": "user",
            "content": "Hello, agent."
        });
        let msg: ConversationHistoryMessage = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(msg.role, ConversationRole::User);
        assert_eq!(msg.content, "Hello, agent.");
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn conversation_assistant_role() {
        let raw = json!({
            "type": "History",
            "role": "assistant",
            "content": "Hi there!"
        });
        let msg: ConversationHistoryMessage = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(msg.role, ConversationRole::Assistant);
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn function_call_round_trip_minimal() {
        let raw = json!({
            "type": "History",
            "function_calls": [{
                "id": "f1",
                "name": "get_weather",
                "client_side": true,
                "arguments": "{\"city\":\"NYC\"}",
                "response": "{\"temp\":72}"
            }]
        });
        let msg: FunctionCallHistoryMessage = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(msg.function_calls.len(), 1);
        assert_eq!(msg.function_calls[0].id, "f1");
        assert!(msg.function_calls[0].client_side);
        assert!(msg.function_calls[0].thought_signature.is_none());
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn function_call_round_trip_with_thought_signature() {
        let raw = json!({
            "type": "History",
            "function_calls": [{
                "id": "f2",
                "name": "lookup",
                "client_side": false,
                "arguments": "{}",
                "response": "{}",
                "thought_signature": "sig-abc"
            }]
        });
        let msg: FunctionCallHistoryMessage = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(
            msg.function_calls[0].thought_signature.as_deref(),
            Some("sig-abc")
        );
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn enum_dispatches_conversation() {
        let raw = json!({
            "type": "History",
            "role": "user",
            "content": "Hi"
        });
        let msg: HistoryMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(msg, HistoryMessage::Conversation(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn enum_dispatches_function_call() {
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
        let msg: HistoryMessage = serde_json::from_value(raw.clone()).unwrap();
        assert!(matches!(msg, HistoryMessage::FunctionCall(_)));
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn enum_constructor_helpers() {
        let conv = HistoryMessage::conversation(ConversationRole::User, "Hello");
        match &conv {
            HistoryMessage::Conversation(c) => {
                assert_eq!(c.role, ConversationRole::User);
                assert_eq!(c.content, "Hello");
            }
            _ => panic!("expected conversation"),
        }

        let fc = HistoryMessage::function_calls([HistoryFunctionCall::new(
            "f1", "fn", true, "{}", "{}",
        )]);
        match &fc {
            HistoryMessage::FunctionCall(f) => {
                assert_eq!(f.function_calls.len(), 1);
                assert!(f.function_calls[0].client_side);
            }
            _ => panic!("expected function call"),
        }
    }

    #[test]
    fn function_call_with_thought_signature_builder() {
        let call = HistoryFunctionCall::new("f1", "lookup", false, "{}", "{}")
            .with_thought_signature("sig");
        assert_eq!(call.thought_signature.as_deref(), Some("sig"));
    }

    #[test]
    fn history_type_defaults_to_history() {
        // When constructing via `new`, the message_type field is set to History.
        let msg = ConversationHistoryMessage::new(ConversationRole::Assistant, "Hi");
        assert_eq!(msg.message_type, HistoryMessageType::History);
        let serialized = serde_json::to_value(&msg).unwrap();
        assert_eq!(serialized["type"], "History");
    }

    #[test]
    fn history_type_default_when_absent_in_input() {
        // Serde uses Default for `message_type` since the field is `default`d.
        // This makes JSON without `type` still parse — convenient for the
        // "construct in Rust without writing the constant" path.
        let raw = json!({ "role": "user", "content": "x" });
        let msg: ConversationHistoryMessage = serde_json::from_value(raw).unwrap();
        assert_eq!(msg.message_type, HistoryMessageType::History);
    }

    /// `AGENTS.md`: a modeled event carrying an unmodeled *value* must
    /// not error. A role this release does not name round-trips to the
    /// exact wire string instead of ending the session.
    #[test]
    fn unknown_role_round_trips_to_the_wire_string() {
        let raw = json!({
            "type": "History",
            "role": "system",
            "content": "You are a helpful agent."
        });
        let msg: ConversationHistoryMessage = serde_json::from_value(raw.clone())
            .expect("an unrecognized role must deserialize, not error");
        assert_eq!(msg.role, ConversationRole::Unknown("system".to_string()));
        assert_eq!(msg.role.as_str(), "system");
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    /// The same payload through the untagged enum: an unknown role still
    /// matches the `Conversation` shape, so it does not fall through to
    /// [`HistoryMessage::Unknown`].
    #[test]
    fn unknown_role_still_dispatches_to_conversation() {
        let raw = json!({
            "type": "History",
            "role": "tool",
            "content": "{}"
        });
        let msg: HistoryMessage = serde_json::from_value(raw.clone())
            .expect("an unrecognized role must deserialize, not error");
        match &msg {
            HistoryMessage::Conversation(c) => {
                assert_eq!(c.role, ConversationRole::Unknown("tool".to_string()));
            }
            other => panic!("expected a conversation entry, got {other:?}"),
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    #[test]
    fn known_roles_still_serialize_lowercase() {
        assert_eq!(ConversationRole::User.as_str(), "user");
        assert_eq!(ConversationRole::Assistant.as_str(), "assistant");
        assert_eq!(
            serde_json::to_value(ConversationRole::User).unwrap(),
            json!("user")
        );
        assert_eq!(
            serde_json::to_value(ConversationRole::Assistant).unwrap(),
            json!("assistant")
        );
    }

    /// A third history shape — neither `role`/`content` nor
    /// `function_calls` — lands in `Unknown` with its JSON intact rather
    /// than failing the untagged match.
    #[test]
    fn unknown_history_shape_lands_in_unknown_with_its_json() {
        let raw = json!({
            "type": "History",
            "tool_result": { "id": "t1", "output": "42" }
        });
        let msg: HistoryMessage = serde_json::from_value(raw.clone())
            .expect("an unmodeled history shape must deserialize, not error");
        match &msg {
            HistoryMessage::Unknown(value) => assert_eq!(value, &raw),
            other => panic!("expected the Unknown catch-all, got {other:?}"),
        }
        assert_eq!(serde_json::to_value(&msg).unwrap(), raw);
    }

    /// The catch-all must not shadow the two modeled shapes.
    #[test]
    fn unknown_variant_is_tried_last() {
        let conversation = json!({ "type": "History", "role": "user", "content": "Hi" });
        let function_call = json!({
            "type": "History",
            "function_calls": [{
                "id": "f1", "name": "fn", "client_side": true,
                "arguments": "{}", "response": "{}"
            }]
        });
        assert!(matches!(
            serde_json::from_value::<HistoryMessage>(conversation).unwrap(),
            HistoryMessage::Conversation(_)
        ));
        assert!(matches!(
            serde_json::from_value::<HistoryMessage>(function_call).unwrap(),
            HistoryMessage::FunctionCall(_)
        ));
    }

    #[test]
    fn list_of_history_messages() {
        // Models the wire shape of `agent.context.messages[]`.
        let raw = json!([
            { "type": "History", "role": "user", "content": "Hi" },
            {
                "type": "History",
                "function_calls": [{
                    "id": "f1", "name": "fn", "client_side": true,
                    "arguments": "{}", "response": "{}"
                }]
            },
            { "type": "History", "role": "assistant", "content": "Hello!" }
        ]);
        let msgs: Vec<HistoryMessage> = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(msgs.len(), 3);
        assert!(matches!(msgs[0], HistoryMessage::Conversation(_)));
        assert!(matches!(msgs[1], HistoryMessage::FunctionCall(_)));
        assert!(matches!(msgs[2], HistoryMessage::Conversation(_)));
        assert_eq!(serde_json::to_value(&msgs).unwrap(), raw);
    }
}
