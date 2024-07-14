//! Types used for speech to text
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/text-to-speech-api

use options::{Options, SerializableOptions};
use reqwest::RequestBuilder;
use serde_json::Value;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use url::Url;

use super::Speak;

pub mod options;

static DEEPGRAM_API_URL_SPEAK: &str = "v1/speak";

impl<'a> Speak<'a> {
    /// Sends a request to Deepgram to transcribe pre-recorded audio.
    pub async fn speak(
        &self,
        text: &str,
        options: &Options,
        output_file: &Path,
    ) -> crate::Result<()> {
        let payload = Value::Object(
            [("text".to_string(), Value::String(text.to_string()))]
                .iter()
                .cloned()
                .collect(),
        );

        let request_builder = self
            .0
            .client
            .post(self.speak_url())
            .query(&SerializableOptions(options))
            .json(&payload);

        self.send_and_translate_response(request_builder, output_file)
            .await
    }

    async fn send_and_translate_response(
        &self,
        request_builder: RequestBuilder,
        output_file: &Path,
    ) -> crate::Result<()> {
        let mut response = request_builder.send().await?;

        // Ensure the request was successful
        if response.status().is_success() {
            // Create the output file
            let mut file = File::create(output_file)?;

            // Stream the response body to the file
            while let Some(chunk) = response.chunk().await? {
                copy(&mut chunk.as_ref(), &mut file)?;
            }

            println!("Audio saved to {:?}", output_file);
        } else {
            eprintln!("Failed to generate speech: {}", response.status());
            let error_text = response.text().await?;
            eprintln!("Error details: {}", error_text);
        }

        Ok(())
    }

    fn speak_url(&self) -> Url {
        self.0.base_url.join(DEEPGRAM_API_URL_SPEAK).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn listen_url() {
        let dg = Deepgram::new("token");
        assert_eq!(
            &dg.text_to_speech().speak_url().to_string(),
            "https://api.deepgram.com/v1/speak"
        );
    }
}
