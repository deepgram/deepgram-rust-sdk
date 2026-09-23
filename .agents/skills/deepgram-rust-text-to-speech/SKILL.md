---
name: deepgram-rust-text-to-speech
description: Use when implementing Deepgram text-to-speech in the Rust SDK, including Aura and Flux TTS model selection, speak feature flags, output file or byte-stream handling, the Flux TTS WebSocket handle, and real crate APIs under speak::options, speak::flux, and Speak.
---

# Using Deepgram Text-to-Speech (Rust SDK)

Use this skill when generating audio from text with the Rust SDK's `Speak` surface.

## When to use this product

- Converting text into audio files with `speak_to_file(...)`.
- Streaming TTS bytes with `speak_to_stream(...)`.
- Capturing the `dg-request-id` Deepgram support asks for (plus model name/uuid, character count, content type) with `speak_to_file_with_metadata(...)` / `speak_to_stream_with_metadata(...)`, which return a `SpeakMetadata` alongside the audio. Every field is optional, so read it through `metadata.request_id()`.
- Selecting Aura voices and output encodings with `speak::options::Options`.
- Synthesizing with Flux TTS (`/v2/speak`) in batch with `flux_speak_to_file(...)` or `flux_speak_to_stream(...)`, or turn by turn over the WebSocket with `flux_request(options).handle()`.

## Authentication

For a TTS-only install:

```toml
[dependencies]
deepgram = { version = "0.12", default-features = false, features = ["speak"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
# Only add `bytes = "1"` if you need to name `bytes::Bytes` in your own signatures.
# The code below relies on type inference and does not import bytes directly.
```

```rust
let dg = deepgram::Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

- API keys use `Authorization: Token <api_key>`.
- Aura text-to-speech (`/v1/speak`) is REST only in this crate: a saved file or a stream of bytes. Flux TTS (`/v2/speak`) has both transports in the `speak::flux` module: `Speak::flux_speak_to_file` / `flux_speak_to_stream` for batch and `Speak::flux_request(options).handle()` for the streaming WebSocket (`FluxSpeakHandle`: `speak`, `flush`, `interrupt`, `configure_speed`, `close`, `receive`).

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

## Quick start: Flux TTS batch (REST)

```rust
use std::path::Path;

