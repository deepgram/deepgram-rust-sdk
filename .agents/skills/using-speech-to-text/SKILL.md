---
name: using-speech-to-text
description: Use when implementing Deepgram speech-to-text in the Rust SDK, including prerecorded REST transcription, live WebSocket streaming, listen feature flags, Options builder usage, and response handling.
---

# Using Deepgram Speech-to-Text (Rust SDK)

Use this skill for prerecorded transcription, live streaming transcription, or when mapping Deepgram docs to the Rust crate's real `listen` surface.

## When to use this product

- Transcribing local files, URLs, or in-memory audio with `Deepgram::transcription()`.
- Streaming audio over WebSocket with `stream_request()` / `stream_request_with_options(...)`.
- Using `common::options::Options` for STT features such as `model`, `language`, `punctuate`, `diarize`, `smart_format`, `utterances`, and streaming knobs like `endpointing`.

## Authentication

`deepgram` defaults to `manage + listen + speak`. For STT-only installs, trim features explicitly:

```toml
[dependencies]
deepgram = { version = "0.9.2", default-features = false, features = ["listen"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

```rust
use deepgram::Deepgram;

let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

- API keys use `Authorization: Token <api_key>`.
- Temporary tokens use `Deepgram::with_temp_token(...)` and send `Bearer`, but are mainly useful for voice APIs rather than Manage APIs.
- Self-hosted installs can use `Deepgram::with_base_url(...)` or `Deepgram::with_base_url_and_api_key(...)`.

## Quick start

## Quick start: prerecorded file transcription

```rust
use deepgram::{
    common::{
        audio_source::AudioSource,
        options::{Language, Options},
    },
    Deepgram,
};
use tokio::fs::File;

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let file = File::open(PATH_TO_FILE).await?;
    let source = AudioSource::from_buffer_with_mime_type(file, "audio/wav");

    let options = Options::builder()
        .punctuate(true)
        .language(Language::en_US)
        .build();

    let response = dg.transcription().prerecorded(source, &options).await?;
    println!("{}", response.results.channels[0].alternatives[0].transcript);
    Ok(())
}
```

## Quick start: live WebSocket transcription

```rust
use std::time::Duration;

use deepgram::{
    common::options::{Encoding, Endpointing, Language, Options},
    Deepgram,
};
use futures::stream::StreamExt;

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";
static AUDIO_CHUNK_SIZE: usize = 3174;
static FRAME_DELAY: Duration = Duration::from_millis(16);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let mut results = dg
        .transcription()
        .stream_request_with_options(options)
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .endpointing(Endpointing::CustomDurationMs(300))
        .interim_results(true)
        .utterance_end_ms(1000)
        .vad_events(true)
        .no_delay(true)
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY)
        .await?;

    println!("Deepgram Request ID: {}", results.request_id());
    while let Some(result) = results.next().await {
        println!("{result:?}");
    }

    Ok(())
}
```

## Key parameters

- Prerecorded entrypoints: `prerecorded(...)`, `prerecorded_callback(...)`, `make_prerecorded_request_builder(...)`.
- Streaming entrypoints: `stream_request()`, `stream_request_with_options(options)`, then `.file(...)`, `.stream(...)`, or `.handle().await?`.
- Core `Options` builder fields: `model`, `language`, `punctuate`, `smart_format`, `diarize`, `multichannel`, `utterances`, `detect_language`, `keywords`, `search`, `replace`, `paragraphs`.
- Streaming-only builder fields: `encoding`, `sample_rate`, `channels`, `endpointing`, `utterance_end_ms`, `interim_results`, `no_delay`, `vad_events`, `keep_alive`, `callback`.
- Main response types: prerecorded `common::batch_response::Response`; live `common::stream_response::StreamResponse`.

## API reference (layered)

1. **In-repo**
   - `README.md`
   - `src/listen/rest.rs`
   - `src/listen/websocket.rs`
   - `src/common/options.rs`
   - `examples/transcription/rest/prerecorded_from_file.rs`
   - `examples/transcription/websocket/simple_stream.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Pre-recorded reference: `https://developers.deepgram.com/reference/speech-to-text/listen-pre-recorded`
3. **AsyncAPI**
   - Raw spec: `https://developers.deepgram.com/asyncapi.yaml`
   - Streaming reference: `https://developers.deepgram.com/reference/speech-to-text/listen-streaming`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/stt/getting-started`

## Gotchas

1. **Use `listen` feature gates correctly.** STT modules are behind the `listen` Cargo feature.
2. **`Options` is by value for WebSocket builders.** `stream_request_with_options(options)` takes ownership, unlike prerecorded APIs that take `&Options`.
3. **Live and prerecorded responses differ.** Intelligence-heavy fields such as `summary`, `topics`, and `sentiments` live on prerecorded response types, not `StreamResponse`.
4. **Audio pacing matters.** The example `.file(...)` helpers assume realistic chunk sizes and delays; sending audio too fast can produce bad streaming behavior.
5. **Use `Token`, not `Bearer`, for API keys.** `Bearer` is only for temporary tokens.

## Example files in this repo

- `examples/transcription/rest/prerecorded_from_file.rs`
- `examples/transcription/rest/prerecorded_from_url.rs`
- `examples/transcription/rest/callback.rs`
- `examples/transcription/rest/make_prerecorded_request_builder.rs`
- `examples/transcription/websocket/simple_stream.rs`
- `examples/transcription/websocket/callback_stream.rs`
- `examples/transcription/websocket/microphone_stream.rs`
- `examples/transcription/websocket/16_keepalive_close_stream.rs`

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
