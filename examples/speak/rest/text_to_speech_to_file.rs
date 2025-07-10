use std::{env, path::Path, time::Instant};

use deepgram::{
    speak::options::{Container, Encoding, Model, Options},
    Deepgram, DeepgramError,
};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let options = Options::builder()
        .model(Model::AuraAsteriaEn)
        .encoding(Encoding::Linear16)
        .sample_rate(16000)
        .container(Container::Wav)
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
    println!("Time to download audio: {elapsed_time:.2?}");

    Ok(())
}
