//! Generate subtitle/caption files ([SRT] and [WebVTT]) from a Deepgram
//! pre-recorded transcription [`Response`].
//!
//! Deepgram returns word-level timestamps; this module groups those words into
//! caption cues and renders them in the two most widely supported subtitle
//! formats. This mirrors the standalone [`@deepgram/captions`][js] helper
//! shipped with the JavaScript SDK, including how it chooses cue boundaries
//! and how it labels speakers:
//!
//! - When the response contains `results.utterances` (the
//!   [Utterances feature][utterances]), every utterance becomes its own cue.
//!   An utterance longer than [`CaptionOptions::max_words_per_cue`] is split
//!   into consecutive cues, each carrying the utterance's speaker. Utterances
//!   from **every** channel are included, in the order the response lists
//!   them, with no channel marker in the output.
//! - Otherwise the words of `channels[0].alternatives[0]` are grouped into
//!   cues of at most `max_words_per_cue` words; other channels are ignored.
//!   When the transcript is [diarized][diarize], a speaker change always
//!   starts a new cue, so a cue never contains words from more than one
//!   speaker.
//!
//! Speaker labels for diarized transcripts follow the JS helper too: SRT
//! writes `[Speaker N]` on its own line above the cue text, and only on a cue
//! whose speaker differs from the previous cue's (so the first cue is always
//! labeled); WebVTT prefixes every cue's text with a `<v Speaker N>` voice
//! tag. Timestamps are truncated, not rounded, to whole milliseconds.
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
//! [utterances]: https://developers.deepgram.com/docs/utterances
//! [diarize]: https://developers.deepgram.com/docs/diarization

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
    pub fn with_max_words_per_cue(mut self, max_words_per_cue: usize) -> Self {
        self.max_words_per_cue = max_words_per_cue;
        self
    }

    /// Set whether to include the WebVTT `NOTE` metadata header.
    pub fn with_include_metadata_header(mut self, include_metadata_header: bool) -> Self {
        self.include_metadata_header = include_metadata_header;
        self
    }
}

/// A single caption cue: a contiguous group of words with a start and end time
/// and, when diarization is enabled, the speaker they are attributed to.
struct Cue<'a> {
    words: &'a [Word],
    speaker: Option<usize>,
}

impl<'a> Cue<'a> {
    /// A cue built directly from a run of words, attributed to the speaker of
    /// its first word (every word in a run shares one speaker, see [`cues`]).
    fn from_words(words: &'a [Word]) -> Self {
        Cue {
            words,
            speaker: words.first().and_then(|w| w.speaker),
        }
    }

    fn start(&self) -> f64 {
        self.words.first().map(|w| w.start).unwrap_or(0.0)
    }

    fn end(&self) -> f64 {
        self.words.last().map(|w| w.end).unwrap_or(0.0)
    }

    fn speaker(&self) -> Option<usize> {
        self.speaker
    }

