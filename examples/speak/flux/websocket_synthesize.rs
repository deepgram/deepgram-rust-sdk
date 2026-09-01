/* Expected result from running this example program.
Flux TTS Request ID: <uuid>
Connected: model flux-haley-en (version <version>)
SpeechStarted: dg_sp_<hex>
Flushed: dg_sp_<hex>
SpeechMetadata: dg_sp_<hex> (2340ms, 52 billable chars)
SessionMetadata: 2340ms total audio, 52 total billable chars
Audio saved to "flux-tts-websocket.wav" (<n> bytes of linear16 @ 24000 Hz)
*/

//! Flux TTS streaming (WebSocket) example.
//!
//! Streams text into the `/v2/speak` WebSocket the way a voice agent
//! streams LLM tokens: several `Speak` messages, then a `Flush` to end
//! the turn. The server assigns the turn a `speech_id` and reports its
//! lifecycle (`SpeechStarted`, binary audio frames, `Flushed`,
//! `SpeechMetadata`); a graceful `Close` drains remaining audio and
//! ends with `SessionMetadata`. The synthesized audio is collected and
//! saved as a playable WAV file — but only after the session completed:
//! a fatal error, a session that ends without its terminal
//! `SessionMetadata`, or empty audio exits nonzero and writes nothing.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features speak --example flux_tts_websocket
//! ```

use std::env;
use std::error::Error;
use std::io::Write;

use deepgram::{
    speak::flux::{
        options::{Encoding, Model, Options},
        response::FluxSpeakResponse,
    },
    Deepgram,
};

static SAMPLE_RATE: u32 = 24_000;

/// The agent reply to synthesize, in the token-sized pieces an LLM
/// would stream. The server handles flush placement at sentence and
/// clause boundaries internally — you only mark the end of the turn.
static TOKENS: &[&str] = &[
    "Hello! ",
    "This audio was synthesized ",
    "over Deepgram's Flux ",
    "text to speech websocket.",
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = env::var("DEEPGRAM_API_KEY")?;
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder(Model::FluxHaleyEn)
        .encoding(Encoding::Linear16)
        .sample_rate(SAMPLE_RATE)
        .build();

    let speak = dg.text_to_speech();
    let mut handle = speak.flux_request(options).handle().await?;

    println!("Flux TTS Request ID: {}", handle.request_id());

    // Stream the turn's text in, then end the turn and close. Events
    // and audio are read below; nothing here blocks on the server.
    for token in TOKENS {
        handle.speak(*token).await?;
    }
    handle.flush().await?;
    handle.close().await?;

    let mut audio: Vec<u8> = Vec::new();
    let mut saw_session_metadata = false;

    while let Some(response) = handle.receive().await {
        match response? {
            FluxSpeakResponse::Audio(chunk) => {
                audio.extend_from_slice(&chunk);
            }
            FluxSpeakResponse::Connected {
                model_name,
                model_version,
                ..
            } => {
                println!("Connected: model {model_name} (version {model_version})");
            }
            FluxSpeakResponse::SpeechStarted { speech_id, .. } => {
                println!("SpeechStarted: {speech_id}");
            }
            FluxSpeakResponse::Flushed { speech_id, .. } => {
                println!("Flushed: {speech_id}");
            }
            FluxSpeakResponse::SpeechMetadata(metadata) => {
                println!(
                    "SpeechMetadata: {} ({}ms, {} billable chars)",
                    metadata.speech_id,
                    metadata.audio_duration_ms,
                    metadata.billable_character_count
                );
            }
            FluxSpeakResponse::SessionMetadata {
                total_audio_duration_ms,
                total_billable_character_count,
                ..
            } => {
                println!(
                    "SessionMetadata: {total_audio_duration_ms}ms total audio, \
                     {total_billable_character_count} total billable chars"
                );
                // SessionMetadata is the terminal event of a graceful
                // close — the session completed.
                saw_session_metadata = true;
            }
            FluxSpeakResponse::Warning {
                code, description, ..
            } => {
                eprintln!("Warning {code}: {description}");
            }
            FluxSpeakResponse::FatalError {
                code, description, ..
            } => {
                return Err(format!("fatal Flux TTS error {code}: {description}").into());
            }
            _ => {}
        }
    }

    // Write the artifact only for a complete, non-empty synthesis: the
    // graceful close must have finished with SessionMetadata, and audio
    // must have actually been produced. Otherwise exit nonzero with no
    // output file.
    if !saw_session_metadata {
        return Err("session ended before its terminal SessionMetadata event".into());
    }
    if audio.is_empty() {
        return Err("session completed but produced no audio".into());
    }

    let output_file = std::path::Path::new("flux-tts-websocket.wav");
    let mut file = std::fs::File::create(output_file)?;
    file.write_all(&wav_header_linear16_mono(SAMPLE_RATE, audio.len() as u32))?;
    file.write_all(&audio)?;

    println!(
        "Audio saved to {:?} ({} bytes of linear16 @ {} Hz)",
        output_file,
        audio.len(),
        SAMPLE_RATE
    );

    Ok(())
}

/// Build a 44-byte WAV header for mono 16-bit PCM (`linear16`) audio.
/// The websocket emits raw audio with no container, so a header is
/// needed to make the file playable.
fn wav_header_linear16_mono(sample_rate: u32, data_len: u32) -> [u8; 44] {
    const CHANNELS: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;
    let byte_rate = sample_rate * u32::from(CHANNELS) * u32::from(BITS_PER_SAMPLE / 8);
    let block_align = CHANNELS * (BITS_PER_SAMPLE / 8);

    let mut header = [0u8; 44];
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&(36 + data_len).to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16u32.to_le_bytes()); // fmt chunk size
    header[20..22].copy_from_slice(&1u16.to_le_bytes()); // format code 1 = PCM
    header[22..24].copy_from_slice(&CHANNELS.to_le_bytes());
    header[24..28].copy_from_slice(&sample_rate.to_le_bytes());
    header[28..32].copy_from_slice(&byte_rate.to_le_bytes());
    header[32..34].copy_from_slice(&block_align.to_le_bytes());
    header[34..36].copy_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&data_len.to_le_bytes());
    header
}
