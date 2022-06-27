use std::env;

use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let member_id =
        env::var("DEEPGRAM_MEMBER_ID").expect("DEEPGRAM_MEMBER_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let members = dg_client.members().list_members(&project_id).await?;
    println!("{:#?}", members);

    let message = dg_client
        .members()
        .remove_member(&project_id, &member_id)
        .await?;
    println!("{}", message.message);

    Ok(())
}
