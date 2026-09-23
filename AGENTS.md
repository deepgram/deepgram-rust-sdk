# Agents

Instructions for AI coding agents (Claude Code, Cursor, Codex, Copilot) and for humans working with them in this repository. `CLAUDE.md` includes this file.

## Repository purpose

This is the Rust SDK for the Deepgram API, published to crates.io as `deepgram`. `Cargo.toml` is at version `0.12.0`; the latest release tag is `0.11.0`, and `Cargo.lock` is committed. The crate is hand-written: there is no code generator, no `fern/` folder, and no `.fernignore`. Edit the source directly.

Never hardcode API keys or access tokens. Examples and the ignored end-to-end tests read `DEEPGRAM_API_KEY` from the environment and construct the client with `Deepgram::new(key)`; `Deepgram::with_temp_token` takes a short-lived token from `POST /v1/auth/grant`.

## Repository map

| Path | What lives there |
| --- | --- |
| `src/lib.rs` | The `Deepgram` client, `DeepgramError`, the `Transcription`, `Speak`, and `TextIntelligence` handles, the `TranscriptionStream` re-export, base URL, and `User-Agent` |
| `src/listen/` | `rest.rs` (pre-recorded), `websocket.rs` (Nova streaming over `/v1/listen`), `flux.rs` (Flux STT over `/v2/listen`) |
| `src/speak/` | `rest.rs` (Aura batch over `POST /v1/speak`), `options.rs`, `response.rs` (`SpeakMetadata`, the `/v1/speak` response headers), `websocket.rs` (Aura streaming over `wss /v1/speak`), `flux/` (Flux TTS over `/v2/speak`: `rest.rs`, `websocket.rs`, `options.rs`, `response.rs`) |
| `src/agent/` | Voice Agent WebSocket client (`wss agent.deepgram.com/v1/agent/converse`): `websocket.rs` (the `Agent` sub-client, `AgentHandle`, `AgentEventStream`), `settings.rs`, `messages.rs` (client-to-server), `response.rs` (server events), and the `audio`, `listen`, `think`, `speak`, `history`, `endpoint`, `aws_credentials` option modules |
| `src/read/` | Text Intelligence over `POST /v1/read`: `rest.rs` (requests), `options.rs` (query builder), `response.rs` |
| `src/manage/` | Management API: `billing`, `invitations`, `keys`, `members`, `models`, `projects`, `scopes`, `self_hosted`, `usage`. Each has response types; `keys` and `projects` have `options.rs`, and `usage` has operation-specific option modules. |
| `src/auth/` | `grant` for temporary tokens |
| `src/common/` | Shared `Options` builder, `Model` enum, audio sources, the batch, stream, and Flux STT response types, and `captions.rs` (the SRT/WebVTT helper) |
| `src/diagnostics.rs` | Opt-in per-phase connect timing for `/v1/listen` (feature `connect-diagnostics`) |
| `src/tls.rs` | The rustls connector every `wss://` WebSocket surface uses, `TlsTrust`, the `rustls-tls-native-roots` merge, and `Deepgram::tls_config` resolution |
| `examples/` | Runnable example programs; registered targets are `[[example]]` entries in `Cargo.toml`, and sample audio is under `examples/audio/` |
| `tests/` | Integration tests: `*_local.rs` run against in-process servers, `*_e2e.rs` are `#[ignore]` and need the live API |
| `.github/workflows/ci.yaml` | The CI matrix (nine jobs, listed below); `context7.yml` refreshes the Context7 index on release |
| `.agents/skills/` | Agent-agnostic skills for using this SDK (speech-to-text, conversational STT, text-to-speech, voice agent, audio intelligence, text intelligence, management API) |

## Cargo features

