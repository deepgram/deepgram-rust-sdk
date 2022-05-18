use deepgram::{BufferSource, Deepgram, DeepgramError, Language, Mimetype, OptionsBuilder};
use tokio::fs::File;

const DEEPGRAM_API_KEY: &str = "YOUR_SECRET";
const PATH_TO_FILE: &str = "examples/prerecorded_from_file/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let dg_client = Deepgram::new(DEEPGRAM_API_KEY);

    let file = File::open(PATH_TO_FILE).await.unwrap();

    let source = BufferSource {
        buffer: file,
        mimetype: Mimetype::audio_wav,
    };

    let options = OptionsBuilder::new()
        .punctuate(true)
        .language(Language::en_US);

    let response = dg_client.prerecorded_request(source, &options).await?;
    println!("{:?}", response);

    Ok(())
}
