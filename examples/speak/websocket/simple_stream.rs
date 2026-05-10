/* Expected result from running this example program.
Connected. dg-request-id: Some(<uuid>)
Metadata: model=aura-asteria-en uuid=<...>
Wrote 38400 bytes of audio to /tmp/deepgram_speak.pcm
Flushed (sequence_id=1)
*/

//! Streaming TTS example.
//!
//! Connects to the Speak WebSocket, sends a single line of text, flushes
//! to force the server to emit all audio, writes the audio bytes to a
//! local `.pcm` file (linear16, 24kHz, mono), then closes.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features speak --example speak_websocket_simple
//! ```
//!
//! Play the output back with e.g. `ffplay -f s16le -ar 24000 -ac 1 /tmp/deepgram_speak.pcm`.

use std::env;
use std::fs::File;
use std::io::Write as _;

use futures::stream::StreamExt;

use deepgram::speak::{
    options::{Encoding, Model},
    response::SpeakResponse,
};
use deepgram::{Deepgram, DeepgramError};

const OUTPUT_PATH: &str = "/tmp/deepgram_speak.pcm";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");

    let dg = Deepgram::new(&api_key)?;
    let (mut handle, mut stream) = dg
        .text_to_speech()
        .websocket()
        .model(Model::aura_asteria_en())
        .encoding(Encoding::Linear16)
        .sample_rate(24_000)
        .start()
        .await?;

    println!("Connected. dg-request-id: {:?}", handle.request_id());

    handle
        .send_text("Hello from Deepgram streaming text-to-speech.")
        .await?;
    handle.flush().await?;

    let mut file = File::create(OUTPUT_PATH).expect("create output file");
    let mut total_bytes = 0usize;

    while let Some(event) = stream.next().await {
        match event? {
            SpeakResponse::Metadata(m) => {
                println!("Metadata: model={} uuid={}", m.model_name, m.model_uuid);
            }
            SpeakResponse::Audio(bytes) => {
                file.write_all(&bytes).expect("write audio chunk");
                total_bytes += bytes.len();
            }
            SpeakResponse::Flushed(f) => {
                println!("Wrote {} bytes of audio to {}", total_bytes, OUTPUT_PATH);
                println!("Flushed (sequence_id={})", f.sequence_id);
                break;
            }
            SpeakResponse::Cleared(c) => {
                println!("Cleared (sequence_id={})", c.sequence_id);
            }
            SpeakResponse::Warning(w) => {
                eprintln!("Warning [{}]: {}", w.code, w.description);
            }
            // SpeakResponse is #[non_exhaustive].
            _ => {}
        }
    }

    handle.close().await?;
    Ok(())
}
