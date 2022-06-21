//! Get the usage data of a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#usage

use crate::Deepgram;

/// Get the usage data of a Deepgram Project.
///
/// Constructed using [`Deepgram::usage`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#usage
#[derive(Debug, Clone)]
pub struct Usage<'a, K: AsRef<str>>(&'a Deepgram<K>);

impl<'a, K: AsRef<str>> Deepgram<K> {
    /// Construct a new [`Usage`] from a [`Deepgram`].
    pub fn usage(&'a self) -> Usage<'a, K> {
        self.into()
    }
}

impl<'a, K: AsRef<str>> From<&'a Deepgram<K>> for Usage<'a, K> {
    /// Construct a new [`Usage`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram<K>) -> Self {
        Self(deepgram)
    }
}
