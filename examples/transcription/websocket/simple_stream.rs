use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{
    common::options::{Encoding, Endpointing, Language, Options},
    Deepgram, DeepgramError,
};

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";
static AUDIO_CHUNK_SIZE: usize = 3174;

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let dg = Deepgram::new(env::var("DEEPGRAM_API_KEY").unwrap());

    let options = Options::builder()
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let mut results = dg
        .transcription()
        .stream_request_with_options(Some(&options))
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .endpointing(Endpointing::CustomDurationMs(300))
        .interim_results(true)
        .utterance_end_ms(1000)
        .vad_events(true)
        .no_delay(true)
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, Duration::from_millis(16))
        .await?
        .start()
        .await?;

    while let Some(result) = results.next().await {
        println!("got: {:?}", result);
    }

    Ok(())
}
