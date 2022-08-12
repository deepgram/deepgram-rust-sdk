use std::env;

use deepgram::{
    usage::{get_fields_options, get_usage_options, list_requests_options},
    Deepgram, DeepgramError,
};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let request_id =
        env::var("DEEPGRAM_REQUEST_ID").expect("DEEPGRAM_REQUEST_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let options = list_requests_options::Options::builder().build();
    let requests = dg_client
        .usage()
        .list_requests(&project_id, &options)
        .await?;
    println!("{:#?}", requests);

    let request = dg_client
        .usage()
        .get_request(&project_id, &request_id)
        .await?;
    println!("{:#?}", request);

    let options = get_usage_options::Options::builder().build();
    let summary = dg_client.usage().get_usage(&project_id, &options).await?;
    println!("{:#?}", summary);

    let options = get_fields_options::Options::builder().build();
    let summary = dg_client.usage().get_fields(&project_id, &options).await?;
    println!("{:#?}", summary);

    Ok(())
}
