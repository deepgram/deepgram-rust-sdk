use super::{Deepgram, Result};

mod audio_source;
mod options;
mod response;

pub use audio_source::{BufferSource, UrlSource};
pub use options::{Language, Model, Options, OptionsBuilder, Redact, Utterances};
pub use response::PrerecordedResponse;

use audio_source::AudioSource;
use options::SerializableOptions;

impl<K: AsRef<str>> Deepgram<K> {
    pub async fn prerecorded_request(
        &self,
        source: impl AudioSource,
        options: &Options<'_>,
    ) -> Result<PrerecordedResponse> {
        let request_builder = self
            .client
            .post("https://api.deepgram.com/v1/listen")
            .header("Authorization", format!("Token {}", self.api_key.as_ref()))
            .query(&SerializableOptions(options));
        let request_builder = source.fill_body(request_builder);

        Ok(request_builder.send().await?.json().await?)
    }
}
