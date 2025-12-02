//! JUCE widget components.
//!
//! This module provides safe Rust wrappers around JUCE's widget components,
//! including buttons, sliders, labels, combo boxes, and more.
//!
//! All widgets inherit from Component and can be used anywhere a Component
//! is expected. The inheritance is implemented through Deref/DerefMut traits.
//!
//! # Thread Safety
//!
//! All widget operations must be performed on the JUCE message thread.
//! This is enforced through the type system - widgets do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::TextButton;
//!
//! let mut button = TextButton::new("Click Me")?;
//! button.set_bounds(10, 10, 100, 30);
//! button.set_on_click(|| {
//!     println!("Button clicked!");
//! });
//! ```

pub mod button;
pub mod combo_box;
pub mod label;
pub mod slider;
pub mod text_editor;
pub mod toggle_button;

pub use button::TextButton;
pub use combo_box::ComboBox;
pub use label::{Justification, Label};
pub use slider::{Slider, SliderStyle};
pub use text_editor::TextEditor;
pub use toggle_button::ToggleButton;
