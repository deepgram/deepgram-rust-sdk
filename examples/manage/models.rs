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
        let multilingual = match model.multilingual {
            Some(true) => " [multilingual]",
            _ => "",
        };
        println!(
            "  {} ({}) v{}{multilingual}",
            model.name.as_deref().unwrap_or("-"),
            model.canonical_name.as_deref().unwrap_or("-"),
            model.version.as_deref().unwrap_or("-"),
        );
    }

    // `display_name` is the human-readable voice name to show in a picker
    // ("Angus"); `name` is the wire form ("angus").
    println!("TTS voices:");
    for model in &models.tts {
        let metadata = model.metadata.as_ref();
        let display_name = metadata
            .and_then(|m| m.display_name.as_deref())
            .or(model.name.as_deref())
            .unwrap_or("-");
        let accent = metadata
            .and_then(|m| m.accent.as_deref())
            .unwrap_or("unknown accent");
        println!(
            "  {display_name} ({}) — {accent}",
            model.canonical_name.as_deref().unwrap_or("-"),
        );
    }

    // The response also names every language the listed models cover, so a
    // BCP-47 tag from `model.languages` can be shown to a human.
    println!("Languages covered: {}", models.languages.len());
    if let Some(name) = models.languages.get("en-US") {
        println!("  en-US is {name}");
    }

    Ok(())
}
