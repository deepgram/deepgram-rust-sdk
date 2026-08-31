//! Flux TTS batch (REST) module — synthesize a complete block of text
//! into a single audio response with `POST /v2/speak`.
//!
//! Use this for pre-rendering fixed audio (IVR prompts, notifications,
//! narration) where the whole text is known up front and you don't need
//! incremental playback or interruption. For live, interruptible,
//! turn-based synthesis, use the streaming WebSocket transport via
//! [`Speak::flux_request`](crate::Speak::flux_request) instead.
//!
//! See the [Deepgram Flux TTS API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/text-to-speech/speak-flux-batch

use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use serde_json::json;
use url::Url;

use super::options::{Options, SerializableOptions};
use crate::{DeepgramError, Speak};

static FLUX_SPEAK_URL_PATH: &str = "v2/speak";

impl Speak<'_> {
    /// Synthesize a complete block of text with the Flux TTS batch
    /// (REST) API and save the returned audio to a file.
    ///
    /// The response body is the synthesized audio in the requested
    /// encoding (`mp3` by default). When [`callback`] is set, the
    /// request is processed asynchronously and the saved body is instead
    /// a JSON acknowledgement of the form `{"request_id": "..."}`.
    ///
    /// [`callback`]: super::options::OptionsBuilder::callback
    pub async fn flux_speak_to_file(
        &self,
        text: &str,
        options: &Options,
        output_file: &std::path::Path,
    ) -> Result<(), DeepgramError> {
        let mut audio = self.flux_speak_to_stream(text, options).await?;

        let mut file = std::fs::File::create(output_file)?;
        while let Some(chunk) = audio.next().await {
            std::io::copy(&mut chunk?.as_ref(), &mut file)?;
        }

        Ok(())
    }

    /// Synthesize a complete block of text with the Flux TTS batch
    /// (REST) API, returning the audio as a byte stream.
    ///
    /// The stream yields the synthesized audio in the requested encoding
    /// (`mp3` by default). When [`callback`] is set, the request is
    /// processed asynchronously and the stream instead yields a JSON
    /// acknowledgement of the form `{"request_id": "..."}` as raw bytes.
    ///
    /// [`callback`]: super::options::OptionsBuilder::callback
    pub async fn flux_speak_to_stream(
        &self,
        text: &str,
        options: &Options,
    ) -> Result<impl Stream<Item = Result<Bytes, DeepgramError>>, DeepgramError> {
        let response = self
            .0
            .client
            .post(self.flux_speak_rest_url())
            .query(&SerializableOptions(options))
            .json(&json!({ "text": text }))
            .send()
            .await?;

        if let Err(err) = response.error_for_status_ref() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| status.to_string());
            return Err(DeepgramError::DeepgramApiError {
                body: error_text,
                err,
            });
        }

        Ok(response.bytes_stream().map(|chunk| Ok(chunk?)))
    }

    fn flux_speak_rest_url(&self) -> Url {
        self.0
            .base_url
            .join(FLUX_SPEAK_URL_PATH)
            .expect("base_url is checked to be a valid base_url when constructing Deepgram client")
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn flux_speak_rest_url() {
        let dg = Deepgram::new("token").unwrap();
        assert_eq!(
            &dg.text_to_speech().flux_speak_rest_url().to_string(),
            "https://api.deepgram.com/v2/speak"
        );
    }

    #[test]
    fn flux_speak_rest_url_custom_host() {
        let dg = Deepgram::with_base_url_and_api_key("http://localhost:8080", "token").unwrap();
        assert_eq!(
            &dg.text_to_speech().flux_speak_rest_url().to_string(),
            "http://localhost:8080/v2/speak"
        );
    }
}
