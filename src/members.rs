//! Manage the members of a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#members

use crate::Deepgram;

/// Manage the members of a Deepgram Project.
///
/// Constructed using [`Deepgram::members`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#members
#[derive(Debug, Clone)]
pub struct Members<'a, K: AsRef<str>>(&'a Deepgram<K>);

impl<'a, K: AsRef<str>> Deepgram<K> {
    /// Construct a new [`Members`] from a [`Deepgram`].
    pub fn members(&'a self) -> Members<'a, K> {
        self.into()
    }
}

impl<'a, K: AsRef<str>> From<&'a Deepgram<K>> for Members<'a, K> {
    /// Construct a new [`Members`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram<K>) -> Self {
        Self(deepgram)
    }
}
