use std::env;

use deepgram::{auth::options::Options, Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    // Example 1: Generate a token with default 30-second TTL
    println!("Generating token with default TTL (30 seconds)...");
    let token = dg_client.auth().grant(None).await?;
    println!("Token: {}", token.access_token);
    if let Some(expires_in) = token.expires_in {
        println!("Expires in: {} seconds", expires_in);
    }
    println!();

    // Example 2: Generate a token with custom TTL (5 minutes)
    println!("Generating token with custom TTL (300 seconds)...");
    let options = Options::builder().ttl_seconds(300.0).build();
    let token_with_ttl = dg_client.auth().grant(Some(&options)).await?;
    println!("Token: {}", token_with_ttl.access_token);
    if let Some(expires_in) = token_with_ttl.expires_in {
        println!("Expires in: {} seconds", expires_in);
    }
    println!();

    // Example 3: Use the generated token to create a new client
    println!("Creating a new Deepgram client with the temporary token...");
    let _temp_client = Deepgram::with_temp_token(&token.access_token)?;
    println!("Successfully created client with temporary token!");
    println!("This client can now be used for transcription requests.");

    Ok(())
}
