//! Query options for [`Billing::get_fields`](super::Billing::get_fields).

use serde::Serialize;

/// Query options for the billing fields endpoint.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
}

/// Builds [`Options`] for the billing fields endpoint.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    start: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end: &'a Option<String>,
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
    /// Set the inclusive start date (`YYYY-MM-DD`).
    pub fn start(mut self, value: impl Into<String>) -> Self {
        self.0.start = Some(value.into());
        self
    }

    /// Set the inclusive end date (`YYYY-MM-DD`).
    pub fn end(mut self, value: impl Into<String>) -> Self {
        self.0.end = Some(value.into());
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
            start: &options.start,
            end: &options.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Options;

    #[test]
    fn serializes_date_range() {
        let options = Options::builder()
            .start("2026-01-01")
            .end("2026-01-31")
            .build();

        assert_eq!(
            options.urlencoded().unwrap(),
            "start=2026-01-01&end=2026-01-31"
        );
    }
}
