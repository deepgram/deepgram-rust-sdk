//! Query options for [`Projects::get_with_options`](super::Projects::get_with_options).

use serde::Serialize;

/// Query options for retrieving a project.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Options {
    limit: Option<usize>,
    page: Option<usize>,
}

/// Builds [`Options`] for project retrieval.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<usize>,
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
    /// Set the maximum number of results.
    pub fn limit(mut self, value: usize) -> Self {
        self.0.limit = Some(value);
        self
    }

    /// Set the zero-based result page.
    pub fn page(mut self, value: usize) -> Self {
        self.0.page = Some(value);
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
            page: options.page,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    #[test]
    fn serializes_pagination() {
        let options = Options::builder().limit(25).page(2).build();

        assert_eq!(options.urlencoded().unwrap(), "limit=25&page=2");
    }
}