    fn text(&self) -> String {
        self.words
            .iter()
            .map(|w| w.punctuated_word.as_deref().unwrap_or(&w.word))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Group the transcript into caption cues.
///
/// Prefers `results.utterances` when present: each utterance is its own cue,
/// split into chunks of at most `max_words_per_cue` words, and every chunk is
/// attributed to the utterance's speaker. Without utterances, the words of the
/// first channel's first alternative are buffered into cues that are flushed
/// when they reach `max_words_per_cue` words or when the diarized speaker
/// changes, so no cue ever spans two speakers.
fn cues(response: &Response, max_words_per_cue: usize) -> Vec<Cue<'_>> {
    let chunk = max_words_per_cue.max(1);

    if let Some(utterances) = &response.results.utterances {
        return utterances
            .iter()
            .flat_map(|utterance| {
                utterance.words.chunks(chunk).map(move |words| Cue {
                    words,
                    speaker: utterance
                        .speaker
                        .or_else(|| words.first().and_then(|w| w.speaker)),
                })
            })
            .collect();
    }

    let words = response
        .results
        .channels
        .first()
        .and_then(|channel| channel.alternatives.first())
        .map(|alt| alt.words.as_slice())
        .unwrap_or(&[]);

    let mut cues = Vec::new();
    let mut cue_start = 0;
    for (index, word) in words.iter().enumerate() {
        let buffered = &words[cue_start..index];
        // Both conditions imply a non-empty buffer (`chunk >= 1`), so an empty
        // cue is never pushed.
        let speaker_changed = buffered
            .first()
            .is_some_and(|first| first.speaker != word.speaker);
        if buffered.len() >= chunk || speaker_changed {
            cues.push(Cue::from_words(buffered));
            cue_start = index;
        }
    }
    if cue_start < words.len() {
        cues.push(Cue::from_words(&words[cue_start..]));
    }

    cues
}

/// Format a number of seconds as an `HH:MM:SS.mmm` timestamp.
///
/// Sub-millisecond precision is truncated rather than rounded, matching the
/// JS helper: `secondsToTimestamp` builds a `Date` from `seconds * 1000`,
/// which drops the fractional milliseconds. So `3.2206` renders as
/// `00:00:03.220`, not `00:00:03.221`.
fn format_timestamp(seconds: f64) -> String {
    let seconds = seconds.max(0.0);
    let total_millis = (seconds * 1000.0).floor() as u64;
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
/// Cues follow utterance boundaries when `results.utterances` is present and
/// otherwise contain at most [`CaptionOptions::max_words_per_cue`] words from
/// a single diarized speaker; see the [module docs](self) for the exact rules.
/// When diarization is enabled, a cue whose speaker differs from the previous
/// cue's carries a `[Speaker N]` label on its own line above the text (the
/// first diarized cue is always labeled); consecutive cues from the same
/// speaker are not re-labeled. This is the shape produced by the
/// [`@deepgram/captions`](https://github.com/deepgram/deepgram-js-captions)
/// JavaScript helper.
/// Returns an empty string if the response contains no words.
///
/// [SRT]: https://en.wikipedia.org/wiki/SubRip
pub fn srt(response: &Response, options: &CaptionOptions) -> String {
    let mut output = String::new();
    let mut previous_speaker = None;
    for (index, cue) in cues(response, options.max_words_per_cue).iter().enumerate() {
        // SRT uses a comma as the millisecond separator.
        let start = format_timestamp(cue.start()).replace('.', ",");
        let end = format_timestamp(cue.end()).replace('.', ",");
        // Like the JS helper, label the speaker only when it changes from the
        // previous cue, on a line of its own above the text.
        let line = match cue.speaker() {
            Some(speaker) if previous_speaker != Some(speaker) => {
                format!("[Speaker {speaker}]\n{}", cue.text())
            }
            _ => cue.text(),
        };
        previous_speaker = cue.speaker();
        output.push_str(&format!("{}\n{start} --> {end}\n{line}\n\n", index + 1));
    }
    output
}

/// Generate a [WebVTT] caption document from a transcription response.
///
/// The response should be produced with word-level timestamps (the default).
/// Cues follow utterance boundaries when `results.utterances` is present and
/// otherwise contain at most [`CaptionOptions::max_words_per_cue`] words from
/// a single diarized speaker; see the [module docs](self) for the exact rules.
/// When diarization is enabled, each cue is prefixed with a WebVTT voice tag
/// (`<v Speaker N>`).
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
    use serde_json::{json, Value};

    use super::*;

    fn metadata() -> Value {
        json!({
            "request_id": "550e8400-e29b-41d4-a716-446655440000",
            "transaction_key": "deprecated",
            "sha256": "abc",
            "created": "2023-10-27T15:35:56.637Z",
            "duration": 2.5,
            "channels": 1
        })
    }

    /// A word fixture; `speaker` is omitted from the JSON when `None`.
    fn word(text: &str, start: f64, end: f64, speaker: Option<usize>) -> Value {
        let mut word = json!({
            "word": text.trim_end_matches(['.', ',']).to_lowercase(),
            "start": start,
            "end": end,
            "confidence": 0.9,
            "punctuated_word": text
        });
        if let Some(speaker) = speaker {
            word["speaker"] = json!(speaker);
        }
        word
    }

    fn response_from_words(words: Vec<Value>) -> Response {
        let json = json!({
            "metadata": metadata(),
            "results": {
                "channels": [{
                    "alternatives": [{
                        "transcript": "",
                        "confidence": 0.99,
                        "words": words
                    }]
                }]
            }
        });
        serde_json::from_value(json).unwrap()
    }

    fn utterance(words: Vec<Value>, speaker: Option<usize>) -> Value {
        let start = words
            .first()
            .map(|w| w["start"].clone())
            .unwrap_or(json!(0.0));
        let end = words.last().map(|w| w["end"].clone()).unwrap_or(json!(0.0));
        let mut utterance = json!({
            "start": start,
            "end": end,
            "confidence": 0.9,
            "channel": 0,
            "transcript": "",
            "words": words,
            "id": "0e4c3d5a-8e2b-4b8e-9c1a-1f2e3d4c5b6a"
        });
        if let Some(speaker) = speaker {
            utterance["speaker"] = json!(speaker);
        }
        utterance
    }

    /// A response carrying both `utterances` and channel words, so tests can
    /// prove which source the caption helper picks.
    fn response_from_utterances(utterances: Vec<Value>, channel_words: Vec<Value>) -> Response {
        let json = json!({
            "metadata": metadata(),
            "results": {
                "channels": [{
                    "alternatives": [{
                        "transcript": "",
                        "confidence": 0.99,
                        "words": channel_words
                    }]
                }],
                "utterances": utterances
            }
        });
        serde_json::from_value(json).unwrap()
    }

    fn sample_response() -> Response {
        response_from_words(vec![
            word("Hello,", 0.0, 0.5, None),
            word("there", 0.5, 1.0, None),
            word("friend.", 1.0, 1.5, None),
        ])
    }

    #[test]
    fn format_timestamp_basic() {
        assert_eq!(format_timestamp(0.0), "00:00:00.000");
        assert_eq!(format_timestamp(1.5), "00:00:01.500");
        assert_eq!(format_timestamp(65.25), "00:01:05.250");
        assert_eq!(format_timestamp(3661.001), "01:01:01.001");
    }

    #[test]
    fn format_timestamp_truncates_sub_millisecond_precision() {
        // The JS helper floors (a `Date` built from `seconds * 1000` drops the
        // fractional milliseconds), so a word ending at 3.2206 s must render
        // as `,220`, not the rounded `,221`.
        assert_eq!(format_timestamp(3.2206), "00:00:03.220");
        assert_eq!(format_timestamp(0.9999), "00:00:00.999");
        assert_eq!(format_timestamp(1.0005), "00:00:01.000");

        let response = response_from_words(vec![word("late", 3.2201, 3.2206, None)]);
        assert_eq!(
            srt(&response, &CaptionOptions::default()),
            "1\n00:00:03,220 --> 00:00:03,220\nlate\n\n"
        );
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
        let out = srt(
            &response,
            &CaptionOptions::default().with_max_words_per_cue(2),
        );
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
            &CaptionOptions::default().with_include_metadata_header(false),
        );
        assert_eq!(
            out,
            "WEBVTT\n\n00:00:00.000 --> 00:00:01.500\nHello, there friend.\n\n"
        );
    }

