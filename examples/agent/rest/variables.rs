/* Expected result from running this example program.
Created variable v1 (key=DG_AGENT_NAME)
Listed 1 variables.
Get -> "Alice"
Update -> "Bob"
Deleted variable v1.
Think models available:
  open_ai      gpt-4o-mini
  anthropic    claude-3-5-haiku-latest
  ...
*/

//! Walkthrough for the Agent template-variables REST endpoints, plus a
//! call to the think-models catalog at the end.
//!
//! Run with:
//!
//! ```bash
//! DEEPGRAM_API_KEY=<your-key> \
//! DEEPGRAM_PROJECT_ID=<project-uuid> \
//!     cargo run --features agent --example agent_variables
//! ```

use std::env;

use serde_json::json;

use deepgram::agent::variables::CreateAgentVariableRequest;
use deepgram::{Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let api_key = env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environment variable");
    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environment variable");

    let dg = Deepgram::new(&api_key)?;
    let agent = dg.agent();
    let variables = agent.variables();

    let created = variables
        .create(
            &project_id,
            &CreateAgentVariableRequest::new("DG_AGENT_NAME", json!("Alice")),
        )
        .await?;
    let variable_id = created.variable_id.clone();
    println!("Created variable {} (key={})", variable_id, created.key);

    let listed = variables.list(&project_id).await?;
    println!("Listed {} variables.", listed.variables.len());

    let fetched = variables.get(&project_id, &variable_id).await?;
    println!("Get -> {}", fetched.value);

    let updated = variables
        .update(&project_id, &variable_id, json!("Bob"))
        .await?;
    println!("Update -> {}", updated.value);

    variables.delete(&project_id, &variable_id).await?;
    println!("Deleted variable {}.", variable_id);

    let think_models = agent.think_models().list().await?;
    println!("Think models available:");
    for model in think_models.models {
        println!(
            "  {:<12} {}",
            format!("{:?}", model.provider).to_lowercase(),
            model.id
        );
    }

    Ok(())
}
