use deepgram::{
    prerecorded::{Language, Options, UrlSource},
    Deepgram, DeepgramError,
};
use std::env;

static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let source = UrlSource { url: AUDIO_URL };

    let options = Options::builder()
        .punctuate(true)
        .language(Language::en_US)
        .build();

    let response = dg_client
        .callback_request(&source, &options, "https://example.com/")
        .await?;

    println!("{}", response.request_id);

    Ok(())
}
