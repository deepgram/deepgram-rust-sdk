//! Set options for [`Usage::list_requests`](super::Usage::list_requests).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#usage-all

use serde::Serialize;

use super::get_usage_options::{Deployment, Endpoint, Method};

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
    page: Option<usize>,
    accessor: Option<String>,
    request_id: Option<String>,
    deployment: Option<Deployment>,
    endpoint: Option<Endpoint>,
    method: Option<Method>,
    status: Option<StatusFilter>,
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

#[derive(Debug, PartialEq, Eq, Clone)]
enum StatusFilter {
    Known(Status),
    Other(String),
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

    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    accessor: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    deployment: &'a Option<Deployment>,

    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: &'a Option<Endpoint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    method: &'a Option<Method>,

    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'a str>,
}

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::new()
    }

    /// Return the Options in urlencoded format. If serialization would
    /// fail, this will also return an error.
    ///
    /// This is intended primarily to help with debugging API requests.
    ///
    /// ```
    /// use deepgram::manage::usage::list_requests_options::Options;
    /// let options = Options::builder()
    ///     .start("2024-04-10T00:00:00Z")
    ///     .end("2024-10-10")
    ///     .limit(100)
    ///     .build();
    /// assert_eq!(&options.urlencoded().unwrap(), "start=2024-04-10T00%3A00%3A00Z&end=2024-10-10&limit=100")
    /// ```
    ///
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions::from(self))
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new() -> Self {
        Self(Options {
            start: None,
            end: None,
            limit: None,
            page: None,
            accessor: None,
            request_id: None,
            deployment: None,
            endpoint: None,
            method: None,
            status: None,
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

    /// Set the maximum number of results to return per page.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::list_requests_options::Options;
    /// #
    /// let options1 = Options::builder()
    ///     .limit(42)
    ///     .build();
    /// ```
    pub fn limit(mut self, limit: usize) -> Self {
        self.0.limit = Some(limit);
        self
    }

    /// Set the zero-based result page.
    pub fn page(mut self, page: usize) -> Self {
        self.0.page = Some(page);
        self
    }

    /// Filter by accessor ID.
    pub fn accessor(mut self, accessor: impl Into<String>) -> Self {
        self.0.accessor = Some(accessor.into());
        self
    }

    /// Filter by Deepgram request ID.
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.0.request_id = Some(request_id.into());
        self
    }

    /// Filter by deployment.
    pub fn deployment(mut self, deployment: Deployment) -> Self {
        self.0.deployment = Some(deployment);
        self
    }

    /// Filter by API endpoint.
    pub fn endpoint(mut self, endpoint: Endpoint) -> Self {
        self.0.endpoint = Some(endpoint);
        self
    }

    /// Filter by processing method.
    pub fn method(mut self, method: Method) -> Self {
        self.0.method = Some(method);
        self
    }

    /// Limits results to requests to requests that either succeeded or failed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::usage::list_requests_options::{Options, Status};
    /// #
    /// let options1 = Options::builder()
    ///     .status(Status::Succeeded)
    ///     .build();
    /// ```
    pub fn status(mut self, status: Status) -> Self {
        self.0.status = Some(StatusFilter::Known(status));
        self
    }

    /// Filter by a server-supported status value not yet represented by
    /// [`Status`].
    pub fn status_other(mut self, status: impl Into<String>) -> Self {
        self.0.status = Some(StatusFilter::Other(status.into()));
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
            page,
            accessor,
            request_id,
            deployment,
            endpoint,
            method,
            status,
        } = options;

        Self {
            start,
            end,
            limit: *limit,
            page: *page,
            accessor,
            request_id,
            deployment,
            endpoint,
            method,
            status: match status {
                Some(StatusFilter::Known(Status::Succeeded)) => Some("succeeded"),
                Some(StatusFilter::Known(Status::Failed)) => Some("failed"),
                Some(StatusFilter::Other(value)) => Some(value.as_str()),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Deployment, Endpoint, Method, Options, Status};

    #[test]
    fn status_remains_copy() {
        fn assert_copy<T: Copy>() {}

        assert_copy::<Status>();
    }

    #[test]
    fn serializes_every_request_list_filter() {
        let options = Options::builder()
            .start("2026-01-01T00:00:00Z")
            .end("2026-01-31T23:59:59Z")
            .limit(25)
            .page(2)
            .accessor("accessor-id")
            .request_id("request-id")
            .deployment(Deployment::Hosted)
            .endpoint(Endpoint::Listen)
            .method(Method::Streaming)
            .status(Status::Succeeded)
            .build();

        assert_eq!(
            options.urlencoded().unwrap(),
            "start=2026-01-01T00%3A00%3A00Z&end=2026-01-31T23%3A59%3A59Z&limit=25&page=2&accessor=accessor-id&request_id=request-id&deployment=hosted&endpoint=listen&method=streaming&status=succeeded"
        );
    }

    #[test]
    fn serializes_server_statuses_not_yet_modeled_by_the_sdk() {
        let options = Options::builder()
            .status_other("partially_succeeded")
            .build();

        assert_eq!(options.urlencoded().unwrap(), "status=partially_succeeded");
    }
}
