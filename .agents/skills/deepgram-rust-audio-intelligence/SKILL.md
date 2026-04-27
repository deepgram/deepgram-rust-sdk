---
name: deepgram-rust-audio-intelligence
description: Use when implementing Deepgram audio intelligence from the Rust SDK, especially when intelligence features are attached to STT Options and batch responses instead of a separate audio-intelligence module.
---

# Using Deepgram Audio Intelligence (Rust SDK)

Use this skill when the user wants transcript plus enrichment from audio, not a standalone text analysis request.

## When to use this product

- Running summarization, topics, intents, sentiments, entity detection, paragraphs, search, diarization, or utterances against audio.
- Explaining that the Rust crate exposes these features through STT `Options`, not a separate `audio_intelligence` module.

## Authentication

Audio intelligence rides on the `listen` feature because it is implemented through prerecorded transcription.

```toml
[dependencies]
deepgram = { version = "0.9.2", default-features = false, features = ["listen"] }
tokio = { version = "1", features = ["full"] }
```

```rust
let dg = deepgram::Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

## Quick start

## Quick start: prerecorded audio + intelligence flags

```rust
use deepgram::{
    common::{
        audio_source::AudioSource,
        options::{Language, Options},
    },
    Deepgram,
};
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let file = File::open("examples/audio/bueller.wav").await?;
    let source = AudioSource::from_buffer_with_mime_type(file, "audio/wav");

    let options = Options::builder()
        .language(Language::en_US)
        .punctuate(true)
        .detect_entities(true)
        .intents(true)
        .sentiment(true)
        .topics(true)
        .summarize(true)
        .paragraphs(true)
        .utterances(true)
        .diarize(true)
        .build();

    let response = dg.transcription().prerecorded(source, &options).await?;

    println!("transcript: {}", response.results.channels[0].alternatives[0].transcript);
    println!("summary: {:?}", response.results.summary);
    println!("topics: {:?}", response.results.topics);
    println!("intents: {:?}", response.results.intents);
    println!("sentiments: {:?}", response.results.sentiments);
    println!("entities: {:?}", response.results.channels[0].alternatives[0].entities);
    Ok(())
}
```

## Key parameters

- Intelligence flags on `common::options::OptionsBuilder`: `detect_entities`, `intents`, `sentiment`, `topics`, `summarize`, `paragraphs`, `utterances`, `diarize`, `search`, `keywords`, `keyterms`, `multichannel`.
- Result locations:
  - `response.results.summary`
  - `response.results.topics`
  - `response.results.intents`
  - `response.results.sentiments`
  - `response.results.channels[0].alternatives[0].entities`
  - `response.results.channels[0].alternatives[0].paragraphs`
  - `response.results.utterances`
  - `response.results.channels[0].search`

## API reference (layered)

1. **In-repo**
   - `src/common/options.rs`
   - `src/common/batch_response.rs`
   - `src/listen/rest.rs`
   - `examples/transcription/rest/prerecorded_from_file.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Endpoint reference: `https://developers.deepgram.com/reference/speech-to-text/listen-pre-recorded`
3. **AsyncAPI**
   - Usually not the primary source for full audio-intelligence response shapes in this crate
   - Raw spec: `https://developers.deepgram.com/asyncapi.yaml`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/docs/audio-intelligence`

## Gotchas

1. **No separate Rust module exists.** Audio intelligence is expressed as STT options plus prerecorded response fields.
2. **Use prerecorded for full coverage.** The richest typed results live in `common::batch_response`; live `StreamResponse` does not expose the same intelligence objects.
3. **Response fields are nested.** Some features live on `results`, others under `channels[...].alternatives[...]`.
4. **Feature availability varies by API mode.** The crate shares one `Options` builder, but not every flag is equally meaningful for live streaming.

## Example files in this repo

- `examples/transcription/rest/prerecorded_from_file.rs`
- `examples/transcription/rest/prerecorded_from_url.rs`
- `examples/transcription/rest/callback.rs`

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
