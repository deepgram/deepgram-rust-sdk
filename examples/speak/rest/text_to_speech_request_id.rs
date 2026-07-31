//! Generate speech and read the response metadata — including the
//! `dg-request-id` — using [`Speak::speak_to_file_with_metadata`].
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key cargo run --example text_to_speech_request_id --features speak
//! ```

use std::{env, path::Path};

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

    let text = "Hello, how can I help you today?";
    let output_file = Path::new("your_output_file.wav");

    let metadata = dg_client
        .text_to_speech()
        .speak_to_file_with_metadata(text, &options, output_file)
        .await?;

    // The request id is essential for debugging and support requests.
    match metadata.request_id() {
        Some(request_id) => println!("dg-request-id: {request_id}"),
        None => println!("No dg-request-id header returned"),
    }
    println!("model: {:?}", metadata.model_name);
    println!("characters billed: {:?}", metadata.char_count);

    Ok(())
}
