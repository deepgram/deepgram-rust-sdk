//! Token-based authentication for Deepgram.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/auth/tokens/grant

use url::Url;

use crate::{
    auth::{
        options::{Options, SerializableOptions},
        response::GrantResponse,
    },
    send_and_translate_response, Deepgram,
};

pub mod options;
pub mod response;

/// Token-based authentication for Deepgram.
///
/// Constructed using [`Deepgram::auth`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/reference/auth/tokens/grant
#[derive(Debug, Clone)]
pub struct Auth<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Auth`] from a [`Deepgram`].
    pub fn auth(&self) -> Auth<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Auth<'a> {
    /// Construct a new [`Auth`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Auth<'_> {
    /// Generate a temporary JSON Web Token (JWT) with a configurable TTL.
    ///
    /// The token will have usage::write permission for core voice APIs.
    /// Requires an API key with Member or higher authorization.
    /// Tokens created with this endpoint will not work with the Manage APIs.
    ///
    /// The request goes to the base URL the client was built with, so a
    /// client built with [`Deepgram::with_base_url_and_api_key`] grants its
    /// token from that host instead of from `https://api.deepgram.com`.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/reference/auth/tokens/grant
    ///
    /// # Examples
    ///
    /// Generate a token with default 30-second TTL:
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let token = dg_client
    ///     .auth()
    ///     .grant(None)
    ///     .await?;
    ///
    /// println!("Token: {}", token.access_token);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Generate a token with custom TTL:
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{auth::options::Options, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let options = Options::builder()
    ///     .ttl_seconds(300.0)  // 5 minutes
    ///     .build();
    ///
    /// let token = dg_client
    ///     .auth()
    ///     .grant(Some(&options))
    ///     .await?;
    ///
    /// println!("Token: {}", token.access_token);
    /// println!("Expires in: {} seconds", token.expires_in.unwrap_or(0.0));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn grant(&self, options: Option<&Options>) -> crate::Result<GrantResponse> {
        let url = self.grant_url()?;

        let request = if let Some(opts) = options {
            self.0
                .client
                .post(url)
                .json(&SerializableOptions::from(opts))
        } else {
            // Send empty JSON object when no options provided
            self.0.client.post(url).json(&serde_json::json!({}))
        };

        send_and_translate_response(request).await
    }

    fn grant_url(&self) -> crate::Result<Url> {
        self.0.api_url("v1/auth/grant")
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn urls_default_base() {
        let dg = Deepgram::new("token").unwrap();

        assert_eq!(
            dg.auth().grant_url().unwrap().as_str(),
            "https://api.deepgram.com/v1/auth/grant"
        );
    }

    #[test]
    fn urls_custom_base() {
        let dg = Deepgram::with_base_url("http://deepgram.internal").unwrap();

        assert_eq!(
            dg.auth().grant_url().unwrap().as_str(),
            "http://deepgram.internal/v1/auth/grant"
        );
    }

    #[test]
    fn urls_custom_base_with_path_prefix() {
        let dg = Deepgram::with_base_url("http://gateway/deepgram/").unwrap();

        assert_eq!(
            dg.auth().grant_url().unwrap().as_str(),
            "http://gateway/deepgram/v1/auth/grant"
        );
    }
}
