//! Speak module

pub mod options;
pub mod response;
pub mod rest;
pub mod websocket;

pub use response::{
    ClearedEvent, ClearedType, FlushedEvent, FlushedType, MetadataEvent, MetadataType,
    SpeakResponse, WarningEvent, WarningType,
};
pub use websocket::{SpeakHandle, SpeakStream, WebsocketBuilder};
