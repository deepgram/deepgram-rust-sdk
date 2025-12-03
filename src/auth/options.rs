//! Set options for [`Auth::grant`](super::Auth::grant).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/auth/tokens/grant

use serde::Serialize;

/// Used as a parameter for [`Auth::grant`](super::Auth::grant).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/reference/auth/tokens/grant
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Options {
    ttl_seconds: Option<f64>,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone, Default)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl_seconds: Option<f64>,
}

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::new()
    }

    /// Return the Options in json format. If serialization would
    /// fail, this will also return an error.
    ///
    /// This is intended primarily to help with debugging API requests.
    ///
    /// ```
    /// use deepgram::auth::options::Options;
    /// let options = Options::builder()
    ///     .ttl_seconds(60.0)
    ///     .build();
    /// assert_eq!(
    ///     &options.json().unwrap(),
    ///     r#"{"ttl_seconds":60.0}"#)
    /// ```
    ///
    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&SerializableOptions::from(self))
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new() -> Self {
        Self(Options { ttl_seconds: None })
    }

    /// Set the time to live in seconds for the token.
    ///
    /// Valid range is 1-3600 seconds. Defaults to 30 seconds if not specified.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::auth::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .ttl_seconds(300.0)
    ///     .build();
    /// ```
    pub fn ttl_seconds(mut self, ttl_seconds: f64) -> Self {
        self.0.ttl_seconds = Some(ttl_seconds);
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options {
        self.0
    }
}

impl From<&Options> for SerializableOptions {
    fn from(options: &Options) -> Self {
        Self {
            ttl_seconds: options.ttl_seconds,
        }
    }
}
