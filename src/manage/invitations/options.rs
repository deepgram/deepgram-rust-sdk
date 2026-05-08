//! Set options for [`Invitations::create`](super::super::invitations::Invitations::create).

use serde::Serialize;

/// Request body for creating a project invite.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct Options {
    /// Email address of the invitee.
    pub email: String,
    /// Scope (role) to grant the invitee — e.g. `member`, `admin`.
    pub scope: String,
}

impl Options {
    /// Construct an invite request.
    pub fn new(email: impl Into<String>, scope: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            scope: scope.into(),
        }
    }
}
