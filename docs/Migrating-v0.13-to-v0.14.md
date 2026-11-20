# v0.13 to v0.14 Migration Guide

This guide covers the breaking API change in Deepgram Rust SDK `0.14.0`.

## Update the dependency

Upgrade the crate to the latest `0.14` release:

```sh
cargo add deepgram@0.14
```

Or update your `Cargo.toml` directly:

```toml
[dependencies]
deepgram = "0.14"
```

Then update the lockfile:

```sh
cargo update -p deepgram
```

## Handle an absent WebSocket request ID

WebSocket upgrade responses can omit the `dg-request-id` header. In `0.14.0`,
the following accessors therefore return `Option<Uuid>` instead of `Uuid`:

- `WebsocketHandle::request_id`
- `TranscriptionStream::request_id`
- `FluxHandle::request_id`
- `FluxStream::request_id`
- `FluxSpeakHandle::request_id`

Update code that assumed every connection had an ID. For example, replace:

```rust
let request_id = handle.request_id();
println!("Deepgram request ID: {request_id}");
```

with explicit absent-value handling:

```rust
match handle.request_id() {
    Some(request_id) => println!("Deepgram request ID: {request_id}"),
    None => eprintln!("The WebSocket upgrade did not include dg-request-id"),
}
```

`None` means only that the server omitted the header. A present
`dg-request-id` header that is not a UUID remains an
`DeepgramError::UnexpectedServerResponse` error.
