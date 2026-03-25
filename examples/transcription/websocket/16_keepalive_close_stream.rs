/// Example: WebSocket Keep-Alive with Close Stream
///
/// Demonstrates that enabling keep_alive() and then calling close_stream()
/// works correctly without panicking — even when the keep-alive timer fires
/// after the channel has been closed.
///
/// This validates the fix for the race condition where close_stream() closes
/// the internal channel, and a subsequent keep-alive send would previously
/// panic on .expect().
///
/// Usage:
///   DEEPGRAM_API_KEY=your-key cargo run --example 16_keepalive_close_stream
use std::env;
use std::time::Duration;

use deepgram::{
    common::options::{Encoding, Endpointing, Options},
    Deepgram, DeepgramError,
};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let options = Options::builder()
        .query_params([("mip_opt_out".to_string(), "true".to_string())])
        .build();

    println!("Connecting to Deepgram with keep-alive enabled...");

    let mut handle = dg_client
        .transcription()
        .stream_request_with_options(options)
        .encoding(Encoding::Linear16)
        .endpointing(Endpointing::Disabled)
        .keep_alive()
        .handle()
        .await?;

    println!("Connected. Request ID: {}", handle.request_id());

    // Brief pause — no audio is sent, so the worker only has the keep-alive
    // timer ticking. This simulates a connection that is idle before being
    // torn down.
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("Closing stream...");
    handle.close_stream().await?;
    println!("Stream closed.");

    // Wait longer than the 3s keep-alive interval so the keep-alive send
    // path runs after close. Before the fix this would panic; now the send
    // error is silently ignored and the worker exits cleanly.
    println!("Waiting for keep-alive timer to fire (>3s)...");
    tokio::time::sleep(Duration::from_secs(4)).await;

    println!("No panic — keep-alive + close_stream works correctly.");

    Ok(())
}
