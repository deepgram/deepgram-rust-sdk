//! Generate subtitle/caption files ([SRT] and [WebVTT]) from a Deepgram
//! pre-recorded transcription [`Response`].
//!
//! Deepgram returns word-level timestamps; this module groups those words into
//! caption cues and renders them in the two most widely supported subtitle
//! formats. This mirrors the standalone [`@deepgram/captions`][js] helper
//! shipped with the JavaScript SDK.
//!
//! ```no_run
//! # use deepgram::common::batch_response::Response;
//! use deepgram::common::captions::{srt, webvtt, CaptionOptions};
//!
//! # fn demo(response: &Response) {
//! // Default: at most 8 words per cue.
//! let srt_file = srt(response, &CaptionOptions::default());
//! let vtt_file = webvtt(response, &CaptionOptions::default());
//! # }
//! ```
//!
//! [SRT]: https://en.wikipedia.org/wiki/SubRip
//! [WebVTT]: https://www.w3.org/TR/webvtt1/
//! [js]: https://github.com/deepgram/deepgram-js-captions

use super::batch_response::{Response, Word};

/// Options controlling how captions are generated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CaptionOptions {
    /// The maximum number of words placed in a single caption cue.
    ///
    /// Defaults to `8`, matching the Deepgram captions libraries.
    pub max_words_per_cue: usize,

    /// Whether to include the informational `NOTE` header block (request id,
    /// creation time, duration, channel count) at the top of WebVTT output.
    ///
    /// Has no effect on SRT output. Defaults to `true`.
    pub include_metadata_header: bool,
}

impl Default for CaptionOptions {
    fn default() -> Self {
        Self {
            max_words_per_cue: 8,
            include_metadata_header: true,
        }
    }
}

impl CaptionOptions {
    /// Construct a new [`CaptionOptions`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum number of words per caption cue.
    ///
    /// A value of `0` is treated as `1` to avoid producing empty cues.
    pub fn max_words_per_cue(mut self, max_words_per_cue: usize) -> Self {
        self.max_words_per_cue = max_words_per_cue;
        self
    }

    /// Set whether to include the WebVTT `NOTE` metadata header.
    pub fn include_metadata_header(mut self, include_metadata_header: bool) -> Self {
        self.include_metadata_header = include_metadata_header;
        self
    }
}

/// A single caption cue: a contiguous group of words with a start and end time.
struct Cue<'a> {
    words: &'a [Word],
}

impl Cue<'_> {
    fn start(&self) -> f64 {
        self.words.first().map(|w| w.start).unwrap_or(0.0)
    }

    fn end(&self) -> f64 {
        self.words.last().map(|w| w.end).unwrap_or(0.0)
    }

    fn speaker(&self) -> Option<usize> {
        self.words.first().and_then(|w| w.speaker)
    }

    fn text(&self) -> String {
        self.words
            .iter()
            .map(|w| w.punctuated_word.as_deref().unwrap_or(&w.word))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Split the transcript's words into cues of at most `max_words_per_cue` words.
fn cues(response: &Response, max_words_per_cue: usize) -> Vec<Cue<'_>> {
    let chunk = max_words_per_cue.max(1);
    let words = response
        .results
        .channels
        .first()
        .and_then(|channel| channel.alternatives.first())
        .map(|alt| alt.words.as_slice())
        .unwrap_or(&[]);

    words.chunks(chunk).map(|words| Cue { words }).collect()
}

/// Format a number of seconds as an `HH:MM:SS.mmm` timestamp.
fn format_timestamp(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    let total_millis = (seconds * 1000.0).round() as u64;
    let millis = total_millis % 1000;
    let total_seconds = total_millis / 1000;
    let secs = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;
    let hours = total_seconds / 3600;
    format!("{hours:02}:{minutes:02}:{secs:02}.{millis:03}")
}

/// Generate an [SRT] (SubRip) caption document from a transcription response.
///
/// The response should be produced with word-level timestamps (the default).
/// Words are grouped into cues of at most [`CaptionOptions::max_words_per_cue`]
/// words. Returns an empty string if the response contains no words.
///
/// [SRT]: https://en.wikipedia.org/wiki/SubRip
pub fn srt(response: &Response, options: &CaptionOptions) -> String {
    let mut output = String::new();
    for (index, cue) in cues(response, options.max_words_per_cue).iter().enumerate() {
        // SRT uses a comma as the millisecond separator.
        let start = format_timestamp(cue.start()).replace('.', ",");
        let end = format_timestamp(cue.end()).replace('.', ",");
        let line = match cue.speaker() {
            Some(speaker) => format!("[Speaker {speaker}] {}", cue.text()),
            None => cue.text(),
        };
        output.push_str(&format!("{}\n{start} --> {end}\n{line}\n\n", index + 1));
    }
    output
}

