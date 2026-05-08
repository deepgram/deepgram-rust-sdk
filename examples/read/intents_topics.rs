/* Expected result from running this example program.
Request ID: <uuid>
--- Topics ---
  travel
  recommendation
--- Intents ---
  ask_recommendation
*/

//! Read API: run topics + intents on inline text, with custom topics
//! and intents to demonstrate the `extended` and `strict` modes.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features read --example read_intents_topics
//! ```

use std::env;

use deepgram::common::options::{CustomIntentMode, CustomTopicMode};
use deepgram::read::{options::Options, request::ReadRequest};
use deepgram::{Deepgram, DeepgramError};

const TEXT: &str = "Hi! I'm planning a trip to Tokyo next month and I'd love any \
                    restaurant recommendations near Shibuya. Also, what's the best \
                    way to get from Narita Airport to the city?";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let dg = Deepgram::new(&api_key)?;

    let options = Options::builder()
        .language("en")
        .topics(true)
        .custom_topics(["travel", "recommendation"])
        .custom_topic_mode(CustomTopicMode::Extended)
        .intents(true)
        .custom_intents(["ask_recommendation"])
        .custom_intent_mode(CustomIntentMode::Strict)
        .build();

    let response = dg
        .read()
        .analyze(&ReadRequest::text(TEXT), &options)
        .await?;

    if let Some(meta) = response.metadata_inner() {
        if let Some(id) = &meta.request_id {
            println!("Request ID: {id}");
        }
    }

    println!("--- Topics ---");
    if let Some(topics) = response.results.topics.as_ref() {
        let v = serde_json::to_value(topics)?;
        if let Some(segments) = v.get("segments").and_then(|s| s.as_array()) {
            for seg in segments {
                if let Some(arr) = seg.get("topics").and_then(|t| t.as_array()) {
                    for topic in arr {
                        if let Some(t) = topic.get("topic").and_then(|t| t.as_str()) {
                            println!("  {t}");
                        }
                    }
                }
            }
        }
    } else {
        println!("  (none)");
    }

    println!("--- Intents ---");
    if let Some(intents) = response.results.intents.as_ref() {
        let v = serde_json::to_value(intents)?;
        if let Some(segments) = v.get("segments").and_then(|s| s.as_array()) {
            for seg in segments {
                if let Some(arr) = seg.get("intents").and_then(|i| i.as_array()) {
                    for intent in arr {
                        if let Some(name) = intent.get("intent").and_then(|n| n.as_str()) {
                            println!("  {name}");
                        }
                    }
                }
            }
        }
    } else {
        println!("  (none)");
    }

    Ok(())
}
