use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::{Deepgram, DeepgramError};

#[cfg(not(feature = "tokio"))]
fn main() {
    println!("This example requires the `tokio` feature");
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let dg = Deepgram::new(env::var("DEEPGRAM_API_KEY").unwrap());

    let mut results = dg
        .transcription()
        .stream_request()
        .file(
            env::var("FILENAME").unwrap(),
            3174,
            Duration::from_millis(16),
        )
        .await?
        .start()
        .await?;

    while let Some(result) = results.next().await {
        println!("got: {:?}", result);
    }

    Ok(())
}
