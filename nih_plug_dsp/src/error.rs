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

    /// Invalid FFT size (must be power of 2).
    #[error("Invalid FFT size: {size} (must be power of 2)")]
    InvalidFFTSize {
        /// The invalid size that was provided.
        size: usize,
    },

    /// FFT size out of valid range.
    #[error("FFT size {size} out of range (min: {min}, max: {max})")]
    FFTSizeOutOfRange {
        /// The size that was provided.
        size: usize,
        /// Minimum valid size.
        min: usize,
        /// Maximum valid size.
        max: usize,
    },
}
