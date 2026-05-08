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
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
    limit: Option<usize>,
    page: Option<u64>,
    accessor: Option<String>,
    request_id: Option<String>,
    deployment: Option<DeploymentFilter>,
    endpoint: Option<EndpointFilter>,
    method: Option<HttpMethodFilter>,
    status: Option<Status>,
}

/// `?status=` filter values.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Status {
    #[allow(missing_docs)]
    Succeeded,

    #[allow(missing_docs)]
    Failed,
}

/// `?deployment=` filter values for `list_requests`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum DeploymentFilter {
    /// Hosted (SaaS).
    Hosted,
    /// Beta program.
    Beta,
    /// Self-hosted.
    SelfHosted,
}

impl DeploymentFilter {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Hosted => "hosted",
            Self::Beta => "beta",
            Self::SelfHosted => "self-hosted",
        }
    }
}

/// `?endpoint=` filter values for `list_requests`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum EndpointFilter {
    /// `/v1/listen` (STT).
    Listen,
    /// `/v1/read` (text intelligence).
    Read,
    /// `/v1/speak` (TTS).
    Speak,
    /// `/v1/agent/converse` (Voice Agent).
    Agent,
}

impl EndpointFilter {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Listen => "listen",
            Self::Read => "read",
            Self::Speak => "speak",
            Self::Agent => "agent",
        }
    }
}

/// `?method=` filter values for `list_requests`. Distinct from
/// [`super::get_usage_options::Method`] (Listen/Speak product method).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum HttpMethodFilter {
    /// Synchronous request/response.
    Sync,
    /// Asynchronous (callback-based) request/response.
    Async,
    /// Streaming WebSocket connection.
    Streaming,
}

impl HttpMethodFilter {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::Async => "async",
            Self::Streaming => "streaming",
        }
    }
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
    page: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    accessor: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    deployment: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'static str>,
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
        Self(Options::default())
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
    pub fn limit(mut self, limit: usize) -> Self {
        self.0.limit = Some(limit);
        self
    }

    /// Set the page number to return.
    pub fn page(mut self, page: u64) -> Self {
        self.0.page = Some(page);
        self
    }

    /// Filter by accessor (typically an API key UUID).
    pub fn accessor(mut self, accessor: impl Into<String>) -> Self {
        self.0.accessor = Some(accessor.into());
        self
    }

    /// Filter by request ID.
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.0.request_id = Some(request_id.into());
        self
    }

    /// Filter by deployment type (hosted / beta / self-hosted).
    pub fn deployment(mut self, deployment: DeploymentFilter) -> Self {
        self.0.deployment = Some(deployment);
        self
    }

    /// Filter by endpoint (listen / read / speak / agent).
    pub fn endpoint(mut self, endpoint: EndpointFilter) -> Self {
        self.0.endpoint = Some(endpoint);
        self
    }

    /// Filter by HTTP method (sync / async / streaming).
    pub fn method(mut self, method: HttpMethodFilter) -> Self {
        self.0.method = Some(method);
        self
    }

    /// Limit results to requests that succeeded or failed.
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
            deployment: deployment.as_ref().map(DeploymentFilter::as_str),
            endpoint: endpoint.as_ref().map(EndpointFilter::as_str),
            method: method.as_ref().map(HttpMethodFilter::as_str),
            status: match status {
                Some(Status::Succeeded) => Some("succeeded"),
                Some(Status::Failed) => Some("failed"),
                None => None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_new_filters_serialize() {
        let q = Options::builder()
            .page(3)
            .accessor("abc")
            .request_id("def")
            .deployment(DeploymentFilter::SelfHosted)
            .endpoint(EndpointFilter::Agent)
            .method(HttpMethodFilter::Streaming)
            .build()
            .urlencoded()
            .unwrap();
        assert_eq!(
            q,
            "page=3&accessor=abc&request_id=def&deployment=self-hosted\
             &endpoint=agent&method=streaming"
        );
    }
}
