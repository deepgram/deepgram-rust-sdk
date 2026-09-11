# Deepgram Rust SDK

[![Discord](https://img.shields.io/badge/Discord-Deepgram-5865F2?logo=discord&logoColor=white&style=flat)](https://discord.gg/deepgram)
[![CI](https://github.com/deepgram/deepgram-rust-sdk/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/deepgram/deepgram-rust-sdk/actions/workflows/ci.yaml)
[![crates.io](https://img.shields.io/crates/v/deepgram)](https://crates.io/crates/deepgram)
[![downloads](https://img.shields.io/crates/d/deepgram)](https://crates.io/crates/deepgram)
[![docs](https://img.shields.io/docsrs/deepgram)](https://docs.rs/deepgram)
[![license](https://img.shields.io/crates/l/deepgram)](./LICENSE)

A Community Rust SDK for [Deepgram](https://www.deepgram.com/). Start building with our powerful transcription & speech understanding API.

## SDK Documentation

This SDK implements the Deepgram API found at [https://developers.deepgram.com](https://developers.deepgram.com).

Documentation and examples can be found on our [Docs.rs page](https://docs.rs/deepgram/latest/deepgram/).

## Quick Start

Check out the [examples folder](./examples/) for practical code examples showing how to use the SDK.

## Authentication

🔑 To access the Deepgram API you will need a [free Deepgram API Key](https://console.deepgram.com/signup?jump=keys).

There are two ways to authenticate with the Deepgram API:

1.  **API Key**: This is the simplest method. You can get a free API key from the
    [Deepgram Console](https://console.deepgram.com/signup?jump=keys).

    ```rust
    use deepgram::Deepgram;

    let dg = Deepgram::new("YOUR_DEEPGRAM_API_KEY");
    ```

2.  **Temporary Tokens**: If you are building an application where you need to
    grant temporary access to the Deepgram API, you can use temporary tokens.
    This is useful for client-side applications where you don't want to expose
    your API key.

    You can create temporary tokens using the Deepgram API. Learn more about
    [token-based authentication](https://developers.deepgram.com/guides/fundamentals/token-based-authentication).

    ```rust
    use deepgram::Deepgram;

    let dg = Deepgram::with_temp_token("YOUR_TEMPORARY_TOKEN");
    ```

## Current Status

This SDK is currently Community owned but is moving to a stable `1.0` version soon.

## Install

From within your Cargo project directory, run the following command:

```sh
cargo add deepgram
```

You will also probably need to install [`tokio`](https://crates.io/crates/tokio):

```sh
cargo add tokio --features full
```

### Cargo features

The product features (`listen`, `speak`, `read`, `manage`) are enabled by
default; `connect-diagnostics` and `rustls-tls-native-roots` are opt-in. To
trim dependencies, disable the defaults and pick only what you need:

```sh
cargo add deepgram --no-default-features --features read
```

| Feature  | Enables                                                                                  |
| -------- | ---------------------------------------------------------------------------------------- |
| `listen` | Speech-to-text: pre-recorded REST, live WebSocket streaming, Flux STT                    |
| `speak`  | Text-to-speech: Aura REST and streaming WebSocket, Flux TTS REST and streaming WebSocket |
| `read`   | Text Intelligence (`/v1/read`): sentiment, summary, topics, intents                      |
| `manage` | Management API: projects, keys, members, usage, billing, models                          |
| `agent`  | Voice Agent WebSocket client (`/v1/agent/converse`)                                      |

Token-based authentication (`auth`) is always available. The `listen`,
`speak`, and `agent` features pull in WebSocket dependencies; `read` and
`manage` are HTTP-only. The table lists the product features only: the two
opt-in features, `connect-diagnostics` and `rustls-tls-native-roots`, are
described below.

### Voice Agent

The `agent` feature's live Voice Agent client (`Deepgram::agent().start()`)
is demonstrated in [`examples/agent/websocket/`](examples/agent/websocket/):
`simple_agent` opens a session, applies `Settings`, and prints the event
stream; `function_calling` shows the client-side function-call round-trip.

For streaming text-to-speech, `dg.text_to_speech().speak_stream()` opens the
Aura socket (`wss /v1/speak`); see
[`examples/speak/websocket/`](./examples/speak/websocket/) for a runnable
example, and [`examples/speak/flux/`](./examples/speak/flux/) for the Flux TTS
equivalent.

## Connect Diagnostics

For diagnosing connection latency on live transcription (`/v1/listen`)
streaming requests, the optional `connect-diagnostics` feature emits one
structured record per WebSocket connect attempt, with per-phase timings
(DNS, TCP, TLS, WebSocket upgrade), socket addresses, and the Deepgram
request ID. Records are delivered even when the connect is cancelled by a
caller-side timeout. Other WebSocket surfaces (Flux, streaming TTS) are not
covered yet.

```sh
cargo add deepgram --features connect-diagnostics
```

```rust
use deepgram::{Deepgram, common::options::Options, diagnostics::ConnectRecord};

let dg = Deepgram::new("YOUR_DEEPGRAM_API_KEY")?;
let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
// Drain diag_rx to your JSONL sink of choice, then:
let builder = dg
    .transcription()
    .stream_request_with_options(Options::default())
    .diagnostics(diag_tx);
```

See the `connect_diagnostics` example and the `deepgram::diagnostics` module
docs for the record schema and integration details.

## TLS Trust (Corporate Proxies, Private CAs)

`wss://` WebSocket connections verify Deepgram's certificate against the
bundled public roots ([webpki-roots](https://crates.io/crates/webpki-roots)).
That is the default, and it needs no OS certificate store. The REST client
(`reqwest`) already trusts the operating system's store, so behind a
TLS-inspecting proxy the REST calls typically work while `wss://` connections
fail with `UntrustedTlsCertificate` until you enable one of the options below.

> **Only `wss://` is covered.** A client built from an `http://` base URL
> (`Deepgram::with_base_url("http://localhost:8080")`) opens plaintext
> `ws://` WebSockets: no TLS handshake, no certificate verification, and
> neither option below has any effect. Credentials and audio travel
> unencrypted. Keep `http://` to local testing and use an `https://` base URL
> whenever an API key, a temporary token, or private traffic is involved,
> including self-hosted deployments.

If your traffic goes through a TLS-inspecting proxy (Zscaler, Netskope, …),
an internal CA, or a self-hosted deployment, the certificate the SDK sees is
signed by a CA the public bundle does not know, and the connection fails
with `DeepgramError::UntrustedTlsCertificate`. Its message names the fix.
There are two:

**Also trust the operating system's certificate store.** One cargo feature,
no code change. The OS roots are added on top of the public roots, never
instead of them. Named after the `tokio-tungstenite` and `reqwest` features
it mirrors.

```sh
cargo add deepgram --features rustls-tls-native-roots
```

Install the proxy's or internal CA in the OS store the way your platform
does it. If you use `SSL_CERT_FILE` / `SSL_CERT_DIR` instead, point them at
a PEM bundle that holds the CA *together with* the public roots you rely on:
once set, the variables replace the OS store for these WebSockets and, on
Linux, for the REST client too, so a file containing only the CA breaks
requests to hosts that CA did not sign.

**Or supply your own `rustls` config.** Set it once on the client and every
`wss://` WebSocket it opens (live transcription, Flux speech-to-text,
streaming text-to-speech, Flux text-to-speech) uses it verbatim: pin a
private CA, present a client certificate, plug in a custom verifier. Build
it from `deepgram::rustls` so the versions match.

```rust
use deepgram::{rustls, Deepgram};

let mut roots = rustls::RootCertStore::empty();
roots.add(rustls::pki_types::CertificateDer::from(my_private_ca_der))?; // DER bytes of your CA certificate
let config = rustls::ClientConfig::builder()
    .with_root_certificates(roots)
    .with_no_client_auth();

let dg = Deepgram::new("YOUR_DEEPGRAM_API_KEY")?.tls_config(config);
```

If the feature is enabled but the OS store cannot be loaded (an
`SSL_CERT_FILE` that points at a missing or non-PEM file, a container with
no store), the client still works with the public roots, and a rejected
certificate then says the native roots could not be loaded rather than
claiming they were checked.

See the `deepgram::tls` module docs for details. REST requests are made with
`reqwest` and are not affected by either option.

## Development and Contributing

Interested in contributing? We ❤️ pull requests!

To make sure our community is safe for all, be sure to review and agree to our
[Code of Conduct](./CODE_OF_CONDUCT.md) and review our
[Contributing Guidelines](./CONTRIBUTING.md).

### Build the SDK

```sh
cargo build
```

## Getting Help

We love to hear from you so if you have questions, comments or find a bug in the
project, let us know! You can either:

- [Open an issue in this repository](https://github.com/deepgram/deepgram-rust-sdk/issues/new)
- [Join the Deepgram Github Discussions Community](https://github.com/orgs/deepgram/discussions)
- [Join the Deepgram Discord Community](https://discord.gg/xWRaCDBtW4)

[license]: LICENSE.txt
