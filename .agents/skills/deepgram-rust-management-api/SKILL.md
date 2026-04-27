---
name: deepgram-rust-management-api
description: Use when implementing Deepgram project, key, member, scope, billing, invitation, or usage operations from the Rust SDK, including manage feature flags and the real Deepgram::projects/keys/members/scopes/billing/usage APIs.
---

# Using Deepgram Management API (Rust SDK)

Use this skill for project/account administration against Deepgram's Manage APIs.

## When to use this product

- Listing or updating projects.
- Managing project API keys, members, scopes, invitations, balances, and usage.
- Explaining which admin surfaces are available in the current crate.

## Authentication

For a management-only install:

```toml
[dependencies]
deepgram = { version = "0.9.2", default-features = false, features = ["manage"] }
tokio = { version = "1", features = ["full"] }
```

```rust
let dg = deepgram::Deepgram::new(std::env::var("DEEPGRAM_API_KEY")?)?;
```

- Manage APIs require a regular API key with `Token` auth.
- Temporary auth tokens created via `auth().grant(...)` do **not** work for Manage APIs.

## Quick start

## Quick start: projects + usage

```rust
use deepgram::{
    manage::{projects::options::Options as ProjectOptions, usage::get_usage_options},
    Deepgram,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPGRAM_API_KEY")?;
    let project_id = std::env::var("DEEPGRAM_PROJECT_ID")?;
    let dg = Deepgram::new(&api_key)?;

    let projects = dg.projects().list().await?;
    println!("{projects:#?}");

    let project = dg.projects().get(&project_id).await?;
    println!("{project:#?}");

    let update = ProjectOptions::builder()
        .name("The Transcribinator")
        .company("Doofenshmirtz Evil Incorporated")
        .build();
    let message = dg.projects().update(&project_id, &update).await?;
    println!("{}", message.message);

    let usage_options = get_usage_options::Options::builder().build();
    let usage = dg.usage().get_usage(&project_id, &usage_options).await?;
    println!("{usage:#?}");

    Ok(())
}
```

## Key parameters

- Project APIs: `projects().list()`, `get(project_id)`, `update(project_id, &options)`, `delete(project_id)`.
- Key APIs: `keys().list(project_id)`, `get(project_id, key_id)`, `create(project_id, &options)`, `delete(project_id, key_id)`.
- Member APIs: `members().list_members(project_id)`, `remove_member(project_id, member_id)`.
- Scope APIs: `scopes().get_scope(project_id, member_id)`, `update_scope(project_id, member_id, scope)`.
- Billing APIs: `billing().list_balance(project_id)`, `get_balance(project_id, balance_id)`.
- Usage APIs: `usage().list_requests(...)`, `get_request(...)`, `get_usage(...)`, `get_fields(...)`.
- Invitation API: `invitations().leave_project(project_id)`.

## API reference (layered)

1. **In-repo**
   - `src/manage/projects.rs`
   - `src/manage/keys.rs`
   - `src/manage/members.rs`
   - `src/manage/scopes.rs`
   - `src/manage/billing.rs`
   - `src/manage/usage.rs`
   - `src/manage/invitations.rs`
   - `examples/manage/*.rs`
2. **OpenAPI**
   - Raw spec: `https://developers.deepgram.com/openapi.yaml`
   - Examples: `https://developers.deepgram.com/reference/manage/usage/get`, `https://developers.deepgram.com/reference/manage/keys/list`
3. **AsyncAPI**
   - Not applicable for the Rust crate's current management surface
4. **Context7**
   - `/llmstxt/developers_deepgram_llms_txt`
5. **Product docs**
   - `https://developers.deepgram.com/reference/`
   - `https://developers.deepgram.com/docs/create-additional-api-keys`

## Gotchas

1. **Use API keys, not temp tokens.** `auth().grant(...)` explicitly does not authorize Manage APIs.
2. **Examples in `examples/manage/` are useful, but source files are the source of truth.** Prefer `src/manage/*.rs` when you need exact method names and return types.
3. **Admin traffic stays on hosted Deepgram.** Even with `with_base_url(...)`, billing/usage/key-management still target `https://api.deepgram.com`.

## Example files in this repo

- `examples/manage/projects.rs`
- `examples/manage/keys.rs`
- `examples/manage/members.rs`
- `examples/manage/scopes.rs`
- `examples/manage/billing.rs`
- `examples/manage/usage.rs`
- `examples/manage/invitations.rs`

## Central product skills

For cross-language Deepgram product knowledge — the consolidated API reference, documentation finder, focused runnable recipes, third-party integration examples, and MCP setup — install the central skills:

```bash
npx skills add deepgram/skills
```

This SDK ships language-idiomatic code skills; `deepgram/skills` ships cross-language product knowledge (see `api`, `docs`, `recipes`, `examples`, `starters`, `setup-mcp`).
