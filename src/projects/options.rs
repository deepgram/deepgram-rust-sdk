//! Set options for [`Projects::update`](super::Projects::update).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#projects-update

use serde::Serialize;

/// Used as a parameter for [`Projects::update`](super::Projects::update).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#projects-update
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Options<'a> {
    name: Option<&'a str>,
    company: Option<&'a str>,
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder<'a>(Options<'a>);

#[derive(Serialize)]
pub(super) struct SerializableOptions<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: &'a Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) company: &'a Option<&'a str>,
}

impl<'a> Options<'a> {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder() -> OptionsBuilder<'a> {
        OptionsBuilder::new()
    }
}

impl<'a> OptionsBuilder<'a> {
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
    /// # use deepgram::projects::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .name("The Transcribinator")
    ///     .build();
    /// ```
    pub fn name(mut self, name: &'a str) -> Self {
        self.0.name = Some(name);
        self
    }

    /// Set the project company.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::projects::options::Options;
    /// #
    /// let options = Options::builder()
    ///     .company("Doofenshmirtz Evil Incorporated")
    ///     .build();
    /// ```
    pub fn company(mut self, company: &'a str) -> Self {
        self.0.company = Some(company);
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options<'a> {
        self.0
    }
}

impl<'a> Default for OptionsBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a Options<'a>> for SerializableOptions<'a> {
    fn from(options: &'a Options<'a>) -> Self {
        Self {
            name: &options.name,
            company: &options.company,
        }
    }
}
