use std::env;

use deepgram::{
    transcription::{audio_source::AudioSource, common_options::{Language, Options}, prerecorded::response::Response},
    Deepgram,
};

static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> reqwest::Result<()> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let source = AudioSource::from_url(AUDIO_URL);

    let options = Options::builder()
        .punctuate(true)
        .language(Language::en_US)
        .build();

    let request_builder = dg_client
        .transcription()
        .make_prerecorded_request_builder(source, &options);

    // Customize the RequestBuilder here
    let customized_request_builder = request_builder
        .query(&[("custom_query_key", "custom_query_value")])
        .header("custom_header_key", "custom_header_value");

    // It is necessary to annotate the type of response here
    // That way it knows what type to deserialize the JSON into
    let response: Response = customized_request_builder.send().await?.json().await?;

    let transcript = &response.results.channels[0].alternatives[0].transcript;
    println!("{}", transcript);

    println!("{:?}", response); 

    Ok(())
}
