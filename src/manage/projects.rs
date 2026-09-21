//! Manage Deepgram Projects.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#projects

use url::Url;

use crate::{send_and_translate_response, Deepgram};

use options::{Options, SerializableOptions};

use response::{Message, Project};

pub mod options;
pub mod response;

/// Manage Deepgram Projects.
///
/// Constructed using [`Deepgram::projects`].
///
/// You can create new Deepgram Projects on the [Deepgram Console][console].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [console]: https://console.deepgram.com/
/// [api]: https://developers.deepgram.com/api-reference/#projects
#[derive(Debug, Clone)]
pub struct Projects<'a>(&'a Deepgram);

impl Deepgram {
    /// Construct a new [`Projects`] from a [`Deepgram`].
    pub fn projects(&self) -> Projects<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Projects<'a> {
    /// Construct a new [`Projects`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Projects<'_> {
    /// Get all projects.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#projects-get-projects
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
    /// let dg_client = Deepgram::new(&deepgram_api_key)?;
    ///
    /// let projects = dg_client
    ///     .projects()
    ///     .list()
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> crate::Result<response::Projects> {
        let request = self.0.client.get(self.projects_url()?);

        send_and_translate_response(request).await
    }

    /// Get a specific project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#projects-get-project
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
    /// let project = dg_client
    ///     .projects()
    ///     .get(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, project_id: &str) -> crate::Result<Project> {
        let url = self.project_url(project_id)?;

        send_and_translate_response(self.0.client.get(url)).await
    }

    /// Update the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#projects-update
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::env;
    /// #
    /// # use deepgram::{manage::projects::options::Options, Deepgram, DeepgramError};
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
    /// let options = Options::builder()
    ///     .name("The Transcribinator")
    ///     .company("Doofenshmirtz Evil Incorporated")
    ///     .build();
    ///
    /// dg_client
    ///     .projects()
    ///     .update(&project_id, &options)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(&self, project_id: &str, options: &Options) -> crate::Result<Message> {
        let url = self.project_url(project_id)?;
        let request = self
            .0
            .client
            .patch(url)
            .json(&SerializableOptions::from(options));

        send_and_translate_response(request).await
    }

    /// Delete the specified project.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#projects-get-delete
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
    ///     .projects()
    ///     .delete(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, project_id: &str) -> crate::Result<Message> {
        let url = self.project_url(project_id)?;
        let request = self.0.client.delete(url);

        send_and_translate_response(request).await
    }

    fn projects_url(&self) -> crate::Result<Url> {
        self.0.api_url("v1/projects")
    }

    fn project_url(&self, project_id: &str) -> crate::Result<Url> {
        self.0.api_url(&format!("v1/projects/{project_id}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Deepgram;

    #[test]
    fn urls_default_base() {
        let dg = Deepgram::new("token").unwrap();

        assert_eq!(
            dg.projects().projects_url().unwrap().as_str(),
            "https://api.deepgram.com/v1/projects"
        );
        assert_eq!(
            dg.projects().project_url("proj").unwrap().as_str(),
            "https://api.deepgram.com/v1/projects/proj"
        );
    }

    #[test]
    fn urls_custom_base() {
        let dg = Deepgram::with_base_url("http://deepgram.internal").unwrap();

        assert_eq!(
            dg.projects().projects_url().unwrap().as_str(),
            "http://deepgram.internal/v1/projects"
        );
        assert_eq!(
            dg.projects().project_url("proj").unwrap().as_str(),
            "http://deepgram.internal/v1/projects/proj"
        );
    }

    /// A base URL that carries a path prefix keeps it, the same way
    /// `Transcription`'s `/v1/listen` URL does. The trailing slash matters:
    /// without it the last path segment is replaced, per RFC 3986 relative
    /// resolution. Both forms are pinned here because the difference is what
    /// the `Deepgram::with_base_url*` rustdoc tells developers to expect.
    #[test]
    fn urls_custom_base_with_path_prefix() {
        let dg = Deepgram::with_base_url("http://gateway.internal/deepgram/").unwrap();

        assert_eq!(
            dg.projects().projects_url().unwrap().as_str(),
            "http://gateway.internal/deepgram/v1/projects"
        );

        let dg = Deepgram::with_base_url("http://gateway.internal/deepgram").unwrap();

        assert_eq!(
            dg.projects().projects_url().unwrap().as_str(),
            "http://gateway.internal/v1/projects"
        );
    }
}
