//! Manage a project's self-hosted (on-prem) distribution credentials.
//!
//! By default this lists the project's credentials. Set
//! `DEEPGRAM_SELF_HOSTED_CREATE=1` to also create a set of credentials, print
//! the one-time registry `username` / `secret`, fetch it back by id, and then
//! delete it again (set `DEEPGRAM_SELF_HOSTED_KEEP=1` to keep it instead).
//!
//! Run with:
//!
//! ```sh
//! DEEPGRAM_API_KEY=your-key DEEPGRAM_PROJECT_ID=your-project \
//!   cargo run --example self_hosted_credentials --features manage
//! ```

use std::env;

use deepgram::{manage::self_hosted::CreateDistributionCredentials, Deepgram, DeepgramError};

#[tokio::main]
async fn main() -> Result<(), DeepgramError> {
    let deepgram_api_key =
        env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    let project_id =
        env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");

    let dg_client = Deepgram::new(&deepgram_api_key)?;
    let self_hosted = dg_client.self_hosted();

    let credentials = self_hosted
        .list_distribution_credentials(&project_id)
        .await?;

    println!(
        "{} set(s) of distribution credentials:",
        credentials.distribution_credentials.len()
    );
    for entry in &credentials.distribution_credentials {
        println!(
            "  {} (provider: {}, scopes: {:?}, owner: {})",
            entry.distribution_credentials.distribution_credentials_id,
            entry.distribution_credentials.provider,
            entry.distribution_credentials.scopes,
            entry.member.email,
        );
    }

    if env::var_os("DEEPGRAM_SELF_HOSTED_CREATE").is_none() {
        println!(
            "\nSet DEEPGRAM_SELF_HOSTED_CREATE=1 to create (and delete) a set of credentials."
        );
        return Ok(());
    }

    // `comment` is required by the API; `provider` defaults to `quay` and
    // `scopes` to `["self-hosted:products"]`.
    let request = CreateDistributionCredentials::new("created by the deepgram-rust-sdk example")
        .scopes(["self-hosted:product:api", "self-hosted:product:engine"]);

    let created = self_hosted
        .create_distribution_credentials(&project_id, &request)
        .await?;

    println!("\nCreated {}", created.distribution_credentials_id);
    println!("  provider: {}", created.provider);
    println!("  scopes:   {:?}", created.scopes);
    println!(
        "  username: {}",
        created.username.as_deref().unwrap_or("<none>")
    );
    // The secret is returned exactly once and cannot be retrieved again. It
    // is redacted in Debug output; expose it explicitly to use it.
    match &created.secret {
        Some(secret) => println!(
            "  secret:   {}  <- shown once; store it now, it cannot be retrieved again",
            secret.expose_secret()
        ),
        None => println!("  secret:   <not returned>"),
    }

    let id = created.distribution_credentials_id.to_string();

    let fetched = self_hosted
        .get_distribution_credentials(&project_id, &id)
        .await?;
    println!(
        "\nFetched {} (created {}, owner {})",
        fetched.distribution_credentials.distribution_credentials_id,
        fetched.distribution_credentials.created,
        fetched.member.email,
    );

    if env::var_os("DEEPGRAM_SELF_HOSTED_KEEP").is_some() {
        println!("\nDEEPGRAM_SELF_HOSTED_KEEP is set; leaving {id} in place.");
        return Ok(());
    }

    let deleted = self_hosted
        .delete_distribution_credentials(&project_id, &id)
        .await?;
    println!("\nDeleted {id}: {}", deleted.message);

    Ok(())
}
