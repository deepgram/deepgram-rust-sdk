#![warn(missing_docs)]

//! This module provides various types that are used for transcription requests to Deepgram.

use crate::Deepgram;

pub mod prerecorded;

/// Transcribe audio using Deepgram's automated speech recognition.
///
/// Constructed using [`Deepgram::transcription`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription
#[derive(Debug, Clone)]
pub struct Transcription<'a, K: AsRef<str>>(&'a Deepgram<K>);

impl<'a, K: AsRef<str>> Deepgram<K> {
    /// Construct a new [`Transcription`] from a [`Deepgram`].
    pub fn transcription(&'a self) -> Transcription<'a, K> {
        Transcription(self)
    }
}

impl<'a, K: AsRef<str>> From<&'a Deepgram<K>> for Transcription<'a, K> {
    /// Construct a new [`Transcription`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram<K>) -> Self {
        deepgram.transcription()
    }
}
