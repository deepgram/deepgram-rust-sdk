//! Transcribe a hosted audio file and render SRT and WebVTT captions from the
//! response.
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key cargo run --example captions --features listen
//! ```

use std::env;

use deepgram::{
    common::{
        audio_source::AudioSource,
        captions::{srt, webvtt, CaptionOptions},
        options::{Model, Options},
    },
    Deepgram, DeepgramError,
};

static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let source = AudioSource::from_url(AUDIO_URL);

    // Word-level timestamps (returned by default) drive the caption timing;
    // `smart_format` and `punctuate` make the caption text readable.
    let options = Options::builder()
        .model(Model::Nova3)
        .smart_format(true)
        .punctuate(true)
        .build();

    let response = dg_client
        .transcription()
        .prerecorded(source, &options)
        .await?;

    // At most 8 words per cue by default; tune with `CaptionOptions`.
    let caption_options = CaptionOptions::default();

    println!("===== SRT =====");
    println!("{}", srt(&response, &caption_options));

    println!("===== WebVTT =====");
    println!("{}", webvtt(&response, &caption_options));

    Ok(())
}
