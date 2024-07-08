//! Transcribe audio using Deepgram's automated speech recognition.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription

use crate::Deepgram;

#[cfg(feature = "live")]
pub mod live;
#[cfg(feature = "prerecorded")]
pub mod prerecorded;

/// Transcribe audio using Deepgram's automated speech recognition.
///
/// Constructed using [`Deepgram::transcription`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription
#[derive(Debug, Clone)]
pub struct Transcription<'a>(#[allow(unused)] &'a Deepgram);

impl Deepgram {
    /// Construct a new [`Transcription`] from a [`Deepgram`].
    pub fn transcription(&self) -> Transcription<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Transcription<'a> {
    /// Construct a new [`Transcription`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}
