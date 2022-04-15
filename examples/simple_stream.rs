use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").unwrap();
    let dg = Deepgram::new(&api_key);

    let mut results = dg
        .stream_request()
        .file(
            &env::var("FILENAME").unwrap(),
            128,
            Duration::from_millis(16),
        )
        .await?
        .start()
        .await?;

    while let Some(result) = results.next().await {
        println!("got: {result}");
    }

    Ok(())
}
