use deepgram::{
    transcription::prerecorded::{AudioSource, Language, Options},
    Deepgram, DeepgramError,
};
use std::env;

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

    let response = dg_client
        .transcription()
        .prerecorded(source, &options)
        .await?;

    let transcript = &response.results.channels[0].alternatives[0].transcript;
    println!("{}", transcript);

    Ok(())
}