    #[test]
    fn empty_response_produces_minimal_output() {
        let json = json!({
            "metadata": metadata(),
            "results": { "channels": [] }
        });
        let response: Response = serde_json::from_value(json).unwrap();
        assert_eq!(srt(&response, &CaptionOptions::default()), "");
        assert!(webvtt(&response, &CaptionOptions::default()).starts_with("WEBVTT"));
    }

    // --- Word path: diarized speaker changes -------------------------------

    #[test]
    fn speaker_change_starts_new_cue() {
        // Four words fit in one default cue (8), but the speaker changes after
        // the second word, so the helper must emit two cues.
        let response = response_from_words(vec![
            word("Hello", 0.0, 0.5, Some(0)),
            word("there.", 0.5, 1.0, Some(0)),
            word("Hi", 1.2, 1.5, Some(1)),
            word("back.", 1.5, 2.0, Some(1)),
        ]);

        // Matching `@deepgram/captions`, the label sits on its own line above
        // the text and appears because the speaker differs from the previous
        // cue (the first cue is always labeled).
        let out = srt(&response, &CaptionOptions::default());
        assert_eq!(
            out,
            "1\n00:00:00,000 --> 00:00:01,000\n[Speaker 0]\nHello there.\n\n\
             2\n00:00:01,200 --> 00:00:02,000\n[Speaker 1]\nHi back.\n\n"
        );

        // The first speaker's cue must never contain the second speaker's words.
        let speaker_0_cue = out.split("\n\n").next().unwrap();
        assert!(!speaker_0_cue.contains("Hi"));
        assert!(!speaker_0_cue.contains("back."));
    }

