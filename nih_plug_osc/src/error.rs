//! Error types for OSC operations.

use thiserror::Error;

/// Errors that can occur during OSC operations.
#[derive(Debug, Error)]
pub enum OscError {
    /// Network error occurred.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Invalid OSC message format.
    #[error("Invalid OSC message")]
    InvalidMessage,

    /// I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
