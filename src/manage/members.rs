//! Manage the members of a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#members

use crate::{send_and_translate_response, Deepgram};

use response::Message;

pub mod response;

/// Manage the members of a Deepgram Project.
///
/// Constructed using [`Deepgram::members`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#members
#[derive(Debug, Clone)]
pub struct Members<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Members`] from a [`Deepgram`].
    pub fn members(&self) -> Members<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Members<'a> {
    /// Construct a new [`Members`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Members<'_> {
    /// Get all members of the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#members-get-members
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
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let members = dg_client
    ///     .members()
    ///     .list_members(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_members(&self, project_id: &str) -> crate::Result<response::Members> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/members",);

        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Remove the specified member from the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#members-delete
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
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// dg_client
    ///     .members()
    ///     .remove_member(&project_id, &member_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_member(&self, project_id: &str, member_id: &str) -> crate::Result<Message> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/members/{member_id}",);

        send_and_translate_response(self.0.client.delete(url)).await
    }
}
