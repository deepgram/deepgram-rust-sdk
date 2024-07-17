//! Manage Deepgram Projects.
//!
//! See the [Deepgram API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/api-reference/#projects

use crate::{send_and_translate_response, Deepgram};

use crate::common::response::Message;
use crate::manage::projects::options::{Options, SerializableOptions};
use crate::manage::projects::response::{self, Project};

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
    /// # use deepgram::{projects::options::Options, Deepgram, DeepgramError};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), DeepgramError> {
    /// # let deepgram_api_key =
    /// #     env::var("DEEPGRAM_API_KEY").expect("DEEPGRAM_API_KEY environmental variable");
    /// #
    /// let dg_client = Deepgram::new(&deepgram_api_key);
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
        let request = self.0.client.get("https://api.deepgram.com/v1/projects");

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
    /// # use deepgram::{projects::options::Options, Deepgram, DeepgramError};
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
    /// let project = dg_client
    ///     .projects()
    ///     .get(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, project_id: &str) -> crate::Result<Project> {
        let url = format!("https://api.deepgram.com/v1/projects/{}", project_id);

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
    /// # use deepgram::{projects::options::Options, Deepgram, DeepgramError};
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
        let url = format!("https://api.deepgram.com/v1/projects/{}", project_id);
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
    /// # use deepgram::{projects::options::Options, Deepgram, DeepgramError};
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
    ///     .projects()
    ///     .delete(&project_id)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, project_id: &str) -> crate::Result<Message> {
        let url = format!("https://api.deepgram.com/v1/projects/{}", project_id);
        let request = self.0.client.delete(url);

        send_and_translate_response(request).await
    }
}
