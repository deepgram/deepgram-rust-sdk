//! Query options for [`Billing::list_purchases`](super::Billing::list_purchases).

use serde::Serialize;

/// Query options for the project purchases endpoint.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Options {
    limit: Option<usize>,
}

/// Builds [`Options`] for the project purchases endpoint.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
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
    /// Set the maximum number of purchase records to return.
    pub fn limit(mut self, value: usize) -> Self {
        self.0.limit = Some(value);
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
            limit: options.limit,
        }
    }
}
