use std::env;

use deepgram::{
    transcription::prerecorded::{
        audio_source::AudioSource,
        options::{Language, Options},
    },
    Deepgram, DeepgramError,
};
use tokio::fs::File;

static PATH_TO_FILE: &str = "examples/prerecorded_from_file/Bueller-Life-moves-pretty-fast.mp3";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let file = File::open(PATH_TO_FILE).await.unwrap();

    let source = AudioSource::from_buffer_with_mime_type(file, "audio/mpeg3");

    let options = Options::builder()
        .punctuate(true)
        .language(Language::en_US)
        .build();

    let response = dg_client
        .transcription()
        .prerecorded(source, &options)
        .await?;

    if let Some(transcript) = response.results
        .as_ref()
        .and_then(|result| result.channels.get(0))
        .and_then(|channel| channel.alternatives.get(0))
        .map(|alternative| &alternative.transcript) {
        
        // Now you can use the `transcript` variable safely here.
        println!("Transcript: {}", transcript);
    } else {
        // Handle callback responses
        println!("{:?}", response);
    }

    

    Ok(())
}
