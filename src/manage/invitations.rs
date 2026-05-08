//! Manage the invitations to a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#invitations

use crate::{send_and_translate_response, Deepgram};

use response::{Invites, Message};

pub mod options;
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
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
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
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/leave",);

        send_and_translate_response(self.0.client.delete(url)).await
    }

    /// `GET /v1/projects/{project_id}/invites` — list every pending
    /// invite for the project.
    pub async fn list(&self, project_id: &str) -> crate::Result<Invites> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/invites");
        send_and_translate_response(self.0.client.get(url)).await
    }

    /// `POST /v1/projects/{project_id}/invites` — invite an email to
    /// the project with the given scope.
    pub async fn create(
        &self,
        project_id: &str,
        request: &options::Options,
    ) -> crate::Result<Message> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/invites");
        send_and_translate_response(self.0.client.post(url).json(request)).await
    }

    /// `DELETE /v1/projects/{project_id}/invites/{email}` — revoke a
    /// pending invite by email address.
    pub async fn delete(&self, project_id: &str, email: &str) -> crate::Result<Message> {
        let url = format!("https://api.deepgram.com/v1/projects/{project_id}/invites/{email}");
        send_and_translate_response(self.0.client.delete(url)).await
    }
}
