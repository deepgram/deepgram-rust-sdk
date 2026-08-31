//! Flux text-to-speech (`/v2/speak`): streaming, turn-based TTS built
//! for voice-agent pipelines, plus a batch (REST) transport for
//! pre-rendering fixed audio.
//!
//! - Batch (REST): [`Speak::flux_speak_to_file`](crate::Speak::flux_speak_to_file)
//!   and [`Speak::flux_speak_to_stream`](crate::Speak::flux_speak_to_stream)
//! - Streaming (WebSocket): [`Speak::flux_request`](crate::Speak::flux_request)
//!
//! See the [Deepgram Flux TTS docs][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/docs/text-to-speech/flux/quickstart

pub mod options;
pub mod response;
pub mod rest;
pub mod websocket;