`default = ["manage", "listen", "speak", "read", "agent"]`. `listen`, `speak`, and `agent` each pull in `tungstenite` and `tokio-tungstenite` plus the internal `__tls` feature (`rustls`, `rustls-pki-types`, `tokio-rustls`, `webpki-roots`, `tracing`), so every `wss://` surface shares one rustls connector; `rustls-tls-native-roots` adds `rustls-native-certs` and enables the same-named `tokio-tungstenite` feature to trust the OS certificate store on top of the bundled roots; `connect-diagnostics` implies `listen` and adds `uuid/v4`. `read` and `manage` are HTTP-only and pull in no extra dependencies. `auth` has no feature gate and is always compiled; `src/common/` is gated `#[cfg(any(feature = "listen", feature = "read"))]`, because both surfaces share its `Options` builder and response types. Gate a new module with `#[cfg(feature = "...")]` in `src/lib.rs` and add a `cargo check --no-default-features --features=<name>` line to `ci.yaml` if you add a feature.

## Client surfaces

Every row below was checked against `src/` on 2026-09-15.

| Product | Endpoint | Entry point | Status |
| --- | --- | --- | --- |
| Speech-to-text, pre-recorded | `POST /v1/listen` | `dg.transcription().prerecorded(source, &options)`, `prerecorded_callback`, `make_prerecorded_request_builder` | Shipped (`listen`) |
| Speech-to-text, streaming (Nova) | `wss /v1/listen` | `dg.transcription().stream_request()` or `stream_request_with_options(options)`, then `.file(...)`, `.stream(...)`, or `.handle()` for a `WebsocketHandle` (`send_data`, `finalize`, `keep_alive`, `close_stream`, `receive`) | Shipped (`listen`) |
| Flux STT (conversational speech-to-text) | `wss /v2/listen` | `dg.transcription().flux_request()` or `flux_request_with_options(options)`, then `.handle()` for a `FluxHandle` (`send_data`, `configure`, `force_end_turn`, `close_stream`, `receive`); models `Model::FluxGeneralEn`, `Model::FluxGeneralMulti` | Shipped (`listen`) since 0.8.0; `configure` and `language_hint` since 0.10.0; `force_end_turn` since 0.10.1 |
| Text-to-speech, batch (Aura) | `POST /v1/speak` | `dg.text_to_speech().speak_to_file(...)`, `speak_to_stream(...)`; `speak_to_file_with_metadata(...)`, `speak_to_stream_with_metadata(...)` also return a `SpeakMetadata` whose `request_id()` is the `dg-request-id` header | Shipped (`speak`) |
| Text-to-speech, streaming (Aura) | `wss /v1/speak` | `dg.text_to_speech().speak_stream()` for a `SpeakStreamBuilder` (`model`, `encoding`, `sample_rate`, `speed`, `mip_opt_out`, `query_params`), then `.handle()` for a `SpeakStreamHandle` (`speak`, `flush`, `clear`, `close`, `receive`, `split`, `sender`, `request_id`); events arrive as `SpeakResponse` | Shipped (`speak`) |
| Flux TTS, batch | `POST /v2/speak` | `dg.text_to_speech().flux_speak_to_file(...)`, `flux_speak_to_stream(...)` | Shipped (`speak`) since 0.10.1 |
| Flux TTS, streaming | `wss /v2/speak` | `dg.text_to_speech().flux_request(options).handle()` for a `FluxSpeakHandle` (`speak`, `flush`, `interrupt`, `configure_speed`, `close`, `receive`); events arrive as `FluxSpeakResponse` | Shipped (`speak`) since 0.10.1 |
| Voice Agent | `wss agent.deepgram.com/v1/agent/converse` | `dg.agent().start()` (or `start_at_url(url)` for self-hosted hosts and local mock servers, which requires `wss://` except on loopback), returning an `AgentHandle` (`send_settings`, `send_update_listen`, `send_update_speak`, `send_update_think`, `send_update_prompt`, `send_inject_user_message`, `send_inject_agent_message`, `send_function_call_response`, `send_raw_json`, `send_data`, `keep_alive`, `force_end_turn`, `close`) and an `AgentEventStream` of `AgentEvent` (`Json(AgentResponse)` / `Audio(Bytes)`) | Shipped (`agent`) |
| Text intelligence | `POST /v1/read` | `dg.text_intelligence()` for a `TextIntelligence` (`analyze_text`, `analyze_url`, `analyze_text_callback`, `analyze_url_callback`, `make_read_request_builder`, `make_read_callback_request_builder`) | Shipped (`read`) |
| Management API | `/v1/projects/...` | `dg.projects()`, `dg.keys()`, `dg.members()`, `dg.scopes()`, `dg.invitations()`, `dg.usage()`, `dg.billing()`, and `dg.models()` for a `Models` (`get_models`, `get_models_including_outdated`, `get_model`, `get_project_models`, `get_project_models_including_outdated`, `get_project_model`) | Shipped (`manage`) |
| Self-hosted credentials | `/v1/projects/{id}/self-hosted/distribution/credentials` | `dg.self_hosted()` with `list_distribution_credentials`, `get_distribution_credentials`, `create_distribution_credentials`, `delete_distribution_credentials` | Shipped (`manage`) |
| Auth (grant token) | `POST /v1/auth/grant` | `dg.auth().grant(options)` | Shipped |

