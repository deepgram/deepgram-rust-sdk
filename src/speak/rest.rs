//! Rest TTS module

use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use reqwest::RequestBuilder;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use url::Url;

use crate::{speak::response::SpeakMetadata, DeepgramError, Speak};

use super::options::{Options, SerializableOptions};

static DEEPGRAM_API_URL_SPEAK: &str = "v1/speak";

impl Speak<'_> {
    /// Sends a request to Deepgram to generate speech and save it to a file.
    pub async fn speak_to_file(
        &self,
        text: &str,
        options: &Options,
        output_file: &std::path::Path,
    ) -> Result<(), DeepgramError> {
        self.speak_to_file_with_metadata(text, options, output_file)
            .await
            .map(|_metadata| ())
    }

    /// Sends a request to Deepgram to generate speech, save it to a file, and
    /// return the response [`SpeakMetadata`] (including the `dg-request-id`).
    ///
    /// This behaves exactly like [`Speak::speak_to_file`] but additionally
    /// surfaces the response headers.
    ///
    /// See the [Deepgram Text-to-Speech docs][docs] for the returned metadata.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-to-speech#results
    pub async fn speak_to_file_with_metadata(
        &self,
        text: &str,
        options: &Options,
        output_file: &std::path::Path,
    ) -> Result<SpeakMetadata, DeepgramError> {
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

        self.send_and_save_response(request_builder, output_file)
            .await
    }

    async fn send_and_save_response(
        &self,
        request_builder: RequestBuilder,
        output_file: &std::path::Path,
    ) -> Result<SpeakMetadata, DeepgramError> {
        let mut response = request_builder.send().await?;

        if let Err(err) = response.error_for_status_ref() {
            let status = response.status();
            let error_text = response.text().await?;
            eprintln!("Failed to generate speech: {status}");
            eprintln!("Error details: {error_text}");
            return Err(DeepgramError::DeepgramApiError {
                body: error_text,
                err,
            });
        }

        let metadata = SpeakMetadata::from_headers(response.headers());

        // Create the output file
        let mut file = std::fs::File::create(output_file)?;

        // Stream the response body to the file
        while let Some(chunk) = response.chunk().await? {
            std::io::copy(&mut chunk.as_ref(), &mut file)?;
        }

        Ok(metadata)
    }

    /// Sends a request to Deepgram to generate speech and stream back the audio.
    pub async fn speak_to_stream(
        &self,
        text: &str,
        options: &Options,
    ) -> Result<impl Stream<Item = Bytes>, DeepgramError> {
        self.speak_to_stream_with_metadata(text, options)
            .await
            .map(|(_metadata, stream)| stream)
    }

    /// Sends a request to Deepgram to generate speech, returning the response
    /// [`SpeakMetadata`] (including the `dg-request-id`) alongside the audio
    /// stream.
    ///
    /// The metadata is read from the response headers, which are available
    /// before the audio body is consumed, so it is returned immediately.
    ///
    /// See the [Deepgram Text-to-Speech docs][docs] for the returned metadata.
    ///
    /// [docs]: https://developers.deepgram.com/docs/text-to-speech#results
    pub async fn speak_to_stream_with_metadata(
        &self,
        text: &str,
        options: &Options,
    ) -> Result<(SpeakMetadata, impl Stream<Item = Bytes>), DeepgramError> {
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

        self.send_and_stream_response(request_builder).await
    }

    async fn send_and_stream_response(
        &self,
        request_builder: RequestBuilder,
    ) -> Result<(SpeakMetadata, impl Stream<Item = Bytes>), DeepgramError> {
        let response = request_builder.send().await?;

        if let Err(err) = response.error_for_status_ref() {
            let status = response.status();
            let error_text = response.text().await?;
            eprintln!("Failed to generate speech: {status}");
            eprintln!("Error details: {error_text}");
            return Err(DeepgramError::DeepgramApiError {
                body: error_text,
                err,
            });
        }

        let metadata = SpeakMetadata::from_headers(response.headers());

        let (tx, rx) = mpsc::channel(1024);
        let rx_stream = ReceiverStream::new(rx);

        tokio::spawn(async move {
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(data) => {
                        if tx.send(data).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error streaming response: {e}");
                        break;
                    }
                }
            }
        });

        Ok((metadata, rx_stream))
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
        let dg = Deepgram::new("token").unwrap();
        assert_eq!(
            &dg.text_to_speech().speak_url().to_string(),
            "https://api.deepgram.com/v1/speak"
        );
    }
}
