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
pub struct Options {
    comment: String,
    tags: Vec<String>,
    scopes: Vec<String>,
    expiration: Option<Expiration>,
}

#[derive(Debug, PartialEq, Clone)]
enum Expiration {
    ExpirationDate(String),
    TimeToLiveInSeconds(usize),
}

/// Builds an [`Options`] object using [the Builder pattern][builder].
#[derive(Debug, PartialEq, Clone)]
pub struct OptionsBuilder(Options);

#[derive(Serialize)]
pub(crate) struct SerializableOptions<'a> {
    comment: &'a String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: &'a Vec<String>,

    scopes: &'a Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    expiration_date: Option<&'a String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_to_live_in_seconds: Option<usize>,
}

impl Options {
    /// Construct a new [`OptionsBuilder`].
    pub fn builder<'a>(
        comment: impl Into<String>,
        scopes: impl IntoIterator<Item = &'a str>,
    ) -> OptionsBuilder {
        OptionsBuilder::new(comment, scopes)
    }
}

impl OptionsBuilder {
    /// Construct a new [`OptionsBuilder`].
    pub fn new<'a>(comment: impl Into<String>, scopes: impl IntoIterator<Item = &'a str>) -> Self {
        Self(Options {
            comment: comment.into(),
            tags: Vec::new(),
            scopes: scopes.into_iter().map(String::from).collect(),
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
    /// # use deepgram::manage::keys::options::Options;
    /// #
    /// let options1 = Options::builder("Old comment", ["member"])
    ///     .comment("New comment")
    ///     .build();
    ///
    /// let options2 = Options::builder("New comment", ["member"]).build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.0.comment = comment.into();
        self
    }

    /// Set the tags.
    ///
    /// Calling this when already set will append to the existing tags, not overwrite them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .tag(["Tag 1", "Tag 2"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
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
    pub fn tag<'a>(mut self, tags: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.tags.extend(tags.into_iter().map(String::from));
        self
    }

    /// Set additional scopes.
    ///
    /// Calling this when already set will append to the existing scopes, not overwrite them.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .scopes(["admin"])
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
    /// #
    /// let options1 = Options::builder("New Key", ["member"])
    ///     .scopes(["admin"])
    ///     .build();
    ///
    /// let options2 = Options::builder("New Key", ["member", "admin"]).build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn scopes<'a>(mut self, scopes: impl IntoIterator<Item = &'a str>) -> Self {
        self.0.scopes.extend(scopes.into_iter().map(String::from));
        self
    }

    /// Set the expiration date.
    ///
    /// This will unset the time to live in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .expiration_date("2038-01-19")
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
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
    pub fn expiration_date(mut self, expiration_date: impl Into<String>) -> Self {
        self.0.expiration = Some(Expiration::ExpirationDate(expiration_date.into()));
        self
    }

    /// Set the time to live in seconds.
    ///
    /// This will unset the expiration date.
    ///
    /// # Examples
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
    /// #
    /// let options = Options::builder("New Key", ["member"])
    ///     .time_to_live_in_seconds(7776000)
    ///     .build();
    /// ```
    ///
    /// ```
    /// # use deepgram::manage::keys::options::Options;
    /// #
    /// let options1 = Options::builder("New Key", ["member"])
    ///     .expiration_date("2038-01-19")
    ///     .time_to_live_in_seconds(7776000)
    ///     .build();
    ///
    /// let options2 = Options::builder("New Key", ["member"])
    ///     .time_to_live_in_seconds(7776000)
    ///     .build();
    ///
    /// assert_eq!(options1, options2);
    /// ```
    pub fn time_to_live_in_seconds(mut self, time_to_live_in_seconds: usize) -> Self {
        self.0.expiration = Some(Expiration::TimeToLiveInSeconds(time_to_live_in_seconds));
        self
    }

    /// Finish building the [`Options`] object.
    pub fn build(self) -> Options {
        self.0
    }
}

impl<'a> From<&'a Options> for SerializableOptions<'a> {
    fn from(options: &'a Options) -> Self {
        // Destructuring it makes sure that we don't forget to use any of it
        let Options {
            comment,
            tags,
            scopes,
            expiration,
        } = options;

        let mut serializable_options = Self {
            comment,
            tags,
            scopes,
            expiration_date: None,
            time_to_live_in_seconds: None,
        };

        match expiration {
            Some(Expiration::ExpirationDate(expiration_date)) => {
                serializable_options.expiration_date = Some(expiration_date);
            }
            Some(Expiration::TimeToLiveInSeconds(time_to_live_in_seconds)) => {
                serializable_options.time_to_live_in_seconds = Some(*time_to_live_in_seconds);
            }
            None => {}
        };

        serializable_options
    }
}
