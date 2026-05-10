//! Options for
//! [`DistributionCredentials::create`](super::super::DistributionCredentials::create).
//!
//! Mirrors `schemas.selfHosted.v1.yml#CreateProjectDistributionCredentialsV1Request`
//! (body) plus the `scopes` and `provider` query params from
//! `parameters.selfHosted.v1.yml`.

use serde::Serialize;

/// One of the eight permission scopes a credential set can grant.
///
/// Sent as repeated `?scopes=…` query params at create time.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Scope {
    /// Umbrella scope — all self-hosted products.
    Products,
    /// API container.
    ProductApi,
    /// Engine container.
    ProductEngine,
    /// License-proxy container.
    ProductLicenseProxy,
    /// dgtools container.
    ProductDgtools,
    /// Billing container.
    ProductBilling,
    /// Hotpepper container.
    ProductHotpepper,
    /// Metrics-server container.
    ProductMetricsServer,
}

impl Scope {
    /// Wire string for this scope.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Products => "self-hosted:products",
            Self::ProductApi => "self-hosted:product:api",
            Self::ProductEngine => "self-hosted:product:engine",
            Self::ProductLicenseProxy => "self-hosted:product:license-proxy",
            Self::ProductDgtools => "self-hosted:product:dgtools",
            Self::ProductBilling => "self-hosted:product:billing",
            Self::ProductHotpepper => "self-hosted:product:hotpepper",
            Self::ProductMetricsServer => "self-hosted:product:metrics-server",
        }
    }
}

/// Distribution provider. Currently only Quay is supported by the API.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Provider {
    /// Quay container registry.
    Quay,
}

impl Provider {
    /// Wire string for this provider.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quay => "quay",
        }
    }
}

/// Request body for `POST /…/distribution/credentials`.
///
/// `scopes` and `provider` are sent as query params on the request,
/// while `comment` is the JSON body.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Options {
    scopes: Vec<Scope>,
    provider: Option<Provider>,
    comment: Option<String>,
}

/// Builder for [`Options`].
#[derive(Debug, Default, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

impl Options {
    /// Construct a new builder.
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    pub(super) fn query_pairs(&self) -> Vec<(&'static str, &'static str)> {
        let mut pairs = Vec::with_capacity(self.scopes.len() + 1);
        for scope in &self.scopes {
            pairs.push(("scopes", scope.as_str()));
        }
        if let Some(provider) = self.provider {
            pairs.push(("provider", provider.as_str()));
        }
        pairs
    }

    pub(super) fn body(&self) -> CreateBody<'_> {
        CreateBody {
            comment: self.comment.as_deref(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct CreateBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<&'a str>,
}

impl OptionsBuilder {
    /// Construct a fresh empty builder.
    pub fn new() -> Self {
        Self(Options::default())
    }

    /// Replace the scopes list. Multiple scopes are sent as repeated
    /// `?scopes=…` query params.
    pub fn scopes<I: IntoIterator<Item = Scope>>(mut self, scopes: I) -> Self {
        self.0.scopes = scopes.into_iter().collect();
        self
    }

    /// Distribution provider. Defaults to [`Provider::Quay`]
    /// server-side when unset.
    pub fn provider(mut self, provider: Provider) -> Self {
        self.0.provider = Some(provider);
        self
    }

    /// Optional human-readable comment stored with the credential set.
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.0.comment = Some(comment.into());
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
    fn query_pairs_round_trip() {
        let opts = Options::builder()
            .scopes([Scope::ProductApi, Scope::ProductEngine])
            .provider(Provider::Quay)
            .comment("ops")
            .build();
        let pairs = opts.query_pairs();
        assert_eq!(
            pairs,
            vec![
                ("scopes", "self-hosted:product:api"),
                ("scopes", "self-hosted:product:engine"),
                ("provider", "quay"),
            ]
        );
        let body = serde_json::to_value(opts.body()).unwrap();
        assert_eq!(body, serde_json::json!({"comment": "ops"}));
    }

    #[test]
    fn empty_options() {
        let opts = Options::builder().build();
        assert!(opts.query_pairs().is_empty());
        let body = serde_json::to_value(opts.body()).unwrap();
        assert_eq!(body, serde_json::json!({}));
    }
}
