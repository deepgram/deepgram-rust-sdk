use std::env;

use deepgram::{
    common::{
        audio_source::AudioSource,
        options::{CustomIntentMode, DetectLanguage, Encoding, Extra, Language, Model, Options, Redact},
    },
    Deepgram, DeepgramError,
};

static AUDIO_URL: &str = "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.wav";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let source = AudioSource::from_url(AUDIO_URL);

    let options = Options::builder()
        .model(Model::CustomId(String::from("nova-2-general")))
        .punctuate(true)
        .paragraphs(true)
        .redact([Redact::Pci, Redact::Other(String::from("cvv"))])
        // .detect_language(DetectLanguage::Enabled(true))
        .detect_language(DetectLanguage::Restricted(vec![Language::en, Language::es]))
        .diarize(true)
        .diarize_version("2021-07-14.0")
        .filler_words(true)
        .smart_format(true)
        .encoding(Encoding::Linear16)
        .language(Language::en_US)
        .detect_entities(true)
        .intents(true)
        .custom_intent_mode(CustomIntentMode::Extended)
        .custom_intents(["Phone repair", "Phone cancellation"])
        .sentiment(true)
        .topics(true)
        .custom_intent_mode(CustomIntentMode::Strict)
        .custom_intents(["Get support", "Complain"])
        .summarize(true)
        .dictation(true)
        .measurements(true)
        .extra(Extra::new("key", "value"))
        .build();

    let response = dg_client
        .transcription()
        .prerecorded(source, &options)
        .await?;

    let transcript = &response.results.channels[0].alternatives[0].transcript;
    println!("{}", transcript);

    println!("{:?}", response);

    Ok(())
}
