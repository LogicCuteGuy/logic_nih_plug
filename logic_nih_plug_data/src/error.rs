//! Error types for value-tree operations.

use thiserror::Error;

/// Errors that can occur when working with [`crate::ValueTree`], [`crate::UndoManager`]
/// or related types.
#[derive(Debug, Error)]
pub enum DataError {
    /// Attempted to access a child whose index is out of range.
    #[error("child index {index} out of range (num_children = {num_children})")]
    ChildIndexOutOfRange {
        /// The bad index.
        index: usize,
        /// The actual number of children on the parent at the time.
        num_children: usize,
    },

    /// Attempted to add a child to itself or to one of its own descendants,
    /// which would create a cycle.
    #[error("cannot add a tree as a child of itself or one of its descendants")]
    WouldCreateCycle,

    /// Attempted to close a transaction that was never opened on this
    /// [`crate::UndoManager`].
    #[error("no transaction was active on this undo manager")]
    NoActiveTransaction,
}