/// Generate a [WebVTT] caption document from a transcription response.
///
/// The response should be produced with word-level timestamps (the default).
/// Words are grouped into cues of at most [`CaptionOptions::max_words_per_cue`]
/// words. When diarization is enabled, each cue is prefixed with a WebVTT voice
/// tag (`<v Speaker N>`) taken from the speaker of the cue's first word (cues
/// are not split on speaker changes).
///
/// [WebVTT]: https://www.w3.org/TR/webvtt1/
pub fn webvtt(response: &Response, options: &CaptionOptions) -> String {
    let mut output = String::from("WEBVTT\n\n");

    if options.include_metadata_header {
        let metadata = &response.metadata;
        output.push_str("NOTE\nTranscription provided by Deepgram\n");
        output.push_str(&format!("Request Id: {}\n", metadata.request_id));
        output.push_str(&format!("Created: {}\n", metadata.created));
        output.push_str(&format!("Duration: {}\n", metadata.duration));
        output.push_str(&format!("Channels: {}\n\n", metadata.channels));
    }

    for cue in cues(response, options.max_words_per_cue) {
        let start = format_timestamp(cue.start());
        let end = format_timestamp(cue.end());
        let line = match cue.speaker() {
            Some(speaker) => format!("<v Speaker {speaker}>{}", cue.text()),
            None => cue.text(),
        };
        output.push_str(&format!("{start} --> {end}\n{line}\n\n"));
    }

    output
}

impl Response {
    /// Generate an [SRT] (SubRip) caption document from this response.
    ///
    /// Convenience wrapper around [`srt`].
    ///
    /// [SRT]: https://en.wikipedia.org/wiki/SubRip
    pub fn to_srt(&self, options: &CaptionOptions) -> String {
        srt(self, options)
    }

    /// Generate a [WebVTT] caption document from this response.
    ///
    /// Convenience wrapper around [`webvtt`].
    ///
    /// [WebVTT]: https://www.w3.org/TR/webvtt1/
    pub fn to_webvtt(&self, options: &CaptionOptions) -> String {
        webvtt(self, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_response() -> Response {
        let json = serde_json::json!({
            "metadata": {
                "request_id": "550e8400-e29b-41d4-a716-446655440000",
                "transaction_key": "deprecated",
                "sha256": "abc",
                "created": "2023-10-27T15:35:56.637Z",
                "duration": 2.5,
                "channels": 1
            },
            "results": {
                "channels": [{
                    "alternatives": [{
                        "transcript": "hello there friend",
                        "confidence": 0.99,
                        "words": [
                            {"word": "hello", "start": 0.0, "end": 0.5, "confidence": 0.9, "punctuated_word": "Hello,"},
                            {"word": "there", "start": 0.5, "end": 1.0, "confidence": 0.9, "punctuated_word": "there"},
                            {"word": "friend", "start": 1.0, "end": 1.5, "confidence": 0.9, "punctuated_word": "friend."}
                        ]
                    }]
                }]
            }
        });
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn format_timestamp_basic() {
        assert_eq!(format_timestamp(0.0), "00:00:00.000");
        assert_eq!(format_timestamp(1.5), "00:00:01.500");
        assert_eq!(format_timestamp(65.25), "00:01:05.250");
        assert_eq!(format_timestamp(3661.001), "01:01:01.001");
    }

    #[test]
    fn srt_single_cue() {
        let response = sample_response();
        let out = srt(&response, &CaptionOptions::default());
        let expected = "1\n00:00:00,000 --> 00:00:01,500\nHello, there friend.\n\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn srt_respects_line_length() {
        let response = sample_response();
        let out = srt(&response, &CaptionOptions::default().max_words_per_cue(2));
        assert!(out.starts_with("1\n00:00:00,000 --> 00:00:01,000\nHello, there\n\n2\n"));
        assert!(out.contains("2\n00:00:01,000 --> 00:00:01,500\nfriend.\n\n"));
    }

    #[test]
    fn webvtt_has_header_and_cue() {
        let response = sample_response();
        let out = webvtt(&response, &CaptionOptions::default());
        assert!(out.starts_with("WEBVTT\n\n"));
        assert!(out.contains("Request Id: 550e8400-e29b-41d4-a716-446655440000"));
        assert!(out.contains("00:00:00.000 --> 00:00:01.500\nHello, there friend.\n\n"));
    }

    #[test]
    fn webvtt_without_header() {
        let response = sample_response();
        let out = webvtt(
            &response,
            &CaptionOptions::default().include_metadata_header(false),
        );
        assert_eq!(
            out,
            "WEBVTT\n\n00:00:00.000 --> 00:00:01.500\nHello, there friend.\n\n"
        );
    }

    #[test]
    fn empty_response_produces_minimal_output() {
        let json = serde_json::json!({
            "metadata": {
                "request_id": "550e8400-e29b-41d4-a716-446655440000",
                "transaction_key": "k", "sha256": "s",
                "created": "c", "duration": 0.0, "channels": 1
            },
            "results": { "channels": [] }
        });
        let response: Response = serde_json::from_value(json).unwrap();
        assert_eq!(srt(&response, &CaptionOptions::default()), "");
        assert!(webvtt(&response, &CaptionOptions::default()).starts_with("WEBVTT"));
    }
}
