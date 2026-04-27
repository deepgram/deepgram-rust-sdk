---
name: deepgram-rust-maintaining-sdk
description: Use when maintaining the Deepgram Rust SDK itself: feature flags, examples, tests, cargo fmt/clippy/build/test, CHANGELOG updates, crate releases, and adding new endpoints without Fern workflows.
---

# Maintaining Deepgram Rust SDK

Use this skill when changing the SDK itself rather than consuming it.

## When to use this skill

- Adding or updating API surfaces under `src/`.
- Changing feature flags, examples, tests, or docs.
- Preparing a release to crates.io.
- Auditing the repo's current hand-maintained conventions.

## Authentication

Not applicable for repository maintenance.

## Quick start

## Quick start: local verification loop

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features
cargo test --all-features
```

## Quick start: feature-aware development

- Default features are `manage`, `listen`, and `speak`.
- `listen` pulls in WebSocket dependencies.
- If you add a module, decide whether it belongs behind a Cargo feature and wire examples accordingly in `Cargo.toml`.

## Key parameters

- Core files:
  - `Cargo.toml`
  - `src/lib.rs`
  - `src/`
  - `examples/`
  - `tests/`
  - `CHANGELOG.md`
  - `CONTRIBUTING.md`
- Current quality gates:
  - `cargo fmt`
  - `cargo clippy`
  - `cargo build`
  - `cargo test`
- Contribution rules from `CONTRIBUTING.md`:
  - PRs should target `dev`, not `main`
  - tests must be complete and pass
  - commit messages must be descriptive
  - include a test for bug fixes

## API reference (layered)

1. **In-repo**
   - `README.md`
   - `CONTRIBUTING.md`
   - `CHANGELOG.md`
   - `Cargo.toml`
   - `src/lib.rs`
   - `examples/README.md`
2. **OpenAPI**
   - `https://developers.deepgram.com/openapi.yaml`
3. **AsyncAPI**
   - `https://developers.deepgram.com/asyncapi.yaml`
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/reference/`
   - `https://docs.rs/deepgram/latest/deepgram/`

## Gotchas

1. **This SDK is not Fern-generated.** Do not describe or assume any Fern regeneration workflow.
2. **Match existing hand-maintained module patterns.** Add new product surfaces with explicit modules, option structs, response structs, examples, and feature gating where appropriate.
3. **Update examples and tests with new APIs.** This repo treats example programs and integration tests as part of the public developer experience.
4. **Keep `CHANGELOG.md` honest.** Follow the existing Keep a Changelog + SemVer style already used in the repo.
5. **Release flow is branch-sensitive.** `main` is release-oriented; normal contribution PRs target `dev`.
6. **If you publish a release, verify crate metadata first.** Ensure `version`, features, examples, and changelog all line up before `cargo publish`.

## Example files in this repo

- `examples/README.md`
- `examples/transcription/`
- `examples/speak/rest/`
- `examples/manage/`
- `tests/flux_unknown_messages.rs`
- `tests/flux_e2e.rs`
