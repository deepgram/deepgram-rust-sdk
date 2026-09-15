---
name: deepgram-rust-text-to-speech
description: Use when implementing Deepgram text-to-speech in the Rust SDK, including Aura and Flux TTS model selection, speak feature flags, output file or byte-stream handling, the streaming Aura and Flux TTS WebSocket handles, and real crate APIs under speak::options, speak::flux, and Speak.
---

# Using Deepgram Text-to-Speech (Rust SDK)

Use this skill when generating audio from text with the Rust SDK's `Speak` surface.

## When to use this product

- Converting text into audio files with `speak_to_file(...)`.
- Streaming TTS bytes with `speak_to_stream(...)`.
- Capturing the `dg-request-id` Deepgram support asks for (plus model name/uuid, character count, content type) with `speak_to_file_with_metadata(...)` / `speak_to_stream_with_metadata(...)`, which return a `SpeakMetadata` alongside the audio. Every field is optional, so read it through `metadata.request_id()`.
- Selecting Aura voices and output encodings with `speak::options::Options`.
- Streaming text in and audio out over the Aura WebSocket with `speak_stream().handle()` — the shape to use when the text comes from an LLM and the audio goes to a player.
- Synthesizing with Flux TTS (`/v2/speak`) in batch with `flux_speak_to_file(...)` or `flux_speak_to_stream(...)`, or turn by turn over the WebSocket with `flux_request(options).handle()`.

## Authentication

For a TTS-only install:

```toml
[dependencies]
deepgram = { version = "0.10.1", default-features = false, features = ["speak"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
# Only add `bytes = "1"` if you need to name `bytes::Bytes` in your own signatures.
# The code below relies on type inference and does not import bytes directly.
```

```rust
let dg = deepgram::Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

- API keys use `Authorization: Token <api_key>`.
- Aura text-to-speech (`/v1/speak`) has both transports: REST (`Speak::speak_to_file` / `speak_to_stream`, a saved file or a stream of bytes) and the WebSocket (`Speak::speak_stream().handle()`, returning a `SpeakStreamHandle`: `speak`, `flush`, `clear`, `close`, `receive`, plus `split` / `sender` for sending and receiving from different tasks). Flux TTS (`/v2/speak`) likewise has both, in the `speak::flux` module: `Speak::flux_speak_to_file` / `flux_speak_to_stream` for batch and `Speak::flux_request(options).handle()` for the streaming WebSocket (`FluxSpeakHandle`: `speak`, `flush`, `interrupt`, `configure_speed`, `close`, `receive`).

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

## Quick start: Aura streaming (WebSocket)

```rust
use deepgram::{
    speak::{
        options::{Encoding, Model},
        SpeakResponse,
    },
    Deepgram,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    // The streaming endpoint emits raw audio only: linear16, mulaw, or alaw.
    let handle = dg
        .text_to_speech()
        .speak_stream()
        .model(Model::Aura2ThaliaEn)
        .encoding(Encoding::Linear16)
        .sample_rate(24_000)
        .handle()
        .await?;

    // Split so text can be sent while audio is still arriving.
    let (sender, mut events) = handle.split();

    let producer = tokio::spawn(async move {
        for piece in ["Hello, ", "this is streaming text to speech."] {
            sender.speak(piece).await?;
        }
        sender.flush().await?;
        sender.close().await
    });

    while let Some(message) = events.receive().await {
        match message? {
            SpeakResponse::Audio(_chunk) => { /* play or buffer the audio */ }
            SpeakResponse::Flushed { .. } => { /* the flushed segment is complete */ }
            SpeakResponse::Warning { description, .. } => {
                eprintln!("warning: {description:?}");
            }
            _ => {}
        }
    }

    producer.await.expect("producer task panicked")?;
    Ok(())
}
```

`clear()` discards text the server has not synthesized yet — use it on barge-in. See `examples/speak/websocket/text_to_speech_websocket.rs` for the full loop, including writing the audio to a file.

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

- Entrypoints: `Deepgram::text_to_speech()`, `Speak::speak_to_file(...)`, `Speak::speak_to_stream(...)`, `Speak::speak_stream()`.
- TTS `Options` builder fields: `model`, `encoding`, `sample_rate`, `container`, `bit_rate`.
- Model enum lives in `deepgram::speak::options::Model` and names every Aura-1 and Aura-2 voice the API serves — `AuraAsteriaEn`, `AuraLunaEn`, `AuraOrionEn`, `Aura2ThaliaEn`, `Aura2AgustinaEs`, `Aura2UzumeJa`, … — plus `CustomId(String)` for anything newer. `Model::from("aura-2-thalia-en")` resolves a wire string to its named variant; a `CustomId` built by hand is not `==` to the named variant it spells, so normalize through `Model::from` before comparing.
- Streaming (`speak_stream()`) builder methods: `model`, `encoding`, `sample_rate`, `speed`, `mip_opt_out`, and a `query_params` escape hatch. `handle()` validates locally first: the streaming endpoint accepts only `linear16` / `mulaw` / `alaw`, a `sample_rate` of 8000 / 16000 / 24000 / 32000 / 48000, and `speed` in `0.7..=1.5`; anything else is `DeepgramError::InvalidOptions` before a socket is opened. There is no `container` or `bit_rate` — the stream is raw audio.
- `SpeakStreamHandle` (and `SpeakStreamEvents`) implements `futures::Stream<Item = Result<SpeakResponse>>`; `SpeakResponse` is `Audio` / `Metadata` / `Flushed` / `Cleared` / `Warning` / `Unknown`. Sending text never blocks on undrained audio, and sends after the session ends return an error instead of being dropped.
- `speak_to_stream(...)` returns `impl Stream<Item = bytes::Bytes>`.
- Flux TTS entrypoints: `Speak::flux_speak_to_file(text, &options, path)`, `Speak::flux_speak_to_stream(text, &options)`, `Speak::flux_request(options).handle()` returning `FluxSpeakHandle`.
- Flux TTS `Options::builder(Model)` methods: `encoding`, `sample_rate`, `speed`, `expressivity`, `mip_opt_out`, `tag`, and the REST-only `container`, `bit_rate`, `callback`, `callback_method`, `priority_low`. `priority_low()` serializes as `priority=low`. Models are `flux-{voice}-{language}`, for example `Model::FluxHaleyEn`; the WebSocket rejects REST-only options and the compressed encodings (`mp3`, `opus`, `flac`, `aac`) with `DeepgramError::InvalidOptions`.

## API reference (layered)

1. **In-repo**
   - `README.md`
   - `src/speak/rest.rs`
   - `src/speak/options.rs`
   - `src/speak/websocket.rs`
   - `src/speak/flux/` (`rest.rs`, `websocket.rs`, `options.rs`, `response.rs`)
   - `examples/speak/rest/text_to_speech_to_file.rs`
   - `examples/speak/rest/text_to_speech_to_stream.rs`
   - `examples/speak/websocket/text_to_speech_websocket.rs`
   - `examples/speak/flux/batch_synthesize.rs`
   - `examples/speak/flux/websocket_synthesize.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Endpoint reference: `https://developers.deepgram.com/reference/text-to-speech/speak-request`
