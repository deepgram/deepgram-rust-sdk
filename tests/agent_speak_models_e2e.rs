//! End-to-end smoke test for the current multilingual Voice Agent Speak catalog.
//!
//! Requires `DEEPGRAM_API_KEY` in the environment.
//! Run with: `cargo test --test agent_speak_models_e2e --features agent -- --ignored`

#[cfg(feature = "agent")]
mod e2e {
    use std::time::Duration;

    use deepgram::agent::{
        audio::{AudioConfig, AudioInput, AudioInputEncoding},
        listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
        settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
        speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
        think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
        AgentEvent, AgentResponse,
    };
    use deepgram::Deepgram;
    use futures::StreamExt;

    #[tokio::test]
    #[ignore = "requires DEEPGRAM_API_KEY and access to Voice Agent Flux TTS"]
    async fn multilingual_flux_speak_model_is_accepted() {
        let client =
            Deepgram::new(std::env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY is set"))
                .expect("valid Deepgram client");
        let (mut handle, mut events) = client.agent().start().await.expect("start Voice Agent");

        let settings = SettingsMessage::new(
            AudioConfig::new(
                Some(AudioInput::new(AudioInputEncoding::Linear16, 16_000)),
                None,
            ),
            AgentConfig::inline(InlineAgentConfig::from_parts(
                AgentListenSettings::new(AgentListenProvider::DeepgramV2(
                    DeepgramListenV2Provider::new("flux-general-en"),
                )),
                ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
                    OpenAiModel::Gpt4oMini,
                ))),
                SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::v2(
                    DeepgramSpeakModel::FluxAnoukNl,
                ))),
            )),
        );
        handle.send_settings(settings).await.expect("send settings");

        tokio::time::timeout(Duration::from_secs(30), async {
            while let Some(event) = events.next().await {
                match event.expect("Voice Agent event") {
                    AgentEvent::Json(AgentResponse::SettingsApplied(_)) => return,
                    AgentEvent::Json(AgentResponse::Error(error)) => {
                        panic!(
                            "Voice Agent rejected the multilingual Flux TTS model: {}: {}",
                            error.code, error.description
                        );
                    }
                    AgentEvent::Json(_) | AgentEvent::Audio(_) => {}
                    _ => {}
                }
            }
            panic!("Voice Agent closed before SettingsApplied");
        })
        .await
        .expect("timed out waiting for SettingsApplied");

        handle.close().await.expect("close Voice Agent");
    }
}
