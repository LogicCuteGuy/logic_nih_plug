//! Mouse event handling for JUCE components.
//!
//! This module provides types and traits for handling mouse events on JUCE
//! components. Mouse events include clicks, drags, and hover events.
//!
//! # Thread Safety
//!
//! All mouse event callbacks are invoked on the JUCE message thread. MouseEvent
//! and related types do not implement `Send` or `Sync`, enforcing that they can
//! only be used on the message thread.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::{Component, MouseListener, MouseEvent};
//!
//! struct MyListener;
//!
//! impl MouseListener for MyListener {
//!     fn mouse_down(&mut self, event: &MouseEvent) {
//!         println!("Mouse clicked at ({}, {})", event.x, event.y);
//!     }
//!     
//!     fn mouse_drag(&mut self, event: &MouseEvent) {
//!         println!("Mouse dragged to ({}, {})", event.x, event.y);
//!     }
//! }
//!
//! let mut component = Component::new()?;
//! component.set_mouse_listener(Box::new(MyListener))?;
//! ```

use std::marker::PhantomData;

/// Modifier keys state for mouse and keyboard events.
///
/// This struct represents the state of modifier keys (Shift, Ctrl, Alt, Cmd)
/// at the time of an event.
///
/// # Thread Safety
///
/// ModifierKeys does not implement `Send` or `Sync`, enforcing that it can
/// only be used on the message thread where events are delivered.
#[derive(Debug, Clone, Copy)]
pub struct ModifierKeys {
    /// True if the Shift key is pressed.
    pub shift: bool,
    
    /// True if the Ctrl key is pressed (Command on macOS).
    pub ctrl: bool,
    
    /// True if the Alt key is pressed (Option on macOS).
    pub alt: bool,
    
    /// True if the Cmd key is pressed (macOS only, false on other platforms).
    pub cmd: bool,
    
    /// PhantomData to make ModifierKeys !Send + !Sync.
    _phantom: PhantomData<*mut ()>,
}

impl ModifierKeys {
    /// Create a new ModifierKeys with the specified state.
    ///
    /// # Arguments
    ///
    /// * `shift` - True if Shift is pressed
    /// * `ctrl` - True if Ctrl is pressed
    /// * `alt` - True if Alt is pressed
    /// * `cmd` - True if Cmd is pressed (macOS only)
    ///
    /// # Returns
    ///
    /// Returns a new ModifierKeys instance.
    pub fn new(shift: bool, ctrl: bool, alt: bool, cmd: bool) -> Self {
        ModifierKeys {
            shift,
            ctrl,
            alt,
            cmd,
            _phantom: PhantomData,
        }
    }
    
    /// Create a ModifierKeys with no modifiers pressed.
    ///
    /// # Returns
    ///
    /// Returns a ModifierKeys with all modifiers set to false.
    pub fn none() -> Self {
        ModifierKeys::new(false, false, false, false)
    }
    
    /// Check if any modifier key is pressed.
    ///
    /// # Returns
    ///
    /// Returns true if any modifier key is pressed, false otherwise.
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt || self.cmd
    }
}

/// Mouse event data.
///
/// This struct contains information about a mouse event, including the
/// position of the mouse and the state of modifier keys.
///
/// # Thread Safety
///
/// MouseEvent does not implement `Send` or `Sync`, enforcing that it can
/// only be used on the message thread where events are delivered.
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// X coordinate of the mouse position, relative to the component.
    pub x: i32,
    
    /// Y coordinate of the mouse position, relative to the component.
    pub y: i32,
    
    /// State of modifier keys at the time of the event.
    pub mods: ModifierKeys,
    
    /// PhantomData to make MouseEvent !Send + !Sync.
    _phantom: PhantomData<*mut ()>,
}

impl MouseEvent {
    /// Create a new MouseEvent.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the mouse position
    /// * `y` - Y coordinate of the mouse position
    /// * `mods` - State of modifier keys
    ///
    /// # Returns
    ///
    /// Returns a new MouseEvent instance.
    pub fn new(x: i32, y: i32, mods: ModifierKeys) -> Self {
        MouseEvent {
            x,
            y,
            mods,
            _phantom: PhantomData,
        }
    }
}

/// Trait for handling mouse events on a component.
///
/// Implement this trait to receive mouse event callbacks from a JUCE component.
/// All methods have default implementations that do nothing, so you only need
/// to implement the events you care about.
///
/// # Thread Safety
///
/// All callbacks are invoked on the JUCE message thread. The trait does not
/// require `Send` or `Sync`, allowing implementations to contain non-thread-safe
/// data.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::{MouseListener, MouseEvent};
///
/// struct MyListener {
///     click_count: usize,
/// }
///
/// impl MouseListener for MyListener {
///     fn mouse_down(&mut self, event: &MouseEvent) {
///         self.click_count += 1;
///         println!("Click {} at ({}, {})", self.click_count, event.x, event.y);
///     }
/// }
/// ```
pub trait MouseListener {
    /// Called when a mouse button is pressed on the component.
    ///
    /// # Arguments
    ///
    /// * `event` - Information about the mouse event
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn mouse_down(&mut self, _event: &MouseEvent) {}
    
    /// Called when the mouse is dragged on the component.
    ///
    /// This is called repeatedly as the mouse moves while a button is held down.
    ///
    /// # Arguments
    ///
    /// * `event` - Information about the mouse event
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn mouse_drag(&mut self, _event: &MouseEvent) {}
    
    /// Called when a mouse button is released on the component.
    ///
    /// # Arguments
    ///
    /// * `event` - Information about the mouse event
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn mouse_up(&mut self, _event: &MouseEvent) {}
    
    /// Called when the mouse enters the component's bounds.
    ///
    /// # Arguments
    ///
    /// * `event` - Information about the mouse event
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn mouse_enter(&mut self, _event: &MouseEvent) {}
    
    /// Called when the mouse exits the component's bounds.
    ///
    /// # Arguments
    ///
    /// * `event` - Information about the mouse event
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn mouse_exit(&mut self, _event: &MouseEvent) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_modifier_keys_creation() {
        let mods = ModifierKeys::new(true, false, true, false);
        assert!(mods.shift);
        assert!(!mods.ctrl);
        assert!(mods.alt);
        assert!(!mods.cmd);
    }
    
    #[test]
    fn test_modifier_keys_none() {
        let mods = ModifierKeys::none();
        assert!(!mods.shift);
        assert!(!mods.ctrl);
        assert!(!mods.alt);
        assert!(!mods.cmd);
        assert!(!mods.any());
    }
    
    #[test]
    fn test_modifier_keys_any() {
        let mods1 = ModifierKeys::new(true, false, false, false);
        assert!(mods1.any());
        
        let mods2 = ModifierKeys::none();
        assert!(!mods2.any());
    }
    
    #[test]
    fn test_mouse_event_creation() {
        let mods = ModifierKeys::new(true, false, false, false);
        let event = MouseEvent::new(100, 200, mods);
        assert_eq!(event.x, 100);
        assert_eq!(event.y, 200);
        assert!(event.mods.shift);
    }
    
    #[test]
    fn test_mouse_listener_trait() {
        // Test that we can create a struct implementing MouseListener
        struct TestListener;
        
        impl MouseListener for TestListener {
            fn mouse_down(&mut self, event: &MouseEvent) {
                // Custom implementation
                assert!(event.x >= 0);
            }
        }
        
        let mut listener = TestListener;
        let mods = ModifierKeys::none();
        let event = MouseEvent::new(50, 75, mods);
        listener.mouse_down(&event);
    }
}
