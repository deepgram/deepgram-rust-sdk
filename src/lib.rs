#![forbid(unsafe_code)]
#![warn(missing_debug_implementations, missing_docs, clippy::cargo)]
#![allow(clippy::multiple_crate_versions, clippy::derive_partial_eq_without_eq)]

//! Official Rust SDK for Deepgram's automated speech recognition APIs.
//!
//! Get started transcribing with a [`Transcription`] object.

use core::fmt;
pub use http::Error as HttpError;
pub use reqwest::Error as ReqwestError;
pub use serde_json::Error as SerdeJsonError;
pub use serde_urlencoded::ser::Error as SerdeUrlencodedError;
use std::io;
use std::ops::Deref;
#[cfg(feature = "listen")]
pub use tungstenite::Error as TungsteniteError;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    RequestBuilder,
};
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

#[cfg(feature = "listen")]
pub mod common;
#[cfg(feature = "listen")]
pub mod listen;
#[cfg(feature = "manage")]
pub mod manage;
#[cfg(feature = "speak")]
pub mod speak;

static DEEPGRAM_BASE_URL: &str = "https://api.deepgram.com";

/// Transcribe audio using Deepgram's automated speech recognition.
///
/// Constructed using [`Deepgram::transcription`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/api-reference/#transcription
#[derive(Debug, Clone)]
pub struct Transcription<'a>(#[allow(unused)] pub &'a Deepgram);

/// Generate speech from text using Deepgram's text to speech api.
///
/// Constructed using [`Deepgram::text_to_speech`].
///
/// See the [Deepgram API Reference][api] for more info.
///
/// [api]: https://developers.deepgram.com/reference/text-to-speech-api
#[derive(Debug, Clone)]
pub struct Speak<'a>(#[allow(unused)] pub &'a Deepgram);

impl Deepgram {
    /// Construct a new [`Transcription`] from a [`Deepgram`].
    pub fn transcription(&self) -> Transcription<'_> {
        self.into()
    }

    /// Construct a new [`Speak`] from a [`Deepgram`].
    pub fn text_to_speech(&self) -> Speak<'_> {
        self.into()
    }
}

impl<'a> From<&'a Deepgram> for Transcription<'a> {
    /// Construct a new [`Transcription`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl<'a> From<&'a Deepgram> for Speak<'a> {
    /// Construct a new [`Speak`] from a [`Deepgram`].
    fn from(deepgram: &'a Deepgram) -> Self {
        Self(deepgram)
    }
}

