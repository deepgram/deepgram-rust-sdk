//! Voice Agent (`/v1/agent/converse`) types.
//!
//! This module is being built out incrementally. So far it contains:
//!
//! - [`endpoint::Endpoint`] — custom LLM/TTS endpoint URL + headers.
//! - [`aws_credentials::AwsCredentials`] — credential block shared by AWS Bedrock (Think) and AWS Polly (Speak).
//! - [`think`] — `ThinkSettings` and the five Think provider variants
//!   (OpenAI, Anthropic, AWS Bedrock, Google, Groq).
//! - [`speak`] — `SpeakSettings` and the five Speak provider variants
//!   (Deepgram, ElevenLabs, Cartesia, OpenAI, AWS Polly).
//! - [`history`] — `HistoryMessage` (conversation + function-call history)
//!   used by `agent.context.messages[]` and the server-emitted `History` event.
//!
//! Wire format matches the AsyncAPI schemas in `deepgram-docs` under
//! `api/specs/asyncapi/schemas/agent/`. Connection helpers, message
//! types, and the WebSocket handle land in subsequent commits.

pub mod aws_credentials;
pub mod endpoint;
pub mod history;
pub mod speak;
pub mod think;

pub use aws_credentials::{AwsCredentials, AwsCredentialsType};
pub use endpoint::Endpoint;
pub use history::{
    ConversationHistoryMessage, ConversationRole, FunctionCallHistoryMessage, HistoryFunctionCall,
    HistoryMessage, HistoryMessageType,
};
pub use speak::{SpeakProvider, SpeakSettings};
pub use think::{ContextLength, FunctionEndpoint, ThinkFunction, ThinkProvider, ThinkSettings};
