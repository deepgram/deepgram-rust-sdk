---
name: using-text-intelligence
description: Use when a user asks for Deepgram text intelligence from Rust. Route to raw HTTP guidance because this crate does not currently expose a dedicated /v1/read client or typed text-intelligence module.
---

# Using Deepgram Text Intelligence (Rust SDK)

Use this skill when the request is about summarization, sentiment, topics, or intents for text input rather than audio.

## When to use this product

- Analyzing plain text via Deepgram's `/v1/read` API.
- Building text summarization, topic detection, intent recognition, or sentiment analysis.
- Explaining the gap between Deepgram product support and the current Rust crate surface.

## Authentication

**Not yet supported in this crate as a first-class module.** There is no `deepgram::read`, `deepgram::text_intelligence`, or equivalent typed client in `src/` today.

Use raw HTTP with `reqwest`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.13", default-features = false, features = ["json", "rustls"] }
serde_json = "1"
```

## Quick start

## Quick start: call `/v1/read` with `reqwest`

```rust
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.deepgram.com/v1/read")
        .header(AUTHORIZATION, format!("Token {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&json!({
            "text": "Customer is asking to cancel the subscription next month.",
            "language": "en",
            "summarize": true,
            "topics": true,
            "intents": true,
            "sentiment": true
        }))
        .send()
        .await?
        .error_for_status()?;

    let body: serde_json::Value = response.json().await?;
    println!("{body:#}");
    Ok(())
}
```

## Key parameters

- Request body: `text` or hosted `url`.
- Required for current text-intelligence docs: `language: "en"`.
- Analysis flags: `summarize`, `topics`, `intents`, `sentiment`.
- Optional callback fields exist in the HTTP API, but the Rust crate does not provide typed helpers for them.

## API reference (layered)

1. **In-repo**
   - No dedicated support in this crate today; note the absence of any `read` / `text_intelligence` module under `src/`
   - `README.md` for install/auth patterns only
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Endpoint reference: `https://developers.deepgram.com/reference/text-intelligence/analyze-text`
3. **AsyncAPI**
   - Not applicable for `/v1/read`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/text-intelligence`

## Gotchas

1. **No Rust SDK wrapper yet.** Do not invent `deepgram::read` APIs; use `reqwest` until the crate adds a typed surface.
2. **This is different from audio intelligence.** Audio intelligence is piggybacked on STT; text intelligence is its own `/v1/read` API.
3. **Use English explicitly.** Current text-intelligence docs require `language` to be English.
4. **API keys use `Token`.** This API does not use `Bearer` for standard API keys.

## Example files in this repo

- No dedicated text-intelligence examples are present in this Rust repository.
- Closest related examples are transcription examples under `examples/transcription/` if you need transcript-first workflows.

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
