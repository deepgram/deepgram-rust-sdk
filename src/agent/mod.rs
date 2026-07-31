//! Voice Agent (`/v1/agent/converse`) types.
//!
//! This module is being built out incrementally. So far it contains:
//!
//! - [`endpoint::Endpoint`] — custom LLM/TTS endpoint URL + headers.
//! - [`aws_credentials::AwsCredentials`] — credential block shared by AWS Bedrock (Think) and AWS Polly (Speak).
//! - [`audio`] — `AudioConfig`, `AudioInput`, `AudioOutput`, encodings,
//!   and container for the `audio` block on `Settings`.
//! - [`listen`] — `AgentListenSettings` and the V1/V2 Deepgram STT
//!   provider sub-types.
//! - [`think`] — `ThinkSettings` and the five Think provider variants
//!   (OpenAI, Anthropic, AWS Bedrock, Google, Groq).
//! - [`speak`] — `SpeakSettings` and the five Speak provider variants
//!   (Deepgram, ElevenLabs, Cartesia, OpenAI, AWS Polly).
//! - [`history`] — `HistoryMessage` (conversation + function-call history)
//!   used by `agent.context.messages[]` and the server-emitted `History` event.
//! - [`settings`] — top-level `SettingsMessage` and the `AgentConfig`
//!   oneOf (`Inline(InlineAgentConfig)` vs. `Saved(Uuid)`).
//! - [`messages`] — the remaining client-to-server messages
//!   (`UpdateSpeak`, `UpdateThink`, `UpdatePrompt`, `InjectUserMessage`,
//!   `InjectAgentMessage`, `FunctionCallResponse`, `KeepAlive`) plus the
//!   `ClientMessage` discriminated union over all eight.
//! - [`response`] — every server-emitted JSON event (`Welcome`,
//!   `SettingsApplied`, `ConversationText`, `UserStartedSpeaking`,
//!   `AgentThinking`, `FunctionCallRequest`, `AgentStartedSpeaking`,
//!   `AgentAudioDone`, `Error`, `Warning`, `History`, `PromptUpdated`,
//!   `SpeakUpdated`, `ThinkUpdated`, `InjectionRefused`,
//!   `FunctionCallResponse`) plus an `Unknown` catch-all for forward
//!   compatibility, all unified under the [`response::AgentResponse`] enum.
//! - [`websocket`] — the [`Agent`] sub-client and live-session
//!   primitives ([`AgentHandle`], [`AgentEventStream`], [`AgentEvent`])
//!   that connect to `wss://agent.deepgram.com/v1/agent/converse`.
//!
//! Wire format matches the AsyncAPI schemas in `deepgram-docs` under
//! `api/specs/asyncapi/schemas/agent/`. Examples and additional
//! convenience layers (e.g. file-based audio input) follow in
//! subsequent commits.

pub mod audio;
pub mod aws_credentials;
pub mod endpoint;
pub mod history;
pub mod listen;
pub mod messages;
pub mod response;
pub mod settings;
pub mod speak;
pub mod think;
pub mod websocket;

pub use audio::{
    AudioConfig, AudioContainer, AudioInput, AudioInputEncoding, AudioOutput, AudioOutputEncoding,
};
pub use aws_credentials::{AwsCredentials, AwsCredentialsType};
pub use endpoint::Endpoint;
pub use history::{
    ConversationHistoryMessage, ConversationRole, FunctionCallHistoryMessage, HistoryFunctionCall,
    HistoryMessage, HistoryMessageType,
};
pub use listen::{
    AgentListenProvider, AgentListenSettings, DeepgramListenV1Provider, DeepgramListenV1Version,
    DeepgramListenV2Provider, DeepgramListenV2Version, DeepgramProviderType,
};
pub use messages::{
    ClientMessage, FunctionCallResponseMessage, FunctionCallResponseType, InjectAgentBehavior,
    InjectAgentMessageMessage, InjectAgentMessageType, InjectUserMessageMessage,
    InjectUserMessageType, KeepAliveMessage, KeepAliveType, UpdatePromptMessage, UpdatePromptType,
    UpdateSpeakMessage, UpdateSpeakType, UpdateThinkMessage, UpdateThinkType,
};
pub use response::{
    AgentAudioDoneEvent, AgentAudioDoneType, AgentFunctionCall, AgentResponse,
    AgentStartedSpeakingEvent, AgentStartedSpeakingType, AgentThinkingEvent, AgentThinkingType,
    ConversationTextEvent, ConversationTextType, ErrorEvent, ErrorType, FunctionCallRequestEvent,
    FunctionCallRequestType, InjectionRefusedEvent, InjectionRefusedType, PromptUpdatedEvent,
    PromptUpdatedType, SettingsAppliedEvent, SettingsAppliedType, SpeakUpdatedEvent,
    SpeakUpdatedType, ThinkUpdatedEvent, ThinkUpdatedType, UserStartedSpeakingEvent,
    UserStartedSpeakingType, WarningEvent, WarningType, WelcomeEvent, WelcomeType,
};
pub use settings::{
    AgentConfig, AgentContext, InlineAgentConfig, SettingsFlags, SettingsMessage,
    SettingsMessageType,
};
pub use speak::{SpeakProvider, SpeakSettings};
pub use think::{ContextLength, FunctionEndpoint, ThinkFunction, ThinkProvider, ThinkSettings};
pub use websocket::{Agent, AgentEvent, AgentEventStream, AgentHandle};
