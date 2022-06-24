use std::env;

use deepgram::{projects::options::Options, Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let projects = dg_client.projects().list().await?;
    println!("{:#?}", projects);

    let project = dg_client.projects().get(&project_id).await?;
    println!("{:#?}", project);

    let options = Options::builder()
        .name("The Transcribinator")
        .company("Doofenshmirtz Evil Incorporated")
        .build();
    let message = dg_client.projects().update(&project_id, &options).await?;
    println!("{}", message.message);

    let message = dg_client.projects().delete(&project_id).await?;
    println!("{}", message.message);

    Ok(())
}
