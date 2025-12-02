//! JUCE container components.
//!
//! This module provides safe Rust wrappers around JUCE's container components,
//! which are used to organize and manage other components.
//!
//! # Thread Safety
//!
//! All container operations must be performed on the JUCE message thread.
//! This is enforced through the type system - containers do not implement
//! `Send` or `Sync`.

pub mod document_window;
pub mod list_box;
pub mod resizable_window;
pub mod tabbed_component;
pub mod tree_view;
pub mod viewport;

pub use document_window::DocumentWindow;
pub use list_box::{ListBox, ListBoxModel};
pub use resizable_window::ResizableWindow;
pub use tabbed_component::{TabbedComponent, TabOrientation};
pub use tree_view::{TreeView, TreeViewItem};
pub use viewport::Viewport;
