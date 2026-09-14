# Agents

Instructions for AI coding agents (Claude Code, Cursor, Codex, Copilot) and for humans working with them in this repository. `CLAUDE.md` includes this file.

## Repository purpose

This is the Rust SDK for the Deepgram API, published to crates.io as `deepgram`. `Cargo.toml` is at version `0.10.1`, which is also the latest release tag, and `Cargo.lock` is committed. The crate is hand-written: there is no code generator, no `fern/` folder, and no `.fernignore`. Edit the source directly.

Never hardcode API keys or access tokens. Examples and the ignored end-to-end tests read `DEEPGRAM_API_KEY` from the environment and construct the client with `Deepgram::new(key)`; `Deepgram::with_temp_token` takes a short-lived token from `POST /v1/auth/grant`.

## Repository map

| Path | What lives there |
| --- | --- |
| `src/lib.rs` | The `Deepgram` client, `DeepgramError`, the `Transcription` and `Speak` handles, base URL, and `User-Agent` |
| `src/listen/` | `rest.rs` (pre-recorded), `websocket.rs` (Nova streaming over `/v1/listen`), `flux.rs` (Flux STT over `/v2/listen`) |
| `src/speak/` | `rest.rs` (Aura over `/v1/speak`), `flux/` (Flux TTS over `/v2/speak`: `rest.rs`, `websocket.rs`, `options.rs`, `response.rs`) |
| `src/manage/` | Management API: `billing`, `invitations`, `keys`, `members`, `projects`, `scopes`, `usage`, each with `options.rs` and `response.rs` |
| `src/auth/` | `grant` for temporary tokens |
| `src/common/` | Shared `Options` builder, `Model` enum, audio sources, and the batch, stream, and Flux STT response types |
| `src/diagnostics.rs` | Opt-in per-phase connect timing for `/v1/listen` (feature `connect-diagnostics`) |
| `examples/` | Runnable programs registered as `[[example]]` targets in `Cargo.toml`; sample audio under `examples/audio/` |
| `tests/` | Integration tests: `*_local.rs` run against in-process servers, `*_e2e.rs` are `#[ignore]` and need the live API |
| `.github/workflows/ci.yaml` | The CI matrix (nine jobs, listed below); `context7.yml` refreshes the Context7 index on release |
| `.agents/skills/` | Agent-agnostic skills for using this SDK (speech-to-text, conversational STT, text-to-speech, voice agent, audio intelligence, text intelligence, management API) |

## Cargo features

`default = ["manage", "listen", "speak"]`. `listen` and `speak` each pull in `tungstenite` and `tokio-tungstenite`; `connect-diagnostics` implies `listen` and adds `rustls`, `rustls-pki-types`, `tokio-rustls`, `webpki-roots`, and `uuid/v4`. `auth` is always compiled. Gate a new module with `#[cfg(feature = "...")]` in `src/lib.rs` and add a `cargo check --no-default-features --features=<name>` line to `ci.yaml` if you add a feature.

## Client surfaces

Every row below was checked against `src/` on 2026-09-13.