3. **AsyncAPI**
   - Rust SDK support: the Aura `/v1/speak` WebSocket via `Speak::speak_stream().handle()`, and the Flux TTS `/v2/speak` WebSocket via `Speak::flux_request(options).handle()`
   - Raw spec: `https://developers.deepgram.com/asyncapi.yaml`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/text-to-speech`
   - `https://developers.deepgram.com/docs/tts-rest`

## Gotchas

1. **Aura and Flux TTS are separate endpoints with separate clients.** Aura voices (`aura-*`) go to `/v1/speak` — REST via `speak_to_file` / `speak_to_stream`, WebSocket via `speak_stream()`. Flux TTS voices (`flux-*`) go to `/v2/speak` via `speak::flux`. Aura voices are rejected on `/v2/speak` and vice versa; the two `Model` enums (`speak::options::Model` and `speak::flux::options::Model`) are distinct types.
2. **Pick encoding/container pairs deliberately.** For raw output use `Container::None`; for `.wav` output use `Container::Wav`.
3. **`speak_to_stream(...)` is not the WebSocket.** It streams HTTP response bytes from `POST /v1/speak`, so the whole request is one block of text. To send text incrementally and get audio back as it is generated, use `speak_stream()` (Aura, `/v1/speak`) or `flux_request(options)` (Flux TTS, `/v2/speak`).
4. **Use API keys with `Token`.** Do not send API keys as `Bearer`.

## Example files in this repo

- `examples/speak/rest/text_to_speech_to_file.rs`
- `examples/speak/rest/text_to_speech_to_stream.rs`
- `examples/speak/websocket/text_to_speech_websocket.rs`
- `examples/speak/flux/batch_synthesize.rs`
- `examples/speak/flux/websocket_synthesize.rs`

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
