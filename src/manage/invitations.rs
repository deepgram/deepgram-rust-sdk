//! Manage the invitations to a Deepgram Project.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#invitations

use url::Url;

use crate::{send_and_translate_response, Deepgram};

use options::{Options, SerializableOptions};
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
    /// List the outstanding invitations for a project.
    pub async fn list(&self, project_id: &str) -> crate::Result<Invites> {
        send_and_translate_response(self.0.client.get(self.invites_url(project_id)?)).await
    }

    /// Invite an email address to a project.
    pub async fn create(&self, project_id: &str, options: &Options) -> crate::Result<Message> {
        let request = self
            .0
            .client
            .post(self.invites_url(project_id)?)
            .json(&SerializableOptions::from(options));

        send_and_translate_response(request).await
    }

    /// Revoke a project invitation.
    pub async fn delete(&self, project_id: &str, email: &str) -> crate::Result<Message> {
        send_and_translate_response(self.0.client.delete(self.invite_url(project_id, email)?)).await
    }

    /// Deprecated alias for [`Projects::leave`](crate::manage::projects::Projects::leave).
    #[deprecated(note = "use `Deepgram::projects().leave(project_id)` instead")]
    pub async fn leave_project(&self, project_id: &str) -> crate::Result<Message> {
        let url = self.leave_url(project_id)?;

        send_and_translate_response(self.0.client.delete(url)).await
    }

    fn leave_url(&self, project_id: &str) -> crate::Result<Url> {
        self.0.api_url(&format!("v1/projects/{project_id}/leave"))
    }

    fn invites_url(&self, project_id: &str) -> crate::Result<Url> {
        self.0.api_url(&format!("v1/projects/{project_id}/invites"))
    }

    fn invite_url(&self, project_id: &str, email: &str) -> crate::Result<Url> {
        let email = url::form_urlencoded::byte_serialize(email.as_bytes()).collect::<String>();
        self.0
            .api_url(&format!("v1/projects/{project_id}/invites/{email}"))
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

    #[test]
    fn invitation_email_is_percent_encoded_in_its_path_segment() {
        let dg = Deepgram::new("token").unwrap();

        assert_eq!(
            dg.invitations()
                .invite_url("proj", "person+test@example.com")
                .unwrap()
                .as_str(),
            "https://api.deepgram.com/v1/projects/proj/invites/person%2Btest%40example.com"
        );
    }
}
