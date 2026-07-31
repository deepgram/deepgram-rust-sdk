# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Text Intelligence (`/v1/read`) support: `Deepgram::text_intelligence()` returns a `Read` client with `analyze_text` and `analyze_url` for running sentiment, summarization, topic, and intent analysis over text. Reuses the existing sentiment/topic/intent response types and adds a Read-specific `Summary` (`text`) ([#94](https://github.com/deepgram/deepgram-rust-sdk/issues/94)). New example `analyze_text`.
- Model-listing endpoints on the management API: `Deepgram::models()` returns a `Models` client with `get_models`, `get_model`, `get_project_models`, and `get_project_model`. New example `models`.
- `futures::Stream` for live transcription is confirmed and documented: `listen::websocket::TranscriptionStream` already implements `Stream<Item = Result<StreamResponse>>`, so results compose with `StreamExt` combinators. New example `stream_futures` demonstrates idiomatic combinator usage ([#163](https://github.com/deepgram/deepgram-rust-sdk/issues/163)).
- Text-to-Speech REST responses now expose their metadata headers, including the `dg-request-id`. New `Speak::speak_to_file_with_metadata` and `Speak::speak_to_stream_with_metadata` return a `SpeakMetadata` (request id, model name/uuid, character count, content type) alongside the audio ([#89](https://github.com/deepgram/deepgram-rust-sdk/issues/89)). New example `text_to_speech_request_id`.
- Named Deepgram Whisper Cloud model variants: `Model::Whisper`, `Model::WhisperTiny`, `Model::WhisperBase`, `Model::WhisperSmall`, `Model::WhisperMedium`, and `Model::WhisperLarge` (previously only reachable via `Model::CustomId`) ([#128](https://github.com/deepgram/deepgram-rust-sdk/issues/128)).
- Additional redaction options: `Redact::Pii`, `Redact::Phi`, and `Redact::AggressiveNumbers` ([#87](https://github.com/deepgram/deepgram-rust-sdk/issues/87)).
- Public fields on the Audio Intelligence and paragraph response types (`Paragraph`, `Sentence`, `Paragraphs`, `Entity`, `Intent`, `Segment`, `Intents`, `SentimentSegment`, `SentimentAverage`, `Sentiments`, `TopicDetail`, `TopicSegment`, `Topics`, `Summary`), which were previously inaccessible. All are now `#[non_exhaustive]` ([#129](https://github.com/deepgram/deepgram-rust-sdk/issues/129)).
- Caption generation: `common::captions::srt` / `webvtt` (and `Response::to_srt` / `to_webvtt`) render SRT and WebVTT subtitle files from a pre-recorded transcription, following the same cue-grouping rules as the [`@deepgram/captions`](https://github.com/deepgram/deepgram-js-captions) JavaScript helper. When the response includes `results.utterances` (the `utterances` feature), each utterance becomes its own cue (split further at `max_words_per_cue`) and carries that utterance's speaker; otherwise words from `channels[0].alternatives[0]` are grouped up to `max_words_per_cue`, and a diarized speaker change always starts a new cue so no words are attributed to the previous speaker. Speaker labels match the JS helper as well: SRT writes `[Speaker N]` on its own line above the cue text, only on cues where the speaker changes from the previous cue, and WebVTT prefixes every diarized cue with a `<v Speaker N>` voice tag. Timestamps are truncated to whole milliseconds, as in the JS helper. Words-per-cue and the WebVTT `NOTE` header are configurable via `CaptionOptions`. New example `captions`.
- `Entity::raw_value`: the entity text as it was spoken, before a formatting feature rewrote it. The pre-recorded entity response carries it whenever a feature such as `smart_format` reshapes an entity (`PHONE_NUMBER` arrives as `raw_value: "five five five"` with `value: "(555"`); it is `None` with no formatting feature enabled, and `None` for an entity formatting left unchanged. Previously the field was returned by the API and discarded by the SDK ([#129](https://github.com/deepgram/deepgram-rust-sdk/issues/129)).
- `extra` metadata is now surfaced on both the pre-recorded and the streaming responses: `common::batch_response::ListenMetadata`, `common::stream_response::Metadata`, and the `StreamResponse::TerminalResponse` (Metadata) message each expose a public `extra: Option<HashMap<String, String>>` field holding the key-value pairs echoed back from the request ([#130](https://github.com/deepgram/deepgram-rust-sdk/issues/130)).

### Changed

- **BREAKING**: `common::stream_response::Metadata` is now `#[non_exhaustive]` and gained an `extra` field, and `StreamResponse::TerminalResponse` gained an `extra` field (the variant is now `#[non_exhaustive]`). Code outside the crate can no longer build either with a struct literal at all: `#[non_exhaustive]` rejects it, and there is no `..` form that restores it (functional-record-update is forbidden too) — obtain these by deserializing a server response instead. Exhaustive destructuring and pattern matching still work, but must add `..`. (These types are produced by deserialization, so most callers are unaffected.)
- **BREAKING**: `Model::from(String)` and `Redact::from(String)` now return the new named variants instead of the catch-all ones. `"whisper"`, `"whisper-tiny"`, `"whisper-base"`, `"whisper-small"`, `"whisper-medium"`, and `"whisper-large"` parse to `Model::Whisper` … `Model::WhisperLarge` rather than `Model::CustomId(..)`, and `"pii"`, `"phi"`, and `"aggressive_numbers"` parse to `Redact::Pii`, `Redact::Phi`, and `Redact::AggressiveNumbers` rather than `Redact::Other(..)`. Nothing fails to compile and the serialized wire value is unchanged (`AsRef<str>` still yields the same string, so request query strings are byte-identical), but code that matches `Model::CustomId(id)` or compares against `Redact::Other("pii".to_string())` for these strings silently takes a different branch than it did on 0.11.0. Match the named variants instead, or key the comparison off the wire string with `AsRef::<str>::as_ref` so both the named and the custom form are handled.
- **BREAKING**: `Speak::speak_to_file` no longer prints `Audio saved to …` to stdout on success; libraries should not write to stdout. If your program relied on that line, print it yourself after the call returns. Error diagnostics are unchanged.

### Fixed

- WebSocket handshakes (live transcription, Flux speech-to-text, Flux text-to-speech) now send the URL's full authority in the `Host` header, so a base URL on a non-default port (`https://dg.internal.example:8443`, `http://127.0.0.1:54321`) produces `Host: dg.internal.example:8443` rather than `Host: dg.internal.example`. Strict reverse proxies and virtual-host routing on self-hosted deployments could reject or misroute the old value. Default ports are still omitted, so requests to `api.deepgram.com` are unchanged.
- `cargo doc` with a single Cargo feature (for example `--no-default-features --features manage`) no longer reports unresolved intra-doc links from the crate-level feature list and the `with_base_url*` docs to modules that are compiled only under other features. Documentation builds with all features are unchanged.

## [0.11.0](https://github.com/deepgram/deepgram-rust-sdk/compare/0.10.1...0.11.0)

### Added

- `rustls-tls-native-roots` cargo feature: `wss://` WebSocket connections also trust the operating system's certificate store, on top of the bundled webpki roots (never instead of them). For TLS-inspecting proxies (Zscaler, Netskope, …), internal CAs, and self-hosted deployments. Named after the `tokio-tungstenite` and `reqwest` features it mirrors; `rustls-native-certs` honors `SSL_CERT_FILE` / `SSL_CERT_DIR` in place of the platform store. If the store cannot be loaded (a missing or non-PEM `SSL_CERT_FILE`, a container with no store), the client continues with the public roots and reports that state explicitly (see `TlsTrust` below).
- `Deepgram::tls_config(impl Into<Arc<rustls::ClientConfig>>)`: supply your own rustls configuration once, on the client, and every `wss://` WebSocket it opens (live transcription, Flux speech-to-text, Flux text-to-speech) uses it verbatim. `rustls` is re-exported as `deepgram::rustls` so the versions match.
- `DeepgramError::UntrustedTlsCertificate { host, trust, source }`: returned instead of a bare `WsError` when the server certificate's issuer is not in the trust roots. The message ends with the remedy that applies to the trust roots actually in effect: enable the feature, install the CA in the OS store, fix an `SSL_CERT_FILE` whose roots could not be loaded, or adjust the supplied config. Where a hint mentions `SSL_CERT_FILE`, it asks for a PEM bundle holding the CA together with the public roots you rely on, not the CA alone: once set, the variable replaces the OS store for these WebSockets and, on Linux, for the REST client (`reqwest`'s platform verifier reads the same variables), so a file with only the proxy CA would break REST calls to hosts that CA did not sign.
- `deepgram::tls` module with `TlsTrust` (`webpki`, `webpki_and_native`, `webpki_native_unavailable`, `custom`), the trust roots in effect for a connection. `webpki_native_unavailable` means the `rustls-tls-native-roots` feature is on but no native root could be loaded, so only the bundled roots were checked; the load errors are logged at `tracing` WARN level.
- Connect-diagnostics records gain `tls_trust` (present on every `wss://` attempt; absent for plaintext `ws://`, where no TLS handshake occurs) and `tls_resumed` (present once the TLS phase completed). Resumed handshakes are cheaper than full ones, so `tls_handshake_ms` should be compared within one value of `tls_resumed`. Additive; `schema_version` stays 1.

### Changed

- **BREAKING**: A Flux speech-to-text or Flux text-to-speech `wss://` connection behind a TLS-inspecting proxy or private CA that worked on 0.10.1 only because another crate in your dependency graph enabled `tokio-tungstenite/rustls-tls-native-roots` or `native-tls` now fails with `UntrustedTlsCertificate` until you enable `rustls-tls-native-roots` on `deepgram` (or pass `Deepgram::tls_config`). No public signature changed. Cause: every `wss://` WebSocket surface (live transcription, Flux speech-to-text, Flux text-to-speech) now connects through one explicit rustls connector owned by the `Deepgram` client, so trust roots and TLS provider are identical across surfaces and no longer depend on which TLS features other crates enable on `tokio-tungstenite`. Previously only `/v1/listen` with `connect-diagnostics` used an explicit connector, and the Flux surfaces took whatever feature unification produced. None of this applies to plaintext `ws://` connections (an `http://` base URL): they perform no TLS handshake and verify no certificate, so neither the feature nor `tls_config` affects them; keep `http://` to local testing and use `https://` whenever credentials or private traffic are involved.
- The default TLS configuration is built once per `Deepgram` client (on its first WebSocket connect) and reused, so TLS sessions can be resumed across connections from the same client. Before, a fresh configuration was built per attempt and no session was ever resumed.
- The TLS dependencies (`rustls`, `tokio-rustls`, `rustls-pki-types`, `webpki-roots`) are now enabled by the `listen` and `speak` features rather than only by `connect-diagnostics`. They were already present in the dependency graph through `tokio-tungstenite`; nothing new is downloaded.

### Fixed

- With `connect-diagnostics` enabled, connections behind a TLS-inspecting proxy failed even where the same application's 0.10.0 build succeeded: the explicit connector introduced in 0.10.1 bypassed the OS-root merge that `tokio-tungstenite/rustls-tls-native-roots` (enabled elsewhere in the consumer's dependency graph) had been providing through feature unification. Enable `rustls-tls-native-roots` on `deepgram` to restore that behavior explicitly.
- On a client's first connect the OS certificate store (when enabled) is read before any phase timer starts, so it is never charged to `tls_handshake_ms`. That first connect's `connect_duration_ms` can therefore exceed the sum of the phase timings by the one-time trust-store load, which is attributed to no phase. A plaintext `ws://` client never resolves TLS at all, so it does not read the store (or log about it).
- `{:?}` on a `Deepgram` client (or on a sub-client holding one, such as `Transcription` or `Speak`) printed the API key or temporary token: `reqwest::Client`'s Debug output includes its default headers, and the `Authorization` header value was not marked sensitive. It now is, so the header prints as `Sensitive`.

## [0.10.1](https://github.com/deepgram/deepgram-rust-sdk/compare/0.10.0...0.10.1)

### Added

- Flux text-to-speech (`/v2/speak`) support in the new `speak::flux` module:
  - Batch (REST): `Speak::flux_speak_to_file` and `Speak::flux_speak_to_stream` synthesize a complete block of text with `POST /v2/speak`. Query options cover `model` (required, e.g. `Model::FluxHaleyEn`), `encoding`, `sample_rate`, `speed`, `expressivity`, `mip_opt_out`, `tag`, and the REST-only `container`, `bit_rate`, `callback`, `callback_method`, and `priority`.
  - Streaming (WebSocket): `Speak::flux_request(options).handle()` connects to the `/v2/speak` websocket and returns a `FluxSpeakHandle` with `speak`, `flush`, `interrupt`, `configure_speed`, `close`, and `receive`. Server events surface as `FluxSpeakResponse`: binary `Audio` frames plus `Connected`, `SpeechStarted`, `Flushed`, `SpeechMetadata`, `SpeechInterrupted`, `SessionMetadata`, `ConfigureSuccess`/`ConfigureFailure`, `Warning`, `FatalError`, and a forward-compatible `Unknown`.
  - The websocket rejects REST-only options and the REST-only compressed encodings (`mp3`, `opus`, `flac`, `aac`) up front with the new `DeepgramError::InvalidOptions` variant; custom encodings are left for the server to validate.
  - New examples: `flux_tts_batch` and `flux_tts_websocket` under `examples/speak/flux/`.
- `FluxHandle::force_end_turn()` sends the `ForceEndTurn` control message to end the current turn immediately on an external signal (push-to-talk release, DTMF tone, UI event, or an external endpointing stack). Sent while no turn is active, it is silently ignored.
- `FluxResponse::TurnInfo` now exposes `trigger` (`Option<TurnTrigger>`), the cause of a turn ending, reported on `EndOfTurn` events: `Model` (native end-of-turn detection), `Manual` (a `ForceEndTurn` was sent), or `Timeout` (`eot_timeout_ms` elapsed). `TurnTrigger` is an open enum — unrecognized values deserialize as `TurnTrigger::Unknown(String)`, preserving the original wire value so it survives re-serialization.
- `FluxWord` now exposes optional per-word `start` and `end` timings, in seconds.
- New example: `flux_force_end_turn` under `examples/transcription/flux/`.
- Opt-in connect diagnostics behind the new `connect-diagnostics` cargo feature, for live transcription (`/v1/listen`) WebSocket connections (other WebSocket surfaces are not covered yet). Configuring a sink via `WebsocketBuilder::diagnostics` makes the SDK establish the streaming connection in four individually timed phases (DNS, TCP connect, TLS handshake, WebSocket upgrade) and emit one `diagnostics::ConnectRecord` per connect attempt. New example: `connect_diagnostics`.
  - Records are emitted for every attempt — including attempts cancelled by a caller-side `tokio::time::timeout`, and failed upgrades, which capture the `dg-request-id` and `dg-error` response headers.
  - Records serialize to flat JSON for JSONL pipelines (`schema_version: 1`, additive-only). The recorded URL is reduced to scheme, host, port, and path — userinfo and query parameters are never persisted.
  - TLS note: with the feature enabled, both the timed and the untimed `/v1/listen` connect paths are handed one explicit rustls connector (webpki trust roots, crate-default provider), so downstream feature unification — e.g. a consumer also enabling `tokio-tungstenite/native-tls` — cannot make the two paths select different TLS providers. Without the feature, the stock connect path runs unchanged.

### Changed

- The `speak` feature now enables the websocket dependencies (`tungstenite`, `tokio-tungstenite`), and `DeepgramError::WsError` is available under `speak` as well as `listen`. No API change for default-feature users.

### Fixed

- The Flux connection workers (STT and TTS) now end on the first terminal transport error (forwarding it exactly once) and complete the WebSocket closing handshake promptly when the server initiates the close, instead of leaving the close acknowledgement queued until socket teardown. After the session ends — peer close or terminal error — sends from the handle (`FluxHandle::send_data`/`configure`/`force_end_turn`, `FluxSpeakHandle::speak`/`flush`/`interrupt`/`configure_speed`) return an error instead of silently discarding the message.

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

