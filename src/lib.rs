#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, missing_docs, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

//! Official Rust SDK for Deepgram's automated speech recognition APIs.
//!
//! Get started transcribing with a [`Transcription`](transcription::Transcription) object.

use std::io;

use reqwest::header::{HeaderMap, HeaderValue};
use thiserror::Error;

pub mod billing;
pub mod invitations;
pub mod keys;
pub mod members;
pub mod projects;
pub mod scopes;
pub mod transcription;
pub mod usage;

/// A client for the Deepgram API.
///
/// Make transcriptions requests using [`Deepgram::transcription`].
#[derive(Debug, Clone)]
pub struct Deepgram<K>
where
    K: AsRef<str>,
{
    api_key: K,
    client: reqwest::Client,
}

/// Errors that may arise from the [`deepgram`](crate) crate.
// TODO sub-errors for the different types?
#[derive(Debug, Error)]
pub enum DeepgramError {
    /// No source was provided to the request builder.
    #[error("No source was provided to the request builder.")]
    NoSource,

    /// Something went wrong during transcription.
    #[error("Something went wrong during transcription.")]
    TranscriptionError {
        /// Error message from the Deepgram API.
        body: String,

        /// Underlying [`reqwest::Error`] from the HTTP request.
        err: reqwest::Error,
    },

    /// Something went wrong when generating the http request.
    #[error("Something went wrong when generating the http request: {0}")]
    HttpError(#[from] http::Error),

    /// Something went wrong when making the HTTP request.
    #[error("Something went wrong when making the HTTP request: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Something went wrong during I/O.
    #[error("Something went wrong during I/O: {0}")]
    IoError(#[from] io::Error),

    /// Something went wrong with WS.
    #[error("Something went wrong with WS: {0}")]
    WsError(#[from] tungstenite::Error),

    /// Something went wrong during serialization/deserialization.
    #[error("Something went wrong during serialization/deserialization: {0}")]
    SerdeError(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, DeepgramError>;

impl<K> Deepgram<K>
where
    K: AsRef<str>,
{
    /// Construct a new Deepgram client.
    ///
    /// Create your first API key on the [Deepgram Console][console].
    ///
    /// [console]: https://console.deepgram.com/
    ///
    /// # Panics
    ///
    /// Panics under the same conditions as [`reqwest::Client::new`].
    pub fn new(api_key: K) -> Self {
        static USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " rust",
        );

        let authorization_header = {
            let mut header = HeaderMap::new();
            header.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Token {}", api_key.as_ref()))
                    .expect("Invalid API key"),
            );
            header
        };

        Deepgram {
            api_key,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .default_headers(authorization_header)
                .build()
                // Even though `reqwest::Client::new` is not used here, it will always panic under the same conditions
                .expect("See reqwest::Client::new docs for cause of panic"),
        }
    }
}
