# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.8.0...HEAD)

### Changed

- Upgrade `reqwest` from 0.12 to 0.13. Consumers using re-exported `ReqwestError`, `RequestBuilder`, or `reqwest::Body` types must also upgrade to `reqwest` 0.13.
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

