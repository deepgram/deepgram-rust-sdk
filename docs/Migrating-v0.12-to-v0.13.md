# v0.12 to v0.13 Migration Guide

This guide covers the breaking source and behavior changes in Deepgram Rust SDK
`0.13.0`. The release adds more complete response metadata and routes all
configured-base-URL requests consistently, which changes a few existing
integration assumptions.

## Update the dependency

Upgrade the crate to the latest `0.13` release:

```sh
cargo add deepgram@0.13
```

Or update your `Cargo.toml` directly:

```toml
[dependencies]
deepgram = "0.13"
```

Then update the lockfile:

```sh
cargo update -p deepgram
```

## Handle absent usage meter fields

The Management API can omit the legacy audio meter group for requests that do
not meter audio there, including Voice Agent, text-to-speech, and Text
Intelligence requests. Previously, the SDK decoded these absent values as zero.
`Details::duration` and `Details::total_audio` are now `Option<f64>`;
`Details::channels` and `Details::streams` are now `Option<usize>`.

Do not default an absent value to zero unless that matches your accounting
policy. `None` means that meter group is absent, not that the request consumed
no audio.

Replace direct reads such as:

```rust
let audio_seconds = response.details.duration;
```

with deliberate absent-value handling:

```rust
match response.details.duration {
    Some(audio_seconds) => record_audio_usage(audio_seconds),
    None => record_usage_from_the_operation_specific_meter(&response),
}
```

For `/v1/listen` and `/v2/listen`, an actual zero-audio request is still
reported as `Some(0.0)`.

## Use a separate hosted client for hosted management and auth calls

`Deepgram::with_base_url*` now sends Management API and `Auth::grant` requests
to the configured base URL. Before 0.13.0, those calls always targeted the
hosted API, even when the client sent transcription or speech requests to a
self-hosted deployment or gateway.

If your integration intentionally uses a custom host for audio APIs but the
hosted Deepgram API for management or temporary-token grants, create two
clients:

```rust
use deepgram::Deepgram;

let self_hosted = Deepgram::with_base_url_and_api_key(
    "https://deepgram.internal/",
    "SELF_HOSTED_API_KEY",
)?;

let hosted = Deepgram::new("HOSTED_DEEPGRAM_API_KEY")?;

// Use the custom-host client for its intended audio APIs.
let _ = self_hosted.transcription();

// Use the default-hosted client for management and token grants.
let projects = hosted.projects().list().await?;
```

If your custom base URL has a path prefix, it must end in `/`. URL resolution
otherwise replaces its last segment:

```rust
// Keeps the `deepgram` prefix.
let client = Deepgram::with_base_url_and_api_key(
    "https://gateway.internal/deepgram/",
    "API_KEY",
)?;

// Does not keep the `deepgram` prefix when endpoint paths are joined.
let incorrect = Deepgram::with_base_url_and_api_key(
    "https://gateway.internal/deepgram",
    "API_KEY",
)?;
```

## Update response patterns and fixtures for non-exhaustive types

Streaming `Metadata` and `StreamResponse::TerminalResponse`, plus
pre-recorded `ListenMetadata`, are now `#[non_exhaustive]`. This allows future
SDK releases to add server metadata without another breaking change.

External code can no longer construct these types with struct literals. For
test fixtures, deserialize a representative API payload instead, or construct
your own application type at the boundary.

Add `..` to patterns that destructure these types. For example, replace:

```rust
match response {
    StreamResponse::TerminalResponse {
        request_id,
        created,
        duration,
        channels,
    } => finish(request_id, created, duration, channels),
    _ => {}
}
```

with:

```rust
match response {
    StreamResponse::TerminalResponse {
        request_id,
        created,
        duration,
        channels,
        ..
    } => finish(request_id, created, duration, channels),
    _ => {}
}
```

The same change applies to `Metadata { ... }` and `ListenMetadata { ... }`
patterns. The newly exposed `extra` metadata is optional, so code that reads it
must also handle `None`.

## Match named models and redaction values

`Model::from(String)` and `Redact::from(String)` now return dedicated variants
for values the SDK knows about, including Whisper model IDs and named redaction
groups. Code that used `CustomId` or `Other` to recognize those known values
must match the dedicated variants instead.

For example, replace:

```rust
match Model::from("whisper-tiny".to_owned()) {
    Model::CustomId(name) if name == "whisper-tiny" => use_whisper_tiny(),
    model => use_model(model),
}
```

with:

```rust
match Model::from("whisper-tiny".to_owned()) {
    Model::WhisperTiny => use_whisper_tiny(),
    model => use_model(model),
}
```

When your code needs to compare a wire value rather than branch on SDK-known
variants, compare the value returned by `AsRef<str>`:

```rust
let model = Model::from(model_id);
if <Model as AsRef<str>>::as_ref(&model) == "whisper-tiny" {
    use_whisper_tiny();
}
```

Continue to use `Model::CustomId` and `Redact::Other` for API values added
after the SDK release.

## Print application output after `speak_to_file`

`Speak::speak_to_file` no longer prints a success message to stdout. This
removes a library side effect that could corrupt command output or structured
logs. Print any application-level confirmation yourself:

```rust
dg.text_to_speech()
    .speak_to_file("Hello", &options, output_file)
    .await?;

println!("Wrote audio to {}", output_file.display());
```
