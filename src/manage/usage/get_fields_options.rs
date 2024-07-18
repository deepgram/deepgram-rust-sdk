//! Set options for [`Usage::get_fields`](super::Usage::get_fields).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#usage-fields

use serde::Serialize;

/// Used as a parameter for [`Usage::get_fields`](super::Usage::get_fields).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#usage-fields
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    start: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    end: &'a Option<String>,
}

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::new()
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new() -> Self {
        Self(Options {
            start: None,
            end: None,
        })
    }

    /// Set the time range start date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::list_requests_options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .start("1970-01-01")
    ///     .build();
    /// ```
    pub fn start(mut self, start: impl Into<String>) -> Self {
        self.0.start = Some(start.into());
        self
    }

    /// Set the time range end date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::list_requests_options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .end("2038-01-19")
    ///     .build();
    /// ```
    pub fn end(mut self, end: impl Into<String>) -> Self {
        self.0.end = Some(end.into());
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options {
        self.0
    }
}

impl Default for OptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        // Destructuring it makes sure that we don't forget to use any of it
        let Options { start, end } = options;

        Self { start, end }
    }
}
