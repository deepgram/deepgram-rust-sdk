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

/// Outstanding project invitations.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Invites {
    /// Invitations returned by the API.
    pub invites: Option<Vec<Invite>>,
}

/// One outstanding project invitation.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Invite {
    /// Invited email address.
    pub email: String,
    /// Scope granted when the invitation is accepted.
    pub scope: String,
}

#[cfg(test)]
mod tests {
    use super::Invites;

    #[test]
    fn deserializes_outstanding_invitations() {
        let invites: Invites = serde_json::from_str(
            r#"{
                "invites": [
                    { "email": "developer@example.com", "scope": "member" }
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(invites.invites.unwrap()[0].scope, "member");
    }
}
