use std::{env, path::Path, time::Instant};

use deepgram::{speak::options::Options, Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let options = Options::builder()
        .model("aura-asteria-en")
        .encoding("linear16")
        .sample_rate(16000)
        .container("wav")
        .build();

    let text = "Hello, how can I help you today? This is a longer sentence to increase the time taken to process the audio, so that the streaming shows the full delta vs downloading the whole file.";
    let output_file = Path::new("your_output_file.wav");

    // Record the start time
    let start_time = Instant::now();

    dg_client
        .text_to_speech()
        .speak_to_file(text, &options, output_file)
        .await?;

    let elapsed_time = start_time.elapsed();
    println!("Time to download audio: {:.2?}", elapsed_time);

    Ok(())
}
