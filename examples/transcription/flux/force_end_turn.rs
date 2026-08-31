/* Expected result from running this example program.
Flux Request ID: <uuid>
Connected: <uuid> (seq: 0)
[Turn 0] update: Yep.
Sending ForceEndTurn
[Turn 0] EndOfTurn (trigger: Some(Manual)): Yep.
[Turn 1] EndOfTurn (trigger: Some(Model)): And I said it before, and I'll say it again. ...
*/

//! Flux `ForceEndTurn` example.
//!
//! Pumps a file into the Flux WebSocket and, on the first non-empty
//! `Update` (proof that a turn is active), sends a `ForceEndTurn`
//! message to end the current turn immediately. The server replies
//! with a standard `EndOfTurn`
//! [`crate::common::flux_response::FluxResponse::TurnInfo`] event whose
//! `trigger` is [`crate::common::flux_response::TurnTrigger::Manual`];
//! turns that end naturally afterwards carry `trigger: Model`.
//!
//! `ForceEndTurn` operates on the turn currently in progress — sent
//! while no turn is active (e.g. during leading silence), it is
//! ignored.
//!
//! `ForceEndTurn` is useful when your application has a definitive
//! turn-end signal of its own: a push-to-talk release, a DTMF tone, a
//! "send" button, or an external VAD/endpointing stack.
//!
//! NOTE: `ForceEndTurn` is gated per deployment. On a deployment where
//! it is not enabled, the server responds with a fatal
//! `UNPARSABLE_CLIENT_MESSAGE` error and closes the connection.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features listen --example flux_force_end_turn
//! ```

use std::env;
use std::time::Duration;

use deepgram::{
    common::{
        flux_response::{FluxResponse, TurnEvent},
        options::{Encoding, Model, Options},
    },
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
    let mut sent_force_end_turn = false;

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
                        trigger,
                        ..
                    } => match event {
                        TurnEvent::Update if !transcript.is_empty() => {
                            println!("[Turn {}] update: {}", turn_index, transcript);

                            // A real application would send ForceEndTurn on
                            // its own signal (push-to-talk release, DTMF
                            // tone, UI event). Here the first non-empty
                            // Update stands in for that signal — it also
                            // proves a turn is active, which ForceEndTurn
                            // requires (sent with no active turn, it is
                            // ignored).
                            if !sent_force_end_turn {
                                sent_force_end_turn = true;
                                println!("Sending ForceEndTurn");
                                handle.force_end_turn().await?;
                            }
                        }
                        TurnEvent::EndOfTurn => {
                            println!(
                                "[Turn {}] EndOfTurn (trigger: {:?}): {}",
                                turn_index, trigger, transcript
                            );
                        }
                        _ => {}
                    },
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
