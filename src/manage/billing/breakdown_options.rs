//! Options for [`Billing::breakdown`](super::Billing::breakdown).
//!
//! Mirrors the query params on `GET /v1/projects/{id}/billing/breakdown`.

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

/// Options for the billing breakdown endpoint.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
    accessor: Option<String>,
    deployment: Option<String>,
    tag: Option<String>,
    line_item: Option<String>,
    grouping: Vec<BillingGrouping>,
}

/// `?grouping=` value for `billing/breakdown`. Distinct from
/// [`crate::manage::usage::get_usage_breakdown_options::UsageGrouping`]
/// — billing supports a strict 4-value subset.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum BillingGrouping {
    /// Group by accessor (e.g. API key UUID).
    Accessor,
    /// Group by deployment type.
    Deployment,
    /// Group by line item (e.g. `streaming::nova-3`).
    LineItem,
    /// Group by tag.
    Tags,
}

impl BillingGrouping {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Accessor => "accessor",
            Self::Deployment => "deployment",
            Self::LineItem => "line_item",
            Self::Tags => "tags",
        }
    }
}

/// Builder for [`Options`].
#[derive(Debug, Default, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

/// Wire serializer for [`Options`]. Emits a flat sequence of
/// `(key, value)` tuples so that repeated `grouping=…` params work
/// (`serde_urlencoded` doesn't support sequence-typed fields inside a
/// struct, only at the top level).
#[doc(hidden)]
#[derive(Debug)]
pub struct SerializableOptions<'a>(pub(crate) &'a Options);

impl Serialize for SerializableOptions<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let opts = self.0;
        let mut seq = serializer.serialize_seq(None)?;

        if let Some(start) = &opts.start {
            seq.serialize_element(&("start", start))?;
        }
        if let Some(end) = &opts.end {
            seq.serialize_element(&("end", end))?;
        }
        if let Some(accessor) = &opts.accessor {
            seq.serialize_element(&("accessor", accessor))?;
        }
        if let Some(deployment) = &opts.deployment {
            seq.serialize_element(&("deployment", deployment))?;
        }
        if let Some(tag) = &opts.tag {
            seq.serialize_element(&("tag", tag))?;
        }
        if let Some(line_item) = &opts.line_item {
            seq.serialize_element(&("line_item", line_item))?;
        }
        for g in &opts.grouping {
            seq.serialize_element(&("grouping", g.as_str()))?;
        }

        seq.end()
    }
}

impl Options {
    /// Construct a new builder.
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    /// URL-encoded query string (without leading `?`).
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions(self))
    }
}

impl OptionsBuilder {
    /// Construct a fresh empty builder.
    pub fn new() -> Self {
        Self(Options::default())
    }

    /// Start of the requested date range. `YYYY-MM-DD` format.
    pub fn start(mut self, start: impl Into<String>) -> Self {
        self.0.start = Some(start.into());
        self
    }

    /// End of the requested date range. `YYYY-MM-DD` format.
    pub fn end(mut self, end: impl Into<String>) -> Self {
        self.0.end = Some(end.into());
        self
    }

    /// Filter by accessor (UUID).
    pub fn accessor(mut self, accessor: impl Into<String>) -> Self {
        self.0.accessor = Some(accessor.into());
        self
    }

    /// Filter by deployment type.
    pub fn deployment(mut self, deployment: impl Into<String>) -> Self {
        self.0.deployment = Some(deployment.into());
        self
    }

    /// Filter by tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.0.tag = Some(tag.into());
        self
    }

    /// Filter by line item (e.g. `streaming::nova-3`).
    pub fn line_item(mut self, line_item: impl Into<String>) -> Self {
        self.0.line_item = Some(line_item.into());
        self
    }

    /// Replace the grouping list. Multiple groupings are sent as
    /// repeated query params.
    pub fn grouping<I>(mut self, grouping: I) -> Self
    where
        I: IntoIterator<Item = BillingGrouping>,
    {
        self.0.grouping = grouping.into_iter().collect();
        self
    }

    /// Finish building.
    pub fn build(self) -> Options {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_options() {
        let q = Options::builder().build().urlencoded().unwrap();
        assert_eq!(q, "");
    }

    #[test]
    fn full_options_serialize() {
        let q = Options::builder()
            .start("2025-01-01")
            .end("2025-01-31")
            .accessor("acc-1")
            .deployment("hosted")
            .tag("prod")
            .line_item("streaming::nova-3")
            .grouping([BillingGrouping::Deployment, BillingGrouping::LineItem])
            .build()
            .urlencoded()
            .unwrap();
        assert_eq!(
            q,
            "start=2025-01-01&end=2025-01-31&accessor=acc-1\
             &deployment=hosted&tag=prod&line_item=streaming%3A%3Anova-3\
             &grouping=deployment&grouping=line_item"
        );
    }
}
