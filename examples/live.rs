use std::{env, time::Duration};

use deepgram::{
    transcription::live::{
        options::{Language, Options, Tier},
        response::Response,
        DeepgramLive,
    },
    Deepgram, DeepgramError,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};

static PATH_TO_FILE: &str = "examples/Bueller-Life-moves-pretty-fast.mp3";

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key);

    let options = Options::builder()
        .tier(Tier::Enhanced)
        .punctuate(true)
        .language(Language::en_US)
        .build();

    let dg_live = dg_client.transcription().live(&options).await?;
    let (sink, stream) = dg_live.split();

    let send_task = tokio::spawn(send_to_deepgram(sink));
    let receive_task = tokio::spawn(receive_from_deepgram(stream));

    send_task.await.unwrap()?;
    receive_task.await.unwrap()?;

    Ok(())
}

async fn send_to_deepgram(mut sink: SplitSink<DeepgramLive, &[u8]>) -> Result<(), DeepgramError> {
    let audio_data = tokio::fs::read(PATH_TO_FILE).await.unwrap();

    static SLICE_SIZE: usize = 250;

    // Simulate an audio stream by sending the contents of a file in chunks
    let mut slice_begin = 0;
    while slice_begin < audio_data.len() {
        let slice_end = std::cmp::min(slice_begin + SLICE_SIZE, audio_data.len());

        let slice = &audio_data[slice_begin..slice_end];
        sink.send(slice).await?;

        slice_begin += SLICE_SIZE;

        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    // Tell Deepgram that we've finished sending audio data by sending a zero-byte message
    sink.send(&[]).await?;

    Ok(())
}

async fn receive_from_deepgram(mut stream: SplitStream<DeepgramLive>) -> Result<(), DeepgramError> {
    while let Some(response) = stream.next().await {
        match response? {
            Response::Results(results) => {
                println!("{}", results.channel.alternatives[0].transcript)
            }
            Response::Metadata(metadata) => println!("{:?}", metadata),
        }
    }

    Ok(())
}
