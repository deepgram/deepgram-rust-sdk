//! Stream text to Deepgram's Text-to-Speech WebSocket and save the audio.
//!
//! Text is sent from one task while audio is written from another, the way
//! an LLM-driven voice agent would feed tokens in while playing audio out.
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key cargo run --example text_to_speech_websocket --features speak
//! ```

use std::env;
use std::fs::File;
use std::io::Write;
use std::time::Duration;

use deepgram::{
    speak::{
        options::{Encoding, Model},
        SpeakResponse,
    },
    Deepgram, DeepgramError,
};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let handle = dg_client
        .text_to_speech()
        .speak_stream()
        .model(Model::Aura2ThaliaEn)
        .encoding(Encoding::Linear16)
        .sample_rate(24000)
        .handle()
        .await?;

    println!("Deepgram request id: {}", handle.request_id());

    // Split the handle so text can be sent while audio is still arriving.
    let (sender, mut events) = handle.split();

    // Producer: send text in pieces — as an LLM might emit tokens — then
    // flush and close. This runs concurrently with the consumer below.
    let producer = tokio::spawn(async move {
        for piece in [
            "Hello, this is streaming text to speech. ",
            "The audio arrives as it is generated, ",
            "while more text is still being sent.",
        ] {
            sender.speak(piece).await?;
            // Simulate an LLM producing text over time.
            tokio::time::sleep(Duration::from_millis(150)).await;
        }
        sender.flush().await?;
        sender.close().await
    });

    // Consumer: raw linear16 PCM at 24 kHz; play with e.g.
    // `ffplay -f s16le -ar 24000 -ac 1 output.raw`.
    let mut file = File::create("output.raw")?;
    let mut audio_bytes = 0usize;

    while let Some(message) = events.receive().await {
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

    producer.await.expect("producer task panicked")?;

    println!("wrote {audio_bytes} bytes of audio to output.raw");

    Ok(())
}
