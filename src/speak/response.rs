//! Response metadata for Text-to-Speech (TTS) REST requests.

use reqwest::header::HeaderMap;

/// Metadata returned alongside a Text-to-Speech REST response.
///
/// Deepgram returns useful information in the response headers of a `/v1/speak`
/// request — most notably the `dg-request-id`, which is required to debug or
/// track a request. The audio-producing methods
/// ([`Speak::speak_to_file`](crate::Speak::speak_to_file) and
/// [`Speak::speak_to_stream`](crate::Speak::speak_to_stream)) do not surface
/// these headers; use
/// [`Speak::speak_to_file_with_metadata`](crate::Speak::speak_to_file_with_metadata)
/// or
/// [`Speak::speak_to_stream_with_metadata`](crate::Speak::speak_to_stream_with_metadata)
/// to obtain them.
///
/// Every field is optional because a header may be absent (for example, on an
/// error response or from a self-hosted instance).
///
/// See the [Deepgram Text-to-Speech docs][docs] for the full list of response
/// headers.
///
/// [docs]: https://developers.deepgram.com/docs/text-to-speech#results
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct SpeakMetadata {
    /// A unique identifier for the request (`dg-request-id`), useful for
    /// debugging and tracking.
    pub request_id: Option<String>,

    /// The name of the model used to process the request (`dg-model-name`).
    pub model_name: Option<String>,

    /// The unique identifier of the model that processed the request
    /// (`dg-model-uuid`).
    pub model_uuid: Option<String>,

    /// The number of characters in the input text (`dg-char-count`).
    pub char_count: Option<u32>,

    /// The media type of the returned audio (`content-type`), e.g.
    /// `audio/mpeg`.
    pub content_type: Option<String>,

    /// The transfer encoding used for the response body (`transfer-encoding`).
    pub transfer_encoding: Option<String>,

    /// The date and time the response was sent (`date`).
    pub date: Option<String>,
}

impl SpeakMetadata {
    /// Extract the Deepgram TTS metadata headers from a response's
    /// [`HeaderMap`].
    pub(crate) fn from_headers(headers: &HeaderMap) -> Self {
        let get = |name: &str| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        };

        SpeakMetadata {
            request_id: get("dg-request-id"),
            model_name: get("dg-model-name"),
            model_uuid: get("dg-model-uuid"),
            char_count: get("dg-char-count").and_then(|value| value.parse().ok()),
            content_type: get("content-type"),
            transfer_encoding: get("transfer-encoding"),
            date: get("date"),
        }
    }

    /// The unique identifier for the request (`dg-request-id`), if present.
    ///
    /// This is a convenience accessor for [`SpeakMetadata::request_id`].
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
}
