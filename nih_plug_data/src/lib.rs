//! # nih_plug_data
//!
//! Data structures ported from JUCE.
//!
//! This crate provides:
//!
//! - **ValueTree**: Hierarchical data structure with change notifications
//! - **UndoManager**: Undo/redo functionality
//!
//! ## Examples
//!
//! ```
//! use nih_plug_data::{ValueTree, Value};
//!
//! let mut tree = ValueTree::new("root");
//! tree.set_property("name", Value::String("value".to_string()));
//! ```

#![warn(missing_docs)]

pub mod error;

#[cfg(feature = "valuetree")]
pub mod valuetree;

#[cfg(feature = "undo")]
pub mod undo;

pub use error::DataError;

#[cfg(feature = "valuetree")]
pub use valuetree::{Value, ValueTree, ValueTreeListener};

#[cfg(feature = "undo")]
pub use undo::{UndoManager, UndoableAction};
