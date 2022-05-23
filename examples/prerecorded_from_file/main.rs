use deepgram::{BufferSource, Deepgram, DeepgramError, Language, OptionsBuilder};
use std::env;
use tokio::fs::File;

const DEEPGRAM_API_KEY: &str = "YOUR_SECRET";
const PATH_TO_FILE: &str = "examples/prerecorded_from_file/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let file = File::open(PATH_TO_FILE).await.unwrap();

    let source = BufferSource {
        buffer: file,
        mimetype: Some("audio/wav"),
    };

    let options = OptionsBuilder::new()
        .punctuate(true)
        .language(Language::en_US);

    let response = dg_client.prerecorded_request(source, &options).await?;

    let transcript = &response.results.channels[0].alternatives[0].transcript;
    println!("{}", transcript);

    Ok(())
}
