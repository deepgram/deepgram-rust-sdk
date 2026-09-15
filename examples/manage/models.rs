//! List the STT and TTS models available on your Deepgram account.
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key cargo run --example models --features manage
//! ```

use std::env;

use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let models = dg_client.models().get_models().await?;

    // The Models API marks these display fields optional, so fall back to a
    // placeholder rather than failing when one is absent.
    println!("STT models:");
    for model in &models.stt {
        println!(
            "  {} ({}) v{}",
            model.name.as_deref().unwrap_or("-"),
            model.canonical_name.as_deref().unwrap_or("-"),
            model.version.as_deref().unwrap_or("-"),
        );
    }

    println!("TTS models:");
    for model in &models.tts {
        let accent = model
            .metadata
            .as_ref()
            .and_then(|m| m.accent.as_deref())
            .unwrap_or("unknown accent");
        println!(
            "  {} ({}) — {accent}",
            model.name.as_deref().unwrap_or("-"),
            model.canonical_name.as_deref().unwrap_or("-"),
        );
    }

    Ok(())
}
