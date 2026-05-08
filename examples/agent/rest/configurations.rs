/* Expected result from running this example program.
Created agent: <uuid> (config_keys: ["language", "listen", "speak", "think"])
Listed 1 agents on the project.
Updated metadata: {"env": "staging"}
Deleted agent <uuid>.
*/

//! Saved-configuration CRUD walkthrough for the Agent REST API.
//!
//! Creates a configuration from an [`InlineAgentConfig`], lists the
//! project's agents, updates the new entry's metadata, then deletes
//! it.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//! DEEPGRAM_PROJECT_ID=<project-uuid> \
//!     cargo run --features agent --example agent_configurations
//! ```

use std::env;

use deepgram::agent::configurations::{
    CreateAgentConfigurationRequest, UpdateAgentMetadataRequest,
};
use deepgram::agent::settings::InlineAgentConfig;
use deepgram::agent::{
    listen::{AgentListenProvider, AgentListenSettings, DeepgramListenV2Provider},
    speak::{DeepgramSpeakModel, DeepgramSpeakProvider, SpeakProvider, SpeakSettings},
    think::{OpenAiModel, OpenAiThinkProvider, ThinkProvider, ThinkSettings},
};
use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environment variable");

    let dg = Deepgram::new(&api_key)?;
    let configs = dg.agent().configurations();

    // Build a typed inline config and let the SDK serialize it to the
    // string the API expects.
    let inline = InlineAgentConfig::from_parts(
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
    .with_greeting("Hello! Configured by the Rust SDK example.");

    let create_request = CreateAgentConfigurationRequest::from_inline(&inline)
        .map_err(deepgram::DeepgramError::JsonError)?
        .with_metadata([("env", "demo"), ("source", "rust-sdk-example")]);

    let created = configs.create(&project_id, &create_request).await?;
    let agent_id = created.agent_id;
    let mut config_keys: Vec<String> = created
        .config
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    config_keys.sort();
    println!("Created agent: {agent_id} (config_keys: {:?})", config_keys);

    let listed = configs.list(&project_id).await?;
    println!("Listed {} agents on the project.", listed.agents.len());

    let updated = configs
        .update_metadata(
            &project_id,
            &agent_id,
            &UpdateAgentMetadataRequest::new([("env", "staging")]),
        )
        .await?;
    println!("Updated metadata: {:?}", updated.metadata);

    configs.delete(&project_id, &agent_id).await?;
    println!("Deleted agent {agent_id}.");

    Ok(())
}
