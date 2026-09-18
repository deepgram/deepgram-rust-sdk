# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Voice Agent WebSocket client (`agent` feature, enabled by default): `Deepgram::agent()` returns an `Agent` sub-client; `start()` / `start_at_url()` open a live session at `wss://agent.deepgram.com/v1/agent/converse` and return an `AgentHandle` and an `AgentEventStream` (implements `futures::Stream`). The stream yields `AgentEvent::Json(AgentResponse)` for typed JSON events and `AgentEvent::Audio(Bytes)` for the agent's synthesized speech, so both arrive in order on one stream.
  - **Client controls** (typed `send_*` methods plus binary audio via `send_data`): `Settings`, `UpdateListen`, `UpdateSpeak`, `UpdateThink`, `UpdatePrompt`, `InjectUserMessage`, `InjectAgentMessage` (behaviors `default` / `queue` / `interrupt`), `FunctionCallResponse`, `KeepAlive`, and `ForceEndTurn` (`force_end_turn()`, Flux STT listen providers only; a v1 listen provider answers with a `Warning` carrying `FORCE_END_TURN_UNSUPPORTED` and the turn is not ended), plus `send_raw_json` as an escape hatch for a client message this release does not model.
  - **Server events** (typed `AgentResponse` variants): `Welcome`, `SettingsApplied`, `ConversationText`, `UserStartedSpeaking`, `AgentThinking`, `FunctionCallRequest`, `FunctionCallCancelled` (cancelled `functions[].{id,name}` — do not respond to these), `AgentStartedSpeaking`, `AgentAudioDone`, `LatencyReport` (per-turn STT/LLM/TTS latencies), `History`, `Error`, `Warning`, `ListenUpdated` / `PromptUpdated` / `SpeakUpdated` / `ThinkUpdated`, `InjectionRefused`, server-side `FunctionCallResponse`. Deserialization dispatches on the event's `type` field, so only an *unrecognized* (or absent) `type` lands in the `Unknown(serde_json::Value)` catch-all, with its raw JSON preserved — a future server event never breaks a deployed client. An event whose `type` *is* recognized but whose payload does not match that event's schema (say an `Error` event missing its `code`) is reported as a `DeepgramError::JsonError` on the event stream naming the event type, rather than being silently downgraded to `Unknown` where a consumer matching on `AgentResponse::Error` would never see it. A recognized event carrying an *unrecognized value* is neither an error nor an `Unknown` event — it stays on the happy path through that value's own catch-all, so a new server enum value never ends a session: a `role` this release does not name (on `ConversationText` or `History`) arrives as `ConversationRole::Unknown(String)` holding the wire string verbatim and re-serializing to it exactly, and a `History` entry matching neither modeled shape as `HistoryMessage::Unknown(serde_json::Value)` with its JSON intact. One known off-spec message: Flux STT (V2) sessions can emit `{"type":"EndOfTurn","trigger":"model"|"timeout"}`, which is not in the AsyncAPI spec yet and currently arrives as `Unknown`.
  - **Settings**: inline or saved (`Uuid`) agent configs; `listen` providers Deepgram V1 (Nova) and V2 (Flux STT, with `language_hints` array, `eot_threshold`, `eager_eot_threshold`, `eot_timeout_ms`, `keyterms`); `think` providers OpenAI/Anthropic/AWS Bedrock/Google/Groq with function definitions (including `defer_until_eot` for irreversible actions); `speak` providers Deepgram (`version: v1` Aura and `version: v2` Flux TTS voices, both taking `speed` — the accepted range differs per family — with `expressivity` on `v2` only), ElevenLabs, Cartesia, OpenAI, AWS Polly; audio I/O config; conversation history/context.
  - **Connection behavior**: the worker reserves event-channel capacity before it reads an inbound frame and forwards without blocking, and selects between inbound frames and outbound commands without bias, so continuous agent audio cannot starve outgoing audio, function responses, `KeepAlive`, or the Close request — including when the consumer stops reading the event stream, which previously stalled every outbound message behind the backlog; after the session ends (peer close, or a terminal read or write error) every retained `AgentHandle` send fails and the worker exits. A terminal transport error reaches the event stream exactly once, whichever direction notices it: the worker ends on a failed write rather than accepting further commands it can no longer deliver, and ends on a read error rather than polling a broken socket, which can report the same fault twice. If the consumer drops the `AgentEventStream` instead, the worker notices on its next inbound frame or close (on a silent connection sends keep succeeding until then) and then fails sends the same way. A normal WebSocket close (code 1000) ends the event stream without an error item; only abnormal close codes are surfaced as `DeepgramError::WebsocketClose`. After `AgentHandle::close()` the wait for the server's answering close is bounded: a ten-second gap with nothing from the server ends the event stream with `DeepgramError::UnexpectedServerResponse` and drops the connection, so a server that never completes the closing handshake cannot leave the worker task alive and the event stream never ending. Completing the closing handshake still requires the event stream to be drained, since the worker has to deliver the server's remaining frames somewhere; writing the client's Close frame does not.
  - **Security**: `start_at_url` requires `wss://` — plain `ws://` is accepted only for loopback hosts (`localhost`, `127.0.0.0/8`, `::1`) so local mock servers work; any other cleartext URL is rejected with `DeepgramError::InsecureAgentUrl` (carrying an `InsecureAgentUrl` whose `url` is the rejected origin only, so it is safe to log) before the credential is sent, which callers can `match` on directly. `wss://` sessions connect through the client's shared rustls connector (see `tls`), so the `rustls-tls-native-roots` feature and `Deepgram::tls_config` apply to the Voice Agent exactly as they do to live transcription and text-to-speech, and an untrusted server certificate surfaces as `DeepgramError::UntrustedTlsCertificate`. `Debug` output for `AwsCredentials`, `Endpoint`, and `FunctionEndpoint` redacts AWS keys/tokens, header values, and any `user:pass@` userinfo in an endpoint URL.
  - New examples `simple_agent` and `function_calling` ([#152](https://github.com/deepgram/deepgram-rust-sdk/issues/152)). The REST saved-configuration/variables namespace (`/v1/agent/settings`) is not included in this release.
- Streaming Text-to-Speech over a WebSocket: `Speak::speak_stream()` returns a `SpeakStreamBuilder` (`model` / `encoding` / `sample_rate` / `speed` / `mip_opt_out`, plus a `query_params` escape hatch for parameters not yet modeled); `handle()` validates the options locally and opens the connection, returning a `SpeakStreamHandle` for sending text (`speak`, `flush`, `clear`, `close`) and receiving audio + events. The streaming endpoint accepts only `linear16`, `mulaw`, and `alaw` output, `sample_rate` in `8000` / `16000` / `24000` / `32000` / `48000`, and `speed` in `0.7..=1.5`; REST-only encodings, an unsupported sample rate, or an out-of-range speed are rejected before connecting with `DeepgramError::InvalidOptions`. `SpeakStreamHandle::split()` returns a `Clone`able `SpeakStreamSender` (all methods take `&self`) and a `SpeakStreamEvents` stream so text can be sent from one task while audio is consumed in another (LLM-driven use); `SpeakStreamHandle::sender()` hands out a sender clone without splitting. Text is written to the socket as it is enqueued regardless of how quickly events are consumed, so an unconsumed audio backlog never blocks `speak`. Dropping the handle sends `Close` and completes the WebSocket closing handshake, and the wait for the server after `Close` is bounded (a ten-second idle gap ends the stream with `DeepgramError::UnexpectedServerResponse`). `SpeakStreamHandle` and `SpeakStreamEvents` implement `futures::Stream<Item = Result<SpeakResponse>>`; `SpeakResponse` (`Audio` / `Metadata` / `Flushed` / `Cleared` / `Warning` / `Unknown`) and its struct variants are `#[non_exhaustive]`, and unknown message types are preserved rather than breaking the stream. `SpeakResponse::Metadata` carries `request_id`, `model_name`, `model_version`, `model_uuid`, and `additional_model_uuids` (both reported verbatim as strings, so a value the server does not format as a UUID is still delivered instead of downgrading the event to `Unknown`) ([#148](https://github.com/deepgram/deepgram-rust-sdk/issues/148), [#147](https://github.com/deepgram/deepgram-rust-sdk/issues/147), [#95](https://github.com/deepgram/deepgram-rust-sdk/issues/95)). New example `text_to_speech_websocket`. Like every other `wss://` surface, the connection goes through the client's shared rustls connector (see `tls`), so `rustls-tls-native-roots` and `Deepgram::tls_config` apply and an untrusted server certificate surfaces as `DeepgramError::UntrustedTlsCertificate`.
- Self-hosted (on-prem) distribution credentials management: `Deepgram::self_hosted()` returns a `SelfHosted` client. `list_distribution_credentials` / `get_distribution_credentials` return `{ member, distribution_credentials }` entries (now including the server's `tags`); `create_distribution_credentials` takes a `CreateDistributionCredentials` request (required `comment`; `provider` defaults to `quay`, `scopes` to `[Scope::Products]`, optional `tags`), sends it as the JSON body, and returns a `CreatedDistributionCredentials` carrying the one-time registry `username` and `secret` (a `SecretString`, redacted in `Debug`, read with `expose_secret()`); `delete_distribution_credentials` returns the server's `{ message }` acknowledgement. Permission scopes are the typed `Scope` enum (`Products`, `Api`, `Engine`, `LicenseProxy`, `Dgtools`, `Billing`, `Hotpepper`, `MetricsServer`) rather than bare strings, on the same open-enum pattern as the rest of the SDK: a scope this version does not name round-trips through `Scope::Unknown(String)`, which is also how a caller sends one ahead of an SDK release, and `.scopes(["self-hosted:product:api"])` still works because `Scope` implements `From<&str>` / `From<String>`. `get_distribution_credentials` and `delete_distribution_credentials` take the `Uuid` the SDK itself returns in `distribution_credentials_id`, so no `.to_string()` round trip is needed. All four requests are built against the client's configured base URL, so `Deepgram::with_base_url(...)` reaches a self-hosted or proxied API host. New example `self_hosted_credentials`.
- Text Intelligence (`/v1/read`) support: `Deepgram::text_intelligence()` returns a `TextIntelligence` client with `analyze_text` and `analyze_url` for running sentiment, summarization, topic, and intent analysis over text. The handle is named after the product rather than the endpoint so the crate root does not export a `Read` that shadows `std::io::Read`; it is `deepgram::TextIntelligence` and `deepgram::read::TextIntelligence`. Reuses the existing sentiment/topic/intent response types and adds a Text-Intelligence-specific `Summary` (`text`) ([#94](https://github.com/deepgram/deepgram-rust-sdk/issues/94)). Every field of `ReadMetadata` (including `request_id` and `created`) and of `AnalysisInfo` is an `Option`, matching the `/v1/read` reference, which marks them all optional; a response that omits one deserializes rather than failing. Requests default to `language=en`, the only language `/v1/read` accepts (the API returns 400 without it); call `.language(...)` to override. Async callbacks are supported via `analyze_text_callback` / `analyze_url_callback` (returning `CallbackResponse { request_id }`) and `OptionsBuilder::callback_method`. Gated behind a new `read` Cargo feature (enabled by default) so `/v1/read` can be used without the WebSocket-only `listen` feature: `deepgram = { version = "…", default-features = false, features = ["read"] }`. The shared `common` module is available under either `listen` or `read`. New example `analyze_text`.
- Model-listing endpoints on the management API: `Deepgram::models()` returns a `Models` client with `get_models`, `get_model`, `get_project_models`, and `get_project_model`. The two listing methods take no arguments and return the latest versions only; `get_models_including_outdated` and `get_project_models_including_outdated` also return non-latest versions. `include_outdated` is optional on the endpoint and defaults to latest-only, so it is sent only by the `_including_outdated` variants. `Model`'s `name`, `canonical_name`, `architecture`, `version`, and `uuid` are `Option<String>` to match the Models API, which marks them optional. Everything the endpoints return is reachable from the typed response: `ModelsResponse` exposes the top-level `languages` map (BCP-47 tag to display name; 194 entries when checked on 2026-09-18) so a tag in `Model::languages` can be labeled for a human, `Model` exposes `multilingual: Option<bool>` (present on every STT record the endpoint returned on 2026-09-18, and typed `Option` so a record without it still deserializes), and `ModelMetadata` exposes `display_name: Option<String>`, the human-readable voice name for a picker — `"Agathe"` where `name` is `"agathe"`. On 2026-09-18 the endpoint sent `display_name` for every voice, leaving it `null` for the ones without one, so fall back to `name`. `manage::models::response::Model` (the metadata record) and `common::options::Model` (the request-time selector) now cross-link in the docs. The four requests are built against the client's configured base URL, so `Deepgram::with_base_url` reaches them. New example `models`.
- `futures::Stream` for live transcription is confirmed and documented: `listen::websocket::TranscriptionStream` already implements `Stream<Item = Result<StreamResponse>>`, so results compose with `StreamExt` combinators. It is now also re-exported at the crate root as `deepgram::TranscriptionStream`. New example `stream_futures` demonstrates idiomatic combinator usage ([#163](https://github.com/deepgram/deepgram-rust-sdk/issues/163)).
- Text-to-Speech REST responses now expose their metadata headers, including the `dg-request-id`. New `Speak::speak_to_file_with_metadata` and `Speak::speak_to_stream_with_metadata` return a `SpeakMetadata` (request id, model name/uuid, character count, content type) alongside the audio ([#89](https://github.com/deepgram/deepgram-rust-sdk/issues/89)). New example `text_to_speech_request_id`.
- Named Deepgram Whisper Cloud model variants: `Model::Whisper`, `Model::WhisperTiny`, `Model::WhisperBase`, `Model::WhisperSmall`, `Model::WhisperMedium`, and `Model::WhisperLarge` (previously only reachable via `Model::CustomId`) ([#128](https://github.com/deepgram/deepgram-rust-sdk/issues/128)).
- Named Aura-2 text-to-speech voices on `speak::options::Model` (for example `Model::Aura2ThaliaEn`), covering the voices the API served as of this release across English, Spanish, German, Dutch, French, Italian, and Japanese; previously only reachable via `Model::CustomId`. `Model` now also implements `From<&str>` / `From<String>`, resolving a wire string such as `"aura-2-thalia-en"` to its named variant (or `CustomId` when unrecognized). Every named variant carries a generated doc comment naming its wire string and language.
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

- The Flux text-to-speech worker no longer deadlocks when it forwards a terminal write error. A caller that sends on a `FluxSpeakHandle` without draining events could hang: once a write failed, the worker parked waiting for room on the full response channel while the caller parked on the full command channel, so neither side made progress and the error never surfaced. The worker now forwards the terminal error through a channel slot reserved for exactly that error, so the error is delivered even when the audio ahead of it is undrained, and it neither blocks nor is dropped. Concretely: a caller that broke the transport and stopped draining used to hang forever and never learn why; it now reads the queued audio and then the transport error, and its next `speak()` fails. The new streaming text-to-speech worker (`SpeakStreamHandle`) uses the same reserved slot, which is what makes the error reach a consumer that split the handle and drains events from another task while the response channel is full. (A message the worker could not serialize — unreachable in practice — is still forwarded on the shared buffer without blocking.) Fixes behavior released in 0.10.1.
- The `Deepgram::with_base_url` and `Deepgram::with_base_url_and_api_key` docs no longer claim that every admin feature goes to `https://api.deepgram.com` regardless of the configured base URL. They now name what actually honors the base URL (transcription, text-to-speech, Text Intelligence, the model-listing endpoints, and the self-hosted distribution credentials) and what still does not (billing, usage, keys, members, invitations, projects, scopes, and the `/v1/auth/grant` token exchange), and they document that a base URL carrying a path of its own must end in a slash, since endpoint paths are resolved with `Url::join`.
- WebSocket handshakes (live transcription, Flux speech-to-text, Flux text-to-speech) now send the URL's full authority in the `Host` header, so a base URL on a non-default port (`https://dg.internal.example:8443`, `http://127.0.0.1:54321`) produces `Host: dg.internal.example:8443` rather than `Host: dg.internal.example`. Strict reverse proxies and virtual-host routing on self-hosted deployments could reject or misroute the old value. Default ports are still omitted, so requests to `api.deepgram.com` are unchanged.
- `cargo doc` with a single Cargo feature (for example `--no-default-features --features manage`) no longer reports unresolved intra-doc links to modules that are compiled only under other features: from the crate-level feature list, the `with_base_url*` docs, and the shared `common` module's links to `Transcription::prerecorded` / `prerecorded_callback`. Documentation builds with all features, including docs.rs, still render every one of those links as a link.

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

