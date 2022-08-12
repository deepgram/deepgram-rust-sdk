#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, missing_docs, clippy::cargo)]
#![allow(clippy::multiple_crate_versions, clippy::derive_partial_eq_without_eq)]

//! Official Rust SDK for Deepgram's automated speech recognition APIs.
//!
//! Get started transcribing with a [`Transcription`](transcription::Transcription) object.

use reqwest::{
    header::{HeaderMap, HeaderValue},
    RequestBuilder,
};
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio_tungstenite::tungstenite::{self, protocol::CloseFrame};

pub mod billing;
pub mod invitations;
pub mod keys;
pub mod members;
pub mod projects;
pub mod scopes;
pub mod transcription;
pub mod usage;

mod response;

/// A client for the Deepgram API.
///
/// Make transcriptions requests using [`Deepgram::transcription`].
#[derive(Debug, Clone)]
pub struct Deepgram {
    api_key_header: HeaderValue,
    client: reqwest::Client,
}

/// Errors that may arise from the [`deepgram`](crate) crate.
// TODO sub-errors for the different types?
#[derive(Debug, Error)]
pub enum DeepgramError {
    /// The Deepgram API returned an error.
    #[error("The Deepgram API returned an error.")]
    DeepgramApiError {
        /// Error message from the Deepgram API.
        body: String,

        /// Underlying [`reqwest::Error`] from the HTTP request.
        err: reqwest::Error,
    },

    /// A problem occurred when transcribing the live audio stream.
    ///
    /// See the [Deepgram API Reference][api] for more info.
    ///
    /// [api]: https://developers.deepgram.com/api-reference/#error-handling-str
    #[error("A problem occurred when transcribing the live audio stream.")]
    DeepgramLiveError(CloseFrame<'static>),

    /// Something went wrong when making the HTTP request.
    #[error("Something went wrong when making the HTTP request: {0}")]
    ReqwestError(#[from] reqwest::Error),

    /// Something went wrong with WS.
    #[error("Something went wrong with WS: {0}")]
    WsError(#[from] tungstenite::Error),

    /// Something went wrong during serialization/deserialization.
    #[error("Something went wrong during serialization/deserialization: {0}")]
    SerdeError(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, DeepgramError>;

impl Deepgram {
    /// Construct a new Deepgram client.
    ///
    /// Create your first API key on the [Deepgram Console][console].
    ///
    /// [console]: https://console.deepgram.com/
    ///
    /// # Panics
    ///
    /// Panics under the same conditions as [`reqwest::Client::new`].
    pub fn new(api_key: &str) -> Self {
        static USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " rust",
        );

        let api_key_header =
            HeaderValue::from_str(&format!("Token {}", api_key)).expect("Invalid API key");

        let authorization_header = {
            let mut header = HeaderMap::new();
            header.insert("Authorization", api_key_header.clone());
            header
        };

        Deepgram {
            api_key_header,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .default_headers(authorization_header)
                .build()
                // Even though `reqwest::Client::new` is not used here, it will always panic under the same conditions
                .expect("See reqwest::Client::new docs for cause of panic"),
        }
    }
}

/// Sends the request and checks the response for an error.
///
/// If there is an error, it translates it into a [`DeepgramError::DeepgramApiError`].
/// Otherwise, it deserializes the JSON accordingly.
async fn send_and_translate_response<R: DeserializeOwned>(
    request_builder: RequestBuilder,
) -> crate::Result<R> {
    let response = request_builder.send().await?;

    match response.error_for_status_ref() {
        Ok(_) => Ok(response.json().await?),
        Err(err) => Err(DeepgramError::DeepgramApiError {
            body: response.text().await?,
            err,
        }),
    }
}
