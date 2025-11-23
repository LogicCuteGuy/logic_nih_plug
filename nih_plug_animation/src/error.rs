//! Error types for animation operations.

use thiserror::Error;

/// Errors that can occur during animation operations.
#[derive(Debug, Error)]
pub enum AnimationError {
    /// Invalid animation parameters.
    #[error("Invalid animation parameters: {0}")]
    InvalidParameters(String),

    /// Animation already running.
    #[error("Animation already running")]
    AlreadyRunning,

    /// Invalid duration.
    #[error("Invalid duration: {0}")]
    InvalidDuration(f32),

    /// Empty animation sequence.
    #[error("Animation sequence is empty")]
    EmptySequence,

    /// Invalid animation index.
    #[error("Invalid animation index: {0}")]
    InvalidIndex(usize),
}
