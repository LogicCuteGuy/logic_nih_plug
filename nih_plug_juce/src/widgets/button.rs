//! JUCE Button components.
//!
//! This module provides safe Rust wrappers around JUCE's button components,
//! including TextButton and other button types.
//!
//! # Thread Safety
//!
//! All button operations must be performed on the JUCE message thread.
//! This is enforced through the type system - buttons do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::TextButton;
//!
//! let mut button = TextButton::new("Click Me")?;
//! button.set_bounds(10, 10, 100, 30);
//! button.set_enabled(true);
//! button.set_on_click(|| {
//!     println!("Button clicked!");
//! });
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr;

/// A JUCE TextButton - a clickable button with text label.
///
/// TextButton is one of the most commonly used GUI components. It displays
/// a text label and can be clicked to trigger an action. The button's
/// appearance can be customized through colors and the LookAndFeel system.
///
/// # Inheritance
///
/// TextButton inherits from Component through Deref/DerefMut, so all
/// Component methods are available on TextButton instances.
///
/// # Thread Safety
///
/// TextButton does not implement `Send` or `Sync`, enforcing that all
/// button operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::widgets::TextButton;
///
/// // Create a button
/// let mut button = TextButton::new("Click Me")?;
///
/// // Set its position and size (inherited from Component)
/// button.set_bounds(10, 10, 100, 30);
///
/// // Set button-specific properties
/// button.set_button_text("New Text");
/// button.set_enabled(true);
///
/// // Set a click callback
/// button.set_on_click(|| {
///     println!("Button was clicked!");
/// })?;
/// ```
pub struct TextButton {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make TextButton !Send + !Sync.
    /// This enforces that TextButton can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl TextButton {
    /// Create a new TextButton with the specified text.
    ///
    /// This allocates a new juce::TextButton in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Arguments
    ///
    /// * `text` - The initial text to display on the button
    ///
    /// # Returns
    ///
    /// Returns `Ok(TextButton)` on success, or an error if button
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::widgets::TextButton;
    ///
    /// let button = TextButton::new("Click Me")?;
    /// ```
    pub fn new(text: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_text_button(
                text.as_ptr(),
                text.len(),
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::ComponentCreationFailed(
                    "Unknown error creating TextButton".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_text_button
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(TextButton {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the text displayed on the button.
    ///
    /// This updates the button's label text. The button will automatically
    /// repaint to show the new text.
    ///
    /// # Arguments
    ///
    /// * `text` - The new text to display
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut button = TextButton::new("Initial")?;
    /// button.set_button_text("Updated");
    /// ```
    pub fn set_button_text(&mut self, text: &str) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::button_set_text(ptr, text.as_ptr(), text.len());
        }
    }
    
    /// Set whether the button is enabled.
    ///
    /// Disabled buttons are grayed out and do not respond to clicks.
    ///
    /// # Arguments
    ///
    /// * `enabled` - true to enable the button, false to disable it
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut button = TextButton::new("Click Me")?;
    /// button.set_enabled(false); // Disable the button
    /// ```
    pub fn set_enabled(&mut self, enabled: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::button_set_enabled(ptr, enabled);
        }
    }
    
    /// Set a color for the button.
    ///
    /// Buttons have multiple color IDs that control different aspects of
    /// their appearance (background, text, outline, etc.). This method
    /// allows customizing these colors.
    ///
    /// # Arguments
    ///
    /// * `colour_id` - The color ID to set (e.g., button background, text color)
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    /// * `a` - Alpha component (0-255)
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut button = TextButton::new("Click Me")?;
    /// // Set button background to red
    /// button.set_colour(0, 255, 0, 0, 255);
    /// ```
    pub fn set_colour(&mut self, colour_id: i32, r: u8, g: u8, b: u8, a: u8) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::button_set_colour(ptr, colour_id, r, g, b, a);
        }
    }
    
    /// Set a callback to be invoked when the button is clicked.
    ///
    /// The callback will be invoked on the message thread whenever the
    /// button is clicked by the user.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to invoke when the button is clicked
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting the callback failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    /// The callback will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut button = TextButton::new("Click Me")?;
    /// button.set_on_click(|| {
    ///     println!("Button was clicked!");
    /// })?;
    /// ```
    pub fn set_on_click<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn() + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("Button pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize)
        where
            F: Fn(),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure
            closure();
        }
        
        // Define the drop function that will be called when the button is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::button_set_on_click(
                ptr,
                raw as usize,
                trampoline::<F> as usize,
                drop_closure::<F> as usize,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if result == 0 {
            Ok(())
        } else {
            // If setting the callback failed, we need to clean up the boxed closure
            unsafe {
                let _ = Box::from_raw(raw);
            }
            
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::CallbackError(
                    "Unknown error setting button click callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for TextButton {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for TextButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure TextButton is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents buttons from being moved or shared across threads,
// which is required by JUCE's threading model.
