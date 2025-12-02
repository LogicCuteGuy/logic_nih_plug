//! JUCE dialog components.
//!
//! This module provides safe Rust wrappers around JUCE's dialog classes,
//! including AlertWindow for showing messages and FileChooser for file selection.
//!
//! # Thread Safety
//!
//! All dialog operations must be performed on the JUCE message thread.
//! This is enforced through the type system where applicable.

pub mod alert_window;
pub mod file_chooser;

pub use alert_window::AlertWindow;
pub use file_chooser::FileChooser;
