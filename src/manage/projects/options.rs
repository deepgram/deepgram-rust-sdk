//! Set options for [`Projects::update`](super::mod_projects::Projects::update).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#projects-update

use serde::Serialize;

/// Used as a parameter for [`Projects::update`](super::mod_projects::Projects::update).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#projects-update
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Options {
    name: Option<String>,
    company: Option<String>,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) company: &'a Option<String>,
}

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder {
        OptionsBuilder::new()
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new() -> Self {
        Self(Options {
            name: None,
            company: None,
        })
    }

    /// Set the project name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::projects::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .name("The Transcribinator")
    ///     .build();
    /// ```
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.0.name = Some(name.into());
        self
    }

    /// Set the project company.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::projects::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .company("Doofenshmirtz Evil Incorporated")
    ///     .build();
    /// ```
    pub fn company(mut self, company: impl Into<String>) -> Self {
        self.0.company = Some(company.into());
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options {
        self.0
    }
}

impl Default for OptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        // Destructuring it makes sure that we don't forget to use any of it
        let Options { name, company } = options;

        Self { name, company }
    }
}
