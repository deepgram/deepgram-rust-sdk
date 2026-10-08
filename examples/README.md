## Running Examples

### Configuration

```sh
export DEEPGRAM_API_KEY="<your key>"
```

Examples that read local audio use a path defined in the example source. For Flux STT, change `PATH_TO_FILE` in `transcription/flux/simple_flux.rs`.

### Running the examples

```sh
cargo run --example prerecorded_from_url
```

```sh
cargo run --example simple_stream
```

```sh
cargo run --example callback
```

```sh
cargo run --example make_prerecorded_request_builder
```

```sh
cargo run --example microphone_stream
```

```sh
cargo run --example simple_flux
```

```sh
cargo run --example microphone_flux
```

```sh
cargo run --example text_to_speech_to_file
```

```sh
cargo run --example text_to_speech_to_stream
```

```sh
cargo run --example text_to_speech_websocket
```

```sh
DEEPGRAM_PROJECT_ID=your-project cargo run --example self_hosted_credentials
```
