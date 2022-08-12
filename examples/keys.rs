use std::env;

use deepgram::{keys::options::Options, Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let key_id = env::var("DEEPGRAM_KEY_ID").expect("DEEPGRAM_KEY_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let keys = dg_client.keys().list(&project_id).await?;
    println!("{:#?}", keys);

    let key = dg_client.keys().get(&project_id, &key_id).await?;
    println!("{:#?}", key);

    let options = Options::builder("New Key", ["member"]).build();
    let new_key = dg_client.keys().create(&project_id, &options).await?;
    println!("{:#?}", new_key);

    let message = dg_client.keys().delete(&project_id, &key_id).await?;
    println!("{}", message.message);

    Ok(())
}
