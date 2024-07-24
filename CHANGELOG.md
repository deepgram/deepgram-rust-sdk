# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2024-07-23

### Migrating from 0.4.0 -> 0.6.0

Module Imports

```
use deepgram::{
---    transcription::prerecorded::{
+++    common::{
        audio_source::AudioSource,
        options::{Language, Options},
    },
    Deepgram, DeepgramError,
};
```

Streaming Changes

Now you can pass Options using stream_request_with_options
```
let options = Options::builder()
    .smart_format(true)
    .language(Language::en_US)
    .build();

let mut results = dg
    .transcription()
    .stream_request_with_options(Some(&options))
    .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, Duration::from_millis(16))
    .await?
    .start()
    .await?;
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

## [0.5.0]
- Deprecate tiers and add explicit support for all currently available models.
- Expand language enum to include all currently-supported languages.
- Add (default on) feature flags for live and prerecorded transcription.
- Support arbitrary query params in transcription options.

## [0.4.0] - 2023-11-01

### Added
- `detect_language` option.

### Changed
- Remove generic from `Deepgram` struct.
- Upgrade dependencies: `tungstenite`, `tokio-tungstenite`, `reqwest`.

## [0.3.0]

### Added
- Derive `Serialize` for all response types.

### Fixed
- Use the users builder options when building a streaming URL.
- Make sure that `Future` returned from `StreamRequestBuilder::start()` is `Send`.

### Changed
- Use Rustls instead of OpenSSL.

[Unreleased]: https://github.com/deepgram-devs/deepgram-rust-sdk/compare/0.4.0...HEAD
[0.4.0]: https://github.com/deepgram-devs/deepgram-rust-sdk/compare/0.3.0...0.4.0
[0.3.0]: https://github.com/deepgram-devs/deepgram-rust-sdk/compare/0.2.1...0.3.0
