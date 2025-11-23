//! Error types for DSP operations.

use thiserror::Error;

/// Errors that can occur during DSP operations.
#[derive(Debug, Error)]
pub enum DspError {
    /// Invalid sample rate provided.
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(f32),

    /// Invalid buffer size provided.
    #[error("Invalid buffer size: {0}")]
    InvalidBufferSize(usize),

    /// Invalid filter coefficients provided.
    #[error("Invalid coefficients")]
    InvalidCoefficients,

    /// Invalid frequency value provided.
    #[error("Invalid frequency: {0}")]
    InvalidFrequency(f32),
}
