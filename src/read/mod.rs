//! Read API — analyze plain text or a remote document for sentiment,
//! summary, topics, and intents.
//!
//! Mirrors `POST /v1/read` (`openapi/paths/read.v1.yml`).
//!
//! Entry point: [`crate::Deepgram::read`].
//!
//! ```no_run
//! use deepgram::Deepgram;
//! use deepgram::read::{options::Options, request::ReadRequest};
//!
//! # async fn run() -> Result<(), deepgram::DeepgramError> {
//! let dg = Deepgram::new(std::env::var("DEEPGRAM_API_KEY").unwrap_or_default())?;
//! let options = Options::builder()
//!     .language("en")
//!     .sentiment(true)
//!     .summarize(true)
//!     .topics(true)
//!     .build();
//!
//! let response = dg
//!     .read()
//!     .analyze(&ReadRequest::text("Deepgram makes voice AI fast and easy."), &options)
//!     .await?;
//!
//! if let Some(text) = response.summary_text() {
//!     println!("Summary: {text}");
//! }
//! # Ok(())
//! # }
//! ```

pub mod options;
pub mod request;
pub mod response;
pub mod rest;

pub use options::{Options, OptionsBuilder};
pub use request::ReadRequest;
pub use response::{
    ReadMetadata, ReadMetadataWrapper, ReadResponse, ReadResults, ReadSummaryInner,
    ReadSummaryText, ReadSummaryWrapper, TokenInfo,
};
