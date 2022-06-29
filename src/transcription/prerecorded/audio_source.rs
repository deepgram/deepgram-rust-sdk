//! Sources of audio that can be transcribed.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded

use reqwest::{header::CONTENT_TYPE, RequestBuilder};
use serde::Serialize;

/// Used as a parameter for [`Transcription::prerecorded`](crate::transcription::Transcription::prerecorded) and similar functions.
///
/// This trait is [sealed], and thus cannot be implemented by anything new.
///
/// [sealed]: https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed
pub trait AudioSource: private::Sealed {
    #[doc(hidden)]
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder;
}

/// Used to prevent other crates from implementing AudioSource
/// See <https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed>
mod private {
    use super::{BufferSource, UrlSource};

    pub trait Sealed {}

    impl Sealed for &UrlSource<'_> {}
    impl<B: Into<reqwest::Body>> Sealed for BufferSource<'_, B> {}
}

/// Used as a parameter for [`Transcription::prerecorded`](crate::transcription::Transcription::prerecorded) and similar functions.
///
/// Instructs Deepgram to download the audio from the specified URL.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize)]
pub struct UrlSource<'a> {
    /// URL of audio file to transcribe.
    pub url: &'a str,
}

/// Used as a parameter for [`Transcription::prerecorded`](crate::transcription::Transcription::prerecorded) and similar functions.
///
/// Uploads the raw binary audio data to Deepgram as part of the request.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct BufferSource<'a, B: Into<reqwest::Body>> {
    /// The source of the raw binary audio data, such as a [`tokio::fs::File`].
    ///
    /// It can be any type that implements [`Into<reqwest::Body>`].
    /// See [trait implementations for `reqwest::Body`](reqwest::Body#trait-implementations)
    /// for a list of types that already implement it.
    pub buffer: B,

    /// Optionally specify the [MIME type][mime] of the raw binary audio data.
    ///
    /// [mime]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types#audio_and_video_types
    pub mimetype: Option<&'a str>,
}

impl AudioSource for &UrlSource<'_> {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        request_builder.json(self)
    }
}

impl<B: Into<reqwest::Body>> AudioSource for BufferSource<'_, B> {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        let request_builder = request_builder.body(self.buffer);

        if let Some(mimetype) = self.mimetype {
            request_builder.header(CONTENT_TYPE, mimetype)
        } else {
            request_builder
        }
    }
}
