//! End-to-end test for Flux WebSocket streaming against the real Deepgram API.
//!
//! Requires DEEPGRAM_API_KEY in the environment.
//! Run with: cargo test --test flux_e2e --features listen -- --ignored

#[cfg(feature = "listen")]
mod e2e {
    use std::time::Duration;

    use deepgram::{
        common::{
            flux_response::{FluxResponse, TurnEvent},
            options::{Encoding, Model, Options},
        },
        Deepgram,
    };
    use futures::stream::StreamExt;

    static PATH_TO_FILE: &str = "examples/audio/sample-mono.wav";
    static AUDIO_CHUNK_SIZE: usize = 18_063;
    static FRAME_DELAY: Duration = Duration::from_millis(100);

    #[tokio::test]
    #[ignore] // requires DEEPGRAM_API_KEY
    async fn flux_real_connection_streams_audio() {
        let api_key =
            std::env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY must be set for e2e tests");

        let dg = Deepgram::new(&api_key).unwrap();

        let options = Options::builder()
            .model(Model::FluxGeneralEn)
            .eot_threshold(0.75)
            .eot_timeout_ms(5000)
            .build();

        let mut results = dg
            .transcription()
            .flux_request_with_options(options)
            .encoding(Encoding::Linear32)
            .sample_rate(44100)
            .file(PATH_TO_FILE, AUDIO_CHUNK_SIZE, FRAME_DELAY)
            .await
            .expect("failed to start flux stream");

        let mut got_connected = false;
        let mut got_turn_info = false;
        let mut got_transcript = false;

        while let Some(result) = results.next().await {
            let response = result.expect("flux stream produced an error");
            match response {
                FluxResponse::Connected { .. } => {
                    got_connected = true;
                }
                FluxResponse::TurnInfo {
                    event, transcript, ..
                } => {
                    got_turn_info = true;
                    if !transcript.is_empty()
                        && matches!(
                            event,
                            TurnEvent::EndOfTurn | TurnEvent::Update | TurnEvent::EagerEndOfTurn
                        )
                    {
                        got_transcript = true;
                    }
                }
                FluxResponse::FatalError {
                    code, description, ..
                } => {
                    panic!("received fatal error: {code}: {description}");
                }
                _ => {
                    // Unknown or future variants — fine, just keep going
                }
            }
        }

        assert!(got_connected, "never received Connected message");
        assert!(got_turn_info, "never received any TurnInfo messages");
        assert!(
            got_transcript,
            "never received a non-empty transcript from the audio file"
        );
    }
}
