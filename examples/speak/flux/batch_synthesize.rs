/* Expected result from running this example program.
Synthesizing with flux-haley-en (batch REST)...
Audio saved to "flux-tts-batch.mp3"
*/

//! Flux TTS batch (REST) example.
//!
//! Synthesizes a complete block of text into a single audio file with
//! `POST /v2/speak`. Use the batch transport for pre-rendering fixed
//! audio (IVR prompts, notifications, narration); use the streaming
//! WebSocket transport (see the `flux_tts_websocket` example) for live,
//! interruptible, turn-based synthesis.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features speak --example flux_tts_batch
//! ```

use std::env;

use deepgram::{
    speak::flux::options::{Model, Options},
    Deepgram, DeepgramError,
};

const TEXT: &str = "Hello! This audio was synthesized in a single batch request \
                    with Deepgram's Flux text to speech API.";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let dg = Deepgram::new(&api_key)?;

    // model is required; encoding defaults to mp3 on the batch transport.
    let options = Options::builder(Model::FluxHaleyEn).build();

    let output_file = std::path::Path::new("flux-tts-batch.mp3");

    println!("Synthesizing with flux-haley-en (batch REST)...");
    dg.text_to_speech()
        .flux_speak_to_file(TEXT, &options, output_file)
        .await?;

    println!("Audio saved to {output_file:?}");

    Ok(())
}
