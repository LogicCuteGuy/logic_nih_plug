//! Event handling for JUCE components.
//!
//! This module provides event handling functionality for JUCE components,
//! including mouse events, keyboard events, and timers.
//!
//! # Thread Safety
//!
//! All event callbacks are invoked on the JUCE message thread. Event types
//! do not implement `Send` or `Sync`, enforcing that they can only be used
//! on the message thread.

pub mod mouse;
pub mod keyboard;
pub mod timer;

pub use mouse::{ModifierKeys, MouseEvent, MouseListener};
pub use keyboard::{KeyPress, KeyListener};
pub use timer::Timer;
