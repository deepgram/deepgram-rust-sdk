use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{
    transcription::common_options::{Language, Options},
    Deepgram, DeepgramError,
};

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
        .encoding("linear16".to_string())
        .sample_rate(44100)
        .channels(2)
        .endpointing("300".to_string())
        .interim_results(true)
        .utterance_end_ms(1000)
        .vad_events(true)
        .no_delay(true)
        .file(
            "./examples/prerecorded_from_file/bueller.wav",
            3174,
            Duration::from_millis(16),
        )
        .await?
        .start()
        .await?;

    while let Some(result) = results.next().await {
        println!("got: {:?}", result);
    }

    Ok(())
}
