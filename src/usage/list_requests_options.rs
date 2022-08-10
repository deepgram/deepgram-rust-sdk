//! Set options for [`Usage::list_requests`](super::Usage::list_requests).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#usage-all

use serde::Serialize;

/// Used as a parameter for [`Usage::list_requests`](super::Usage::list_requests).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#usage-all
#[derive(Debug, PartialEq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
    limit: Option<usize>,
    status: Option<Status>,
}

/// Used as a parameter for [`OptionsBuilder::status`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Status {
    #[allow(missing_docs)]
    Succeeded,

    #[allow(missing_docs)]
    Failed,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(super) struct SerializableOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    start: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    end: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'static str>,
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
            limit: None,
            status: None,
        })
    }

    /// Set the time range start date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::usage::list_requests_options::Options;
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
    /// # use deepgram::usage::list_requests_options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .end("2038-01-19")
    ///     .build();
    /// ```
    pub fn end(mut self, end: impl Into<String>) -> Self {
        self.0.end = Some(end.into());
        self
    }

    /// Set the maximum number of results to return per page.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::usage::list_requests_options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .limit(42)
    ///     .build();
    /// ```
    pub fn limit(mut self, limit: usize) -> Self {
        self.0.limit = Some(limit);
        self
    }

    /// Limits results to requests to requests that either succeeded or failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::usage::list_requests_options::{Options, Status};
    /// #
    /// let options1 = Options::builder()
    ///     .status(Status::Succeeded)
    ///     .build();
    /// ```
    pub fn status(mut self, status: Status) -> Self {
        self.0.status = Some(status);
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
        let Options {
            start,
            end,
            limit,
            status,
        } = options;

        Self {
            start,
            end,
            limit: *limit,
            status: match status {
                Some(Status::Succeeded) => Some("succeeded"),
                Some(Status::Failed) => Some("failed"),
                None => None,
            },
        }
    }
}
