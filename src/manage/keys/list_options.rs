//! Query options for [`Keys::list_with_options`](super::Keys::list_with_options).

use serde::Serialize;

/// Status filter for project API keys.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Status {
    /// Include active keys.
    Active,
    /// Include expired keys.
    Expired,
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
pub(crate) struct SerializableOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'static str>,
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

impl From<&Options> for SerializableOptions {
    fn from(options: &Options) -> Self {
        Self {
            status: match options.status {
                Some(Status::Active) => Some("active"),
                Some(Status::Expired) => Some("expired"),
                None => None,
            },
        }
    }
}
