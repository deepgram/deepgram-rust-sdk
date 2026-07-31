//! Idiomatic `futures::Stream` consumption of live transcription results.
//!
//! [`TranscriptionStream`](deepgram::listen::websocket::TranscriptionStream)
//! implements [`futures::Stream`], so live results compose with the async
//! ecosystem via [`StreamExt`] combinators (`filter_map`, `take_while`, `map`,
//! …) exactly like any other stream — no callbacks required.
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key cargo run --example stream_futures --features listen
//! ```

use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{
    common::{
        options::{Encoding, Endpointing, Language, Options},
        stream_response::StreamResponse,
    },
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

    let results = dg_client
        .transcription()
        .stream_request_with_options(options)
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .endpointing(Endpointing::CustomDurationMs(300))
        .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY)
        .await?;

    // Compose with StreamExt: keep only final transcripts and pull out their
    // text, ignoring interim results and non-transcript events.
    let final_transcripts: Vec<String> = results
        .filter_map(|event| async move {
            match event.ok()? {
                StreamResponse::TranscriptResponse {
                    is_final: true,
                    channel,
                    ..
                } => channel
                    .alternatives
                    .into_iter()
                    .next()
                    .map(|alt| alt.transcript)
                    .filter(|t| !t.is_empty()),
                _ => None,
            }
        })
        .collect()
        .await;

    for transcript in final_transcripts {
        println!("{transcript}");
    }

    Ok(())
}