use deepgram::{
    speak::flux::options::{Model, Options},
    Deepgram,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    // model is required; encoding defaults to mp3 on the batch transport.
    let options = Options::builder(Model::FluxHaleyEn).build();

    dg.text_to_speech()
        .flux_speak_to_file("Your appointment is confirmed.", &options, Path::new("flux-tts-batch.mp3"))
        .await?;

    Ok(())
}
```

## Quick start: Flux TTS streaming (WebSocket)

```rust
use deepgram::{
    speak::flux::{
        options::{Encoding, Model, Options},
        response::FluxSpeakResponse,
    },
    Deepgram,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder(Model::FluxHaleyEn)
        .encoding(Encoding::Linear16)
        .sample_rate(24_000)
        .build();

    // Connect to the /v2/speak WebSocket and get the handle.
    let speak = dg.text_to_speech();
    let mut handle = speak.flux_request(options).handle().await?;

    // Stream the turn's text in, end the turn with flush, then close.
    handle.speak("Hello! ").await?;
    handle.speak("This audio was synthesized over the Flux TTS websocket.").await?;
    handle.flush().await?;
    handle.close().await?;

    // Binary audio frames and JSON control events arrive on receive().
    let mut saw_session_metadata = false;
    while let Some(response) = handle.receive().await {
        match response? {
            FluxSpeakResponse::Audio(_bytes) => { /* play or buffer the audio */ }
            FluxSpeakResponse::SpeechMetadata(_) => { /* end-of-turn signal */ }
            FluxSpeakResponse::FatalError { code, description, .. } => {
                return Err(format!("fatal Flux TTS error {code}: {description}").into());
            }
            FluxSpeakResponse::SessionMetadata { .. } => {
                saw_session_metadata = true;
                break;
            }
            _ => {}
        }
    }

    if !saw_session_metadata {
        return Err("session ended before its terminal SessionMetadata event".into());
    }

    Ok(())
}
```

`handle.interrupt(playback_offset_ms)` cancels the active turn on barge-in and the server answers with `SpeechInterrupted`; `handle.configure_speed(speed)` changes the speaking rate mid-session and the server answers with `ConfigureSuccess` or `ConfigureFailure`. See `examples/speak/flux/websocket_synthesize.rs` for the full event loop, including saving the audio as a WAV file.

## Key parameters

- Entrypoints: `Deepgram::text_to_speech()`, `Speak::speak_to_file(...)`, `Speak::speak_to_stream(...)`.
- TTS `Options` builder fields: `model`, `encoding`, `sample_rate`, `container`, `bit_rate`.
- Model enum lives in `deepgram::speak::options::Model` and includes voices such as `AuraAsteriaEn`, `AuraLunaEn`, `AuraOrionEn`, plus `CustomId(String)`.
- `speak_to_stream(...)` returns `impl Stream<Item = bytes::Bytes>`.
- Flux TTS entrypoints: `Speak::flux_speak_to_file(text, &options, path)`, `Speak::flux_speak_to_stream(text, &options)`, `Speak::flux_request(options).handle()` returning `FluxSpeakHandle`.
- Flux TTS `Options::builder(Model)` methods: `encoding`, `sample_rate`, `speed`, `expressivity`, `mip_opt_out`, `tag`, and the REST-only `container`, `bit_rate`, `callback`, `callback_method`, `priority_low`. `priority_low()` serializes as `priority=low`. Models are `flux-{voice}-{language}`, for example `Model::FluxHaleyEn`; the WebSocket rejects REST-only options and the compressed encodings (`mp3`, `opus`, `flac`, `aac`) with `DeepgramError::InvalidOptions`.

## API reference (layered)

1. **In-repo**
   - `README.md`
   - `src/speak/rest.rs`
   - `src/speak/options.rs`
   - `src/speak/flux/` (`rest.rs`, `websocket.rs`, `options.rs`, `response.rs`)
   - `examples/speak/rest/text_to_speech_to_file.rs`
   - `examples/speak/rest/text_to_speech_to_stream.rs`
   - `examples/speak/flux/batch_synthesize.rs`
   - `examples/speak/flux/websocket_synthesize.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Endpoint reference: `https://developers.deepgram.com/reference/text-to-speech/speak-request`
3. **AsyncAPI**
   - Rust SDK support: the Flux TTS `/v2/speak` WebSocket via `Speak::flux_request(options).handle()`; the Aura `/v1/speak` WebSocket is not implemented in this crate
   - Raw spec: `https://developers.deepgram.com/asyncapi.yaml`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/text-to-speech`
   - `https://developers.deepgram.com/docs/tts-rest`

## Gotchas

1. **Aura TTS is REST only in this crate.** The only TTS WebSocket in `src/speak/` is the Flux TTS client in `speak::flux::websocket`; Aura voices are rejected on `/v2/speak`.
2. **Pick encoding/container pairs deliberately.** For raw output use `Container::None`; for `.wav` output use `Container::Wav`.
3. **`speak_to_stream(...)` still uses the REST endpoint.** It streams HTTP response bytes from `POST /v1/speak`; the WebSocket surface is `Speak::flux_request` on `/v2/speak`.
4. **Use API keys with `Token`.** Do not send API keys as `Bearer`.

## Example files in this repo

- `examples/speak/rest/text_to_speech_to_file.rs`
- `examples/speak/rest/text_to_speech_to_stream.rs`
- `examples/speak/flux/batch_synthesize.rs`
- `examples/speak/flux/websocket_synthesize.rs`

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
