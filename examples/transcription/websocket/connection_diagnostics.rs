//! Log WebSocket connection diagnostics for a live transcription request.
//!
//! Once the WebSocket upgrade completes, `connection_info()` exposes connection
//! metadata (request ID, final URL, local/peer socket addresses, and total
//! connect-plus-upgrade duration). Emitting it as a single JSONL record makes it
//! easy to collect from production traffic and share with Deepgram when
//! troubleshooting connectivity or latency.

use std::env;
use std::time::Duration;

use futures::stream::StreamExt;
use serde_json::json;

use deepgram::{
    common::options::{Encoding, Endpointing, Language, Options},
    Deepgram, DeepgramError,
};

static PATH_TO_FILE: &str = "examples/audio/bueller.wav";
static AUDIO_CHUNK_SIZE: usize = 3174;
static FRAME_DELAY: Duration = Duration::from_millis(16);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let options = Options::builder()
        .smart_format(true)
        .language(Language::en_US)
        .build();

    let mut results = dg_client
        .transcription()
        .stream_request_with_options(options)
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .endpointing(Endpointing::CustomDurationMs(300))
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY)
        .await?;

    // Emit connection diagnostics as a single JSONL record.
    let info = results.connection_info();
    let record = json!({
        "request_id": info.request_id.to_string(),
        "url": info.url,
        "local_addr": info.local_addr.map(|addr| addr.to_string()),
        "peer_addr": info.peer_addr.map(|addr| addr.to_string()),
        "connect_duration_ms": info.connect_duration.as_millis(),
    });
    println!("{record}");

    while let Some(result) = results.next().await {
        println!("got: {result:?}");
    }

    Ok(())
}
