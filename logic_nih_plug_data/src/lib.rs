//! # logic_nih_plug_data
//!
//! `ValueTree`, `UndoManager` and `CachedValue` ported from JUCE for nih-plug.
//!
//! This crate provides pure-Rust implementations of JUCE's data-structures module:
//!
//! - **`ValueTree`** — a hierarchical, observable, reference-counted tree of named
//!   properties and child nodes. Useful for plugin state, presets and undo.
//! - **`UndoManager`** — transactional undo/redo for `ValueTree` mutations.
//! - **`CachedValue<T>`** — typed binding from a `ValueTree` property to a Rust
//!   variable.
//!
//! ## Feature flags
//!
//! | Feature | Default | What it adds |
//! |---|---|---|
//! | `valuetree` | ✅ | `Identifier`, `Value`, `ValueTree`, `ValueTreeListener`, `CachedValue` |
//! | `undo` | ✅ | `UndoManager`, `UndoableAction`, the concrete action types |
//! | `full` | — | Re-exports everything (same as the default set) |
//!
//! ## Threading
//!
//! `ValueTree` and `UndoManager` are **single-threaded** by default, matching JUCE's
//! semantics. `ValueTree`'s underlying `Arc` makes the data shareable, but mutations
//! should happen from one thread at a time.
//!
//! ## Example
//!
//! ```rust
//! use logic_nih_plug_data::{UndoManager, Value, ValueTree};
//!
//! let tree = ValueTree::new("Preset");
//! let undo = UndoManager::new();
//!
//! tree.set_property_with("name", "Init".to_owned(), &undo);
//! tree.set_property_with("gain", 0.5_f64, &undo);
//!
//! let child = ValueTree::new("Oscillator");
//! child.set_property_with("waveform", "saw".to_owned(), &undo);
//! tree.add_child_with(child, 0, &undo);
//!
//! assert_eq!(tree.get_string(&"name".into(), ""), "Init");
//! assert_eq!(tree.num_children(), 1);
//!
//! undo.undo(); // Undo the add_child
//! assert_eq!(tree.num_children(), 0);
//! ```

#![warn(missing_docs)]

pub mod error;

#[cfg(feature = "valuetree")]
mod cached_value;

#[cfg(feature = "valuetree")]
mod identifier;

#[cfg(feature = "valuetree")]
mod value;

#[cfg(feature = "valuetree")]
mod value_tree;

#[cfg(feature = "undo")]
mod undo_manager;

#[cfg(feature = "valuetree")]
pub use cached_value::{CachedValue, CachedValueTrait};

pub use error::DataError;

#[cfg(feature = "valuetree")]
pub use identifier::Identifier;

#[cfg(feature = "undo")]
pub use undo_manager::{
    AddChildAction, RemoveChildAction, RemovePropertyAction, SetPropertyAction, UndoManager,
    UndoableAction,
};

#[cfg(feature = "valuetree")]
pub use value::Value;

#[cfg(feature = "valuetree")]
pub use value_tree::{ListenerHandle, ValueTree, ValueTreeListener};
