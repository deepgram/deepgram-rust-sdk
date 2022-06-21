//! Manage the invitations to a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#invitations

use crate::Deepgram;

/// Manage the invitations to a Deepgram Project.
///
/// Constructed using [`Deepgram::invitations`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#invitations
#[derive(Debug, Clone)]
pub struct Invitations<'a, K: AsRef<str>>(&'a Deepgram<K>);

impl<'a, K: AsRef<str>> Deepgram<K> {
    /// Construct a new [`Invitations`] from a [`Deepgram`].
    pub fn invitations(&'a self) -> Invitations<'a, K> {
        self.into()
    }
}

impl<'a, K: AsRef<str>> From<&'a Deepgram<K>> for Invitations<'a, K> {
    /// Construct a new [`Invitations`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram<K>) -> Self {
        Self(deepgram)
    }
}
