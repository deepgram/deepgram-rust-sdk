use deepgram::{Deepgram, DeepgramError, Language, OptionsBuilder, UrlSource};

const DEEPGRAM_API_KEY: &str = "YOUR_SECRET";
const AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let dg_client = Deepgram::new(DEEPGRAM_API_KEY);

    let options = OptionsBuilder::new()
        .punctuate(true)
        .language(Language::en_US);

    let response = dg_client
        .prerecorded_request(UrlSource { url: AUDIO_URL }, &options)
        .await?;
    println!("{:?}", response);

    Ok(())
}
