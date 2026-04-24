---
name: deepgram-rust-text-to-speech
description: Use when implementing Deepgram text-to-speech in the Rust SDK, including Aura model selection, speak feature flags, output file or byte-stream handling, and real crate APIs under speak::options and Speak.
---

# Using Deepgram Text-to-Speech (Rust SDK)

Use this skill when generating audio from text with the Rust SDK's `Speak` surface.

## When to use this product

- Converting text into audio files with `speak_to_file(...)`.
- Streaming TTS bytes with `speak_to_stream(...)`.
- Selecting Aura voices and output encodings with `speak::options::Options`.

## Authentication

For a TTS-only install:

```toml
[dependencies]
deepgram = { version = "0.9.2", default-features = false, features = ["speak"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
bytes = "1"
```

```rust
let dg = deepgram::Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

- API keys use `Authorization: Token <api_key>`.
- The crate does not expose a TTS WebSocket client today; the supported Rust surface is REST returning a saved file or a stream of bytes.

## Quick start

## Quick start: save audio to a file

```rust
use std::{path::Path, time::Instant};

use deepgram::{
    speak::options::{Container, Encoding, Model, Options},
    Deepgram,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .model(Model::AuraAsteriaEn)
        .encoding(Encoding::Linear16)
        .sample_rate(16000)
        .container(Container::Wav)
        .build();

    let start = Instant::now();
    dg.text_to_speech()
        .speak_to_file("Hello from Rust.", &options, Path::new("output.wav"))
        .await?;

    println!("Time to download audio: {:.2?}", start.elapsed());
    Ok(())
}
```

## Quick start: stream response bytes

```rust
use deepgram::{
    speak::options::{Container, Encoding, Model, Options},
    Deepgram,
};
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .model(Model::AuraAsteriaEn)
        .encoding(Encoding::Linear16)
        .sample_rate(16000)
        .container(Container::Wav)
        .build();

    let mut stream = dg
        .text_to_speech()
        .speak_to_stream("Hello from Rust.", &options)
        .await?;

    while let Some(chunk) = stream.next().await {
        println!("received {} bytes", chunk.len());
    }

    Ok(())
}
```

## Key parameters

- Entrypoints: `Deepgram::text_to_speech()`, `Speak::speak_to_file(...)`, `Speak::speak_to_stream(...)`.
- TTS `Options` builder fields: `model`, `encoding`, `sample_rate`, `container`, `bit_rate`.
- Model enum lives in `deepgram::speak::options::Model` and includes voices such as `AuraAsteriaEn`, `AuraLunaEn`, `AuraOrionEn`, plus `CustomId(String)`.
- `speak_to_stream(...)` returns `impl Stream<Item = bytes::Bytes>`.

## API reference (layered)

1. **In-repo**
   - `README.md`
   - `src/speak/rest.rs`
   - `src/speak/options.rs`
   - `examples/speak/rest/text_to_speech_to_file.rs`
   - `examples/speak/rest/text_to_speech_to_stream.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Endpoint reference: `https://developers.deepgram.com/reference/text-to-speech/speak-request`
3. **AsyncAPI**
   - Rust SDK support: not implemented in this crate
   - Raw spec if you need unsupported WS TTS: `https://developers.deepgram.com/asyncapi.yaml`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/text-to-speech`
   - `https://developers.deepgram.com/docs/tts-rest`

## Gotchas

1. **This crate is REST-only for TTS.** There is no supported Rust WebSocket TTS surface in `src/speak/` today.
2. **Pick encoding/container pairs deliberately.** For raw output use `Container::None`; for `.wav` output use `Container::Wav`.
3. **`speak_to_stream(...)` still uses the REST endpoint.** It streams HTTP response bytes; it is not the separate TTS WebSocket API.
4. **Use API keys with `Token`.** Do not send API keys as `Bearer`.

## Example files in this repo

- `examples/speak/rest/text_to_speech_to_file.rs`
- `examples/speak/rest/text_to_speech_to_stream.rs`

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
