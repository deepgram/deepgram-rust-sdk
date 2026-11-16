# Streaming Runtime and Transport Parity Audit

Status: decision recorded — no SDK-managed reconnect or message replay.

This record covers streaming behavior outside the pinned OpenAPI and
AsyncAPI contracts. It is the Phase 3 evidence for the Rust SDK full-parity
plan; it does not add a reconnect or custom-transport API.

## Evidence baseline

| Source | Revision |
| --- | --- |
| Deepgram API specs | `2677eb4ddbf533b700725bb71a46633e858b8466` |
| JavaScript SDK | `62482ff46b996d5c9733fde07ba23078271d28f6` |
| Python SDK | `5a420ce3abd416255c04acff504522398e8e5f1c` |
| Java SDK | `c25deb966fd37b2828d1672955dd3a349cb7a61f` |
| Rust audit baseline | `e42ef37a` |

The streaming surfaces are Listen V1 (`/v1/listen`), Listen V2 / Flux STT
(`/v2/listen`), Speak V1 / Aura (`/v1/speak`), Speak V2 / Flux TTS
(`/v2/speak`), and Voice Agent (`agent.deepgram.com/v1/agent/converse`).

## Capability matrix

| Capability | JavaScript | Python | Java | Rust current behavior | Disposition |
| --- | --- | --- | --- | --- | --- |
| SDK-managed reconnect | Yes. The wrapper reconnects native WebSockets; each connection defaults to 30 retries. | No. `reconnect` is declarative only. | Yes. `ReconnectingWebSocketListener` reconnects with exponential backoff; its default maximum is effectively unbounded. | No. A read/write failure or abnormal close is terminal. | Intentional Rust-native difference. |
| Sending while disconnected | JavaScript queues messages and flushes them after opening. | Not applicable; no wrapper reconnect. | Queues up to `maxEnqueuedMessages`, then drops later messages. | The bounded command channel exists only for a live worker. When the worker exits, retained handle sends fail. | Do not copy without delivery semantics. |
| Custom streaming transport | `transportFactory(url, headers, request)` supports all five surfaces. It owns reconnect by default. | Process-global monkey-patch accepts `factory(url, headers)` for all generated streaming clients. | `DeepgramTransportFactory.create(url, headers)` supports all streaming clients. | No replacement transport seam in this audit's Rust baseline; every surface uses `tokio_tungstenite`. | Separate implementation work; no wrapper retry if added. |
| Close/retry policy | Default does not reconnect after a normal close. Per-connection policy can override this; protocol terminal messages receive special handling. | Native transport behavior only. | Normal close is terminal by default; subclasses can override the policy. | Explicit close is terminal. A non-normal close or read/write error is surfaced once and ends the handle/event stream. | Intentional Rust-native terminal contract. |
| Backpressure / terminal error | Wrapper queues while disconnected; queue ownership is in the reconnect wrapper. | Delegated to the installed transport. | Queues outbound messages across reconnects. | Every worker uses bounded channels; a terminal transport failure closes the command channel so later sends fail rather than disappear. | Preserve this invariant in any transport abstraction. |
| Per-surface custom endpoint | Base URL/environment controls all wrapped endpoints. | Generated client base URL controls all endpoints. | Endpoint configuration is client-level. | `Deepgram::with_base_url*` controls Listen V1/V2 and Speak V1/V2. Voice Agent uses its SaaS endpoint or explicit `Agent::start_at_url`; non-loopback Agent URLs require `wss://`. | Rust-equivalent, intentionally explicit for Voice Agent. |
| Callback lifecycle | WebSocket callbacks plus async iteration. | Sync/async receive and iteration. | Listener callbacks. | Futures handles and streams; no callback API. | Intentional Rust-native API shape. |

## Reconnect is not session recovery

The existing JavaScript and Java reconnect wrappers reconnect a transport and
may queue messages that were submitted while disconnected. They do not
document a protocol-level resume operation, acknowledgement-based replay, or
Voice Agent state restoration.

That distinction is material for every Rust streaming API:

- **Voice Agent:** a new socket has no documented guarantee that it resumes
  the prior agent configuration, conversation history, in-progress function
  call, current user turn, or synthesized audio. Replaying `Settings` may
  create a new conversation; replaying audio or function results can duplicate
  side effects.
- **Listen V1 and Flux STT:** audio submitted before a failure has no
  acknowledgement that permits lossless or exactly-once replay. Continuing
  with subsequent audio changes transcript and turn boundaries.
- **Aura and Flux TTS:** the client cannot tell whether text sent before a
  disconnect was accepted, so automatic replay can duplicate audio. A new
  connection also has a new request ID and distinct session-level metadata.

Therefore, a successful TCP/WebSocket reconnection must not be described as
session recovery unless the service defines resume, state, ordering, and
at-least-once/exactly-once semantics.

## Rust behavior verified by this audit

All five Rust streaming workers use bounded application channels and terminate
their command channels when the peer closes or transport fails. Consequently,
a retained handle rejects later sends instead of accepting data that no worker
can deliver. Workers forward terminal read/write failures once and then close
the response stream.

The existing local backpressure tests cover that lifecycle invariant for the
native WebSocket workers. Any replacement transport must preserve it:

1. outbound messages retain their order while a connection is live;
2. backpressure on responses must not silently discard outbound commands;
3. an ended transport rejects new sends promptly;
4. one terminal error is delivered at most once; and
5. an explicit protocol close never triggers an automatic new session.

## Recorded product contract

1. **Rust does not perform SDK-managed reconnect on any streaming surface.**
   The current terminal-error contract is safe, observable, and does not
   imply a delivery guarantee the protocol has not made.
2. **Rust does not replay a client message or audio after a transport
   failure.** Handles report one terminal error and later sends fail.
3. **Do not add transparent Voice Agent reconnect** until the API defines a
   supported session-resumption protocol. A later contract must specify how
   settings, history, turns, pending function calls, audio, and event ordering
   behave after recovery.
4. **Any future custom transport may reconnect internally.** The SDK must not
   wrap it in a second retry loop. A future factory should receive the
   resolved URL and headers and return a bidirectional
   text/binary/ping/pong/close transport.
5. **Keep retry ownership singular.** A custom transport owns its own retry
   lifecycle unless a later, explicit Rust reconnect policy opts into wrapper
   retries. Stacking both by default risks retry storms.

## Future reconnect prerequisites

The recorded policy closes Phase 3's current runtime-parity decision. Before
changing it to add SDK-managed reconnect, product must approve all of the
following:

1. Whether each service is allowed to start a new session after a transport
   failure, and whether that action is opt-in.
2. Whether the SDK may resend any client message or audio after an
   unacknowledged send.
3. The observable recovery signal and callback/stream ordering at a reconnect
   boundary.
4. Retry eligibility by failure and close code, attempt/time limits, backoff,
   and queue-overflow behavior.
5. Voice Agent recovery semantics, including whether and how settings,
   history, function calls, and active turns may be rebuilt.

Until then, Rust's terminal transport behavior is the intentional, safer
difference.
