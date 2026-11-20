//! Live checks for the Voice Agent REST management APIs.

#![cfg(feature = "manage")]

use std::{
    env, io,
    time::{SystemTime, UNIX_EPOCH},
};

use deepgram::{
    manage::voice_agent::variables::{CreateOptions, UpdateOptions},
    Deepgram, DeepgramError,
};
use serde_json::json;

/// Verifies Voice Agent REST discovery and resource operations.
///
/// The test creates and deletes one uniquely named `DG_E2E_PROBE_*` template
/// variable. It always attempts the deletion after creating the variable,
/// including when validation fails.
#[tokio::test]
#[ignore = "requires DEEPGRAM_API_KEY and live Voice Agent REST API access"]
async fn voice_agent_rest_operations_are_available() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("DEEPGRAM_API_KEY").map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "DEEPGRAM_API_KEY must be set to run this ignored live test",
        )
    })?;
    let dg = Deepgram::new(api_key)?;

    let think_models = dg.voice_agent().list_think_models().await?;
    if think_models.models.is_empty() {
        return Err(io::Error::other("list_think_models() returned no models").into());
    }

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

    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let variable_key = format!("DG_E2E_PROBE_{nonce:x}");
    let expected_variable_key = variable_key.to_ascii_uppercase();
    let created = voice_agent
        .variables()
        .create(
            &project_id,
            &CreateOptions::builder(&variable_key, json!({"source": "rust-sdk-e2e"}))
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
        if variable.agent_variable_uuid != variable_id {
            return Err(io::Error::other("get() returned a different variable identifier").into());
        }
        if variable.key != expected_variable_key {
            return Err(io::Error::other(format!(
                "get() returned a different variable key: expected {expected_variable_key:?}, \
                 got {:?}",
                variable.key,
            ))
            .into());
        }
        if variable.value != json!({"source": "rust-sdk-e2e"}) {
            return Err(io::Error::other("get() returned a different variable value").into());
        }

        voice_agent
            .variables()
            .update(
                &project_id,
                &variable_id,
                &UpdateOptions::new(json!({"source": "rust-sdk-e2e-updated"})),
            )
            .await?;
        let updated = voice_agent
            .variables()
            .get(&project_id, &variable_id)
            .await?;
        if updated.value != json!({"source": "rust-sdk-e2e-updated"}) {
            return Err(io::Error::other("update() did not persist the variable value").into());
        }

        let variables = voice_agent.variables().list(&project_id).await?;
        if !variables
            .variables
            .iter()
            .any(|variable| variable.agent_variable_uuid == variable_id)
        {
            return Err(io::Error::other("the created variable does not appear in list()").into());
        }
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;
    let cleanup = voice_agent
        .variables()
        .delete(&project_id, &variable_id)
        .await;
    combine(verification, cleanup)
}

/// Report a verification failure, a cleanup failure, or both.
fn combine(
    verification: Result<(), Box<dyn std::error::Error>>,
    cleanup: Result<(), DeepgramError>,
) -> Result<(), Box<dyn std::error::Error>> {
    match (verification, cleanup) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(verification), Ok(())) => Err(verification),
        (Ok(()), Err(cleanup)) => Err(cleanup.into()),
        (Err(verification), Err(cleanup)) => Err(io::Error::other(format!(
            "validation failed: {verification}; cleanup also failed: {cleanup}"
        ))
        .into()),
    }
}