| Product | Endpoint | Entry point | Status |
| --- | --- | --- | --- |
| Speech-to-text, pre-recorded | `POST /v1/listen` | `dg.transcription().prerecorded(source, &options)`, `prerecorded_callback`, `make_prerecorded_request_builder` | Shipped (`listen`) |
| Speech-to-text, streaming (Nova) | `wss /v1/listen` | `dg.transcription().stream_request()` or `stream_request_with_options(options)`, then `.file(...)`, `.stream(...)`, or `.handle()` for a `WebsocketHandle` (`send_data`, `finalize`, `keep_alive`, `close_stream`, `receive`) | Shipped (`listen`) |
| Flux STT (conversational speech-to-text) | `wss /v2/listen` | `dg.transcription().flux_request()` or `flux_request_with_options(options)`, then `.handle()` for a `FluxHandle` (`send_data`, `configure`, `force_end_turn`, `close_stream`, `receive`); models `Model::FluxGeneralEn`, `Model::FluxGeneralMulti` | Shipped (`listen`) since 0.8.0; `configure` and `language_hint` since 0.10.0; `force_end_turn` since 0.10.1 |
| Text-to-speech, batch (Aura) | `POST /v1/speak` | `dg.text_to_speech().speak_to_file(...)`, `speak_to_stream(...)` | Shipped (`speak`) |
| Text-to-speech, streaming (Aura) | `wss /v1/speak` | none | Not shipped; `src/speak/` has no v1 WebSocket module (in progress on `origin/feat/phase-3-tts-ws-selfhosted`) |
| Flux TTS, batch | `POST /v2/speak` | `dg.text_to_speech().flux_speak_to_file(...)`, `flux_speak_to_stream(...)` | Shipped (`speak`) since 0.10.1 |
| Flux TTS, streaming | `wss /v2/speak` | `dg.text_to_speech().flux_request(options).handle()` for a `FluxSpeakHandle` (`speak`, `flush`, `interrupt`, `configure_speed`, `close`, `receive`); events arrive as `FluxSpeakResponse` | Shipped (`speak`) since 0.10.1 |
| Voice Agent | `wss agent.deepgram.com/v1/agent/converse` | none | Not shipped (in progress on `origin/feat/agent-websocket` and `origin/feat/phase-4-voice-agent`) |
| Text intelligence | `POST /v1/read` | none | Not shipped (in progress on `origin/feat/phase-2-read-stream-models`) |
| Management API | `/v1/projects/...` | `dg.projects()`, `dg.keys()`, `dg.members()`, `dg.scopes()`, `dg.invitations()`, `dg.usage()`, `dg.billing()` | Shipped (`manage`); no `models` endpoint |
| Self-hosted credentials | `/v1/projects/{id}/onprem/...` | none | Not shipped |
| Auth (grant token) | `POST /v1/auth/grant` | `dg.auth().grant(options)` | Shipped |

## Prerequisites

- Stable Rust. CI installs the current stable toolchain with `actions-rust-lang/setup-rust-toolchain@v1` plus the `rustfmt` component; `Cargo.toml` sets no `rust-version`. The Minimal-Versions job also installs nightly.
- ALSA development headers on Linux (`apt-get install -y pkg-config libasound2-dev`). The `cpal` and `rodio` dev-dependencies behind the microphone examples need them, and `--all-targets` compiles the examples, so `cargo build`, `cargo clippy`, and `cargo test` fail without them.
- `cargo-audit`, `cargo-hack`, and `cargo-semver-checks` for the three CI jobs that use them (`cargo install --locked <tool>`).

## Build, test, lint, format

CI runs every job on every push and pull request with `RUSTFLAGS=-D warnings` and `RUSTDOCFLAGS=-D warnings`, so a single warning fails the build. Export both before running the commands. The first seven rows were run on 2026-09-13 inside a `rust:latest` container (Rust 1.98.0) with the repository mounted at `/work` and the ALSA headers installed; the exit codes are recorded in the pull request that added this file.

| CI job | Command | Notes |
| --- | --- | --- |
| Format | `cargo fmt --check --all` | Run `cargo fmt --all` to fix |
| Features | `cargo check --all-targets --no-default-features`, then the same with `--features=listen`, `--features=speak`, `--features=manage`, `--features=connect-diagnostics` | Each feature must compile alone |
| Build | `cargo build --all-targets --all-features` | About 3 minutes from a cold cache |
| Clippy | `cargo clippy --all-targets --all-features` | Zero warnings at 0.10.1; `#![warn(clippy::cargo)]` is on in `lib.rs` |
| Test | `cargo test --all --all-features` | At 0.10.1: 105 unit tests pass, 2 are ignored, the `*_local.rs` integration tests pass, and the `*_e2e.rs` tests are ignored. No network access needed |
| Documentation | `cargo doc --workspace --all-features` | `missing_docs` is a warning and warnings are errors, so every public item needs a doc comment |
| Live tests | `DEEPGRAM_API_KEY=<key> cargo test --all-features --test flux_e2e -- --ignored` | Also `--test connect_diagnostics_e2e`; each `#[ignore]` reason names the requirement |
| Audit | `cargo install --locked cargo-audit cargo-hack && cargo hack --remove-dev-deps && cargo generate-lockfile && cargo audit` | Run in a throwaway checkout; it rewrites `Cargo.toml` and `Cargo.lock`. Not run for this file |
| Minimal-Versions | `rustup run nightly cargo build --all-targets --all-features -Z minimal-versions` and `rustup run nightly cargo test --all --all-features -Z minimal-versions` | Lower bounds in `Cargo.toml` must be real; the crates under "specified only to satisfy minimal-versions" exist for this job. Not run for this file |
| SemVer | `cargo install --locked cargo-semver-checks && cargo semver-checks check-release --verbose` | Fails a pull request that breaks the public API without a version bump. Not run for this file |

