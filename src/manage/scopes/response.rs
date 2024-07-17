//! Deepgram TODO API response types.

use serde::{Deserialize, Serialize};

/// Scopes associated with the member.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#scopes-get
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Scopes {
    #[allow(missing_docs)]
    pub scopes: Vec<String>,
}
