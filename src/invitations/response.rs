//! Deepgram invitations API response types.

use serde::Deserialize;

/// Success message.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#invitations
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Message {
    #[allow(missing_docs)]
    pub message: String,
}
