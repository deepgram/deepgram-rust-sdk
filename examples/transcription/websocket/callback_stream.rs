use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{
    common::options::{Encoding, Endpointing, Language, Options},
    Deepgram, DeepgramError,
};

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";
static AUDIO_CHUNK_SIZE: usize = 3174;
static FRAME_DELAY: Duration = Duration::from_millis(16);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let options = Options::builder()
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let callback_url = env::var("DEEPGRAM_CALLBACK_URL")
        .expect("DEEPGRAM_CALLBACK_URL environmental variable")
        .parse()
        .expect("DEEPGRAM_CALLBACK_URL not a valid URL");

    let mut results = dg_client
        .transcription()
        .stream_request_with_options(options)
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .endpointing(Endpointing::CustomDurationMs(300))
        .interim_results(true)
        .utterance_end_ms(1000)
        .vad_events(true)
        .no_delay(true)
        .callback(callback_url)
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY)
        .await?;

    while let Some(result) = results.next().await {
        println!("got: {:?}", result);
    }

    Ok(())
}
