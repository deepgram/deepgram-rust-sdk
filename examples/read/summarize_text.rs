/* Expected result from running this example program.
Request ID: <uuid>
Summary: A condensed paraphrase of the input text.
*/

//! Read API: pass an inline string and get back a summary.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features read --example read_summarize_text
//! ```

use std::env;

use deepgram::read::{options::Options, request::ReadRequest};
use deepgram::{Deepgram, DeepgramError};

const TEXT: &str = "Deepgram's Voice AI platform powers speech-to-text, text-to-speech, \
                    and full conversational agents. The Read API analyzes text content \
                    using the same intelligence features (sentiment, summarize, topics, \
                    intents) that the Listen API exposes for audio. This means you can \
                    process transcripts, documents, or any plain text uniformly without \
                    spinning up an audio pipeline.";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder().language("en").summarize(true).build();

    let response = dg
        .read()
        .analyze(&ReadRequest::text(TEXT), &options)
        .await?;

    if let Some(meta) = response.metadata_inner() {
        if let Some(id) = &meta.request_id {
            println!("Request ID: {id}");
        }
    }

    match response.summary_text() {
        Some(text) => println!("Summary: {text}"),
        None => println!("No summary returned."),
    }

    Ok(())
}
