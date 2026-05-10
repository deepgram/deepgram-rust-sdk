use deepgram::{
    self_hosted::distribution_credentials::create_options::{Options, Provider, Scope},
    Deepgram, DeepgramError,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let existing = dg_client
        .distribution_credentials()
        .list(&project_id)
        .await?;
    println!(
        "existing credential sets: {}",
        existing.distribution_credentials.len()
    );

    let opts = Options::builder()
        .scopes([Scope::ProductApi, Scope::ProductEngine])
        .provider(Provider::Quay)
        .comment("created by deepgram-rust-sdk example")
        .build();
    let created = dg_client
        .distribution_credentials()
        .create(&project_id, &opts)
        .await?;
    let credentials_id = created.distribution_credentials.distribution_credentials_id;
    println!("created: {credentials_id}");

    let fetched = dg_client
        .distribution_credentials()
        .get(&project_id, &credentials_id)
        .await?;
    println!(
        "scopes on fetched: {:?}",
        fetched.distribution_credentials.scopes
    );

    dg_client
        .distribution_credentials()
        .delete(&project_id, &credentials_id)
        .await?;
    println!("deleted: {credentials_id}");

    Ok(())
}
