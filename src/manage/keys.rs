//! Manage the keys for a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#keys

use crate::{
    manage::keys::{
        options::{Options, SerializableOptions},
        response::{MemberAndApiKey, MembersAndApiKeys, NewApiKey},
    },
    send_and_translate_response, Deepgram,
};

use response::Message;

pub mod options;
pub mod response;

/// Manage the keys for a Deepgram Project.
///
/// Constructed using [`Deepgram::keys`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#keys
#[derive(Debug, Clone)]
pub struct Keys<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Keys`] from a [`Deepgram`].
    pub fn keys(&self) -> Keys<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Keys<'a> {
    /// Construct a new [`Keys`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Keys<'_> {
    /// Get keys for the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#keys-get-keys
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{manage::keys::options::Options, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let keys = dg_client
    ///     .keys()
    ///     .list(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, project_id: &str) -> crate::Result<MembersAndApiKeys> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/keys");

        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Get details of the specified key.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#keys-get-key
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{manage::keys::options::Options, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// # let key_id = env::var("DEEPGRAM_KEY_ID").expect("DEEPGRAM_KEY_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let key = dg_client
    ///     .keys()
    ///     .get(&project_id, &key_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, project_id: &str, key_id: &str) -> crate::Result<MemberAndApiKey> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/keys/{key_id}",);

        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Create a new key in the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#keys-create
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{manage::keys::options::Options, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// # let key_id = env::var("DEEPGRAM_KEY_ID").expect("DEEPGRAM_KEY_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let options = Options::builder("New Key", ["member"]).build();
    /// let new_key = dg_client
    ///     .keys()
    ///     .create(&project_id, &options)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, project_id: &str, options: &Options) -> crate::Result<NewApiKey> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/keys");
        let request = self
            .0
            .client
            .post(url)
            .json(&SerializableOptions::from(options));

        send_and_translate_response(request).await
    }

    /// Delete the specified key in the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#keys-delete
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{manage::keys::options::Options, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// # let key_id = env::var("DEEPGRAM_KEY_ID").expect("DEEPGRAM_KEY_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// dg_client
    ///     .keys()
    ///     .delete(&project_id, &key_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, project_id: &str, key_id: &str) -> crate::Result<Message> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/keys/{key_id}",);

        send_and_translate_response(self.0.client.delete(url)).await
    }
}
