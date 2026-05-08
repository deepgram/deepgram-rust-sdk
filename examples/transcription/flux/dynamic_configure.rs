/* Expected result from running this example program.
Flux Request ID: <uuid>
Connected: <uuid> (seq: 0)
[Turn 0] update: hello
[Turn 0] EndOfTurn: Hello.
Sending Configure: eot_threshold=0.85, keyterms=["weather", "forecast"]
ConfigureSuccess: thresholds eot=Some(0.85) keyterms=["weather", "forecast"]
[Turn 1] EndOfTurn: What's the weather forecast?
*/

//! Dynamic Flux `Configure` example.
//!
//! Pumps a file into the Flux WebSocket and, after the first turn ends,
//! sends a `Configure` message to update the EOT threshold and
//! keyterms mid-session. Demonstrates the
//! [`crate::common::flux_response::FluxResponse::ConfigureSuccess`]
//! acknowledgement.
//!
//! Single-task `tokio::select!` over the [`FluxHandle`]: audio frames
//! are sent on a paced timer while events are received from the
//! handle's response queue.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features listen --example flux_dynamic_configure
//! ```

use std::env;
use std::time::Duration;

use deepgram::{
    common::{
        flux_response::{ConfigureThresholds, FluxResponse, TurnEvent},
        options::{Encoding, Model, Options},
    },
    listen::flux::ConfigureRequest,
    Deepgram, DeepgramError,
};

static PATH_TO_FILE: &str = "examples/audio/bueller-mono.wav";
static AUDIO_CHUNK_SIZE: usize = 8_820; // 100ms @ 44.1 kHz Linear16 mono
static FRAME_DELAY: Duration = Duration::from_millis(100);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .model(Model::FluxGeneralEn)
        .eot_threshold(0.75)
        .eot_timeout_ms(5_000)
        .build();

    let mut handle = dg
        .transcription()
        .flux_request_with_options(options)
        .encoding(Encoding::Linear16)
        .sample_rate(44_100)
        .handle()
        .await?;

    println!("Flux Request ID: {}", handle.request_id());

    // Read the file up front and slice it into pacable chunks.
    let audio_bytes = tokio::fs::read(PATH_TO_FILE).await?;
    let mut offset = 0usize;
    let mut audio_done = false;
    let mut sent_configure = false;

    let mut frame_timer = tokio::time::interval(FRAME_DELAY);
    frame_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = frame_timer.tick(), if !audio_done => {
                if offset >= audio_bytes.len() {
                    handle.close_stream().await?;
                    audio_done = true;
                    continue;
                }
                let end = (offset + AUDIO_CHUNK_SIZE).min(audio_bytes.len());
                let chunk = audio_bytes[offset..end].to_vec();
                offset = end;
                handle.send_data(chunk).await?;
            }
            response = handle.receive() => {
                let Some(response) = response else { break };
                match response? {
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
                        ..
                    } => match event {
                        TurnEvent::Update if !transcript.is_empty() => {
                            println!("[Turn {}] update: {}", turn_index, transcript);
                        }
                        TurnEvent::EndOfTurn => {
                            println!("[Turn {}] EndOfTurn: {}", turn_index, transcript);
                            if !sent_configure {
                                sent_configure = true;
                                println!(
                                    "Sending Configure: eot_threshold=0.85, \
                                     keyterms=[\"weather\", \"forecast\"]"
                                );
                                let req = ConfigureRequest::new()
                                    .with_thresholds(
                                        ConfigureThresholds::new().with_eot_threshold(0.85),
                                    )
                                    .with_keyterms(["weather", "forecast"]);
                                handle.configure(req).await?;
                            }
                        }
                        _ => {}
                    },
                    FluxResponse::ConfigureSuccess {
                        thresholds,
                        keyterms,
                        ..
                    } => {
                        println!(
                            "ConfigureSuccess: thresholds eot={:?} keyterms={:?}",
                            thresholds.eot_threshold, keyterms
                        );
                    }
                    FluxResponse::ConfigureFailure { sequence_id, .. } => {
                        eprintln!("ConfigureFailure (seq={sequence_id})");
                    }
                    FluxResponse::FatalError {
                        code, description, ..
                    } => {
                        eprintln!("FatalError {code}: {description}");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
