//! Options for the model-listing endpoints.
//!
//! Mirrors the `include_outdated` query param shared by
//! `GET /v1/models` and `GET /v1/projects/{id}/models`.

/// Options for [`Models::list`](super::super::Models::list) and
/// [`Models::list_for_project`](super::super::Models::list_for_project).
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct Options {
    include_outdated: Option<bool>,
}

/// Builder for [`Options`].
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub struct OptionsBuilder(Options);

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::default()
    }

    pub(super) fn query_pairs(&self) -> Vec<(&'static str, &'static str)> {
        let mut pairs = Vec::new();
        if let Some(include_outdated) = self.include_outdated {
            pairs.push((
                "include_outdated",
                if include_outdated { "true" } else { "false" },
            ));
        }
        pairs
    }
}

impl OptionsBuilder {
    /// Construct a fresh empty builder.
    pub fn new() -> Self {
        Self(Options::default())
    }

    /// When `true`, include non-latest model versions in the response.
    pub fn include_outdated(mut self, include_outdated: bool) -> Self {
        self.0.include_outdated = Some(include_outdated);
        self
    }

    /// Finish building.
    pub fn build(self) -> Options {
        self.0
    }
}
