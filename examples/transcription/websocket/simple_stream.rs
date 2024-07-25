use futures_util::stream::StreamExt;
use std::env;
use std::time::Duration;
use tokio::sync::mpsc;

use deepgram::{
    common::options::{Encoding, Endpointing, Language, Model, Options},
    listen::websocket::Event,
    Deepgram, DeepgramError,
};

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";
static AUDIO_CHUNK_SIZE: usize = 3174;

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let dg = Deepgram::new(env::var("DEEPGRAM_API_KEY").unwrap());

    let options = Options::builder()
        .model(Model::Nova2)
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let (event_tx, mut event_rx) = mpsc::channel::<Event>(100);

    // Event handling task
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                Event::Open => println!("Connection opened"),
                Event::Close => println!("Connection closed"),
                Event::Error(e) => eprintln!("Error occurred: {:?}", e),
                Event::Result(result) => println!("got: {:?}", result),
            }
        }
    });

    let (_connection, mut response_stream) = dg
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
        .file(
            PATH_TO_FILE,
            AUDIO_CHUNK_SIZE,
            Duration::from_millis(16),
            event_tx.clone(),
        )
        .await?
        .start(event_tx.clone())
        .await?;

    while let Some(response) = response_stream.next().await {
        match response {
            Ok(result) => println!("Transcription result: {:?}", result),
            Err(e) => eprintln!("Transcription error: {:?}", e),
        }
    }

    Ok(())
}
