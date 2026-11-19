//! Read-only live checks for the Management API additions.
//!
//! Run with:
//! `DEEPGRAM_API_KEY=<key> cargo test --all-features --test manage_e2e -- --ignored`

#![cfg(feature = "manage")]

use std::{env, io};

use deepgram::{
    manage::{billing, keys, projects, usage},
    Deepgram,
};

/// Calls every read-only Management API added in the Phase 1 parity work.
///
/// The test discovers an accessible project with `projects().list()` and
/// performs no mutations: it does not create or revoke invitations, leave a
/// project, modify settings, or create/delete keys.
#[tokio::test]
#[ignore = "requires DEEPGRAM_API_KEY and live Management API access"]
async fn phase_one_management_endpoints_are_available() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("DEEPGRAM_API_KEY").map_err(|_| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "DEEPGRAM_API_KEY must be set to run this ignored live test",
        )
    })?;
    let dg = Deepgram::new(api_key)?;

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

    let project_options = projects::get_options::Options::builder()
        .limit(1)
        .page(0)
        .build();
    let _ = dg
        .projects()
        .get_with_options(&project_id, &project_options)
        .await?;

    let key_options = keys::list_options::Options::builder()
        .status(keys::list_options::Status::Active)
        .build();
    let _ = dg
        .keys()
        .list_with_options(&project_id, &key_options)
        .await?;

    let billing_breakdown_options = billing::breakdown_options::Options::builder().build();
    let _ = dg
        .billing()
        .get_breakdown(&project_id, &billing_breakdown_options)
        .await?;

    let billing_fields_options = billing::fields_options::Options::builder().build();
    let _ = dg
        .billing()
        .get_fields(&project_id, &billing_fields_options)
        .await?;

    let purchase_options = billing::purchases_options::Options::builder()
        .limit(1)
        .build();
    let purchases = dg
        .billing()
        .list_purchases(&project_id, &purchase_options)
        .await?;
    assert!(
        purchases.page.is_some(),
        "the purchases response includes pagination page metadata"
    );
    assert!(
        purchases.limit.is_some(),
        "the purchases response includes pagination limit metadata"
    );

    let _ = dg.invitations().list(&project_id).await?;

    let usage_breakdown_options = usage::get_breakdown_options::Options::builder().build();
    let _ = dg
        .usage()
        .get_breakdown(&project_id, &usage_breakdown_options)
        .await?;

    Ok(())
}