    #[test]
    fn speaker_change_and_max_words_both_flush() {
        // Speaker 0 says three words with a two-word limit (two cues), then
        // speaker 1 says one word (its own cue), then speaker 0 returns.
        // Only cues where the speaker changes carry a label: cue 2 continues
        // speaker 0 and is unlabeled, cue 4 re-labels speaker 0 on return.
        let response = response_from_words(vec![
            word("one", 0.0, 0.1, Some(0)),
            word("two", 0.1, 0.2, Some(0)),
            word("three", 0.2, 0.3, Some(0)),
            word("four", 0.3, 0.4, Some(1)),
            word("five", 0.4, 0.5, Some(0)),
        ]);

        let out = srt(
            &response,
            &CaptionOptions::default().with_max_words_per_cue(2),
        );
        assert_eq!(
            out,
            "1\n00:00:00,000 --> 00:00:00,200\n[Speaker 0]\none two\n\n\
             2\n00:00:00,200 --> 00:00:00,300\nthree\n\n\
             3\n00:00:00,300 --> 00:00:00,400\n[Speaker 1]\nfour\n\n\
             4\n00:00:00,400 --> 00:00:00,500\n[Speaker 0]\nfive\n\n"
        );
    }

    #[test]
    fn speaker_change_webvtt_voice_tags() {
        let response = response_from_words(vec![
            word("Hello", 0.0, 0.5, Some(0)),
            word("Hi", 0.5, 1.0, Some(1)),
        ]);

        let out = webvtt(
            &response,
            &CaptionOptions::default().with_include_metadata_header(false),
        );
        assert_eq!(
            out,
            "WEBVTT\n\n\
             00:00:00.000 --> 00:00:00.500\n<v Speaker 0>Hello\n\n\
             00:00:00.500 --> 00:00:01.000\n<v Speaker 1>Hi\n\n"
        );
    }

    #[test]
    fn undiarized_words_are_not_split() {
        // Without speaker labels only the word limit applies.
        let response = response_from_words(vec![
            word("a", 0.0, 0.1, None),
            word("b", 0.1, 0.2, None),
            word("c", 0.2, 0.3, None),
        ]);

        let out = srt(&response, &CaptionOptions::default());
        assert_eq!(out, "1\n00:00:00,000 --> 00:00:00,300\na b c\n\n");
    }

    // --- Utterance path -----------------------------------------------------

    #[test]
    fn utterance_boundaries_are_preserved() {
        // Two short utterances by the same speaker with a pause between them.
        // Chunking the channel words alone would merge them into one cue.
        let first = vec![
            word("Hello", 0.0, 0.5, Some(0)),
            word("there.", 0.5, 1.0, Some(0)),
        ];
        let second = vec![
            word("How", 3.0, 3.3, Some(0)),
            word("are", 3.3, 3.6, Some(0)),
            word("you?", 3.6, 4.0, Some(0)),
        ];
        let channel_words = first.iter().chain(&second).cloned().collect();
        let response = response_from_utterances(
            vec![utterance(first, Some(0)), utterance(second, Some(0))],
            channel_words,
        );

        let out = srt(&response, &CaptionOptions::default());
        assert_eq!(
            out,
            "1\n00:00:00,000 --> 00:00:01,000\n[Speaker 0]\nHello there.\n\n\
             2\n00:00:03,000 --> 00:00:04,000\nHow are you?\n\n"
        );
    }

