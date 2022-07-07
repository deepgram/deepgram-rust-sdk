//! Set options for [`Keys::create`](super::Keys::create).
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#keys-create

use serde::Serialize;

/// Used as a parameter for [`Keys::create`](super::Keys::create).
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#keys-create
#[derive(Debug, PartialEq, Clone)]
pub struct Options<'a> {
    comment: &'a str,
    tags: Vec<&'a str>,
    scopes: Vec<&'a str>,
    expiration: Option<Expiration<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
enum Expiration<'a> {
    ExpirationDate(&'a str),
    TimeToLiveInSeconds(usize),
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
///
/// [builder]: https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder<'a>(Options<'a>);

#[derive(Serialize)]
pub(super) struct SerializableOptions<'a> {
    comment: &'a str,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: &'a Vec<&'a str>,

    scopes: &'a Vec<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_date: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_to_live_in_seconds: Option<usize>,
}

impl<'a> Options<'a> {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder(
        comment: &'a str,
        scopes: impl IntoIterator<Item = &'a str>,
    ) -> OptionsBuilder<'a> {
        OptionsBuilder::new(comment, scopes)
    }
}

impl<'a> OptionsBuilder<'a> {
    /// Construct a new [`OptionsBuilder`].
    pub fn new(comment: &'a str, scopes: impl IntoIterator<Item = &'a str>) -> Self {
        Self(Options {
            comment,
            tags: Vec::new(),
            scopes: scopes.into_iter().collect(),
            expiration: None,
        })
    }

    /// Set the comment.
    ///
    /// This will overwrite any previously set comment,
    /// including the one set in [`OptionsBuilder::new`] for [`Options::builder`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options1 = Options::builder("Old comment", ["member"])
    ///     .comment("New comment")
    ///     .build();
    ///
    /// let options2 = Options::builder("New comment", ["member"]).build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn comment(mut self, comment: &'a str) -> Self {
        self.0.comment = comment;
        self
    }

    /// Set the tags.
    ///
    /// Calling this when already set will append to the existing tags, not overwrite them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .tag(["Tag 1", "Tag 2"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options1 = Options::builder("New Key", ["member"])
    ///     .tag(["Tag 1"])
    ///     .tag(["Tag 2"])
    ///     .build();
    ///
    /// let options2 = Options::builder("New Key", ["member"])
    ///     .tag(["Tag 1", "Tag 2"])
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn tag(mut self, tags: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.tags.extend(tags);
        self
    }

    /// Set additional scopes.
    ///
    /// Calling this when already set will append to the existing scopes, not overwrite them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .scopes(["admin"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options1 = Options::builder("New Key", ["member"])
    ///     .scopes(["admin"])
    ///     .build();
    ///
    /// let options2 = Options::builder("New Key", ["member", "admin"]).build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn scopes(mut self, scopes: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.scopes.extend(scopes);
        self
    }

    /// Set the expiration date.
    ///
    /// This will unset the time to live in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .expiration_date("2038-01-19")
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options1 = Options::builder("New Key", ["member"])
    ///     .time_to_live_in_seconds(7776000)
    ///     .expiration_date("2038-01-19")
    ///     .build();
    ///
    /// let options2 = Options::builder("New Key", ["member"])
    ///     .expiration_date("2038-01-19")
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn expiration_date(mut self, expiration_date: &'a str) -> Self {
        self.0.expiration = Some(Expiration::ExpirationDate(expiration_date));
        self
    }

    /// Set the expiration date.
    ///
    /// This will unset the time to live in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .expiration_date("2038-01-19")
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::keys::options::Options;
    /// #
    /// let options1 = Options::builder("New Key", ["member"])
    ///     .time_to_live_in_seconds(7776000)
    ///     .expiration_date("2038-01-19")
    ///     .build();
    ///
    /// let options2 = Options::builder("New Key", ["member"])
    ///     .expiration_date("2038-01-19")
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn time_to_live_in_seconds(mut self, time_to_live_in_seconds: usize) -> Self {
        self.0.expiration = Some(Expiration::TimeToLiveInSeconds(time_to_live_in_seconds));
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options<'a> {
        self.0
    }
}

impl<'a> From<&'a Options<'a>> for SerializableOptions<'a> {
    fn from(options: &'a Options<'a>) -> Self {
        let mut serializable_options = Self {
            comment: options.comment,
            tags: &options.tags,
            scopes: &options.scopes,
            expiration_date: None,
            time_to_live_in_seconds: None,
        };

        match options.expiration {
            Some(Expiration::ExpirationDate(expiration_date)) => {
                serializable_options.expiration_date = Some(expiration_date);
            }
            Some(Expiration::TimeToLiveInSeconds(time_to_live_in_seconds)) => {
                serializable_options.time_to_live_in_seconds = Some(time_to_live_in_seconds);
            }
            None => {}
        };

        serializable_options
    }
}
