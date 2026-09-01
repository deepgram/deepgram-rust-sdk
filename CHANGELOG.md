# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/deepgram/deepgram-rust-sdk/compare/0.10.0...HEAD)

### Added

- Opt-in connect diagnostics behind the new `connect-diagnostics` cargo feature, for live transcription (`/v1/listen`) WebSocket connections (other WebSocket surfaces are not covered yet). Configuring a sink via `WebsocketBuilder::diagnostics` makes the SDK establish the streaming connection in four individually timed phases (DNS, TCP connect, TLS handshake, WebSocket upgrade) and emit one `diagnostics::ConnectRecord` per connect attempt — including attempts cancelled by a caller-side `tokio::time::timeout`, and failed upgrades, which capture the `dg-request-id` and `dg-error` response headers. Records serialize to flat JSON for JSONL pipelines (`schema_version: 1`, additive-only); the recorded URL is reduced to scheme, host, port, and path — userinfo and query parameters are never persisted. Without a sink — and always without the feature — the stock connect path runs unchanged. New example: `connect_diagnostics`.

## [0.10.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.9.2...0.10.0)

### Added

- Multilingual Flux support: `Model::FluxGeneralMulti` selects the `flux-general-multi` model, and `OptionsBuilder::language_hint` takes BCP-47 codes that serialize as repeated `language_hint=…` query params.
- `FluxResponse::TurnInfo` now exposes `languages` (detected on the turn) and `languages_hinted` (active hints for the request).
- Mid-session reconfiguration: `FluxHandle::configure(ConfigureRequest)` adjusts thresholds, keyterms, and language hints without restarting the WebSocket. Server replies surface as the new `FluxResponse::ConfigureSuccess` / `ConfigureFailure` variants. `ConfigureRequest` and `ConfigureThresholds` are `#[non_exhaustive]` and built via `new()` + `with_*` methods.
- New examples: `flux_multi_language` and `flux_dynamic_configure` under `examples/transcription/flux/`, plus `examples/audio/bueller-mono.wav` (a mono Linear16 downmix of the existing stereo sample) so the Flux examples have enough audio + trailing silence to trigger `EndOfTurn` against the live server.

### Changed

- **BREAKING**: `FluxResponse::TurnInfo` is now `#[non_exhaustive]` and gained the `languages` and `languages_hinted` fields. Callers destructuring `TurnInfo` with a literal struct pattern must add `..` (or explicitly name the new fields).

## [0.9.2](https://github.com/deepgram/deepgram-rust-sdk/compare/0.9.1...0.9.2)

### Fixed

- Fix occasional panic in WebSocket keep-alive when `close_stream()` closes the internal channel before the worker's keep-alive timer fires ([#143](https://github.com/deepgram/deepgram-rust-sdk/issues/143)).

### Added

- New example `16_keepalive_close_stream` demonstrating correct keep-alive + close_stream usage.

## [0.9.1](https://github.com/deepgram/deepgram-rust-sdk/compare/0.9.0...0.9.1)

### Fixed

- Fix `Container::None` serialization typo (`"nonne"` → `"none"`) that caused 400 errors from the TTS API when requesting raw audio output.
- Add missing `User-Agent` header to WebSocket handshake requests for streaming and Flux endpoints, fixing compatibility with AWS WAF and similar firewalls.
- Replace broken Discord badge in README with working shields.io badge.
- Flux WebSocket now handles unknown message types gracefully instead of producing stream-breaking deserialization errors. `FluxResponse::Unknown` preserves the raw JSON; `TurnEvent::Unknown` catches unrecognized event strings. Both enums are `#[non_exhaustive]`, so this is non-breaking.

## [0.9.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.8.0...0.9.0)

### Changed

- **BREAKING**: Upgrade `reqwest` from 0.12 to 0.13. Consumers using re-exported `ReqwestError`, `RequestBuilder`, or `reqwest::Body` types must also upgrade to `reqwest` 0.13.
- Upgrade `http` from 1.3 to 1.4. Consumers using re-exported `HttpError` must also upgrade to `http` 1.4.
- TLS backend changed from `ring` to `aws-lc-rs` via rustls update. Certificate verification now uses platform-native trust stores via `rustls-platform-verifier`.
- Reqwest feature `rustls-tls` renamed to `rustls`; `query` feature now explicitly enabled.

