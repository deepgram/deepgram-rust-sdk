/* Expected result from running this example program.
Connected. dg-request-id: Some(<uuid>)
Welcome request_id: <uuid>
Settings applied
Conversation (Assistant): I'm using my first voice.
[5s elapsed] Sending UpdateSpeak to switch to a different Aura-2 voice...
SpeakUpdated
Conversation (Assistant): Now my voice has changed.
*/

//! Dynamic provider swap example.
//!
//! Connects with one Speak provider (`aura-2-thalia-en`), then 5 seconds
//! after the agent's initial greeting, sends an `UpdateSpeak` message to
//! swap the voice to `aura-2-zeus-en`. Demonstrates that providers can
//! be changed mid-session without dropping the WebSocket.
//!
//! No real audio I/O — the example surfaces the JSON message round-trip
//! and the `SpeakUpdated` confirmation event.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features agent --example agent_dynamic_provider_swap
//! ```

use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::agent::{
    audio::{AudioConfig, AudioInput, AudioInputEncoding},
    listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
    messages::UpdateSpeakMessage,
    settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
    speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
    think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
    AgentEvent, AgentResponse,
};
use deepgram::{Deepgram, DeepgramError};

static SESSION_DURATION: Duration = Duration::from_secs(60);
static SWAP_AFTER: Duration = Duration::from_secs(5);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");

    let dg = Deepgram::new(&api_key)?;
    let (mut handle, mut events) = dg.agent().start().await?;

    println!("Connected. dg-request-id: {:?}", handle.request_id());

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
                ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
                    OpenAiModel::Gpt4oMini,
                ))),
                SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                    DeepgramSpeakModel::Aura2ThaliaEn,
                ))),
            )
            .with_greeting("I'm using my first voice."),
        ),
    );
    handle.send_settings(settings).await?;

    let timeout = tokio::time::sleep(SESSION_DURATION);
    tokio::pin!(timeout);
    let swap_timer = tokio::time::sleep(SWAP_AFTER);
    tokio::pin!(swap_timer);
    let mut swapped = false;

    loop {
        tokio::select! {
            _ = &mut timeout => {
                println!("\nSession duration reached, closing.");
                break;
            }
            _ = &mut swap_timer, if !swapped => {
                println!(
                    "\n[{}s elapsed] Sending UpdateSpeak to switch to a different Aura-2 voice...",
                    SWAP_AFTER.as_secs()
                );
                let new_speak = SpeakSettings::new(SpeakProvider::Deepgram(
                    DeepgramSpeakProvider::new(DeepgramSpeakModel::Aura2ZeusEn),
                ));
                handle
                    .send_update_speak(UpdateSpeakMessage::one(new_speak))
                    .await?;
                // Optional follow-up: inject an utterance so the agent
                // speaks again with the new voice. Without this the user
                // would have to talk for the swap to be audible.
                handle
                    .send_inject_agent_message(
                        deepgram::agent::messages::InjectAgentMessageMessage::new(
                            "Now my voice has changed.",
                        ),
                    )
                    .await?;
                swapped = true;
            }
            event = events.next() => {
                match event {
                    Some(Ok(AgentEvent::Json(response))) => match response {
                        AgentResponse::Welcome(w) => {
                            println!("Welcome request_id: {}", w.request_id);
                        }
                        AgentResponse::SettingsApplied(_) => {
                            println!("Settings applied");
                        }
                        AgentResponse::SpeakUpdated(_) => {
                            println!("SpeakUpdated");
                        }
                        AgentResponse::ConversationText(c) => {
                            println!("Conversation ({:?}): {}", c.role, c.content);
                        }
                        AgentResponse::Error(e) => {
                            eprintln!("Error [{}]: {}", e.code, e.description);
                            break;
                        }
                        AgentResponse::Warning(w) => {
                            println!("Warning [{}]: {}", w.code, w.description);
                        }
                        _ => {}
                    },
                    Some(Ok(AgentEvent::Audio(_))) => {
                        // Discard audio for brevity; the playback path is
                        // demonstrated in the microphone example.
                    }
                    Some(Ok(_)) => {} // AgentEvent #[non_exhaustive]
                    Some(Err(err)) => {
                        eprintln!("Stream error: {err}");
                        break;
                    }
                    None => {
                        println!("Server closed connection.");
                        break;
                    }
                }
            }
        }
    }

    handle.close().await?;
    Ok(())
}
