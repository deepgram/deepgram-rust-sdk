//! Voice Agent (`/v1/agent/converse`) types.
//!
//! It contains:
//!
//! - [`endpoint::Endpoint`] — custom LLM/TTS endpoint URL + headers.
//! - [`aws_credentials::AwsCredentials`] — credential block shared by AWS Bedrock (Think) and AWS Polly (Speak).
//! - [`audio`] — `AudioConfig`, `AudioInput`, `AudioOutput`, encodings,
//!   and container for the `audio` block on `Settings`.
//! - [`listen`] — `AgentListenSettings` and the V1/V2 Deepgram STT
//!   provider sub-types.
//! - [`think`] — `ThinkSettings` and the Think provider variants (OpenAI,
//!   Anthropic, AWS Bedrock, Deepgram, Google, Groq).
//! - [`think_live`] — Think Live settings for Deepgram backchannels,
//!   presence checks, dynamic fillers, and frustration detection.
//! - [`speak`] — `SpeakSettings` and the five Speak provider variants
//!   (Deepgram, ElevenLabs, Cartesia, OpenAI, AWS Polly).
//! - [`history`] — `HistoryMessage` (conversation + function-call history)
//!   used by `agent.context.messages[]` and the server-emitted `History` event.
//! - [`settings`] — top-level `SettingsMessage` and the `AgentConfig`
//!   oneOf (`Inline(InlineAgentConfig)` vs. `Saved(Uuid)`).
//! - [`messages`] — the remaining client-to-server messages
//!   (`UpdateListen`, `UpdateSpeak`, `UpdateThink`, `UpdateThink_Live`,
//!   `UpdatePrompt`, `InjectUserMessage`, fixed or Think Live-generated
//!   `InjectAgentMessage`, `FunctionCallResponse`, `KeepAlive`,
//!   `ForceEndTurn`, `__customToThinkProvider`) plus the `ClientMessage`
//!   discriminated union.
//! - [`response`] — every server-emitted JSON event (`Welcome`,
//!   `SettingsApplied`, `ConversationText`, `UserStartedSpeaking`,
//!   `AgentThinking`, `FunctionCallRequest`, `FunctionCallCancelled`,
//!   `AgentStartedSpeaking`, `AgentAudioDone`, `Error`, `Warning`,
//!   `History`, `LatencyReport`, `ListenUpdated`, `PromptUpdated`,
//!   `SpeakUpdated`, `ThinkUpdated`, `ThinkLiveUpdated`,
//!   `FrustrationDetected`, `InjectionRefused`, `FunctionCallResponse`,
//!   `__customFromThinkProvider`) plus an `Unknown` catch-all for forward
//!   compatibility, all unified under the [`response::AgentResponse`] enum.
//!   Dispatch is on the event's `type`, so `AgentResponse::Unknown` holds
//!   exactly the *events* this SDK does not model; a recognized event
//!   with a malformed payload is an error on the stream, never a silent
//!   `Unknown`. A recognized event carrying an *unrecognized value* is
//!   neither: it stays on the happy path through that value's own
//!   catch-all. A `role` this release does not name, on `ConversationText`
//!   or `History`, arrives as
//!   [`ConversationRole::Unknown(String)`](history::ConversationRole::Unknown)
//!   holding the wire string verbatim, and a history entry matching
//!   neither modeled shape as
//!   [`HistoryMessage::Unknown(serde_json::Value)`](history::HistoryMessage::Unknown).
//!   So a new server enum value never ends a session.
//! - [`websocket`] — the [`Agent`] sub-client and live-session
//!   primitives ([`AgentHandle`], [`AgentEventStream`], [`AgentEvent`])
//!   that connect to `wss://agent.deepgram.com/v1/agent/converse`.
//!   `start_at_url` requires TLS (`wss://`) except for loopback hosts.
//!
//! `Debug` output for [`AwsCredentials`], [`Endpoint`], and
//! [`FunctionEndpoint`] redacts secrets (AWS keys/tokens and header
//! values), so a `Settings` value can be logged safely.
//!
//! Wire format matches the AsyncAPI schemas in `deepgram-docs` under
//! `api/specs/asyncapi/schemas/agent/`. See the `simple_agent` and
//! `function_calling` examples for end-to-end usage.

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
pub mod think_live;
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
    ClientMessage, CustomToThinkProviderMessage, CustomToThinkProviderType, ForceEndTurnMessage,
    ForceEndTurnType, FunctionCallResponseMessage, FunctionCallResponseType, InjectAgentBehavior,
    InjectAgentMessageMessage, InjectAgentMessageType, InjectDynamicAgentMessage,
    InjectUserMessageMessage, InjectUserMessageType, KeepAliveMessage, KeepAliveType,
    UpdateListenMessage, UpdateListenType, UpdatePromptMessage, UpdatePromptType,
    UpdateSpeakMessage, UpdateSpeakType, UpdateThinkLiveMessage, UpdateThinkLiveType,
    UpdateThinkMessage, UpdateThinkType,
};
pub use response::{
    AgentAudioDoneEvent, AgentAudioDoneType, AgentFunctionCall, AgentResponse,
    AgentStartedSpeakingEvent, AgentStartedSpeakingType, AgentThinkingEvent, AgentThinkingType,
    CancelledFunctionCall, ConversationTextEvent, ConversationTextType, ConversationalBehavior,
    CustomFromThinkProviderEvent, CustomFromThinkProviderType, ErrorEvent, ErrorType,
    FrustrationDetectedEvent, FrustrationDetectedType, FunctionCallCancelledEvent,
    FunctionCallCancelledType, FunctionCallRequestEvent, FunctionCallRequestType,
    InjectionRefusedEvent, InjectionRefusedType, LatencyReportEvent, LatencyReportType,
    ListenUpdatedEvent, ListenUpdatedType, PromptUpdatedEvent, PromptUpdatedType,
    SettingsAppliedEvent, SettingsAppliedType, SpeakUpdatedEvent, SpeakUpdatedType,
    ThinkLiveUpdatedEvent, ThinkLiveUpdatedType, ThinkUpdatedEvent, ThinkUpdatedType,
    UserStartedSpeakingEvent, UserStartedSpeakingType, WarningEvent, WarningType, WelcomeEvent,
    WelcomeType,
};
pub use settings::{
    AgentConfig, AgentContext, InlineAgentConfig, SettingsFlags, SettingsMessage,
    SettingsMessageType,
};
pub use speak::{SpeakProvider, SpeakSettings};
pub use think::{ContextLength, FunctionEndpoint, ThinkFunction, ThinkProvider, ThinkSettings};
pub use think_live::{
    ThinkLiveProvider, ThinkLiveProviderType, ThinkLiveSettings, ThinkLiveVersion,
};
pub use websocket::{Agent, AgentEvent, AgentEventStream, AgentHandle, InsecureAgentUrl};
