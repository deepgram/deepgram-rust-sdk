---
name: deepgram-rust-conversational-stt
description: Use when implementing Deepgram Flux conversational STT from the Rust SDK, including flux_request APIs, turn events, FluxResponse handling, and turn-detection tuning for voice-agent-style pipelines.
---

# Using Deepgram Conversational STT (Rust SDK)

Use this skill for Deepgram Flux, the crate's supported turn-based conversational streaming path.

## When to use this product

- Building turn-based STT for voice-agent pipelines.
- Handling `TurnEvent::{StartOfTurn, EndOfTurn, EagerEndOfTurn, TurnResumed, Update}`.
- Tuning end-of-turn behavior with `eot_threshold`, `eager_eot_threshold`, and `eot_timeout_ms`.

## Authentication

Flux is under the `listen` feature.

```toml
[dependencies]
deepgram = { version = "0.9.2", default-features = false, features = ["listen"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

```rust
let dg = deepgram::Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

## Quick start

```rust
use std::{io::Write, time::Duration};

use deepgram::{
    common::{
        flux_response::{FluxResponse, TurnEvent},
        options::{Encoding, Model, Options},
    },
    Deepgram,
};
use futures::stream::StreamExt;

static PATH_TO_FILE: &str = "examples/audio/sample-mono.wav";
static AUDIO_CHUNK_SIZE: usize = 18_063;
static FRAME_DELAY: Duration = Duration::from_millis(100);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .model(Model::FluxGeneralEn)
        .eot_threshold(0.75)
        .eot_timeout_ms(5000)
        .keyterms(["activate", "cancel"])
        .build();

    let mut results = dg
        .transcription()
        .flux_request_with_options(options)
        .encoding(Encoding::Linear32)
        .sample_rate(44100)
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY)
        .await?;

    println!("Flux Request ID: {}", results.request_id());
    while let Some(result) = results.next().await {
        match result? {
            FluxResponse::Connected { request_id, sequence_id } => {
                println!("Connected: {request_id} (seq: {sequence_id})");
            }
            FluxResponse::TurnInfo { event, turn_index, transcript, end_of_turn_confidence, .. } => {
                match event {
                    TurnEvent::StartOfTurn => println!("▶ [Turn {turn_index}] START"),
                    TurnEvent::EndOfTurn => println!("✓ [Turn {turn_index}] END ({end_of_turn_confidence:.2}): {transcript}"),
                    TurnEvent::Update => {
                        if !transcript.is_empty() {
                            print!("\r[Turn {turn_index}] UPDATE: {transcript}");
                            std::io::stdout().flush().unwrap();
                        }
                    }
                    _ => {}
                }
            }
            FluxResponse::FatalError { code, description, .. } => {
                eprintln!("{code}: {description}");
                break;
            }
            FluxResponse::Unknown(value) => println!("unknown: {value}"),
        }
    }

    Ok(())
}
```

## Key parameters

- Entrypoints: `flux_request()`, `flux_request_with_options(options)`.
- Flux transport builder fields: `encoding`, `sample_rate`, then `.file(...)`, `.stream(...)`, or `.handle().await?`.
- Flux-specific tuning in shared `OptionsBuilder`: `model(Model::FluxGeneralEn)`, `eot_threshold`, `eager_eot_threshold`, `eot_timeout_ms`, `keyterms`.
- Main response type: `common::flux_response::FluxResponse`.

## API reference (layered)

1. **In-repo**
   - `src/listen/flux.rs`
   - `src/common/flux_response.rs`
   - `src/common/options.rs`
   - `examples/transcription/flux/simple_flux.rs`
   - `tests/flux_unknown_messages.rs`
   - `tests/flux_e2e.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Endpoint reference: `https://developers.deepgram.com/reference/speech-to-text/listen-flux`
3. **AsyncAPI**
   - Raw spec: `https://developers.deepgram.com/asyncapi.yaml`
   - Flux channel docs are surfaced from the same product reference page above
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/stt/getting-started`

## Gotchas

1. **Flux is English-only in this crate's model surface.** The default supported model is `Model::FluxGeneralEn`.
2. **Real-time pacing matters even more than standard streaming.** The example warns that bad chunk size / delay values can break turn detection.
3. **Unknown events are intentional.** `FluxResponse::Unknown` and `TurnEvent::Unknown` are there for forward compatibility; handle them instead of assuming exhaustiveness.
4. **Use Flux for turn-taking, not full agent control.** If you need TTS replies, prompts, or tool calls, that is Voice Agent territory and not a typed Rust SDK surface yet.

## Example files in this repo

- `examples/transcription/flux/simple_flux.rs`
- `examples/transcription/flux/simple_flux_token.rs`
- `examples/transcription/flux/microphone_flux.rs`
- `tests/flux_unknown_messages.rs`
- `tests/flux_e2e.rs`

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
