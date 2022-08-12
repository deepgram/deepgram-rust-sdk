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

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let scopes = dg_client
        .scopes()
        .get_scope(&project_id, &member_id)
        .await?;
    println!("{:#?}", scopes);

    let message = dg_client
        .scopes()
        .update_scope(&project_id, &member_id, "member")
        .await?;
    println!("{}", message.message);

    Ok(())
}