    #[test]
    fn utterances_are_chunked_at_max_words_and_keep_speaker() {
        let words = vec![
            word("one", 0.0, 0.1, Some(2)),
            word("two", 0.1, 0.2, Some(2)),
            word("three", 0.2, 0.3, Some(2)),
        ];
        let response = response_from_utterances(vec![utterance(words.clone(), Some(2))], words);

        // WebVTT tags every cue, so it proves each chunk kept the speaker.
        let out = webvtt(
            &response,
            &CaptionOptions::default()
                .with_include_metadata_header(false)
                .with_max_words_per_cue(2),
        );
        assert_eq!(
            out,
            "WEBVTT\n\n\
             00:00:00.000 --> 00:00:00.200\n<v Speaker 2>one two\n\n\
             00:00:00.200 --> 00:00:00.300\n<v Speaker 2>three\n\n"
        );

        // SRT labels only the first chunk; the second continues speaker 2.
        let out = srt(
            &response,
            &CaptionOptions::default().with_max_words_per_cue(2),
        );
        assert_eq!(
            out,
            "1\n00:00:00,000 --> 00:00:00,200\n[Speaker 2]\none two\n\n\
             2\n00:00:00,200 --> 00:00:00,300\nthree\n\n"
        );
    }

    #[test]
    fn utterances_take_precedence_over_channel_words() {
        // The channel words disagree with the utterances; the utterances win.
        let utterance_words = vec![word("Utterance", 0.0, 1.0, Some(1))];
        let channel_words = vec![
            word("Channel", 0.0, 0.5, Some(0)),
            word("words", 0.5, 1.0, Some(0)),
        ];
        let response =
            response_from_utterances(vec![utterance(utterance_words, Some(1))], channel_words);

        let out = srt(&response, &CaptionOptions::default());
        assert_eq!(
            out,
            "1\n00:00:00,000 --> 00:00:01,000\n[Speaker 1]\nUtterance\n\n"
        );
        assert!(!out.contains("Channel"));
    }

    #[test]
    fn utterance_speaker_labels_every_chunk_even_without_word_speakers() {
        // Utterance-level speaker with no per-word speaker fields.
        let words = vec![
            word("a", 0.0, 0.1, None),
            word("b", 0.1, 0.2, None),
            word("c", 0.2, 0.3, None),
        ];
        let response = response_from_utterances(vec![utterance(words.clone(), Some(3))], words);

        let out = webvtt(
            &response,
            &CaptionOptions::default()
                .with_include_metadata_header(false)
                .with_max_words_per_cue(2),
        );
        assert_eq!(
            out,
            "WEBVTT\n\n\
             00:00:00.000 --> 00:00:00.200\n<v Speaker 3>a b\n\n\
             00:00:00.200 --> 00:00:00.300\n<v Speaker 3>c\n\n"
        );
    }

    #[test]
    fn undiarized_utterances_have_no_speaker_label() {
        let words = vec![word("Hello", 0.0, 0.5, None)];
        let response = response_from_utterances(vec![utterance(words.clone(), None)], words);

        let out = srt(&response, &CaptionOptions::default());
        assert_eq!(out, "1\n00:00:00,000 --> 00:00:00,500\nHello\n\n");
    }

    #[test]
    fn empty_utterance_list_produces_no_cues() {
        let response = response_from_utterances(vec![], vec![word("ignored", 0.0, 0.5, None)]);
        assert_eq!(srt(&response, &CaptionOptions::default()), "");
    }
}
