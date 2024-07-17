//! Sources of audio that can be transcribed.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#transcription-prerecorded

use reqwest::{header::CONTENT_TYPE, RequestBuilder};
use serde::Serialize;

/// Used as a parameter for [`Transcription::prerecorded`](crate::transcription::Transcription::prerecorded) and similar functions.
#[derive(Debug)]
pub struct AudioSource(InternalAudioSource);

#[derive(Debug)]
enum InternalAudioSource {
    Url(String),
    Buffer {
        buffer: reqwest::Body,
        mime_type: Option<String>,
    },
}

impl AudioSource {
    /// Constructs an [`AudioSource`] that will instruct Deepgram to download the audio from the specified URL.
    pub fn from_url(url: impl Into<String>) -> Self {
        Self(InternalAudioSource::Url(url.into()))
    }

    /// Constructs an [`AudioSource`] that will upload the raw binary audio data to Deepgram as part of the request.
    ///
    /// The buffer can be any type that implements [`Into<reqwest::Body>`], such as a [`tokio::fs::File`].
    /// See [trait implementations for `reqwest::Body`](reqwest::Body#trait-implementations)
    /// for a list of types that already implement it.
    ///
    /// Use [`AudioSource::from_buffer_with_mime_type`] if you want to specify a [MIME type][mime].
    ///
    /// [mime]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types#audio_and_video_types
    pub fn from_buffer(buffer: impl Into<reqwest::Body>) -> Self {
        Self(InternalAudioSource::Buffer {
            buffer: buffer.into(),
            mime_type: None,
        })
    }

    /// Same as [`AudioSource::from_buffer`], but allows you to specify a [MIME type][mime].
    ///
    /// [mime]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types#audio_and_video_types
    pub fn from_buffer_with_mime_type(
        buffer: impl Into<reqwest::Body>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self(InternalAudioSource::Buffer {
            buffer: buffer.into(),
            mime_type: Some(mime_type.into()),
        })
    }

    /// Fill body
    pub fn fill_body(self, request_builder: RequestBuilder) -> RequestBuilder {
        match self.0 {
            InternalAudioSource::Url(url) => {
                #[derive(Serialize)]
                struct UrlSource {
                    url: String,
                }

                request_builder.json(&UrlSource { url })
            }
            InternalAudioSource::Buffer { buffer, mime_type } => {
                let request_builder = request_builder.body(buffer);

                if let Some(mime_type) = mime_type {
                    request_builder.header(CONTENT_TYPE, mime_type)
                } else {
                    request_builder
                }
            }
        }
    }
}
