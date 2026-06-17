//! Error types for GUI operations.

use thiserror::Error;

/// Errors that can occur during GUI operations.
#[derive(Debug, Error)]
pub enum GuiError {
    /// Invalid component bounds
    #[error("Invalid component bounds: x={0}, y={1}, width={2}, height={3}")]
    InvalidBounds(i32, i32, u32, u32),

    /// Component not found
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Invalid parent-child relationship
    #[error("Invalid parent-child relationship: {0}")]
    InvalidRelationship(String),

    /// Component already has a parent
    #[error("Component already has a parent")]
    AlreadyHasParent,

    /// Cannot add component as its own child
    #[error("Cannot add component as its own child")]
    SelfReference,

    /// Graphics error
    #[error("Graphics error: {0}")]
    GraphicsError(#[from] logic_nih_plug_graphics::GraphicsError),

    /// Invalid range (min >= max)
    #[error("Invalid range: min={0}, max={1}")]
    InvalidRange(f64, f64),

    /// Invalid layout configuration
    #[error("Invalid layout: {0}")]
    InvalidLayout(String),
}

/// Result type for GUI operations.
pub type Result<T> = std::result::Result<T, GuiError>;
