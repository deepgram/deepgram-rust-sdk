# Listen / transcription parity audit

This first product audit is pinned to the revisions in `PARITY_LEDGER.yml`.
It records only findings verified against the current AsyncAPI operation
inventory and the Rust surface; every other Listen row remains `untriaged` in
`operation-audit-ledger.json`.

## Verified operation coverage

- `listen.v1.media.transcribe` is available through
  `Transcription::prerecorded`, `prerecorded_callback`, and their request
  builders in `src/listen/rest.rs`.
- Listen V1 media, `Finalize`, `CloseStream`, and `KeepAlive` are available
  through `WebsocketHandle::send_data`, `finalize`, `close_stream`, and
  `keep_alive` in `src/listen/websocket.rs`.
- Flux STT's documented operations are received and sent through `FluxHandle`,
  `ConfigureRequest`, and `FluxResponse` in `src/listen/flux.rs` and
  `src/common/flux_response.rs`; their detailed row-level schema audit remains
  pending.

## Missing Listen V1 Configure surface

The AsyncAPI operation inventory declares mid-stream `ListenV1Configure` plus
`ListenV1ConfigureSuccess` and `ListenV1ConfigureFailure`. Rust currently has
no typed sender or typed response variants. The operation ledger classifies all
three rows as `missing_rust_implementation` and supplies the wire-fixture plan
for the follow-up implementation PR.

Do not mark the Listen workstream complete from this audit: query-option and
response-field parity still require the row-level schema sweep.
