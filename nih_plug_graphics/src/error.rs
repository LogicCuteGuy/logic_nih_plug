//! Error types for graphics operations.

use thiserror::Error;

/// Errors that can occur during graphics operations.
#[derive(Debug, Error)]
pub enum GraphicsError {
    /// Invalid dimensions provided.
    #[error("Invalid dimensions: {0}x{1}")]
    InvalidDimensions(u32, u32),

    /// Invalid color value.
    #[error("Invalid color value")]
    InvalidColor,
    
    /// Error loading an image.
    #[error("Failed to load image: {0}")]
    ImageLoadError(String),
    
    /// Error saving an image.
    #[error("Failed to save image: {0}")]
    ImageSaveError(String),
    
    /// Invalid image data.
    #[error("Invalid image data: expected {expected} bytes, got {actual}")]
    InvalidImageData {
        /// Expected number of bytes
        expected: usize,
        /// Actual number of bytes
        actual: usize,
    },
    
    /// Error loading a font.
    #[error("Failed to load font: {0}")]
    FontLoadError(String),
}