## [0.8.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.6.2...0.8.0)

- Add Flux conversational speech recognition model support (`flux-general-en`)
  - New `flux_request()` and `flux_request_with_options()` methods for Flux streaming
  - Support for turn-based conversation detection with `FluxResponse` types
  - Configurable end-of-turn detection parameters (`eot_threshold`, `eager_eot_threshold`, `eot_timeout_ms`)
  - New `TurnEvent` enum: `StartOfTurn`, `EndOfTurn`, `EagerEndOfTurn`, `TurnResumed`, `Update`
  - Examples: `simple_flux` (file streaming) and `microphone_flux` (real-time microphone)
  - Uses `/v2/listen` endpoint for Flux API
- Update documentation to point to [deepgram/deepgram-rust-sdk](https://github.com/deepgram/deepgram-rust-sdk).
- Added support for [short-lived auth tokens](https://developers.deepgram.com/reference/auth/tokens/grant) using Deepgram `v1/auth/grant` API

## [0.6.1](https://github.com/deepgram/deepgram-rust-sdk/compare/0.6.1...0.6.2)

## [0.6.1](https://github.com/deepgram/deepgram-rust-sdk/compare/0.6.0...0.6.1)

- Implement `From<String>` for `Model`, `Language`, and `Redact`
- Add callback support to websocket connections.

## [0.6.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.5.0...0.6.0) - 2024-08-08

### Migrating from 0.4.0 -> 0.6.0

#### Module Imports

```rust
use deepgram::{
---    transcription::prerecorded::{
+++    common::{
        audio_source::AudioSource,
        options::{Language, Options},
    },
    Deepgram, DeepgramError,
};
```

#### Streaming Changes

We have exposed a low-level, message-based interface to the websocket API:

```rust
use futures::select;

let mut handle = dg
    .transcription()
    .stream_request()
    .handle()
    .await?;

loop {
    select! {
        _ = tokio::time::sleep(Duration::from_secs(3)) => handle.keep_alive().await,
        _ = handle.send_data(data_chunk()).fuse() => {}
        response = handle.receive().fuse() => {
            match response {
                Some(response) => println!("{response:?}"),
                None => break,
            }
        }
    }
}
handle.close_stream().await;
```

No need to call `.start()` to begin streaming data.

```rust
let mut results = dg
    .transcription()
    .stream_request_with_options(Some(&options))
    .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, Duration::from_millis(16))
---    .await
---    .start()
    .await;
```

Now you can pass Options using stream_request_with_options

```rust
let options = Options::builder()
    .smart_format(true)
    .language(Language::en_US)
    .build();

let mut results = dg
    .transcription()
    .stream_request_with_options(Some(&options))
    .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, Duration::from_millis(16))
    .await?
```

Some Enums have changed and may need to be updated

### Changed

- Add streaming features
- Add support for pre-recorded features when streaming
- Add Speech to Text
- Reorganize Code

### Streaming Features

- endpointing
- utterance_end_ms
- interim_results
- no_delay
- vad_events

### Streaming Functions

- keep_alive

### New Streaming Message Types

- Utterance End
- Speech Started

### Pre-Recorded Features

- encoding
- smart_format
- callback
- callback_method
- filler_words
- paragraphs
- diarize_version
- dictation
- measurements
- extra

### Pre-Recorded Audio Intelligence Features

- detect_entities
- sentiment
- topics
- summarize
- intents
- custom_intents
- custom_intent_mode
- topics
- custom_topics
- custom_topic_mode

## [0.5.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.4.0...0.5.0) - 2024-07-08

- Deprecate tiers and add explicit support for all currently available models.
- Expand language enum to include all currently-supported languages.
- Add (default on) feature flags for live and prerecorded transcription.
- Support arbitrary query params in transcription options.

## [0.4.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.3.0...0.4.0) - 2023-11-01

### Added

- `detect_language` option.

### Changed

- Remove generic from `Deepgram` struct.
- Upgrade dependencies: `tungstenite`, `tokio-tungstenite`, `reqwest`.

## [0.3.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.2.1...0.3.0) - 2023-07-26

### Added

- Derive `Serialize` for all response types.

### Fixed

- Use the users builder options when building a streaming URL.
- Make sure that `Future` returned from `StreamRequestBuilder::start()` is `Send`.

### Changed

- Use Rustls instead of OpenSSL.