## Run an example against the live API

Example names are the `[[example]]` entries in `Cargo.toml`, and each one lists its `required-features`. Both commands below were run on 2026-09-13 in the same `rust:latest` container with the key passed through the environment; the first printed the transcript, the second wrote `flux-tts-batch.mp3` (39168 bytes) with `flux-haley-en`.

```bash
# Every example reads DEEPGRAM_API_KEY from the environment.
export DEEPGRAM_API_KEY="<your key>"

# Transcribe a hosted file with the default model and print the transcript.
cargo run --example prerecorded_from_url

# Synthesize a block of text with Flux TTS over REST and write flux-tts-batch.mp3.
cargo run --features speak --example flux_tts_batch

# Stream a Flux STT session from a local WAV file and print TurnInfo events.
FILENAME=./examples/audio/bueller-mono.wav cargo run --example simple_flux
```

The file-based examples read the path from `FILENAME`. The `microphone_stream` and `microphone_flux` examples capture audio with `cpal`, so they need an audio device and do not run in a container. Examples write output files into the current directory; delete them before you commit.

## Implementation conventions

- `src/lib.rs` sets `#![forbid(unsafe_code)]` and warns on `missing_docs`, `missing_debug_implementations`, and `clippy::cargo`. With `-D warnings` in CI, every public item needs a `///` doc comment and every type needs `Debug`.
- Public enums that mirror server values are `#[non_exhaustive]` and carry an `Unknown` variant, so a new server message or value never breaks a deployed client. `FluxResponse::Unknown` and `TurnTrigger::Unknown(String)` preserve the raw wire value; `TurnEvent::Unknown` identifies an unrecognized event. Do the same for any new response enum, and add a deserialization test in the module's `tests` block.
- Request options are builders: `Options::builder()` with chained setters for the shared query, `ConfigureRequest::new().with_thresholds(...)`, and `#[non_exhaustive]` structs with `new()` plus `with_*` methods. Options serialize with `serde_urlencoded`; multi-value parameters (for example `language_hint`) serialize as repeated keys.
- Errors are `DeepgramError` variants (`thiserror`). Validate what the server would reject anyway only when the failure would otherwise be confusing (the Flux TTS WebSocket rejects REST-only options up front with `DeepgramError::InvalidOptions`); leave everything else to the server.
- WebSocket clients run a worker task and hand the caller a handle. Sends after the session has ended return an error instead of silently dropping the message, and the worker forwards a terminal transport error exactly once. Keep that contract in any new streaming surface (`tests/flux_backpressure_local.rs` and `tests/flux_speak_backpressure_local.rs` show the pattern).
- Keep dependency lower bounds honest. If you call an API that a newer version of a dependency introduced, raise that dependency's minimum in `Cargo.toml`, or the Minimal-Versions job fails.
- A change to a public signature is a semver event. In `0.x`, a breaking change needs a minor bump and a `**BREAKING**` line in `CHANGELOG.md` (the 0.10.0 entry is the model); an additive change needs a patch or minor bump. `cargo-semver-checks` enforces this in CI.
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
- Do not remove the `[[example]]` entries or their `required-features`; `--all-targets` in CI compiles every example.
- Do not add `unsafe` code; `#![forbid(unsafe_code)]` rejects it.
- Do not change a public signature without a version bump and a `CHANGELOG.md` entry.
- Do not run `cargo hack --remove-dev-deps` in your working checkout; it rewrites `Cargo.toml`.
