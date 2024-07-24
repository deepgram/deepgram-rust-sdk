use deepgram::{Deepgram, DeepgramError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let balance_id =
        env::var("DEEPGRAM_BALANCE_ID").expect("DEEPGRAM_BALANCE_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let all_balances = dg_client.billing().list_balance(&project_id).await?;
    println!("{:#?}", all_balances);

    let specific_balance = dg_client
        .billing()
        .get_balance(&project_id, &balance_id)
        .await?;
    println!("{:#?}", specific_balance);

    Ok(())
}
