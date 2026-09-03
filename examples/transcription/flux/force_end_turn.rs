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
//! Pumps a WAV file into the Flux WebSocket and, on the first non-empty
//! `Update` (proof that a turn is active), sends a `ForceEndTurn`
//! message to end the current turn immediately. The server replies
//! with a standard `EndOfTurn`
//! [`deepgram::common::flux_response::FluxResponse::TurnInfo`] event whose
//! `trigger` is [`deepgram::common::flux_response::TurnTrigger::Manual`];
//! turns that end naturally afterwards carry `trigger: Model`.
//!
//! The audio is sent as a containerized WAV, so no `encoding` or
//! `sample_rate` parameters are set — the server detects the format
//! from the container. (Those parameters are for raw, headerless audio
//! such as a microphone PCM stream; see the `microphone_flux` example.)
//!
//! `ForceEndTurn` operates on the turn currently in progress — sent
//! while no turn is active (e.g. during leading silence), it is
//! ignored. `Ok(())` from `force_end_turn()` means only that the
//! message was queued locally: this example therefore treats the
//! session as successful only after observing an `EndOfTurn` with
//! `trigger: Manual`, and exits nonzero otherwise.
//!
//! `ForceEndTurn` is useful when your application has a definitive
//! turn-end signal of its own: a push-to-talk release, a DTMF tone, a
//! "send" button, or an external VAD/endpointing stack.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features listen --example flux_force_end_turn
//! ```

use std::env;
use std::error::Error;
use std::time::Duration;

use deepgram::{
    common::{
        flux_response::{FluxResponse, TurnEvent, TurnTrigger},
        options::{Model, Options},
    },
    Deepgram,
};

static PATH_TO_FILE: &str = "examples/audio/bueller-mono.wav";
static AUDIO_CHUNK_SIZE: usize = 8_820; // 100ms @ 44.1 kHz Linear16 mono
static FRAME_DELAY: Duration = Duration::from_millis(100);
/// How long to wait for the server to finish the session after
/// `CloseStream` before giving up.
static SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .model(Model::FluxGeneralEn)
        .eot_threshold(0.75)
        .eot_timeout_ms(5_000)
        .build();

    // The file is a containerized WAV, so encoding/sample_rate are not set.
    let mut handle = dg
        .transcription()
        .flux_request_with_options(options)
        .handle()
        .await?;

    println!("Flux Request ID: {}", handle.request_id());

    // Read the file up front and slice it into pacable chunks.
    let audio_bytes = tokio::fs::read(PATH_TO_FILE).await?;
    let mut offset = 0usize;
    let mut shutdown_deadline: Option<tokio::time::Instant> = None;
    let mut sent_force_end_turn = false;
    let mut saw_manual_end_of_turn = false;

    let mut frame_timer = tokio::time::interval(FRAME_DELAY);
    frame_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = frame_timer.tick(), if shutdown_deadline.is_none() => {
                if offset >= audio_bytes.len() {
                    handle.close_stream().await?;
                    // The audio is done; the server has SHUTDOWN_TIMEOUT to
                    // deliver the remaining results and end the session.
                    shutdown_deadline =
                        Some(tokio::time::Instant::now() + SHUTDOWN_TIMEOUT);
                    continue;
                }
                let end = (offset + AUDIO_CHUNK_SIZE).min(audio_bytes.len());
                let chunk = audio_bytes[offset..end].to_vec();
                offset = end;
                handle.send_data(chunk).await?;
            }
            _ = tokio::time::sleep_until(shutdown_deadline.unwrap_or_else(tokio::time::Instant::now)),
                if shutdown_deadline.is_some() =>
            {
                return Err(format!(
                    "server did not end the session within {SHUTDOWN_TIMEOUT:?} of CloseStream"
                )
                .into());
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
                            if trigger == Some(TurnTrigger::Manual) {
                                saw_manual_end_of_turn = true;
                            }
                        }
                        _ => {}
                    },
                    FluxResponse::FatalError {
                        code, description, ..
                    } => {
                        return Err(format!("fatal Flux error {code}: {description}").into());
                    }
                    _ => {}
                }
            }
        }
    }

    // Success requires server confirmation, not just a locally queued
    // message: the forced turn must have ended with trigger: Manual.
    if !saw_manual_end_of_turn {
        return Err("session ended without an EndOfTurn with trigger: Manual — \
             ForceEndTurn was not demonstrated"
            .into());
    }

    Ok(())
}