impl Transcription<'_> {
    /// Expose a method to access the inner `Deepgram` reference if needed.
    pub fn deepgram(&self) -> &Deepgram {
        self.0
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A string wrapper that redacts its contents when formatted with `Debug`.
pub(crate) struct RedactedString(pub String);

impl fmt::Debug for RedactedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

impl Deref for RedactedString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Authentication method for Deepgram API requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AuthMethod {
    /// Use an API key with "Token" prefix (e.g., "Token dg_xxx").
    /// This is for permanent API keys created in the Deepgram console.
    ApiKey(RedactedString),

    /// Use a temporary token with "Bearer" prefix (e.g., "Bearer dg_xxx").
    /// This is for temporary tokens obtained via token-based authentication.
    TempToken(RedactedString),
}

impl AuthMethod {
    /// Get the authorization header value for this authentication method.
    pub(crate) fn header_value(&self) -> String {
        match self {
            AuthMethod::ApiKey(key) => format!("Token {}", key.0),
            AuthMethod::TempToken(token) => format!("Bearer {}", token.0),
        }
    }
}

/// A client for the Deepgram API.
///
/// Make transcriptions requests using [`Deepgram::transcription`].
#[derive(Debug, Clone)]
pub struct Deepgram {
    #[cfg_attr(not(feature = "listen"), allow(unused))]
    auth: Option<AuthMethod>,
    #[cfg_attr(not(feature = "listen"), allow(unused))]
    base_url: Url,
    #[cfg_attr(not(feature = "listen"), allow(unused))]
    client: reqwest::Client,
}

/// Errors that may arise from the [`deepgram`](crate) crate.
// TODO sub-errors for the different types?
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DeepgramError {
    /// The Deepgram API returned an error.
    #[error("The Deepgram API returned an error.")]
    DeepgramApiError {
        /// Error message from the Deepgram API.
        body: String,

        /// Underlying [`reqwest::Error`] from the HTTP request.
        err: ReqwestError,
    },

    /// Something went wrong when generating the http request.
    #[error("Something went wrong when generating the http request: {0}")]
    HttpError(#[from] HttpError),

    /// Something went wrong when making the HTTP request.
    #[error("Something went wrong when making the HTTP request: {0}")]
    ReqwestError(#[from] ReqwestError),

    /// Something went wrong during I/O.
    #[error("Something went wrong during I/O: {0}")]
    IoError(#[from] io::Error),

    #[cfg(feature = "listen")]
    /// Something went wrong with WS.
    #[error("Something went wrong with WS: {0}")]
    WsError(#[from] TungsteniteError),

    /// Something went wrong during serialization/deserialization.
    #[error("Something went wrong during json serialization/deserialization: {0}")]
    JsonError(#[from] SerdeJsonError),

    /// Something went wrong during serialization/deserialization.
    #[error("Something went wrong during query serialization: {0}")]
    UrlencodedError(#[from] SerdeUrlencodedError),

    /// The data stream produced an error
    #[error("The data stream produced an error: {0}")]
    StreamError(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// The provided base url is not valid
    #[error("The provided base url is not valid")]
    InvalidUrl,

    /// A websocket close from was received indicating an error
    #[error("websocket close frame received with error content: code: {code}, reason: {reason}")]
    WebsocketClose {
        /// The numerical code indicating the reason for the error
        code: u16,
        /// A textual description of the error reason
        reason: String,
    },

    /// An unexpected error occurred in the client
    #[error("an unepected error occurred in the deepgram client: {0}")]
    InternalClientError(anyhow::Error),

    /// A Deepgram API server response was not in the expected format.
    #[error("The Deepgram API server response was not in the expected format: {0}")]
    UnexpectedServerResponse(anyhow::Error),
}

#[cfg_attr(not(feature = "listen"), allow(unused))]
type Result<T, E = DeepgramError> = std::result::Result<T, E>;

impl Deepgram {
    /// Construct a new Deepgram client.
    ///
    /// The client will be pointed at Deepgram's hosted API.
    ///
    /// Create your first API key on the [Deepgram Console][console].
    ///
    /// [console]: https://console.deepgram.com/
    ///
    /// # Errors
    ///
    /// Errors under the same conditions as [`reqwest::ClientBuilder::build`].
    pub fn new<K: AsRef<str>>(api_key: K) -> Result<Self> {
        let auth = AuthMethod::ApiKey(RedactedString(api_key.as_ref().to_owned()));
        // This cannot panic because we are converting a static value
        // that is known-good.
        let base_url = DEEPGRAM_BASE_URL.try_into().unwrap();
        Self::inner_constructor(base_url, Some(auth))
    }

    /// Construct a new Deepgram client with a temporary token.
    ///
    /// This uses the "Bearer" prefix for authentication, suitable for temporary tokens.
    pub fn with_temp_token<T: AsRef<str>>(temp_token: T) -> Result<Self> {
        let auth = AuthMethod::TempToken(RedactedString(temp_token.as_ref().to_owned()));
        let base_url = DEEPGRAM_BASE_URL.try_into().unwrap();
        Self::inner_constructor(base_url, Some(auth))
    }

    /// Construct a new Deepgram client with the specified base URL.
    ///
    /// When using a self-hosted instance of deepgram, this will be the
    /// host portion of your own instance. For instance, if you would
    /// query your deepgram instance at `http://deepgram.internal/v1/listen`,
    /// the base_url will be `http://deepgram.internal`.
    ///
    /// Admin features, such as billing, usage, and key management will
    /// still go through the hosted site at `https://api.deepgram.com`.
    ///
    /// Self-hosted instances do not in general authenticate incoming
    /// requests, so unlike in [`Deepgram::new`], so no api key needs to be
    /// provided. The SDK will not include an `Authorization` header in its
    /// requests. If an API key is required, consider using
    /// [`Deepgram::with_base_url_and_api_key`].
    ///
    /// [console]: https://console.deepgram.com/
    ///
    /// # Example:
    ///
    /// ```
    /// # use deepgram::Deepgram;
    /// let deepgram = Deepgram::with_base_url(
    ///     "http://localhost:8080",
    /// );
    /// ```
    ///
    /// # Errors
    ///
    /// Errors under the same conditions as [`reqwest::Client::new`], or if `base_url`
    /// is not a valid URL.
    pub fn with_base_url<U>(base_url: U) -> Result<Self>
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
    {
        let base_url = base_url.try_into().map_err(|_| DeepgramError::InvalidUrl)?;
        Self::inner_constructor(base_url, None)
    }

    /// Construct a new Deepgram client with the specified base URL and
    /// API Key.
    ///
    /// When using a self-hosted instance of deepgram, this will be the
    /// host portion of your own instance. For instance, if you would
    /// query your deepgram instance at `http://deepgram.internal/v1/listen`,
    /// the base_url will be `http://deepgram.internal`.
    ///
    /// Admin features, such as billing, usage, and key management will
    /// still go through the hosted site at `https://api.deepgram.com`.
    ///
    /// [console]: https://console.deepgram.com/
    ///
    /// # Example:
    ///
    /// ```
    /// # use deepgram::Deepgram;
    /// let deepgram = Deepgram::with_base_url_and_api_key(
    ///     "http://localhost:8080",
    ///     "apikey12345",
    /// ).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Errors under the same conditions as [`reqwest::ClientBuilder::build`], or if `base_url`
    /// is not a valid URL.
    pub fn with_base_url_and_api_key<U, K>(base_url: U, api_key: K) -> Result<Self>
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
        K: AsRef<str>,
    {
        let base_url = base_url.try_into().map_err(|_| DeepgramError::InvalidUrl)?;
        let auth = AuthMethod::ApiKey(RedactedString(api_key.as_ref().to_owned()));
        Self::inner_constructor(base_url, Some(auth))
    }

    /// Construct a new Deepgram client with the specified base URL and temp token.
    pub fn with_base_url_and_temp_token<U, T>(base_url: U, temp_token: T) -> Result<Self>
    where
        U: TryInto<Url>,
        U::Error: std::fmt::Debug,
        T: AsRef<str>,
    {
        let base_url = base_url.try_into().map_err(|_| DeepgramError::InvalidUrl)?;
        let auth = AuthMethod::TempToken(RedactedString(temp_token.as_ref().to_owned()));
        Self::inner_constructor(base_url, Some(auth))
    }

    fn inner_constructor(base_url: Url, auth: Option<AuthMethod>) -> Result<Self> {
        static USER_AGENT: &str = concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " rust",
        );

        if base_url.cannot_be_a_base() {
            return Err(DeepgramError::InvalidUrl);
        }
        let authorization_header = {
            let mut header = HeaderMap::new();
            if let Some(auth) = &auth {
                let header_value = auth.header_value();
                if let Ok(value) = HeaderValue::from_str(&header_value) {
                    header.insert("Authorization", value);
                }
            }
            header
        };

        Ok(Deepgram {
            auth,
            base_url,
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .default_headers(authorization_header)
                .build()?,
        })
    }
}

/// Sends the request and checks the response for an error.
///
/// If there is an error, it translates it into a [`DeepgramError::DeepgramApiError`].
/// Otherwise, it deserializes the JSON accordingly.
#[cfg_attr(not(feature = "listen"), allow(unused))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_method_header_value() {
        let api_key = AuthMethod::ApiKey(RedactedString("test_api_key".to_string()));
        assert_eq!(api_key.header_value(), "Token test_api_key".to_string());

        let temp_token = AuthMethod::TempToken(RedactedString("test_temp_token".to_string()));
        assert_eq!(
            temp_token.header_value(),
            "Bearer test_temp_token".to_string()
        );
    }

    #[test]
    fn test_deepgram_new_with_temp_token() {
        let client = Deepgram::with_temp_token("test_temp_token").unwrap();
        assert_eq!(
            client.auth,
            Some(AuthMethod::TempToken(RedactedString(
                "test_temp_token".to_string()
            )))
        );
    }

    #[test]
    fn test_deepgram_new_with_api_key() {
        let client = Deepgram::new("test_api_key").unwrap();
        assert_eq!(
            client.auth,
            Some(AuthMethod::ApiKey(RedactedString(
                "test_api_key".to_string()
            )))
        );
    }
}
