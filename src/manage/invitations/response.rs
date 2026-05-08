//! Deepgram invitations API response types.

use serde::{Deserialize, Serialize};

/// Success message.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#invitations
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Message {
    #[allow(missing_docs)]
    pub message: String,
}

/// List of pending invites for a project. Returned by
/// [`Invitations::list`](super::Invitations::list).
#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Invites {
    /// Pending invites on the project.
    #[serde(default)]
    pub invites: Vec<Invite>,
}

/// A single pending invite.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Invite {
    /// Email address of the invitee.
    pub email: String,
    /// Scope (role) granted to the invitee.
    pub scope: String,
}
