//! Response types that are shared by multiple parts of the API.

use serde::Deserialize;

/// A success message.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct Message {
    #[allow(missing_docs)]
    pub message: String,
}
