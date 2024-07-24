//! Deepgram projects API response types.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Returned by [`Projects::list`](super::Projects::list).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#projects-get-projects
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Projects {
    #[allow(missing_docs)]
    pub projects: Vec<Project>,
}

/// Returned by [`Projects::get`](super::Projects::get).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#projects-get-project
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Project {
    #[allow(missing_docs)]
    pub project_id: Uuid,

    #[allow(missing_docs)]
    pub name: String,

    #[allow(missing_docs)]
    pub company: Option<String>,
}
