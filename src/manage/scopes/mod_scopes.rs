//! Manage the permissions of a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#scopes

use serde::Serialize;

use crate::{send_and_translate_response, Deepgram};

use crate::manage::scopes::response;

use super::response::Message;

/// Manage the permissions of a Deepgram Project.
///
/// Constructed using [`Deepgram::scopes`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#scopes
#[derive(Debug, Clone)]
pub struct Scopes<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Scopes`] from a [`Deepgram`].
    pub fn scopes(&self) -> Scopes<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Scopes<'a> {
    /// Construct a new [`Scopes`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Scopes<'_> {
    /// Get the specified project scopes assigned to the specified member.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#scopes-get
    ///
    /// # Examples
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
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// # let member_id =
    /// #     env::var("DEEPGRAM_MEMBER_ID").expect("DEEPGRAM_MEMBER_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key);
    ///
    /// let scopes = dg_client
    ///     .scopes()
    ///     .get_scope(&project_id, &member_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_scope(
        &self,
        project_id: &str,
        member_id: &str,
    ) -> crate::Result<response::Scopes> {
        let url = format!(
            "https://api.deepgram.com/v1/projects/{}/members/{}/scopes ",
            project_id, member_id
        );

        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Update the specified project scopes assigned to the specified member.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#scopes-update
    ///
    /// # Examples
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
    /// # let project_id =
    /// #     env::var("DEEPGRAM_PROJECT_ID").expect("DEEPGRAM_PROJECT_ID environmental variable");
    /// #
    /// # let member_id =
    /// #     env::var("DEEPGRAM_MEMBER_ID").expect("DEEPGRAM_MEMBER_ID environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key);
    ///
    /// dg_client
    ///     .scopes()
    ///     .update_scope(&project_id, &member_id, "member")
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_scope(
        &self,
        project_id: &str,
        member_id: &str,
        scope: &str,
    ) -> crate::Result<Message> {
        #[derive(Serialize)]
        struct Scope<'a> {
            scope: &'a str,
        }

        let url = format!(
            "https://api.deepgram.com/v1/projects/{}/members/{}/scopes",
            project_id, member_id
        );
        let request = self.0.client.put(url).json(&Scope { scope });

        send_and_translate_response(request).await
    }
}
