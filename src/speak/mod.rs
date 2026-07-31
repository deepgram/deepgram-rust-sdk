//! Speak module

pub mod flux;
pub mod options;
pub mod response;
pub mod rest;
pub mod websocket;

pub use response::SpeakMetadata;
pub use websocket::{SpeakResponse, SpeakStreamBuilder, SpeakStreamHandle};
