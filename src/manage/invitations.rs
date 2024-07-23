//! Manage the invitations to a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#invitations

use crate::{send_and_translate_response, Deepgram};

use response::Message;

pub mod response;

/// Manage the invitations to a Deepgram Project.
///
/// Constructed using [`Deepgram::invitations`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#invitations
#[derive(Debug, Clone)]
pub struct Invitations<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Invitations`] from a [`Deepgram`].
    pub fn invitations(&self) -> Invitations<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Invitations<'a> {
    /// Construct a new [`Invitations`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Invitations<'_> {
    /// Remove the authenticated account from the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#invitations
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
    /// let dg_client = Deepgram::new(&deepgram_api_key);
    ///
    /// dg_client
    ///     .invitations()
    ///     .leave_project(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn leave_project(&self, project_id: &str) -> crate::Result<Message> {
        let url = format!("https://api.deepgram.com/v1/projects/{}/leave", project_id,);

        send_and_translate_response(self.0.client.delete(url)).await
    }
}
