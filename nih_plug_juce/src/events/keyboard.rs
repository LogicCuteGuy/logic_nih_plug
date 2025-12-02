//! Keyboard event handling for JUCE components.
//!
//! This module provides types and traits for handling keyboard events on JUCE
//! components. Keyboard events include key presses, key releases, and focus changes.
//!
//! # Thread Safety
//!
//! All keyboard event callbacks are invoked on the JUCE message thread. KeyPress
//! and related types do not implement `Send` or `Sync`, enforcing that they can
//! only be used on the message thread.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::{Component, KeyListener, KeyPress};
//!
//! struct MyListener;
//!
//! impl KeyListener for MyListener {
//!     fn key_pressed(&mut self, key: &KeyPress) -> bool {
//!         println!("Key pressed: code={}", key.key_code);
//!         true // Consume the event
//!     }
//! }
//!
//! let mut component = Component::new()?;
//! component.set_wants_keyboard_focus(true)?;
//! component.set_key_listener(Box::new(MyListener))?;
//! ```

use std::marker::PhantomData;
use super::mouse::ModifierKeys;

/// Key press event data.
///
/// This struct contains information about a keyboard event, including the
/// key code and the state of modifier keys.
///
/// # Thread Safety
///
/// KeyPress does not implement `Send` or `Sync`, enforcing that it can
/// only be used on the message thread where events are delivered.
#[derive(Debug, Clone)]
pub struct KeyPress {
    /// The key code for the pressed key.
    /// 
    /// This follows JUCE's key code conventions. Common values include:
    /// - 32: Space
    /// - 13: Return/Enter
    /// - 27: Escape
    /// - 8: Backspace
    /// - 127: Delete
    /// - Letters: 'A' (65) through 'Z' (90)
    /// - Numbers: '0' (48) through '9' (57)
    pub key_code: i32,
    
    /// State of modifier keys at the time of the event.
    pub mods: ModifierKeys,
    
    /// PhantomData to make KeyPress !Send + !Sync.
    _phantom: PhantomData<*mut ()>,
}

impl KeyPress {
    /// Create a new KeyPress.
    ///
    /// # Arguments
    ///
    /// * `key_code` - The key code for the pressed key
    /// * `mods` - State of modifier keys
    ///
    /// # Returns
    ///
    /// Returns a new KeyPress instance.
    pub fn new(key_code: i32, mods: ModifierKeys) -> Self {
        KeyPress {
            key_code,
            mods,
            _phantom: PhantomData,
        }
    }
    
    /// Check if this is a letter key (A-Z).
    ///
    /// # Returns
    ///
    /// Returns true if the key code represents a letter.
    pub fn is_letter(&self) -> bool {
        self.key_code >= 65 && self.key_code <= 90
    }
    
    /// Check if this is a digit key (0-9).
    ///
    /// # Returns
    ///
    /// Returns true if the key code represents a digit.
    pub fn is_digit(&self) -> bool {
        self.key_code >= 48 && self.key_code <= 57
    }
    
    /// Check if this is the space key.
    ///
    /// # Returns
    ///
    /// Returns true if this is the space key.
    pub fn is_space(&self) -> bool {
        self.key_code == 32
    }
    
    /// Check if this is the return/enter key.
    ///
    /// # Returns
    ///
    /// Returns true if this is the return/enter key.
    pub fn is_return(&self) -> bool {
        self.key_code == 13
    }
    
    /// Check if this is the escape key.
    ///
    /// # Returns
    ///
    /// Returns true if this is the escape key.
    pub fn is_escape(&self) -> bool {
        self.key_code == 27
    }
}

/// Trait for handling keyboard events on a component.
///
/// Implement this trait to receive keyboard event callbacks from a JUCE component.
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
/// use nih_plug_juce::{KeyListener, KeyPress};
///
/// struct MyListener {
///     key_count: usize,
/// }
///
/// impl KeyListener for MyListener {
///     fn key_pressed(&mut self, key: &KeyPress) -> bool {
///         self.key_count += 1;
///         println!("Key {} pressed: code={}", self.key_count, key.key_code);
///         true // Consume the event
///     }
///     
///     fn focus_gained(&mut self) {
///         println!("Component gained keyboard focus");
///     }
/// }
/// ```
pub trait KeyListener {
    /// Called when a key is pressed on the component.
    ///
    /// # Arguments
    ///
    /// * `key` - Information about the key press event
    ///
    /// # Returns
    ///
    /// Return `true` to indicate that the key press was handled and should not
    /// be passed to parent components. Return `false` to allow the event to
    /// propagate.
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn key_pressed(&mut self, _key: &KeyPress) -> bool {
        false
    }
    
    /// Called when the keyboard state changes (key released, etc.).
    ///
    /// This is called when any key state changes, including key releases.
    /// It's less commonly used than `key_pressed`.
    ///
    /// # Returns
    ///
    /// Return `true` to indicate that the state change was handled.
    /// Return `false` to allow the event to propagate.
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn key_state_changed(&mut self) -> bool {
        false
    }
    
    /// Called when the component gains keyboard focus.
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn focus_gained(&mut self) {}
    
    /// Called when the component loses keyboard focus.
    ///
    /// # Thread Safety
    ///
    /// This callback is invoked on the JUCE message thread.
    fn focus_lost(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_key_press_creation() {
        let mods = ModifierKeys::new(true, false, false, false);
        let key = KeyPress::new(65, mods); // 'A' key
        assert_eq!(key.key_code, 65);
        assert!(key.mods.shift);
    }
    
    #[test]
    fn test_key_press_is_letter() {
        let mods = ModifierKeys::none();
        let key_a = KeyPress::new(65, mods); // 'A'
        let key_z = KeyPress::new(90, mods); // 'Z'
        let key_0 = KeyPress::new(48, mods); // '0'
        
        assert!(key_a.is_letter());
        assert!(key_z.is_letter());
        assert!(!key_0.is_letter());
    }
    
    #[test]
    fn test_key_press_is_digit() {
        let mods = ModifierKeys::none();
        let key_0 = KeyPress::new(48, mods); // '0'
        let key_9 = KeyPress::new(57, mods); // '9'
        let key_a = KeyPress::new(65, mods); // 'A'
        
        assert!(key_0.is_digit());
        assert!(key_9.is_digit());
        assert!(!key_a.is_digit());
    }
    
    #[test]
    fn test_key_press_special_keys() {
        let mods = ModifierKeys::none();
        let space = KeyPress::new(32, mods);
        let enter = KeyPress::new(13, mods);
        let escape = KeyPress::new(27, mods);
        
        assert!(space.is_space());
        assert!(enter.is_return());
        assert!(escape.is_escape());
    }
    
    #[test]
    fn test_key_listener_trait() {
        // Test that we can create a struct implementing KeyListener
        struct TestListener {
            handled: bool,
        }
        
        impl KeyListener for TestListener {
            fn key_pressed(&mut self, key: &KeyPress) -> bool {
                self.handled = true;
                key.is_letter()
            }
            
            fn focus_gained(&mut self) {
                self.handled = false;
            }
        }
        
        let mut listener = TestListener { handled: false };
        let mods = ModifierKeys::none();
        let key = KeyPress::new(65, mods); // 'A'
        
        let result = listener.key_pressed(&key);
        assert!(result);
        assert!(listener.handled);
        
        listener.focus_gained();
        assert!(!listener.handled);
    }
}
