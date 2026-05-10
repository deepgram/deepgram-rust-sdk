//! Self-hosted Deepgram management endpoints.
//!
//! Currently exposes [distribution credentials][dc], used by
//! self-hosted operators to provision per-project image-pull
//! credentials for the Quay distribution registry.
//!
//! [dc]: distribution_credentials

pub mod distribution_credentials;