## Prerequisites

- Stable Rust. CI installs the current stable toolchain with `actions-rust-lang/setup-rust-toolchain@v1` plus the `rustfmt` component; `Cargo.toml` sets no `rust-version`. The Minimal-Versions job also installs nightly.
- ALSA development headers on Linux (`apt-get install -y pkg-config libasound2-dev`). The `cpal` dev-dependency powers the microphone examples and `rodio` powers the TTS streaming example; `--all-targets` compiles every example, so `cargo build`, `cargo clippy`, and `cargo test` fail without them.
- `cargo-audit`, `cargo-hack`, and `cargo-semver-checks` for the three CI jobs that use them (`cargo install --locked <tool>`).

## Build, test, lint, format

CI runs every job on every push and pull request with `RUSTFLAGS=-D warnings` and `RUSTDOCFLAGS=-D warnings`, so a single warning fails the build. Export both before running the commands. The first six rows were run on 2026-09-13 inside a `rust:latest` container (Rust 1.98.0) with the repository mounted at `/work` and the ALSA headers installed; the exit codes are recorded in the pull request that added this file.

| CI job | Command | Notes |
| --- | --- | --- |
| Format | `cargo fmt --check --all` | Run `cargo fmt --all` to fix |
| Features | `cargo check --all-targets --no-default-features`, then the same with `--features=listen`, `--features=speak`, `--features=manage`, `--features=read`, `--features=agent`, `--features=connect-diagnostics`, `--features=listen,rustls-tls-native-roots`, `--features=speak,rustls-tls-native-roots`, `--features=agent,rustls-tls-native-roots`, `--features=connect-diagnostics,rustls-tls-native-roots`, then `cargo test --tests` with default features | Each feature must compile alone, and the native-roots feature with each surface it applies to |
| Build | `cargo build --all-targets --all-features` | About 3 minutes from a cold cache |
| Clippy | `cargo clippy --all-targets --all-features` | Zero warnings at 0.10.1; `#![warn(clippy::cargo)]` is on in `lib.rs` |
| Test | `cargo test --all --all-features` | At 0.10.1: 105 unit tests pass, 2 are ignored, the `*_local.rs` integration tests pass, and the `*_e2e.rs` tests are ignored. No network access needed |
| Documentation | `cargo doc --workspace --all-features`, then `cargo doc --no-deps --no-default-features --features <f>` for each of `listen`, `speak`, `manage`, `read`, `agent`, `connect-diagnostics` | `missing_docs` is a warning and warnings are errors, so every public item needs a doc comment. The per-feature runs catch intra-doc links into modules that another feature gates; write those as a `#[cfg_attr(feature = "...", doc = "...")]` pair (see `src/common/options.rs`) so the link survives on docs.rs, which builds all features |
| Audit | `cargo install --locked cargo-audit cargo-hack && cargo hack --remove-dev-deps && cargo generate-lockfile && cargo audit` | Run in a throwaway checkout; it rewrites `Cargo.toml` and `Cargo.lock`. Not run for this file |
| Minimal-Versions | Run the full sequence below | Lower bounds in `Cargo.toml` must be real; the crates under "specified only to satisfy minimal-versions" exist for this job. Not run for this file |
| SemVer | `cargo install --locked cargo-semver-checks && cargo semver-checks check-release --verbose` | Fails a pull request that breaks the public API without a version bump. Not run for this file |

