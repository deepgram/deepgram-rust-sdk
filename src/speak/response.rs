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

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue};

    use super::SpeakMetadata;

    /// Reads one `SpeakMetadata` field, rendered as a string so numeric fields
    /// fit the same table as the textual ones.
    type FieldAccessor = fn(&SpeakMetadata) -> Option<String>;

    /// One row per documented TTS response header: the wire header name, a
    /// sample value, and the `SpeakMetadata` field it must populate.
    type HeaderCase = (&'static str, &'static str, FieldAccessor);

    const HEADER_CASES: &[HeaderCase] = &[
        (
            "dg-request-id",
            "550e8400-e29b-41d4-a716-446655440000",
            |m| m.request_id.clone(),
        ),
        ("dg-model-name", "aura-2-thalia-en", |m| {
            m.model_name.clone()
        }),
        (
            "dg-model-uuid",
            "1e9d0f4a-6b0e-4a3c-9a1b-2c3d4e5f6a7b",
            |m| m.model_uuid.clone(),
        ),
        ("dg-char-count", "42", |m| {
            m.char_count.map(|c| c.to_string())
        }),
        ("content-type", "audio/mpeg", |m| m.content_type.clone()),
        ("transfer-encoding", "chunked", |m| {
            m.transfer_encoding.clone()
        }),
        ("date", "Thu, 11 Sep 2026 10:00:00 GMT", |m| m.date.clone()),
    ];

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(*name, HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    #[test]
    fn each_header_maps_to_exactly_one_field() {
        for (name, value, field) in HEADER_CASES {
            let metadata = SpeakMetadata::from_headers(&headers(&[(name, value)]));

            assert_eq!(
                field(&metadata),
                Some((*value).to_owned()),
                "header `{name}` should populate its field"
            );

            // Every other field must remain unset: the header is only ever read
            // into the one field this row names.
            for (other_name, _, other_field) in HEADER_CASES {
                if other_name != name {
                    assert_eq!(
                        other_field(&metadata),
                        None,
                        "header `{name}` must not populate the `{other_name}` field"
                    );
                }
            }
        }
    }

    #[test]
    fn all_headers_together() {
        let pairs: Vec<(&'static str, &str)> = HEADER_CASES
            .iter()
            .map(|(name, value, _)| (*name, *value))
            .collect();
        let metadata = SpeakMetadata::from_headers(&headers(&pairs));

        for (name, value, field) in HEADER_CASES {
            assert_eq!(
                field(&metadata),
                Some((*value).to_owned()),
                "header `{name}` should populate its field"
            );
        }
        assert_eq!(metadata.char_count, Some(42));
        assert_eq!(metadata.request_id(), Some(HEADER_CASES[0].1));
    }

    #[test]
    fn header_names_are_case_insensitive() {
        let mut map = HeaderMap::new();
        map.insert("DG-Request-Id", HeaderValue::from_static("req-1"));
        map.insert("Content-Type", HeaderValue::from_static("audio/wav"));

        let metadata = SpeakMetadata::from_headers(&map);
        assert_eq!(metadata.request_id.as_deref(), Some("req-1"));
        assert_eq!(metadata.content_type.as_deref(), Some("audio/wav"));
    }

    #[test]
    fn missing_headers_yield_default() {
        let metadata = SpeakMetadata::from_headers(&HeaderMap::new());
        assert_eq!(metadata, SpeakMetadata::default());
        assert_eq!(metadata.request_id(), None);
    }

    #[test]
    fn non_numeric_char_count_is_none() {
        let metadata = SpeakMetadata::from_headers(&headers(&[("dg-char-count", "lots")]));
        assert_eq!(metadata.char_count, None);

        let metadata = SpeakMetadata::from_headers(&headers(&[("dg-char-count", "-1")]));
        assert_eq!(metadata.char_count, None);
    }
}
