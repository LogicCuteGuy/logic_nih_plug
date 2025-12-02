//! JUCE ToggleButton components.
//!
//! This module provides safe Rust wrappers around JUCE's ToggleButton component,
//! which is used for checkboxes, toggle switches, and radio buttons.
//!
//! # Thread Safety
//!
//! All toggle button operations must be performed on the JUCE message thread.
//! This is enforced through the type system - toggle buttons do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::ToggleButton;
//!
//! let mut toggle = ToggleButton::new("Enable Feature")?;
//! toggle.set_bounds(10, 10, 150, 30);
//! toggle.set_toggle_state(true);
//! toggle.set_on_click(|state| {
//!     println!("Toggle state: {}", state);
//! })?;
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A JUCE ToggleButton - a button that can be toggled on/off.
///
/// ToggleButton is commonly used for checkboxes, toggle switches, and radio buttons.
/// It maintains a boolean state (on/off) and can be grouped with other toggle buttons
/// to create radio button groups.
///
/// # Inheritance
///
/// ToggleButton inherits from Component through Deref/DerefMut, so all
/// Component methods are available on ToggleButton instances.
///
/// # Thread Safety
///
/// ToggleButton does not implement `Send` or `Sync`, enforcing that all
/// toggle button operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::widgets::ToggleButton;
///
/// // Create a toggle button
/// let mut toggle = ToggleButton::new("Enable Feature")?;
///
/// // Set its position and size (inherited from Component)
/// toggle.set_bounds(10, 10, 150, 30);
///
/// // Set toggle-specific properties
/// toggle.set_toggle_state(true);
///
/// // Set a click callback that receives the new state
/// toggle.set_on_click(|state| {
///     println!("Toggle is now: {}", if state { "ON" } else { "OFF" });
/// })?;
///
/// // Create radio buttons by setting a radio group ID
/// let mut radio1 = ToggleButton::new("Option 1")?;
/// let mut radio2 = ToggleButton::new("Option 2")?;
/// radio1.set_radio_group_id(1);
/// radio2.set_radio_group_id(1);
/// ```
pub struct ToggleButton {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make ToggleButton !Send + !Sync.
    /// This enforces that ToggleButton can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl ToggleButton {
    /// Create a new ToggleButton with the specified text.
    ///
    /// This allocates a new juce::ToggleButton in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display next to the toggle button
    ///
    /// # Returns
    ///
    /// Returns `Ok(ToggleButton)` on success, or an error if toggle button
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::widgets::ToggleButton;
    ///
    /// let toggle = ToggleButton::new("Enable Feature")?;
    /// ```
    pub fn new(text: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_toggle_button(
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
                    "Unknown error creating ToggleButton".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_toggle_button
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(ToggleButton {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the toggle state of the button.
    ///
    /// This updates the button's on/off state. The button will automatically
    /// repaint to show the new state. This will not trigger the click callback.
    ///
    /// # Arguments
    ///
    /// * `state` - true for on/checked, false for off/unchecked
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut toggle = ToggleButton::new("Enable")?;
    /// toggle.set_toggle_state(true); // Turn on
    /// ```
    pub fn set_toggle_state(&mut self, state: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::toggle_button_set_toggle_state(ptr, state);
        }
    }
    
    /// Get the current toggle state of the button.
    ///
    /// # Returns
    ///
    /// Returns true if the button is on/checked, false if off/unchecked.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let toggle = ToggleButton::new("Enable")?;
    /// let is_on = toggle.get_toggle_state();
    /// println!("Toggle is: {}", if is_on { "ON" } else { "OFF" });
    /// ```
    pub fn get_toggle_state(&self) -> bool {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return false;
        }
        
        unsafe {
            ffi::toggle_button_get_toggle_state(ptr)
        }
    }
    
    /// Set the radio group ID for this toggle button.
    ///
    /// Toggle buttons with the same radio group ID (and the same parent component)
    /// will behave as radio buttons - only one can be selected at a time.
    /// Setting a radio group ID of 0 removes the button from any radio group.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The radio group ID (0 to remove from groups)
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Create radio buttons
    /// let mut radio1 = ToggleButton::new("Option 1")?;
    /// let mut radio2 = ToggleButton::new("Option 2")?;
    /// let mut radio3 = ToggleButton::new("Option 3")?;
    ///
    /// // Group them together
    /// radio1.set_radio_group_id(1);
    /// radio2.set_radio_group_id(1);
    /// radio3.set_radio_group_id(1);
    /// ```
    pub fn set_radio_group_id(&mut self, group_id: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::toggle_button_set_radio_group_id(ptr, group_id);
        }
    }
    
    /// Set the text displayed next to the toggle button.
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
    /// let mut toggle = ToggleButton::new("Initial")?;
    /// toggle.set_button_text("Updated");
    /// ```
    pub fn set_button_text(&mut self, text: &str) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::toggle_button_set_text(ptr, text.as_ptr(), text.len());
        }
    }
    
    /// Set a callback to be invoked when the toggle button is clicked.
    ///
    /// The callback will be invoked on the message thread whenever the
    /// toggle button is clicked by the user. The callback receives the
    /// new toggle state as a parameter.
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
    /// let mut toggle = ToggleButton::new("Enable Feature")?;
    /// toggle.set_on_click(|state| {
    ///     println!("Feature is now: {}", if state { "enabled" } else { "disabled" });
    /// })?;
    /// ```
    pub fn set_on_click<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(bool) + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("ToggleButton pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, state: bool)
        where
            F: Fn(bool),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure with the state
            closure(state);
        }
        
        // Define the drop function that will be called when the button is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(bool),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::toggle_button_set_on_click(
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
                    "Unknown error setting toggle button click callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for ToggleButton {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for ToggleButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure ToggleButton is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents toggle buttons from being moved or shared across threads,
// which is required by JUCE's threading model.