## Manual live verification

These ignored end-to-end tests require `DEEPGRAM_API_KEY` and are not run by CI:

```bash
DEEPGRAM_API_KEY=<key> cargo test --all-features --test flux_e2e -- --ignored
DEEPGRAM_API_KEY=<key> cargo test --all-features --test connect_diagnostics_e2e -- --ignored
```

## Minimal-Versions verification

Run this full CI sequence in a throwaway checkout because `cargo hack --remove-dev-deps` rewrites `Cargo.toml` and `Cargo.lock`:

```bash
rustup run nightly cargo build --all-targets --all-features -Z minimal-versions
rustup run nightly cargo test --all --all-features -Z minimal-versions
cargo install --locked cargo-hack
cargo hack --remove-dev-deps
rustup run nightly cargo generate-lockfile -Z minimal-versions
rustup run nightly cargo build --all-features -Z minimal-versions
```

## Run an example against the live API

Registered example names are the `[[example]]` entries in `Cargo.toml`; entries list `required-features` where needed. Both commands below were run on 2026-09-13 in the same `rust:latest` container with the key passed through the environment; the first printed the transcript, the second wrote `flux-tts-batch.mp3` (39168 bytes) with `flux-haley-en`.

```bash
# Every example reads DEEPGRAM_API_KEY from the environment.
export DEEPGRAM_API_KEY="<your key>"

# Transcribe a hosted file with the default model and print the transcript.
cargo run --example prerecorded_from_url

# Synthesize a block of text with Flux TTS over REST and write flux-tts-batch.mp3.
cargo run --features speak --example flux_tts_batch

# Stream a Flux STT session and print TurnInfo events. To use another file,
# change PATH_TO_FILE in examples/transcription/flux/simple_flux.rs.
cargo run --example simple_flux
```

The `microphone_stream` and `microphone_flux` examples capture audio with `cpal`, so they need an audio device and do not run in a container. Examples write output files into the current directory; delete them before you commit.

## Implementation conventions

- `src/lib.rs` sets `#![forbid(unsafe_code)]` and warns on `missing_docs`, `missing_debug_implementations`, and `clippy::cargo`. With `-D warnings` in CI, every public item needs a `///` doc comment and every type needs `Debug`.
- Public enums that mirror server values are `#[non_exhaustive]` and carry an `Unknown` variant, so a new server message or value never breaks a deployed client. `FluxResponse::Unknown` and `TurnTrigger::Unknown(String)` preserve the raw wire value; `TurnEvent::Unknown` identifies an unrecognized event. Do the same for any new response enum, and add a deserialization test in the module's `tests` block.
- Request options are builders: `Options::builder()` with chained setters for the shared query, `ConfigureRequest::new().with_thresholds(...)`, and `#[non_exhaustive]` structs with `new()` plus `with_*` methods. Options serialize with `serde_urlencoded`; multi-value parameters (for example `language_hint`) serialize as repeated keys.
- Errors are `DeepgramError` variants (`thiserror`). Validate what the server would reject anyway only when the failure would otherwise be confusing (the Flux TTS WebSocket rejects REST-only options up front with `DeepgramError::InvalidOptions`); leave everything else to the server.
- WebSocket clients run a worker task and hand the caller a handle. Sends after the session has ended return an error instead of silently dropping the message, and the worker forwards a terminal transport error exactly once. Keep that contract in any new streaming surface (`tests/flux_backpressure_local.rs` and `tests/flux_speak_backpressure_local.rs` show the pattern).
- Keep dependency lower bounds honest. If you call an API that a newer version of a dependency introduced, raise that dependency's minimum in `Cargo.toml`, or the Minimal-Versions job fails.
- A change to a public signature is a semver event, and so is a behavior change that makes code working on the last release fail until the consumer changes something (a Cargo feature, a config call, an environment variable). In `0.x`, either kind of breaking change needs a minor bump and a `**BREAKING**` line in `CHANGELOG.md` that leads with what breaks and what to do (the 0.10.0 and 0.11.0 entries are the model); an additive change needs a patch or minor bump. `cargo-semver-checks` enforces the signature half in CI; it cannot see behavior, so read every `Changed` and `Fixed` entry for a consequence before labeling a release a patch.
- `CHANGELOG.md` follows Keep a Changelog with `Added`, `Changed`, and `Fixed` sections. Add your entry under the version being prepared in the same pull request as the code.
- Write "Flux STT" or "Flux TTS" in prose and doc comments; never bare "Flux". Identifiers such as `FluxHandle`, `flux_request`, and the `flux-general-en` model name stay as they are.

