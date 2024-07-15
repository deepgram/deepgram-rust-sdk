use std::env;

use deepgram::{
    transcription::{
        audio_source::AudioSource, common_options::{Language, Options},
    },
    Deepgram, DeepgramError,
};

static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let source = AudioSource::from_url(AUDIO_URL);

    let options = Options::builder()
        .punctuate(true)
        .language(Language::en_US)
        .build();

    let callback_url =
        env::var("DEEPGRAM_CALLBACK_URL").expect("DEEPGRAM_CALLBACK_URL environmental variable");

    let response = dg_client
        .transcription()
        .prerecorded_callback(source, &options, &callback_url)
        .await?;

    println!("{}", response.request_id);

    Ok(())
}
