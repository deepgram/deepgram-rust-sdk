/* Expected result from running this example program.
Flux Request ID: <uuid>
Connected: <uuid> (seq: 0)

▶ [Turn 0] START — hint=[en, es]
✓ [Turn 0] END (conf: 0.92): Hola, ¿cómo estás?
  Detected languages: ["es"]
  Active hints: ["en", "es"]
*/

//! Multilingual Flux example.
//!
//! Uses the `flux-general-multi` model with two language hints (`en`,
//! `es`) and prints the per-turn `languages` and `languages_hinted`
//! fields populated by the server.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features listen --example flux_multi_language
//! ```

use std::env;
use std::io::Write;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{
    common::{
        flux_response::{FluxResponse, TurnEvent},
        options::{Encoding, Model, Options},
    },
    Deepgram, DeepgramError,
};

static PATH_TO_FILE: &str = "examples/audio/sample-mono.wav";
static AUDIO_CHUNK_SIZE: usize = 18_063;
static FRAME_DELAY: Duration = Duration::from_millis(100);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .model(Model::FluxGeneralMulti)
        .language_hint(["en", "es"])
        .eot_threshold(0.75)
        .eot_timeout_ms(5_000)
        .build();

    let mut results = dg
        .transcription()
        .flux_request_with_options(options)
        .encoding(Encoding::Linear32)
        .sample_rate(44_100)
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY)
        .await?;

    println!("Flux Request ID: {}", results.request_id());

    while let Some(result) = results.next().await {
        match result? {
            FluxResponse::Connected {
                request_id,
                sequence_id,
            } => {
                println!("Connected: {} (seq: {})", request_id, sequence_id);
            }
            FluxResponse::TurnInfo {
                event,
                turn_index,
                transcript,
                end_of_turn_confidence,
                languages,
                languages_hinted,
                ..
            } => match event {
                TurnEvent::StartOfTurn => {
                    println!(
                        "\n▶ [Turn {}] START — hint={:?}",
                        turn_index, languages_hinted
                    );
                }
                TurnEvent::EndOfTurn => {
                    println!(
                        "\n✓ [Turn {}] END (conf: {:.2}): {}",
                        turn_index, end_of_turn_confidence, transcript
                    );
                    if !languages.is_empty() {
                        println!("  Detected languages: {:?}", languages);
                    }
                    if !languages_hinted.is_empty() {
                        println!("  Active hints: {:?}", languages_hinted);
                    }
                }
                TurnEvent::EagerEndOfTurn => {
                    println!("\n⚡ [Turn {}] EAGER END: {}", turn_index, transcript);
                }
                TurnEvent::TurnResumed => {
                    println!("\n↻ [Turn {}] RESUMED: {}", turn_index, transcript);
                }
                TurnEvent::Update => {
                    if !transcript.is_empty() {
                        print!("\r[Turn {}] UPDATE: {}", turn_index, transcript);
                        std::io::stdout().flush().unwrap();
                    }
                }
                _ => println!("\n[Turn {}] Unknown event: {:?}", turn_index, event),
            },
            FluxResponse::FatalError {
                code, description, ..
            } => {
                eprintln!("Error {}: {}", code, description);
                break;
            }
            FluxResponse::ConfigureSuccess { .. } | FluxResponse::ConfigureFailure { .. } => {
                // Not used in this example — see dynamic_configure.rs.
            }
            _ => {}
        }
    }

    Ok(())
}
