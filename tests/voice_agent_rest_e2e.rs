//! Read-only live checks for the Voice Agent REST management APIs.

#![cfg(feature = "manage")]

use std::{env, io};

use deepgram::{manage::voice_agent::variables::CreateOptions, Deepgram};
use serde_json::json;

/// Verifies Voice Agent REST discovery and read APIs without mutating project state.
#[tokio::test]
#[ignore = "requires DEEPGRAM_API_KEY and live Voice Agent REST API access"]
async fn voice_agent_rest_read_operations_are_available() -> Result<(), Box<dyn std::error::Error>>
{
    let api_key = env::var("DEEPGRAM_API_KEY").map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "DEEPGRAM_API_KEY must be set to run this ignored live test",
        )
    })?;
    let dg = Deepgram::new(api_key)?;

    let _ = dg.voice_agent().list_think_models().await?;

    let projects = dg.projects().list().await?;
    let project_id = projects
        .projects
        .first()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "API key has no accessible projects",
            )
        })?
        .project_id
        .to_string();

    let voice_agent = dg.voice_agent();
    let configurations = voice_agent.configurations().list(&project_id).await?;
    let variables = voice_agent.variables().list(&project_id).await?;
    let _ = (configurations.agents, variables.variables);

    let created = voice_agent
        .variables()
        .create(
            &project_id,
            &CreateOptions::builder("DG_E2E_PROBE", json!({"source": "rust-sdk-e2e"}))
                .api_version(1)
                .build(),
        )
        .await?;
    let variable_id = created.agent_variable_uuid;

    let verification = async {
        let variable = voice_agent
            .variables()
            .get(&project_id, &variable_id)
            .await?;
        assert_eq!(variable.agent_variable_uuid, variable_id);
        assert_eq!(variable.key, "DG_E2E_PROBE");
        assert_eq!(variable.value, json!({"source": "rust-sdk-e2e"}));

        let variables = voice_agent.variables().list(&project_id).await?;
        assert!(
            variables
                .variables
                .iter()
                .any(|variable| variable.agent_variable_uuid == variable_id),
            "the created variable appears in list()"
        );
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;
    let cleanup = voice_agent
        .variables()
        .delete(&project_id, &variable_id)
        .await;
    cleanup?;
    verification?;

    Ok(())
}
