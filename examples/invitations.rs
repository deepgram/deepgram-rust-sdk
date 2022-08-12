use std::env;

use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let message = dg_client.invitations().leave_project(&project_id).await?;
    println!("{:#?}", message);

    Ok(())
}
