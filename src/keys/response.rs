//! Deepgram keys API response types.

use serde::Deserialize;
use uuid::Uuid;

use crate::members::response::Member;

pub use crate::response::Message;

/// Returned by [`Keys::list`](super::Keys::list).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#keys-get-keys
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct MembersAndApiKeys {
    #[allow(missing_docs)]
    pub api_keys: Vec<MemberAndApiKey>,
}

/// Returned by [`Keys::get`](super::Keys::get).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#keys-get-key
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct MemberAndApiKey {
    #[allow(missing_docs)]
    pub member: Member,

    #[allow(missing_docs)]
    pub api_key: ApiKey,
}

/// Details of a single API key.
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#keys-get-key
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct ApiKey {
    #[allow(missing_docs)]
    pub api_key_id: Uuid,

    #[allow(missing_docs)]
    pub comment: String,

    #[allow(missing_docs)]
    pub scopes: Vec<String>,

    #[allow(missing_docs)]
    pub tags: Vec<String>,

    #[allow(missing_docs)]
    pub created: String,

    #[allow(missing_docs)]
    pub expiration_date: Option<String>,
}

/// Returned by [`Keys::create`](super::Keys::create).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#keys-create
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[non_exhaustive]
pub struct NewApiKey {
    #[allow(missing_docs)]
    pub api_key_id: Uuid,

    #[allow(missing_docs)]
    pub key: String,

    #[allow(missing_docs)]
    pub comment: String,

    #[allow(missing_docs)]
    pub scopes: Vec<String>,

    #[allow(missing_docs)]
    pub tags: Vec<String>,

    #[allow(missing_docs)]
    pub created: String,

    #[allow(missing_docs)]
    pub expiration_date: Option<String>,
}
