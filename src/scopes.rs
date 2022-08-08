//! Manage the permissions of a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#scopes

use crate::Deepgram;

/// Manage the permissions of a Deepgram Project.
///
/// Constructed using [`Deepgram::scopes`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#scopes
#[derive(Debug, Clone)]
pub struct Scopes<'a, K: AsRef<str>>(&'a Deepgram<K>);

impl<'a, K: AsRef<str>> Deepgram<K> {
    /// Construct a new [`Scopes`] from a [`Deepgram`].
    pub fn scopes(&'a self) -> Scopes<'a, K> {
        self.into()
    }
}

impl<'a, K: AsRef<str>> From<&'a Deepgram<K>> for Scopes<'a, K> {
    /// Construct a new [`Scopes`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram<K>) -> Self {
        Self(deepgram)
    }
}
