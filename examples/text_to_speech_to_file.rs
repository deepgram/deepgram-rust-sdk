use std::{env, path::Path};

use deepgram::{speak::rest::options::Options, Deepgram, DeepgramError};

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

    let text = "Hello, how can I help you today?";
    let output_file = Path::new("your_output_file.wav");

    dg_client
        .text_to_speech()
        .speak(text, &options, &output_file)
        .await?;

    Ok(())
}