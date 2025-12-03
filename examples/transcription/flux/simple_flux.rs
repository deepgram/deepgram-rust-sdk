/* Expected result from running this example program.
Flux Request ID: 5add2b9f-42a7-406f-9fac-7ac4be9dc2cb
Connected: 5add2b9f-42a7-406f-9fac-7ac4be9dc2cb (seq: 0)

▶ [Turn 0] START
[Turn 0] UPDATE: Hello from Deepgram. Welcome to our voice AI APIs.I
*/

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

// IMPORTANT: To stream a pre-recorded audio file to Flux, you will need to:
//
// - Specify the path to a mono-channel audio file
// - Determine the sample rate and corresponding chunk size (eg. 88.2 KB/sec divided by 10 chunks per second)
// - Determine the desired interval based on the chunk size (eg. send X bytes every Y milliseconds)
//
// You may receive no transcription or unpredictable transcription results if you do not get these numbers close to real-time.
// 
static PATH_TO_FILE: &str = "examples/audio/sample-mono.wav";
static AUDIO_CHUNK_SIZE: usize = 18_063;
static FRAME_DELAY: Duration = Duration::from_millis(100);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    // Configure Flux for more reliable turn detection
    // - eot_threshold: 0.75 (higher = more reliable, less false positives)
    // - eot_timeout_ms: 5000 (default, allows longer pauses before forcing turn end)
    // - eager_eot_threshold: None (disabled - remove if you want early LLM responses)
    let options = Options::builder()
        .model(Model::FluxGeneralEn)
        .eot_threshold(0.75)
        .eot_timeout_ms(5000)
        .keyterms(["activate", "cancel"])
        // Uncomment below if you want early response generation (increases LLM calls by 50-70%)
        // .eager_eot_threshold(0.7)
        .build();

    let mut results = dg_client
        .transcription()
        .flux_request_with_options(options)
        .encoding(Encoding::Linear32)
        .sample_rate(44100)
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
                words,
                ..
            } => match event {
                TurnEvent::StartOfTurn => {
                    println!("\n▶ [Turn {}] START", turn_index);
                }
                TurnEvent::EndOfTurn => {
                    println!(
                        "\n✓ [Turn {}] END (conf: {:.2}): {}",
                        turn_index, end_of_turn_confidence, transcript
                    );
                    println!("  Words: {}", words.len());
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
                _ => {
                    println!("\n[Turn {}] Unknown event: {:?}", turn_index, event);
                }
            },
            FluxResponse::FatalError {
                code, description, ..
            } => {
                eprintln!("Error {}: {}", code, description);
                break;
            }
            _ => {
                println!("Unknown response type");
            }
        }
    }

    Ok(())
}
