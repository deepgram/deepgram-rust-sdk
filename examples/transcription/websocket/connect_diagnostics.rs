//! Stream a file with connect diagnostics enabled and a caller-side connect
//! timeout.
//!
//! Run with:
//! `cargo run --features connect-diagnostics --example connect_diagnostics`
//!
//! One JSON line is emitted per connect attempt — on success, on failure,
//! and when the 1-second timeout below cancels the connect. Requires the
//! `DEEPGRAM_API_KEY` environment variable.

use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{
    common::options::{Encoding, Language, Options},
    diagnostics::ConnectRecord,
    Deepgram, DeepgramError,
};

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";
static AUDIO_CHUNK_SIZE: usize = 3174;
static FRAME_DELAY: Duration = Duration::from_millis(16);
static CONNECT_TIMEOUT: Duration = Duration::from_secs(1);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    // Diagnostics sink: an unbounded channel plus a writer task that prints
    // one JSON line per connect attempt. A production integration would
    // append these lines to a JSONL file instead.
    let (diag_tx, mut diag_rx) = tokio::sync::mpsc::unbounded_channel::<ConnectRecord>();
    let writer = tokio::spawn(async move {
        while let Some(record) = diag_rx.recv().await {
            eprintln!(
                "{}",
                serde_json::to_string(&record).expect("connect records serialize to JSON")
            );
        }
    });

    let options = Options::builder()
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let transcription = dg_client.transcription();
    let request = transcription
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
    match tokio::time::timeout(CONNECT_TIMEOUT, request).await {
        Ok(Ok(mut results)) => {
            println!("Deepgram Request ID: {}", results.request_id());
            while let Some(result) = results.next().await {
                println!("got: {result:?}");
            }
        }
        Ok(Err(err)) => eprintln!("Deepgram connection failed: {err:?}"),
        Err(_) => eprintln!("Deepgram connection attempt timed out after {CONNECT_TIMEOUT:?}"),
    }

    // Dropping the last sender ends the writer task once the queue drains.
    drop(diag_tx);
    let _ = writer.await;

    Ok(())
}
