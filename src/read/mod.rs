//! Analyze text with Deepgram's Text Intelligence (`/v1/read`) API.
//!
//! Text Intelligence applies the same sentiment, summarization, topic, and
//! intent analyses as [Audio Intelligence][audio], but to text you already
//! have (a transcript, document, chat log, or email) rather than to audio.
//!
//! Construct a [`Read`] with [`Deepgram::text_intelligence`].
//!
//! See the [Deepgram Text Intelligence docs][docs] for more info.
//!
//! [docs]: https://developers.deepgram.com/docs/text-intelligence
//! [audio]: https://developers.deepgram.com/docs/audio-intelligence

use crate::Deepgram;

pub mod options;
pub mod response;
pub mod rest;

/// Analyze text using Deepgram's Text Intelligence API.
///
/// Constructed using [`Deepgram::text_intelligence`].
///
/// See the [Deepgram Text Intelligence docs][docs] for more info.
///
/// [docs]: https://developers.deepgram.com/docs/text-intelligence
#[derive(Debug, Clone)]
pub struct Read<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Read`] from a [`Deepgram`].
    pub fn text_intelligence(&self) -> Read<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Read<'a> {
    /// Construct a new [`Read`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}
