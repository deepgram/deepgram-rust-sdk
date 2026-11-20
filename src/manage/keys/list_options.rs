//! Query options for [`Keys::list_with_options`](super::Keys::list_with_options).

use serde::Serialize;

/// Status filter for project API keys.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Status {
    /// Include active keys.
    Active,
    /// Include expired keys.
    Expired,

    /// A server-supported status value not yet represented by this SDK.
    ///
    /// `Other` compares by its string, so `Other("active".into())` is not equal to
    /// [`Status::Active`] even though both serialize to the same query value.
    Other(String),
}

/// Query options for listing project API keys.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Options {
    status: Option<Status>,
}

/// Builds [`Options`] for listing project API keys.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'a str>,
}

impl Options {
    /// Start building query options.
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    /// Serialize these options as a URL query string.
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions::from(self))
    }
}

impl OptionsBuilder {
    /// Filter keys by status.
    pub fn status(mut self, value: Status) -> Self {
        self.0.status = Some(value);
        self
    }

    /// Finish building the options.
    pub fn build(self) -> Options {
        self.0
    }
}

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        Self {
            status: options.status.as_ref().map(AsRef::as_ref),
        }
    }
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Other(value) => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Options, Status};

    #[test]
    fn serializes_status() {
        let options = Options::builder().status(Status::Expired).build();

        assert_eq!(options.urlencoded().unwrap(), "status=expired");
    }

    #[test]
    fn serializes_server_statuses_not_yet_modeled_by_the_sdk() {
        let options = Options::builder()
            .status(Status::Other("pending_rotation".to_owned()))
            .build();

        assert_eq!(options.urlencoded().unwrap(), "status=pending_rotation");
    }
}
