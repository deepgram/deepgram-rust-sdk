//! Set options for [`Usage::get_usage_breakdown`](super::Usage::get_usage_breakdown).
//!
//! Mirrors the query params on `GET /v1/projects/{id}/usage/breakdown`
//! in `openapi/paths/manage.v1.yml`.
//!
//! Note: this endpoint accepts a *large* set of boolean filter params
//! (alternatives, callback, channels, custom_intent_*, custom_topic_*,
//! detect_entities, detect_language, diarize, dictation, encoding,
//! endpoint, extra, filler_words, intents, keyterm, keywords, language,
//! measurements, model, multichannel, numerals, paragraphs,
//! profanity_filter, punctuate, redact, replace, sample_rate, search,
//! sentiment, smart_format, summarize, tag, topics, utt_split,
//! utterances, version) — same shape as the deprecated `/usage`
//! endpoint. For now this options struct exposes the most commonly
//! used filters; additional filter params can be passed via
//! [`OptionsBuilder::extra_query`] until full coverage is added in a
//! follow-up.

use serde::Serialize;

/// Used as a parameter for
/// [`Usage::get_usage_breakdown`](super::Usage::get_usage_breakdown).
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Options {
    start: Option<String>,
    end: Option<String>,
    grouping: Option<UsageGrouping>,
    accessor: Option<String>,
    deployment: Option<String>,
    endpoint: Option<String>,
    model: Option<String>,
    tag: Option<String>,
    extra_query: Vec<(String, String)>,
}

/// `?grouping=` value for `usage/breakdown`. Distinct from
/// [`crate::manage::billing::breakdown_options::BillingGrouping`]
/// (which is a 4-value subset — accessor / deployment / line_item /
/// tags).
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum UsageGrouping {
    /// Group by accessor (e.g. API key).
    Accessor,
    /// Group by endpoint (`listen` / `read` / `speak` / `agent`).
    Endpoint,
    /// Group by feature set.
    FeatureSet,
    /// Group by model UUIDs.
    Models,
    /// Group by HTTP method (`sync` / `async` / `streaming`).
    Method,
    /// Group by tag.
    Tags,
    /// Group by deployment type (`hosted` / `beta` / `self-hosted`).
    Deployment,
}

impl UsageGrouping {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Accessor => "accessor",
            Self::Endpoint => "endpoint",
            Self::FeatureSet => "feature_set",
            Self::Models => "models",
            Self::Method => "method",
            Self::Tags => "tags",
            Self::Deployment => "deployment",
        }
    }
}

/// Builder for [`Options`].
#[derive(Debug, Default, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    start: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    end: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    grouping: Option<&'static str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    accessor: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    deployment: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    model: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    tag: &'a Option<String>,

    /// Flattened list of any additional filter params (e.g.
    /// `("diarize", "true")`).
    #[serde(flatten)]
    extra: ExtraSerializer<'a>,
}

struct ExtraSerializer<'a>(&'a [(String, String)]);

impl Serialize for ExtraSerializer<'_> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = ser.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    /// URL-encoded query string (without leading `?`).
    pub fn urlencoded(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(SerializableOptions::from(self))
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

    /// Group results by the given dimension.
    pub fn grouping(mut self, grouping: UsageGrouping) -> Self {
        self.0.grouping = Some(grouping);
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

    /// Filter by endpoint name.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.0.endpoint = Some(endpoint.into());
        self
    }

    /// Filter by model UUID.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.0.model = Some(model.into());
        self
    }

    /// Filter by tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.0.tag = Some(tag.into());
        self
    }

    /// Pass an arbitrary additional query parameter (e.g.
    /// `("diarize", "true")`). Appends; multiple calls accumulate.
    /// Useful for the long tail of feature filters that aren't
    /// surfaced as named methods.
    pub fn extra_query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.0.extra_query.push((key.into(), value.into()));
        self
    }

    /// Finish building.
    pub fn build(self) -> Options {
        self.0
    }
}

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        Self {
            start: &options.start,
            end: &options.end,
            grouping: options.grouping.as_ref().map(UsageGrouping::as_str),
            accessor: &options.accessor,
            deployment: &options.deployment,
            endpoint: &options.endpoint,
            model: &options.model,
            tag: &options.tag,
            extra: ExtraSerializer(&options.extra_query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_options_have_no_query() {
        let q = Options::builder().build().urlencoded().unwrap();
        assert_eq!(q, "");
    }

    #[test]
    fn full_options_serialize() {
        let q = Options::builder()
            .start("2025-01-01")
            .end("2025-01-31")
            .grouping(UsageGrouping::Endpoint)
            .accessor("acc-1")
            .endpoint("listen")
            .model("model-uuid")
            .tag("prod")
            .extra_query("diarize", "true")
            .extra_query("punctuate", "false")
            .build()
            .urlencoded()
            .unwrap();
        assert_eq!(
            q,
            "start=2025-01-01&end=2025-01-31&grouping=endpoint\
             &accessor=acc-1&endpoint=listen&model=model-uuid&tag=prod\
             &diarize=true&punctuate=false"
        );
    }
}
