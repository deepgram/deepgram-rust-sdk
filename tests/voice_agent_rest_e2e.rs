//! Read-only live checks for the Voice Agent REST management APIs.

#![cfg(feature = "manage")]

use std::{env, io};

use deepgram::Deepgram;

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

    Ok(())
}
