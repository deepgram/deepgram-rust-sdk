---
name: using-voice-agent
description: Use when a user asks for Deepgram Voice Agent support from Rust. Route honestly: this crate does not currently expose the Agent WebSocket API, reusable agent configurations, or typed voice-agent events.
---

# Using Deepgram Voice Agent (Rust SDK)

Use this skill when the user wants the full Voice Agent API from Rust.

## When to use this product

- Building an end-to-end conversational voice agent on Deepgram's Agent WebSocket.
- Explaining the difference between Flux conversational STT support and unsupported full Agent support in this crate.
- Pointing maintainers or app developers to raw WebSocket fallback guidance.

## Authentication

**Not yet supported in this crate as a first-class module.** There is no `deepgram::agent`, `deepgram::voice_agent`, or typed `/v1/agent/converse` client in `src/` today.

Use raw WebSocket code if you must call Agent before the crate adds support.

## Quick start

## Quick start: raw WebSocket fallback

```rust
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;

    let request = http::Request::builder()
        .uri("wss://api.deepgram.com/v1/agent/converse")
        .header("Authorization", format!("Token {api_key}"))
        .body(())?;

    let (mut ws, _) = connect_async(request).await?;

    ws.send(Message::Text(r#"{"type":"Settings","audio":{"input":{"encoding":"linear16","sample_rate":16000}}}"#.into())).await?;

    while let Some(message) = ws.next().await {
        println!("{:?}", message?);
    }

    Ok(())
}
```

## Key parameters

- Endpoint: `wss://api.deepgram.com/v1/agent/converse`.
- Auth header: `Authorization: Token <api_key>`.
- Initial message ordering matters: send settings/config before media.
- If the request is really about turn-based STT only, prefer the supported Flux Rust APIs instead.

## API reference (layered)

1. **In-repo**
   - No dedicated voice-agent module exists in this crate today
   - Closest supported surface is `src/listen/flux.rs` for conversational STT only
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

1. **Flux is not Voice Agent.** `flux_request()` only gives conversational STT events, not the full agent orchestration API.
2. **Do not invent crate APIs.** There is no supported Rust wrapper for agent settings, audio replies, function calls, or reusable agent configs.
3. **Agent is WebSocket-first.** If you implement a workaround, validate message sequencing against the official docs, not the Rust crate.

## Example files in this repo

- No dedicated Voice Agent examples are present in this Rust repository.
- Closest related examples are Flux examples under `examples/transcription/flux/`.
