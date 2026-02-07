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
