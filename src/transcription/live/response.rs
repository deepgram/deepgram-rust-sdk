//! Deepgram live transcription API response types.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-streaming-responses

use serde::Deserialize;
use uuid::Uuid;

pub use super::super::common::response::{
    ChannelResult, Hit, ResultAlternative, SearchResults, Word,
};

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Response {
    pub channel_index: (usize, usize),
    pub duration: f64,
    pub start: f64,
    pub is_final: bool,
    pub speech_final: bool,
    pub channel: ChannelResult,
    pub metadata: Metadata,
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Metadata {
    pub request_id: Uuid,
}
