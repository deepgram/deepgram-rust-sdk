//! Flux streaming response types for turn-based conversations.
//!
//! See the [Deepgram Flux API Reference][api] for more info.
//!
//! [api]: https://developers.deepgram.com/reference/speech-to-text/listen-flux

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Flux WebSocket message types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum FluxResponse {
    /// Initial connection confirmation
    Connected {
        #[allow(missing_docs)]
        request_id: Uuid,

        #[allow(missing_docs)]
        sequence_id: u32,
    },

    /// Turn information with transcript
    TurnInfo {
        #[allow(missing_docs)]
        request_id: Uuid,

        #[allow(missing_docs)]
        sequence_id: u32,

        /// Event type: EndOfTurn, EagerEndOfTurn, or TurnResumed
        event: TurnEvent,

        #[allow(missing_docs)]
        turn_index: u32,

        #[allow(missing_docs)]
        audio_window_start: f64,

        #[allow(missing_docs)]
        audio_window_end: f64,

        #[allow(missing_docs)]
        transcript: String,

        #[allow(missing_docs)]
        words: Vec<FluxWord>,

        /// Confidence that this is end of turn
        end_of_turn_confidence: f64,
    },

    /// Fatal error from server
    #[serde(rename = "Error")]
    FatalError {
        #[allow(missing_docs)]
        sequence_id: u32,

        #[allow(missing_docs)]
        code: String,

        #[allow(missing_docs)]
        description: String,
    },
}

/// Turn event types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TurnEvent {
    /// Start of a new turn
    StartOfTurn,

    /// Normal end of turn
    EndOfTurn,

    /// Eager end of turn (when confidence threshold met early)
    EagerEndOfTurn,

    /// Turn resumed after eager end
    TurnResumed,

    /// Turn update (interim transcript update)
    Update,
}

/// A word in a Flux turn with confidence
#[derive(Debug, Serialize, Deserialize, Clone)]
#[non_exhaustive]
pub struct FluxWord {
    #[allow(missing_docs)]
    pub word: String,

    #[allow(missing_docs)]
    pub confidence: f64,
}
