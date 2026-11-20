# v0.14 to v0.15 Migration Guide

This guide covers the behavioral changes in Deepgram Rust SDK `0.15.0`.
Most additions in this release, including Management API coverage, Voice Agent
REST management, new Voice Agent models, and the custom streaming transport
factory, are additive.

## Update the dependency

Upgrade the crate to the latest `0.15` release:

```sh
cargo add deepgram@0.15
```

Or update your `Cargo.toml` directly:

```toml
[dependencies]
deepgram = "0.15"
```

Then update the lockfile:

```sh
cargo update -p deepgram
```

## Handle terminal Listen V1 errors

Listen V1 now forwards an abnormal peer WebSocket close to `receive()` as
`DeepgramError::WebsocketClose`. Earlier versions could end the stream without
delivering that error because the worker task discarded it.

Continue to treat a normal close (code 1000) as clean end-of-stream. For every
other terminal condition, handle the error item before the stream ends:

```rust
while let Some(item) = handle.receive().await {
    match item {
        Ok(response) => process_response(response),
        Err(DeepgramError::WebsocketClose { code, reason }) => {
            report_transport_close(code, &reason);
        }
        Err(error) => return Err(error),
    }
}
```

The SDK also ends the session with one error when it cannot write an automatic
Pong reply. Applications should already handle `Err` items from `receive()` as
terminal transport failures rather than assuming every item is a response.

## Handle typed custom Think-provider events

`__customFromThinkProvider` events now deserialize as
`AgentResponse::CustomFromThinkProvider` instead of
`AgentResponse::Unknown`. If an application previously inspected the raw
`Unknown` payload to handle custom Think-provider traffic, add the typed
variant:

```rust
match response {
    AgentResponse::CustomFromThinkProvider(event) => {
        forward_to_application(event.content);
    }
    AgentResponse::Unknown(raw) => {
        preserve_an_unrecognized_server_event(raw);
    }
    _ => {}
}
```

A custom Think-provider frame that omits required `content` now returns a
deserialization error instead of being preserved as `Unknown`. Treat that error
as an invalid provider frame and correct the provider payload.

## Review custom transport ownership

`Deepgram::with_streaming_transport_factory` is additive. A factory owns its
TLS configuration, connection handshake, and any reconnect policy. The SDK
does not wrap a factory-provided transport in another retry loop or replay
messages after failure.

See the runnable
[`in_memory_transport_factory`](../examples/transport/in_memory_factory.rs)
example when implementing a factory.
