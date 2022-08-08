//! Manage the keys for a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#keys

use crate::Deepgram;

/// Manage the keys for a Deepgram Project.
///
/// Constructed using [`Deepgram::keys`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#keys
#[derive(Debug, Clone)]
pub struct Keys<'a, K: AsRef<str>>(&'a Deepgram<K>);

impl<'a, K: AsRef<str>> Deepgram<K> {
    /// Construct a new [`Keys`] from a [`Deepgram`].
    pub fn keys(&'a self) -> Keys<'a, K> {
        self.into()
    }
}

impl<'a, K: AsRef<str>> From<&'a Deepgram<K>> for Keys<'a, K> {
    /// Construct a new [`Keys`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram<K>) -> Self {
        Self(deepgram)
    }
}
