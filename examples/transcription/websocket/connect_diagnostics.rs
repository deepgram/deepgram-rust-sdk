//! Stream a file with connect diagnostics enabled and a caller-side connect
//! timeout.
//!
//! Run with:
//! `cargo run --features connect-diagnostics --example connect_diagnostics`
//!
//! Stdout carries diagnostic JSON records only — one JSON object per line,
//! one line per connect attempt (success, failure, or a connect cancelled by
//! the 1-second timeout below). All human-readable output (transcripts,
//! progress, errors) goes to stderr, so stdout can be redirected to a JSONL
//! file:
//!
//! `cargo run --features connect-diagnostics --example connect_diagnostics > connects.jsonl`
//!
//! The process exits nonzero when the connection, the stream, or the record
//! writer fails. Requires the `DEEPGRAM_API_KEY` environment variable.

use std::env;
use std::error::Error;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{
    common::options::{Encoding, Language, Options},
    diagnostics::ConnectRecord,
    Deepgram,
};

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";
static AUDIO_CHUNK_SIZE: usize = 3174;
static FRAME_DELAY: Duration = Duration::from_millis(16);
static CONNECT_TIMEOUT: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let deepgram_api_key = env::var("DEEPGRAM_API_KEY")?;

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    // Diagnostics sink: an unbounded channel plus a writer task that prints
    // one JSON line per connect attempt to stdout — nothing else is written
    // there. A production integration would append these lines to a JSONL
    // file instead.
    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let writer = tokio::spawn(async move {
        while let Some(record) = diag_rx.recv().await {
            println!("{}", serde_json::to_string(&record)?);
        }
        Ok::<(), serde_json::Error>(())
    });

    let options = Options::builder()
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let request = dg_client
        .transcription()
        .stream_request_with_options(options)
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .diagnostics(diag_tx.clone())
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY);

    // A caller-side connect budget. If it fires, the connect future is
    // dropped mid-flight — the diagnostics record is emitted regardless,
    // carrying the furthest phase reached and all completed phase timings.
    let outcome: Result<(), Box<dyn Error>> = match tokio::time::timeout(CONNECT_TIMEOUT, request)
        .await
    {
        Ok(Ok(mut results)) => {
            eprintln!("Deepgram Request ID: {}", results.request_id());
            let mut stream_outcome = Ok(());
            while let Some(result) = results.next().await {
                match result {
                    Ok(response) => eprintln!("got: {response:?}"),
                    Err(err) => {
                        stream_outcome = Err(err.into());
                        break;
                    }
                }
            }
            stream_outcome
        }
        Ok(Err(err)) => Err(format!("Deepgram connection failed: {err}").into()),
        Err(_) => {
            Err(format!("Deepgram connection attempt timed out after {CONNECT_TIMEOUT:?}").into())
        }
    };

    // Dropping the last sender ends the writer task once the queue drains,
    // so every record — including one for a failed or cancelled attempt —
    // reaches stdout before the process exits.
    drop(diag_tx);
    let writer_outcome = writer.await;

    // The operational failure is the primary error; a writer failure only
    // surfaces when the session itself succeeded.
    outcome?;
    writer_outcome??;

    Ok(())
}
