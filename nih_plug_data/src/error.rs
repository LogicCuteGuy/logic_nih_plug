//! Error types for data structure operations.

use thiserror::Error;

/// Errors that can occur during data structure operations.
#[derive(Debug, Error)]
pub enum DataError {
    /// Invalid XML format.
    #[error("Invalid XML: {0}")]
    InvalidXml(String),

    /// Property not found in ValueTree.
    #[error("Property not found: {0}")]
    PropertyNotFound(String),

    /// Invalid value type for property.
    #[error("Invalid value type")]
    InvalidValueType,

    /// No actions available to undo.
    #[error("No actions to undo")]
    NoActionsToUndo,

    /// No actions available to redo.
    #[error("No actions to redo")]
    NoActionsToRedo,

    /// Action execution failed.
    #[error("Action failed: {0}")]
    ActionFailed(String),
}
