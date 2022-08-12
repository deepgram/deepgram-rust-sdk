//! Deepgram live transcription API response types.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-streaming-responses

use serde::Deserialize;
use uuid::Uuid;

pub use super::super::common::response::{
    ChannelResult, Hit, ListenMetadata, ResultAlternative, SearchResults, Word,
};

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(untagged)]
pub enum Response {
    Results(ListenResults),
    Metadata(ListenMetadata),
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ListenResults {
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

// {
// "transaction_key":"deprecated",
// "request_id":"62359324-2d6a-4214-bb9d-947764e9e905",
// "sha256":"295d80fc68d1eaf980ad07f586f62a673b26a5d3aa057e7995abb42189e5207a",
// "created":"2022-08-12T03:25:14.333Z",
// "duration":17.632626,
// "channels":1
//}
