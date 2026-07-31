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
//! specific [`FunctionCallHistoryMessage`] is tried first.

use serde::{Deserialize, Serialize};

/// A single entry in the agent's conversation history.
///
/// Either a user/assistant utterance or a record of one or more function
/// calls executed during a turn. Both forms share the wire-level
/// `type: "History"` discriminator; structural fields differentiate them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum HistoryMessage {
    /// Function call request and response. Ordering note: this variant is
    /// listed first so JSON containing `function_calls` is matched here
    /// rather than falling through to [`HistoryMessage::Conversation`].
    FunctionCall(FunctionCallHistoryMessage),
    /// User or assistant utterance.
    Conversation(ConversationHistoryMessage),
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

/// Speaker role on a [`ConversationHistoryMessage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ConversationRole {
    /// The end user.
    User,
    /// The agent.
    Assistant,
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
