---
name: deepgram-rust-voice-agent
description: Use when implementing a Deepgram Voice Agent from the Rust SDK, including the agent() WebSocket client, Settings/Update messages, typed AgentResponse events, function calling, and saved agent configurations.
---

# Using Deepgram Voice Agent (Rust SDK)

Use this skill when the user wants an end-to-end conversational voice agent — STT, LLM, and TTS orchestrated by Deepgram over one WebSocket.

## When to use this product

- Building a full conversational agent (the server runs the turn loop, the LLM call, and speech synthesis).
- Handling agent events: conversation text, function-call requests, latency reports, interruptions.
- Changing a live session's model, prompt, or voice without reconnecting.

Prefer conversational STT (`flux_request`) instead when the application only needs turn-based transcripts and drives its own LLM and TTS.

## Authentication

The Voice Agent client is behind the `agent` feature (on by default).

```toml
[dependencies]
deepgram = { version = "0.12", default-features = false, features = ["agent"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

```rust
let dg = deepgram::Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

The credential travels in the handshake's `Authorization` header. `start()` uses the SaaS endpoint; `start_at_url()` accepts a custom host but requires `wss://` (plain `ws://` is allowed only for loopback, so local mock servers work). Do not hand-roll a raw `tokio_tungstenite::connect_async` session: the SDK client also supplies the shared rustls connector (so `rustls-tls-native-roots` and `Deepgram::tls_config` apply), the correct `Host` authority for non-default ports, and `Debug` redaction of provider credentials.

## Quick start

```rust
use deepgram::agent::{
    audio::{AudioConfig, AudioInput, AudioInputEncoding},
    listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
    settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
    speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
    think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
    AgentEvent, AgentResponse,
};
use deepgram::{Deepgram, DeepgramError};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap())?;
    let (mut handle, mut events) = dg.agent().start().await?;

    handle
        .send_settings(SettingsMessage::new(
            AudioConfig::new(
                Some(AudioInput::new(AudioInputEncoding::Linear16, 16_000)),
                None,
            ),
            AgentConfig::inline(
                InlineAgentConfig::from_parts(
                    AgentListenSettings::new(AgentListenProvider::DeepgramV2(
                        DeepgramListenV2Provider::new("flux-general-en"),
                    )),
                    ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
                        OpenAiModel::Gpt4oMini,
                    ))),
                    SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                        DeepgramSpeakModel::Aura2ThaliaEn,
                    ))),
                )
                .with_greeting("Hello! How can I help today?"),
            ),
        ))
        .await?;

    // Stream microphone audio with handle.send_data(pcm) from another task.
    while let Some(event) = events.next().await {
        match event? {
            AgentEvent::Json(AgentResponse::ConversationText(c)) => {
                println!("{:?}: {}", c.role, c.content)
            }
            AgentEvent::Json(AgentResponse::Error(e)) => {
                eprintln!("{}: {}", e.code, e.description);
                break;
            }
            AgentEvent::Json(AgentResponse::Unknown(value)) => println!("unmodeled: {value}"),
            AgentEvent::Audio(bytes) => println!("{} bytes of agent audio", bytes.len()),
            _ => {} // AgentEvent and AgentResponse are #[non_exhaustive]
        }
    }
    Ok(())
}
```

## Key parameters

- Entry points: `dg.agent().start()`, `dg.agent().start_at_url(url)`. Both return `(AgentHandle, AgentEventStream)`.
- `AgentHandle` client messages: `send_settings`, `send_update_listen`, `send_update_speak`, `send_update_think`, `send_update_prompt`, `send_inject_user_message`, `send_inject_agent_message`, `send_function_call_response`, `send_data` (binary audio), `keep_alive`, `force_end_turn`, `close`, and `send_raw_json` as the escape hatch for messages the SDK has not typed yet.
- `AgentEventStream` implements `futures::Stream<Item = Result<AgentEvent>>`; `AgentEvent` is `Json(AgentResponse)` or `Audio(Bytes)`, interleaved in protocol order.
- Settings: `AgentConfig::inline(InlineAgentConfig…)` or `AgentConfig::saved(uuid)` for a saved configuration. Listen providers are Deepgram V1 (Nova) and V2 (Flux STT, with `language_hints`, `eot_threshold`, `eager_eot_threshold`, `eot_timeout_ms`, `keyterms`); Think providers are OpenAI, Anthropic, AWS Bedrock, Google, and Groq; Speak providers are Deepgram, ElevenLabs, Cartesia, OpenAI, and AWS Polly.
- Request id: `handle.request_id()` / `events.request_id()` from the upgrade header, or `AgentResponse::Welcome`'s `request_id` when the header is absent.

## API reference (layered)

1. **In-repo**
   - `src/agent/websocket.rs` (client, handle, event stream)
   - `src/agent/settings.rs`, `src/agent/messages.rs`, `src/agent/response.rs`
   - `src/agent/{audio,listen,think,speak,history,endpoint,aws_credentials}.rs`
   - `examples/agent/websocket/simple_agent.rs`, `examples/agent/websocket/function_calling.rs`
   - `tests/agent_websocket_local.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Voice Agent reference page: `https://developers.deepgram.com/reference/voice-agent/voice-agent`
3. **AsyncAPI**
   - Raw spec: `https://developers.deepgram.com/asyncapi.yaml`
   - Voice Agent channel reference: `https://developers.deepgram.com/reference/voice-agent/voice-agent`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/voice-agent`

## Gotchas

1. **Its own host.** Voice Agent runs on `agent.deepgram.com`, not `api.deepgram.com`. `Deepgram::with_base_url` does not redirect it; use `start_at_url` for a self-hosted deployment.
2. **`Settings` comes first.** Send it before any audio, and wait for `SettingsApplied` before assuming the configuration is live. `Update*` messages are confirmed by their own `ListenUpdated` / `SpeakUpdated` / `ThinkUpdated` / `PromptUpdated` events.
3. **KeepAlive while silent.** The server closes sessions that go quiet; send `keep_alive()` every 8 seconds during an idle period. It does not extend the 2-hour session cap.
4. **Drain the event stream.** The handle and the stream are bounded channels, so send audio and consume events from separate tasks rather than queueing audio without reading.
5. **Sends fail once the session ends.** After a peer close or a transport error, every `send_*` on a retained handle returns an error instead of silently dropping the message; `close()` on an ended session is a no-op.
6. **Unmodeled events are preserved.** Anything the SDK does not type lands in `AgentResponse::Unknown(serde_json::Value)` rather than breaking the stream — including the production `{"type":"EndOfTurn","trigger":"model"|"timeout"}` message from Flux STT listen providers, which is not in the published spec yet.
7. **`force_end_turn` needs Flux STT.** With any other listen provider the server answers `FORCE_END_TURN_UNSUPPORTED` as a `Warning` and the turn does not end.
8. **Do not answer `FunctionCallCancelled`.** It reports calls the server abandoned; only `FunctionCallRequest` expects a `send_function_call_response`.

## Example files in this repo

- `examples/agent/websocket/simple_agent.rs` — connect, configure, print events, close.
- `examples/agent/websocket/function_calling.rs` — client-side function execution and `FunctionCallResponse`.

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
