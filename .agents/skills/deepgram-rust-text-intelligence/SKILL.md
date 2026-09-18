---
name: deepgram-rust-text-intelligence
description: Use when a user asks for Deepgram text intelligence from Rust. The crate exposes /v1/read through a typed client behind the `read` Cargo feature.
---

# Using Deepgram Text Intelligence (Rust SDK)

Use this skill when the request is about summarization, sentiment, topics, or intents for text input rather than audio.

## When to use this product

- Analyzing plain text via Deepgram's `/v1/read` API.
- Building text summarization, topic detection, intent recognition, or sentiment analysis.
- Analyzing a transcript, document, chat log, or email you already have, rather than audio.

## Authentication

Text intelligence lives behind the `read` Cargo feature. It is HTTP-only, so it
needs none of the WebSocket dependencies the `listen` and `speak` features pull
in and can be enabled on its own.

```toml
[dependencies]
deepgram = { version = "0.12", default-features = false, features = ["read"] }
tokio = { version = "1", features = ["full"] }
```

`read` is also part of the crate's default features, so a plain
`cargo add deepgram` includes it.

```rust
use deepgram::Deepgram;

let dg_client = Deepgram::new(&std::env::var("DEEPGRAM_API_KEY")?)?;
let text_intelligence = dg_client.text_intelligence();
```

## Quick start: analyze a block of text

```rust
use deepgram::{read::options::Options, Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = std::env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY");
    let dg_client = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .sentiment(true)
        .summarize(true)
        .topics(true)
        .intents(true)
        .build();

    let response = dg_client
        .text_intelligence()
        .analyze_text(
            "Customer is asking to cancel the subscription next month.",
            &options,
        )
        .await?;

    if let Some(summary) = &response.results.summary {
        println!("summary: {}", summary.text);
    }
    if let Some(sentiments) = &response.results.sentiments {
        println!("average sentiment: {:?}", sentiments.average);
    }
    if let Some(topics) = &response.results.topics {
        println!("topic segments: {}", topics.segments.len());
    }
    if let Some(intents) = &response.results.intents {
        println!("intent segments: {}", intents.segments.len());
    }

    Ok(())
}
```

## Analyze a hosted document

`analyze_url` takes the URL of a plain-text document; everything else is the
same as `analyze_text`.

```rust
let response = dg_client
    .text_intelligence()
    .analyze_url("https://example.com/transcript.txt", &options)
    .await?;
```

## Callbacks

For long inputs, have Deepgram deliver the result to a webhook instead of
holding the connection open. The call returns as soon as the request is
accepted, with the `request_id` to correlate against; the full analysis arrives
at the callback URL.

```rust
use deepgram::read::options::{CallbackMethod, Options};

let options = Options::builder()
    .sentiment(true)
    .callback_method(CallbackMethod::POST)
    .build();

let ack = dg_client
    .text_intelligence()
    .analyze_text_callback(text, &options, "https://example.com/hook")
    .await?;
println!("request id: {}", ack.request_id);
```

`analyze_url_callback` is the hosted-document equivalent.

## Key parameters

- Input: `analyze_text(text, &options)` for inline text, `analyze_url(url, &options)` for a hosted plain-text document.
- Analysis flags on `Options::builder()`: `sentiment`, `summarize`, `topics`, `intents`.
- Custom taxonomies: `custom_topics` / `custom_topic_mode` and `custom_intents` / `custom_intent_mode`.
- `language` defaults to English, the only language `/v1/read` accepts; the builder sends it for you because the endpoint returns 400 without it.
- `tag` attaches request tags for usage reporting.
- Callbacks: `analyze_text_callback` / `analyze_url_callback` plus `callback_method`.

## Escape hatch

`make_read_request_builder` and `make_read_callback_request_builder` return the
underlying `reqwest::RequestBuilder` before it is sent, for appending query
parameters the typed options do not cover yet.

## API reference (layered)

1. **In-repo**
   - `src/read/` — `rest.rs` (the four request methods), `options.rs` (the query builder), `response.rs` (`Response`, `ReadMetadata`, `ReadResults`)
   - `README.md` for install/auth patterns
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

1. **This is different from audio intelligence.** Audio intelligence is piggybacked on STT and configured through the transcription `Options`; text intelligence is its own `/v1/read` API with its own `read::options::Options`. The two `Options` types are not interchangeable.
2. **Every analysis result is optional.** `results.sentiments`, `summary`, `topics`, and `intents` are `Option`, populated only for the flags you enabled. Match on them rather than unwrapping.
3. **Every metadata field is optional too.** Each field of `ReadMetadata` (including `request_id`, an `Option<Uuid>`, and `created`) and every field of `AnalysisInfo` is an `Option`, matching the reference, so logging the request id means unwrapping first rather than reading it straight off `response.metadata`.
4. **English only.** `/v1/read` accepts English; the builder sends `language=en` by default because the endpoint rejects a request without it.
5. **`read` can be enabled alone.** If a consumer has `default-features = false`, text intelligence needs `features = ["read"]` — it is not covered by `listen`.
6. **API keys use `Token`.** This API does not use `Bearer` for standard API keys. `Deepgram::new` handles the header for you.

## Example files in this repo

- `examples/read/analyze_text.rs` — runnable end-to-end analysis, registered as the `analyze_text` example (`cargo run --features read --example analyze_text`).

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