## Example: add a Flux STT query parameter

1. Add the field to the Flux STT options in `src/common/options.rs` (or `src/listen/flux.rs` if it is WebSocket-only), with a doc comment and the `serde` attribute that matches the wire name.
2. Add a `urlencoded()` assertion in that module's `tests` block proving the parameter serializes correctly, including the repeated-key form for list values.
3. If the server answers with a new field, extend the matching response type in `src/common/flux_response.rs` and add a fixture-based deserialization test.
4. Update the closest example under `examples/transcription/flux/` and add a `CHANGELOG.md` line.
5. Run `cargo fmt --all`, `cargo clippy --all-targets --all-features`, `cargo test --all --all-features`, and `cargo doc --workspace --all-features`, all with `RUSTFLAGS=-D warnings RUSTDOCFLAGS=-D warnings`.

## Release process

Releases are commits and tags on `main`; there is no release-please and no publish workflow.

1. Open a pull request titled `chore: release X.Y.Z` that bumps `version` in `Cargo.toml`, refreshes `Cargo.lock`, and turns the pending `CHANGELOG.md` heading into `## [X.Y.Z](https://github.com/deepgram/deepgram-rust-sdk/compare/<prev>...X.Y.Z)`. The 0.10.1 release commit (`d884c6bd`) touched exactly those three files.
2. After the merge, tag with plain semver and no `v` prefix (`git tag -m 0.10.1 0.10.1 && git push origin 0.10.1`) and publish a GitHub release from the tag. `context7.yml` refreshes the Context7 index when the release is published.
3. A maintainer publishes the crate from the tagged commit with `cargo publish`.

Pull requests target `main`. Older copies of `CONTRIBUTING.md` and the pull request template named a `dev` branch; that branch no longer exists.

## Pull requests

- Keep diffs focused on one issue or behavior change.
- Add regression coverage for bug fixes, including the exact query string or WebSocket frame when the bug is a serialization or protocol mismatch.
- State the commands you ran and link the related issue. Use `Fixes #<issue>` only when the pull request fully resolves it.
- Follow `.github/PULL_REQUEST_TEMPLATE.md`. Commit messages follow Conventional Commits with a scope (`feat(speak):`, `feat(flux)!:`, `fix(listen):`, `chore:`); `[no-ci]` in the title skips CI for docs-only changes.

## Documentation

- API reference and product guides: <https://developers.deepgram.com/docs>
- Crate docs: <https://docs.rs/deepgram>
- SDK feature matrix: <https://developers.deepgram.com/docs/sdks/sdk-features>
- crates.io: <https://crates.io/crates/deepgram>
- Agent skills that teach this SDK: `.agents/skills/` (install with `npx skills add deepgram/deepgram-rust-sdk`)

## Do not

- Do not commit generated audio (`*.mp3`, `*.wav` outside `examples/audio/`), keys, or `.env` files.
- Do not remove registered `[[example]]` entries or their `required-features`; `--all-targets` in CI compiles every registered example target.
- Do not add `unsafe` code; `#![forbid(unsafe_code)]` rejects it.
- Do not change a public signature without a version bump and a `CHANGELOG.md` entry.
- Do not run `cargo hack --remove-dev-deps` in your working checkout; it rewrites `Cargo.toml`.
