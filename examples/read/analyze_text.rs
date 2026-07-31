//! Analyze plain text with Deepgram's Text Intelligence (`/v1/read`) API.
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key cargo run --example analyze_text --features listen
//! ```

use std::env;

use deepgram::{
    read::options::{Language, Options},
    Deepgram, DeepgramError,
};

const TEXT: &str = "Hi. Thank you for calling Premier Phone Services. My name is Beth. \
    I'm sorry to hear your phone is not working. I would be happy to help you today. \
    Let me pull up your account so we can get this resolved for you right away.";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let options = Options::builder()
        .language(Language::en)
        .sentiment(true)
        .topics(true)
        .intents(true)
        .summarize(true)
        .build();

    let response = dg_client
        .text_intelligence()
        .analyze_text(TEXT, &options)
        .await?;

    if let Some(sentiments) = &response.results.sentiments {
        println!("average sentiment: {:?}", sentiments.average);
    }
    if let Some(summary) = &response.results.summary {
        println!("summary: {}", summary.text);
    }
    if let Some(topics) = &response.results.topics {
        println!("topic segments: {}", topics.segments.len());
    }
    if let Some(intents) = &response.results.intents {
        println!("intent segments: {}", intents.segments.len());
    }

    Ok(())
}
