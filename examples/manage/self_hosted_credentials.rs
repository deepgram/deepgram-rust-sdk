//! List a project's self-hosted (on-prem) distribution credentials.
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key DEEPGRAM_PROJECT_ID=your-project \
//!   cargo run --example self_hosted_credentials --features manage
//! ```

use std::env;

use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let credentials = dg_client
        .self_hosted()
        .list_distribution_credentials(&project_id)
        .await?;

    println!(
        "{} set(s) of distribution credentials:",
        credentials.distribution_credentials.len()
    );
    for entry in &credentials.distribution_credentials {
        println!(
            "  {} (provider: {}, owner: {})",
            entry.distribution_credentials.distribution_credentials_id,
            entry.distribution_credentials.provider,
            entry.member.email,
        );
    }

    Ok(())
}
