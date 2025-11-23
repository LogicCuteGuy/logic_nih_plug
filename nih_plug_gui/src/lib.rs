//! # nih_plug_gui
//!
//! GUI components ported from JUCE.
//!
//! This crate provides a component-based UI framework with:
//!
//! - **Component Infrastructure**: Lifecycle management and parent-child relationships
//! - **Components**: Button, Slider, Label, and other UI controls
//! - **LookAndFeel**: Appearance customization and theming system
//! - **Layout**: Layout managers and constraints (future)
//!
//! ## Component Lifecycle
//!
//! Components go through several lifecycle states:
//! - `Initializing`: Component is being set up
//! - `Active`: Component is ready and visible
//! - `Hidden`: Component is hidden but still in hierarchy
//! - `Destroying`: Component is being cleaned up
//!
//! ## Parent-Child Relationships
//!
//! Components form a tree hierarchy. Each component can have multiple children
//! but only one parent. The framework automatically manages these relationships
//! and ensures proper cleanup.
//!
//! ## LookAndFeel Customization
//!
//! The LookAndFeel system allows you to customize the appearance of UI components:
//!
//! ```
//! use nih_plug_gui::lookandfeel::{DefaultLookAndFeel, LookAndFeel, Theme};
//! use nih_plug_gui::controls::{Button, ButtonState};
//! use nih_plug_gui::components::Bounds;
//!
//! // Create a button with dark theme
//! let mut button = Button::new("Click Me");
//! button.set_bounds(Bounds::new(10, 10, 100, 30)).unwrap();
//!
//! let laf = DefaultLookAndFeel::with_theme(Theme::Dark);
//! let color = laf.button_color(ButtonState::Normal);
//! ```
//!
//! ## Examples
//!
//! ```
//! use nih_plug_gui::components::{Component, Bounds};
//!
//! // Create a parent component
//! let mut parent = Component::new("parent");
//! parent.set_bounds(Bounds::new(0, 0, 400, 300)).unwrap();
//! parent.initialize();
//!
//! // Create and add a child
//! let mut child = Component::new("child");
//! child.set_bounds(Bounds::new(10, 10, 100, 50)).unwrap();
//! parent.add_child(child).unwrap();
//!
//! assert_eq!(parent.child_count(), 1);
//! ```

#![warn(missing_docs)]

pub mod error;

#[cfg(feature = "components")]
pub mod components;

#[cfg(feature = "components")]
pub mod controls;

#[cfg(feature = "components")]
pub mod input;

#[cfg(feature = "layout")]
pub mod layout;

#[cfg(feature = "components")]
pub mod lookandfeel;

// Optional editor helpers (softbuffer-backed windows)
#[cfg(feature = "softbuffer-editor")]
pub mod editor;
#[cfg(feature = "gl-editor")]
pub mod gl_editor;

pub use error::{GuiError, Result};

#[cfg(feature = "layout")]
pub use layout::{
    AbsoluteLayout, FlexAlign, FlexDirection, FlexLayout, GridLayout, SizeConstraint,
};

#[cfg(feature = "components")]
pub use components::{Bounds, Component, ComponentId, ComponentState};

#[cfg(feature = "components")]
pub use controls::{Button, ButtonState, Label, Slider, SliderOrientation, TextAlignment};

#[cfg(feature = "components")]
pub use input::{
    EventResult, InputCallbacks, KeyCode, KeyboardCallback, KeyboardEvent, KeyboardListener,
    Modifiers, MouseButton, MouseCallback, MouseEvent, MouseListener,
};

#[cfg(feature = "components")]
pub use lookandfeel::{ColorScheme, DefaultLookAndFeel, LookAndFeel, Theme};

#[cfg(feature = "softbuffer-editor")]
pub use editor::{SoftbufferWindow, SoftbufferWindowBuilder, render_controls_sample};

#[cfg(feature = "gl-editor")]
pub use gl_editor::{GlWindow, GlWindowBuilder};
