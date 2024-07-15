//! Transcribe audio using Deepgram's automated speech recognition.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription

use crate::Deepgram;

pub mod live;
pub mod prerecorded;

pub mod common_options;
pub mod audio_source;

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
