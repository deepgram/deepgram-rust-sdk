//! Manage the invitations to a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#invitations

use url::Url;

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
        let url = self.leave_url(project_id)?;

        send_and_translate_response(self.0.client.delete(url)).await
    }

    fn leave_url(&self, project_id: &str) -> crate::Result<Url> {
        self.0.api_url(&format!("v1/projects/{project_id}/leave"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn urls_default_base() {
        let dg = Deepgram::new("token").unwrap();

        assert_eq!(
            dg.invitations().leave_url("proj").unwrap().as_str(),
            "https://api.deepgram.com/v1/projects/proj/leave"
        );
    }

    #[test]
    fn urls_custom_base() {
        let dg = Deepgram::with_base_url("http://deepgram.internal").unwrap();

        assert_eq!(
            dg.invitations().leave_url("proj").unwrap().as_str(),
            "http://deepgram.internal/v1/projects/proj/leave"
        );
    }
}
