/* Expected result from running this example program.
Connected. dg-request-id from upgrade headers: Some(<uuid>)
Welcome event request_id: <uuid>
Settings applied
Conversation (assistant): Hello! How can I help today?
Audio chunk: 4096 bytes
Audio chunk: 4096 bytes
...
*/

//! Minimal Voice Agent example.
//!
//! Connects to `wss://agent.deepgram.com/v1/agent/converse`, sends a
//! `Settings` message with a Deepgram-only Listen + Speak setup and an
//! OpenAI Think provider, prints incoming events for a fixed duration,
//! then closes the connection.
//!
//! The agent will speak a greeting on connect, but this example does
//! not capture or send any microphone audio — for that, see the
//! microphone example (when added).
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features agent --example agent_simple
//! ```

use std::env;
use std::time::Duration;

use futures::stream::StreamExt;

use deepgram::agent::{
    audio::{AudioConfig, AudioInput, AudioInputEncoding},
    listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
    settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
    speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
    think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
    AgentEvent, AgentResponse,
};
use deepgram::{Deepgram, DeepgramError};

/// How long to keep the session open before closing.
static SESSION_DURATION: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");

    let dg = Deepgram::new(&api_key)?;
    let (mut handle, mut events) = dg.agent().start().await?;

    println!(
        "Connected. dg-request-id from upgrade headers: {:?}",
        handle.request_id()
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
                ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
                    OpenAiModel::Gpt4oMini,
                ))),
                SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                    DeepgramSpeakModel::Aura2ThaliaEn,
                ))),
            )
            .with_greeting("Hello! How can I help today?"),
        ),
    );
    handle.send_settings(settings).await?;

    let timeout = tokio::time::sleep(SESSION_DURATION);
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => {
                println!("\nSession duration reached, closing.");
                break;
            }
            event = events.next() => {
                match event {
                    Some(Ok(AgentEvent::Json(response))) => match response {
                        AgentResponse::Welcome(w) => {
                            println!("Welcome event request_id: {}", w.request_id);
                        }
                        AgentResponse::SettingsApplied(_) => {
                            println!("Settings applied");
                        }
                        AgentResponse::AgentThinking(t) => {
                            println!("Agent thinking: {}", t.content);
                        }
                        AgentResponse::ConversationText(c) => {
                            println!("Conversation ({:?}): {}", c.role, c.content);
                        }
                        AgentResponse::UserStartedSpeaking(_) => {
                            println!("User started speaking");
                        }
                        AgentResponse::AgentStartedSpeaking(s) => {
                            println!(
                                "Agent started speaking (total_latency={:.3}s)",
                                s.total_latency
                            );
                        }
                        AgentResponse::AgentAudioDone(_) => {
                            println!("Agent audio done");
                        }
                        AgentResponse::Warning(w) => {
                            println!("Warning [{}]: {}", w.code, w.description);
                        }
                        AgentResponse::Error(e) => {
                            eprintln!("Error [{}]: {}", e.code, e.description);
                            break;
                        }
                        other => println!("Other event: {:?}", other),
                    },
                    Some(Ok(AgentEvent::Audio(bytes))) => {
                        println!("Audio chunk: {} bytes", bytes.len());
                    }
                    // AgentEvent is #[non_exhaustive]; future variants land here.
                    Some(Ok(_)) => {}
                    Some(Err(err)) => {
                        eprintln!("Stream error: {}", err);
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
