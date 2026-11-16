//! End-to-end smoke test for Deepgram Think and Think Live.
//!
//! Requires `DEEPGRAM_API_KEY` in the environment.
//! Run with: `cargo test --test agent_think_live_e2e --features agent -- --ignored`

#[cfg(feature = "agent")]
mod e2e {
    use std::time::Duration;

    use deepgram::agent::{
        audio::{AudioConfig, AudioInput, AudioInputEncoding},
        listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
        messages::UpdateThinkLiveMessage,
        settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
        speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
        think::{DeepgramThinkModel, DeepgramThinkProvider, ThinkProvider, ThinkSettings},
        think_live::{ThinkLiveProvider, ThinkLiveSettings},
        AgentEvent, AgentResponse,
    };
    use deepgram::Deepgram;
    use futures::StreamExt;

    const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

    #[tokio::test]
    #[ignore = "requires DEEPGRAM_API_KEY and access to Deepgram Think Live"]
    async fn deepgram_think_and_think_live_are_accepted() {
        let api_key =
            std::env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY must be set for e2e tests");
        let dg = Deepgram::new(api_key).expect("valid Deepgram client");
        let (mut handle, mut events) = dg.agent().start().await.expect("start Voice Agent");

        let think_live = ThinkLiveSettings::new(
            ThinkLiveProvider::new()
                .with_agent_backchannel_sensitivity(0.0)
                .with_presence_check_max_count(0)
                .with_frustration_max_count(0),
        );
        let settings = SettingsMessage::new(
            AudioConfig::new(
                Some(AudioInput::new(AudioInputEncoding::Linear16, 16_000)),
                None,
            ),
            AgentConfig::inline(
                InlineAgentConfig::from_parts(
                    AgentListenSettings::new(AgentListenProvider::DeepgramV2(
                        DeepgramListenV2Provider::new("flux-general-en"),
                    )),
                    ThinkSettings::new(ThinkProvider::Deepgram(DeepgramThinkProvider::new(
                        DeepgramThinkModel::ThinkFast,
                    ))),
                    SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                        DeepgramSpeakModel::Aura2ThaliaEn,
                    ))),
                )
                .with_think_live(think_live.clone()),
            ),
        );
        handle
            .send_settings(settings)
            .await
            .expect("send Think Live settings");

        wait_for(&mut events, |response| {
            matches!(response, AgentResponse::SettingsApplied(_))
        })
        .await;

        handle
            .send_update_think_live(UpdateThinkLiveMessage::new(think_live))
            .await
            .expect("send Think Live update");
        wait_for(&mut events, |response| {
            matches!(response, AgentResponse::ThinkLiveUpdated(_))
        })
        .await;

        handle.close().await.expect("close Voice Agent");
    }

    async fn wait_for(
        events: &mut deepgram::agent::AgentEventStream,
        expected: impl Fn(&AgentResponse) -> bool,
    ) {
        tokio::time::timeout(RESPONSE_TIMEOUT, async {
            while let Some(event) = events.next().await {
                match event.expect("Voice Agent event") {
                    AgentEvent::Json(AgentResponse::Error(error)) => {
                        panic!(
                            "Voice Agent rejected Think Live configuration: {}: {}",
                            error.code, error.description
                        );
                    }
                    AgentEvent::Json(response) if expected(&response) => return,
                    AgentEvent::Json(_) | AgentEvent::Audio(_) => {}
                    _ => {}
                }
            }
            panic!("Voice Agent closed before the expected event");
        })
        .await
        .expect("timed out waiting for Voice Agent event");
    }
}
