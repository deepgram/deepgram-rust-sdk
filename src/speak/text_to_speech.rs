//! Transcribe audio using Deepgram's automated speech recognition.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription

use crate::Deepgram;

/// Generate speech from text using Deepgram's text to speech api.
///
/// Constructed using [`Deepgram::text_to_speech`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/reference/text-to-speech-api
#[derive(Debug, Clone)]
pub struct Speak<'a>(#[allow(unused)] pub &'a Deepgram);

impl Deepgram {
    /// Construct a new [`Speak`] from a [`Deepgram`].
    pub fn text_to_speech(&self) -> Speak<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Speak<'a> {
    /// Construct a new [`Speak`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}
