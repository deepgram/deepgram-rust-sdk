use reqwest::{header::CONTENT_TYPE, RequestBuilder};
use serde::Serialize;
use std::borrow::Borrow;

pub trait AudioSource {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder;
}

/// Used as a parameter for [`Deepgram::prerecorded_request`](crate::Deepgram::prerecorded_request) and similar functions.
///
/// Instructs Deepgram to download the audio from the specified URL.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize)]
pub struct UrlSource<'a> {
    /// URL of audio file to transcribe.
    pub url: &'a str,
}

/// Used as a parameter for [`Deepgram::prerecorded_request`](crate::Deepgram::prerecorded_request) and similar functions.
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

    /// Optionally specify the [MIME type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types#audio_and_video_types)
    /// of the raw binary audio data.
    pub mimetype: Option<&'a str>,
}

impl<'a, B: Borrow<UrlSource<'a>>> AudioSource for B {
    fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        request_builder.json(self.borrow())
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
