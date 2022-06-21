//! Get the outstanding balances for a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#billing

use crate::Deepgram;

/// Get the outstanding balances for a Deepgram Project.
///
/// Constructed using [`Deepgram::billing`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#billing
#[derive(Debug, Clone)]
pub struct Billing<'a, K: AsRef<str>>(&'a Deepgram<K>);

impl<'a, K: AsRef<str>> Deepgram<K> {
    /// Construct a new [`Billing`] from a [`Deepgram`].
    pub fn billing(&'a self) -> Billing<'a, K> {
        Billing(self)
    }
}

impl<'a, K: AsRef<str>> From<&'a Deepgram<K>> for Billing<'a, K> {
    /// Construct a new [`Billing`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram<K>) -> Self {
        deepgram.billing()
    }
}
