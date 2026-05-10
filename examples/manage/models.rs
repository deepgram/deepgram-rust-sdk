use deepgram::{manage::models::list_options, Deepgram, DeepgramError};
use std::env;

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;

    let opts = list_options::Options::builder()
        .include_outdated(false)
        .build();

    let public = dg_client.models().list(&opts).await?;
    println!("public: {} STT, {} TTS", public.stt.len(), public.tts.len());

    let project = dg_client
        .models()
        .list_for_project(&project_id, &opts)
        .await?;
    println!(
        "project: {} STT, {} TTS",
        project.stt.len(),
        project.tts.len()
    );

    if let Some(first) = public.stt.first() {
        let detail = dg_client.models().get(&first.uuid).await?;
        println!("{}: {}", detail.canonical_name, detail.version);
    }

    Ok(())
}
