/* Expected result from running this example program.
Connected. dg-request-id: Some(<uuid>)
Sending sentence 1, then Flush
Flushed (sequence_id=1)
Received N audio bytes for sentence 1
Sending sentence 2, then Clear (we don't want this audio)
Cleared (sequence_id=2)
Sending sentence 3, then Flush
Flushed (sequence_id=3)
Received N audio bytes for sentence 3
Done.
*/

//! Demonstrates the `Flush` and `Clear` control messages on the Speak
//! WebSocket: send three sentences, but use `Clear` to discard the
//! second mid-generation. The audio buffer is flushed between each
//! sentence so its boundaries are observable.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features speak --example speak_websocket_flush_clear
//! ```

use std::env;

use futures::stream::StreamExt;

use deepgram::speak::{
    options::{Encoding, Model},
    response::SpeakResponse,
};
use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");

    let dg = Deepgram::new(&api_key)?;
    let (mut handle, mut stream) = dg
        .text_to_speech()
        .websocket()
        .model(Model::aura_asteria_en())
        .encoding(Encoding::Linear16)
        .sample_rate(24_000)
        .start()
        .await?;

    println!("Connected. dg-request-id: {:?}", handle.request_id());

    println!("Sending sentence 1, then Flush");
    handle.send_text("This is the first sentence.").await?;
    handle.flush().await?;
    drain_until_flushed(&mut stream, 1).await?;

    println!("Sending sentence 2, then Clear (we don't want this audio)");
    handle
        .send_text("This is the second sentence and we will discard it.")
        .await?;
    handle.clear().await?;
    drain_until_cleared(&mut stream).await?;

    println!("Sending sentence 3, then Flush");
    handle.send_text("This is the third sentence.").await?;
    handle.flush().await?;
    drain_until_flushed(&mut stream, 3).await?;

    println!("Done.");
    handle.close().await?;
    Ok(())
}

/// Read events until a `Flushed` arrives, counting audio bytes along the way.
async fn drain_until_flushed(
    stream: &mut deepgram::speak::websocket::SpeakStream,
    sentence: usize,
) -> Result<(), DeepgramError> {
    let mut audio_bytes = 0usize;
    while let Some(event) = stream.next().await {
        match event? {
            SpeakResponse::Audio(bytes) => audio_bytes += bytes.len(),
            SpeakResponse::Flushed(f) => {
                println!("Flushed (sequence_id={})", f.sequence_id);
                println!(
                    "Received {} audio bytes for sentence {}",
                    audio_bytes, sentence
                );
                return Ok(());
            }
            SpeakResponse::Metadata(_) => {}
            SpeakResponse::Cleared(c) => {
                println!("Unexpected Cleared (sequence_id={})", c.sequence_id);
            }
            SpeakResponse::Warning(w) => {
                eprintln!("Warning [{}]: {}", w.code, w.description);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Read events until a `Cleared` arrives.
async fn drain_until_cleared(
    stream: &mut deepgram::speak::websocket::SpeakStream,
) -> Result<(), DeepgramError> {
    while let Some(event) = stream.next().await {
        match event? {
            SpeakResponse::Cleared(c) => {
                println!("Cleared (sequence_id={})", c.sequence_id);
                return Ok(());
            }
            SpeakResponse::Audio(_) => {
                // Audio that arrived before Clear took effect.
            }
            SpeakResponse::Metadata(_) => {}
            SpeakResponse::Flushed(f) => {
                println!("Unexpected Flushed (sequence_id={})", f.sequence_id);
            }
            SpeakResponse::Warning(w) => {
                eprintln!("Warning [{}]: {}", w.code, w.description);
            }
            _ => {}
        }
    }
    Ok(())
}
