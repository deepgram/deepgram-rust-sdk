//! Deepgram auth API response types.

use serde::{Deserialize, Serialize};

/// Returned by [`Auth::grant`](super::Auth::grant).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/reference/auth/tokens/grant
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct GrantResponse {
    /// JSON Web Token (JWT)
    pub access_token: String,

    /// Time in seconds until the JWT expires
    pub expires_in: Option<f64>,
}
