/* Expected result from running this example program.
Request ID: <uuid>
Language: en
Sentiment: positive (avg 0.42)
*/

//! Read API: fetch a remote document and run sentiment analysis on it.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features read --example read_sentiment_url
//! ```

use std::env;

use deepgram::read::{options::Options, request::ReadRequest};
use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder().language("en").sentiment(true).build();

    let response = dg
        .read()
        .analyze(
            &ReadRequest::url(
                "https://static.deepgram.com/examples/Bueller-Life-moves-pretty-fast.txt",
            ),
            &options,
        )
        .await?;

    if let Some(meta) = response.metadata_inner() {
        if let Some(id) = &meta.request_id {
            println!("Request ID: {id}");
        }
        if let Some(lang) = &meta.language {
            println!("Language: {lang}");
        }
    }

    if let Some(sentiments) = response.results.sentiments.as_ref() {
        // Sentiments has private internal fields; surface what serde gives us.
        let v = serde_json::to_value(sentiments)?;
        if let Some(avg) = v.get("average") {
            let label = avg.get("sentiment").and_then(|s| s.as_str()).unwrap_or("?");
            let score = avg
                .get("sentiment_score")
                .and_then(|s| s.as_f64())
                .unwrap_or(0.0);
            println!("Sentiment: {label} (avg {score:.2})");
        }
    } else {
        println!("No sentiment data returned.");
    }

    Ok(())
}
