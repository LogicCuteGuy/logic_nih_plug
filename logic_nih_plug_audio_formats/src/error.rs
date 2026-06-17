//! Error types for audio format operations.

use thiserror::Error;

/// Errors that can occur during audio file operations.
#[derive(Debug, Error)]
pub enum AudioFormatError {
    /// File not found at the specified path.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Unsupported audio format.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    /// I/O error occurred.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Invalid audio data in file.
    #[error("Invalid audio data: {0}")]
    InvalidData(String),

    /// Invalid sample rate.
    #[error("Invalid sample rate: {0} Hz")]
    InvalidSampleRate(f32),

    /// Invalid channel count.
    #[error("Invalid channel count: {0}")]
    InvalidChannelCount(usize),

    /// Invalid bit depth.
    #[error("Invalid bit depth: {0}")]
    InvalidBitDepth(u16),

    /// Sample rate mismatch.
    #[error("Sample rate mismatch: expected {expected} Hz, got {actual} Hz")]
    SampleRateMismatch {
        /// Expected sample rate
        expected: f32,
        /// Actual sample rate
        actual: f32,
    },

    /// Channel count mismatch.
    #[error("Channel count mismatch: expected {expected}, got {actual}")]
    ChannelCountMismatch {
        /// Expected channel count
        expected: usize,
        /// Actual channel count
        actual: usize,
    },

    /// End of file reached unexpectedly.
    #[error("Unexpected end of file")]
    UnexpectedEof,

    /// Feature not enabled.
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

/// Result type for audio format operations.
pub type Result<T> = std::result::Result<T, AudioFormatError>;
