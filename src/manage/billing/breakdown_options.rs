//! Query options for [`Billing::get_breakdown`](super::Billing::get_breakdown).

use serde::{ser::SerializeSeq, Serialize};

pub use crate::manage::usage::get_usage_options::Deployment;

/// Query options for a project billing breakdown.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
    accessor: Option<String>,
    deployment: Option<Deployment>,
    tag: Option<String>,
    line_item: Option<String>,
    grouping: Vec<Grouping>,
}

/// Builds [`Options`] for a project billing breakdown.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct OptionsBuilder(Options);

pub(crate) struct SerializableOptions<'a>(&'a Options);

/// Dimension used to group a billing breakdown.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Grouping {
    #[allow(missing_docs)]
    Accessor,

    #[allow(missing_docs)]
    Deployment,

    #[allow(missing_docs)]
    LineItem,

    #[allow(missing_docs)]
    Tags,

    /// A server-supported grouping dimension not yet represented by this SDK.
    Other(String),
}

impl AsRef<str> for Grouping {
    fn as_ref(&self) -> &str {
        match self {
            Self::Accessor => "accessor",
            Self::Deployment => "deployment",
            Self::LineItem => "line_item",
            Self::Tags => "tags",
            Self::Other(value) => value,
        }
    }
}

impl Serialize for Grouping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
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

    /// Filter by accessor ID.
    pub fn accessor(mut self, value: impl Into<String>) -> Self {
        self.0.accessor = Some(value.into());
        self
    }

    /// Filter by deployment.
    pub fn deployment(mut self, value: Deployment) -> Self {
        self.0.deployment = Some(value);
        self
    }

    /// Filter by tag.
    pub fn tag(mut self, value: impl Into<String>) -> Self {
        self.0.tag = Some(value.into());
        self
    }

    /// Filter by billing line item.
    pub fn line_item(mut self, value: impl Into<String>) -> Self {
        self.0.line_item = Some(value.into());
        self
    }

    /// Group results by one or more dimensions.
    pub fn grouping<I>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = Grouping>,
    {
        self.0.grouping.extend(values);
        self
    }

    /// Finish building the options.
    pub fn build(self) -> Options {
        self.0
    }
}

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        Self(options)
    }
}

impl Serialize for SerializableOptions<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Options {
            start,
            end,
            accessor,
            deployment,
            tag,
            line_item,
            grouping,
        } = self.0;
        let mut seq = serializer.serialize_seq(None)?;

        for (name, value) in [
            ("start", start),
            ("end", end),
            ("accessor", accessor),
            ("tag", tag),
            ("line_item", line_item),
        ] {
            if let Some(value) = value {
                seq.serialize_element(&(name, value))?;
            }
        }
        if let Some(deployment) = deployment {
            seq.serialize_element(&("deployment", deployment))?;
        }
        for value in grouping {
            seq.serialize_element(&("grouping", value))?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests {
    use super::{Grouping, Options};

    #[test]
    fn serializes_repeated_grouping_keys() {
        let options = Options::builder()
            .start("2026-01-01")
            .grouping([Grouping::Deployment, Grouping::LineItem])
            .build();

        assert_eq!(
            options.urlencoded().unwrap(),
            "start=2026-01-01&grouping=deployment&grouping=line_item"
        );
    }

    #[test]
    fn serializes_server_groupings_not_yet_modeled_by_the_sdk() {
        let options = Options::builder()
            .grouping([Grouping::Other("future_dimension".to_owned())])
            .build();

        assert_eq!(options.urlencoded().unwrap(), "grouping=future_dimension");
    }
}
