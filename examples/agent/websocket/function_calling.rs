/* Expected result from running this example program.
Connected. dg-request-id: Some(<uuid>)
Welcome request_id: <uuid>
Settings applied
Conversation (Assistant): Hi! Ask me about the weather.
... (after a user injection asking about NYC)
FunctionCallRequest: get_weather (id=fc_1, client_side=true)
  arguments: {"city":"New York"}
  → responding with: {"temperature":72,"condition":"sunny"}
Conversation (Assistant): It's 72 and sunny in New York.
*/

//! Function-calling Voice Agent example.
//!
//! Configures an agent with a single client-side function (`get_weather`),
//! injects a synthetic user utterance asking about the weather, and
//! responds to the resulting [`FunctionCallRequest`] with a canned
//! `FunctionCallResponse`.
//!
//! No real audio I/O — the example exercises the JSON message round-trip
//! and the function-call protocol on its own.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//!     cargo run --features agent --example agent_function_calling
//! ```

use std::env;
use std::time::Duration;

use futures::stream::StreamExt;
use serde_json::json;

use deepgram::agent::messages::FunctionCallResponseMessage;
use deepgram::agent::{
    audio::{AudioConfig, AudioInput, AudioInputEncoding},
    listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
    settings::{AgentConfig, InlineAgentConfig, SettingsMessage},
    speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
    think::{OpenAiModel, OpenAiThinkProvider, ThinkFunction, ThinkProvider, ThinkSettings},
    AgentEvent, AgentResponse,
};
use deepgram::{Deepgram, DeepgramError};

static SESSION_DURATION: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");

    let dg = Deepgram::new(&api_key)?;
    let (mut handle, mut events) = dg.agent().start().await?;

    println!("Connected. dg-request-id: {:?}", handle.request_id());

    // No `endpoint` → executed client-side. The server emits a
    // FunctionCallRequest and waits for our FunctionCallResponse.
    let weather_function = ThinkFunction::new(
        "get_weather",
        "Look up the current weather for a city.",
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "City name, e.g. \"New York\""
                }
            },
            "required": ["city"]
        }),
    );

    let think = ThinkSettings::new(ThinkProvider::OpenAi(OpenAiThinkProvider::new(
        OpenAiModel::Gpt4oMini,
    )))
    .with_function(weather_function)
    .with_prompt(
        "You are a helpful weather assistant. Use the get_weather function \
         when the user asks about weather.",
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
                think,
                SpeakSettings::new(SpeakProvider::Deepgram(DeepgramSpeakProvider::new(
                    DeepgramSpeakModel::Aura2ThaliaEn,
                ))),
            )
            .with_greeting("Hi! Ask me about the weather."),
        ),
    );
    handle.send_settings(settings).await?;

    // Wait for SettingsApplied, then inject a synthetic user message
    // so the agent has something to respond to without us needing a mic.
    let mut injected = false;

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
                            println!("Welcome request_id: {}", w.request_id);
                        }
                        AgentResponse::SettingsApplied(_) => {
                            println!("Settings applied");
                            if !injected {
                                injected = true;
                                handle
                                    .send_inject_user_message(
                                        deepgram::agent::messages::InjectUserMessageMessage::new(
                                            "What's the weather in New York?",
                                        ),
                                    )
                                    .await?;
                            }
                        }
                        AgentResponse::ConversationText(c) => {
                            println!("Conversation ({:?}): {}", c.role, c.content);
                        }
                        AgentResponse::FunctionCallRequest(req) => {
                            for call in &req.functions {
                                println!(
                                    "FunctionCallRequest: {} (id={}, client_side={})",
                                    call.name, call.id, call.client_side
                                );
                                println!("  arguments: {}", call.arguments);

                                if call.client_side && call.name == "get_weather" {
                                    // Canned response — in a real app, parse
                                    // call.arguments and dispatch to your
                                    // function implementation.
                                    let result =
                                        json!({"temperature": 72, "condition": "sunny"});
                                    println!("  → responding with: {result}");
                                    handle
                                        .send_function_call_response(
                                            FunctionCallResponseMessage::with_id(
                                                call.id.clone(),
                                                call.name.clone(),
                                                result.to_string(),
                                            ),
                                        )
                                        .await?;
                                }
                            }
                        }
                        AgentResponse::AgentAudioDone(_) => {
                            // After the agent finishes its audio response we
                            // could end the demo. For brevity we just log.
                            println!("Agent audio done");
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
                        // Audio chunks arrive between AgentStartedSpeaking and
                        // AgentAudioDone. Discarded here.
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
