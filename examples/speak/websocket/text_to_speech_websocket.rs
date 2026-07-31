//! Stream text to Deepgram's Text-to-Speech WebSocket and save the audio.
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key cargo run --example text_to_speech_websocket --features speak
//! ```

use std::env;
use std::fs::File;
use std::io::Write;

use deepgram::{
    speak::{options::Encoding, SpeakResponse},
    Deepgram, DeepgramError,
};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let mut handle = dg_client
        .text_to_speech()
        .speak_stream()
        .encoding(Encoding::Linear16)
        .sample_rate(24000)
        .handle()
        .await?;

    println!("Deepgram request id: {}", handle.request_id());

    // Send text in chunks — as an LLM might — then flush and close.
    handle
        .speak("Hello, this is streaming text to speech. ")
        .await?;
    handle
        .speak("The audio arrives as it is generated.")
        .await?;
    handle.flush().await?;
    handle.close().await?;

    // Raw linear16 PCM at 24 kHz; play with e.g.
    // `ffplay -f s16le -ar 24000 -ac 1 output.raw`.
    let mut file = File::create("output.raw")?;
    let mut audio_bytes = 0usize;

    while let Some(message) = handle.receive().await {
        match message? {
            SpeakResponse::Audio(chunk) => {
                audio_bytes += chunk.len();
                file.write_all(&chunk)?;
            }
            SpeakResponse::Metadata { request_id, .. } => {
                println!("metadata: request_id={request_id}");
            }
            SpeakResponse::Flushed { .. } => println!("flushed"),
            SpeakResponse::Warning { description, .. } => {
                println!("warning: {description:?}");
            }
            other => println!("event: {other:?}"),
        }
    }

    println!("wrote {audio_bytes} bytes of audio to output.raw");

    Ok(())
}
