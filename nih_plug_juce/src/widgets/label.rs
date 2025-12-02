//! JUCE Label components.
//!
//! This module provides safe Rust wrappers around JUCE's Label component,
//! which is used for displaying text and optionally allowing text editing.
//!
//! # Thread Safety
//!
//! All label operations must be performed on the JUCE message thread.
//! This is enforced through the type system - labels do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::Label;
//!
//! let mut label = Label::new("Hello World")?;
//! label.set_bounds(10, 10, 200, 30);
//! label.set_font(16.0);
//! label.set_justification(Justification::Centred);
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Text justification constants.
///
/// These constants match JUCE's Justification flags and can be combined
/// using bitwise OR operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justification {
    /// Left-aligned text
    Left = 1,
    
    /// Right-aligned text
    Right = 2,
    
    /// Horizontally centered text
    HorizontallyCentred = 4,
    
    /// Top-aligned text
    Top = 8,
    
    /// Bottom-aligned text
    Bottom = 16,
    
    /// Vertically centered text
    VerticallyCentred = 32,
    
    /// Horizontally justified (stretched to fill width)
    HorizontallyJustified = 64,
    
    /// Centered both horizontally and vertically
    Centred = 36, // HorizontallyCentred | VerticallyCentred
    
    /// Centered horizontally, top-aligned vertically
    CentredTop = 12, // HorizontallyCentred | Top
    
    /// Centered horizontally, bottom-aligned vertically
    CentredBottom = 20, // HorizontallyCentred | Bottom
    
    /// Left-aligned horizontally, centered vertically
    CentredLeft = 33, // Left | VerticallyCentred
    
    /// Right-aligned horizontally, centered vertically
    CentredRight = 34, // Right | VerticallyCentred
    
    /// Top-left corner
    TopLeft = 9, // Left | Top
    
    /// Top-right corner
    TopRight = 10, // Right | Top
    
    /// Bottom-left corner
    BottomLeft = 17, // Left | Bottom
    
    /// Bottom-right corner
    BottomRight = 18, // Right | Bottom
}

/// A JUCE Label - a component for displaying and optionally editing text.
///
/// Label is commonly used for displaying static text, but can also be made
/// editable to allow user input. It supports various text formatting options
/// including font, justification, and colors.
///
/// # Inheritance
///
/// Label inherits from Component through Deref/DerefMut, so all
/// Component methods are available on Label instances.
///
/// # Thread Safety
///
/// Label does not implement `Send` or `Sync`, enforcing that all
/// label operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::widgets::{Label, Justification};
///
/// // Create a label
/// let mut label = Label::new("Hello World")?;
///
/// // Set its position and size (inherited from Component)
/// label.set_bounds(10, 10, 200, 30);
///
/// // Set label-specific properties
/// label.set_font(16.0);
/// label.set_justification(Justification::Centred);
///
/// // Make it editable
/// label.set_editable(true);
/// label.set_on_text_change(|text| {
///     println!("Text changed to: {}", text);
/// })?;
/// ```
pub struct Label {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make Label !Send + !Sync.
    /// This enforces that Label can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl Label {
    /// Create a new Label with the specified text.
    ///
    /// This allocates a new juce::Label in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Arguments
    ///
    /// * `text` - The initial text to display in the label
    ///
    /// # Returns
    ///
    /// Returns `Ok(Label)` on success, or an error if label
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::widgets::Label;
    ///
    /// let label = Label::new("Hello World")?;
    /// ```
    pub fn new(text: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_label(
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
                    "Unknown error creating Label".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_label
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(Label {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the text displayed in the label.
    ///
    /// This updates the label's text content. The label will automatically
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
    /// let mut label = Label::new("Initial")?;
    /// label.set_text("Updated");
    /// ```
    pub fn set_text(&mut self, text: &str) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::label_set_text(ptr, text.as_ptr(), text.len());
        }
    }
    
    /// Set the font size of the label.
    ///
    /// This updates the size of the font used to display the label's text.
    ///
    /// # Arguments
    ///
    /// * `font_size` - The font size in points
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut label = Label::new("Text")?;
    /// label.set_font(16.0); // 16 point font
    /// ```
    pub fn set_font(&mut self, font_size: f32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::label_set_font(ptr, font_size);
        }
    }
    
    /// Set the text justification of the label.
    ///
    /// This controls how the text is aligned within the label's bounds.
    ///
    /// # Arguments
    ///
    /// * `justification` - The justification to use
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::widgets::Justification;
    ///
    /// let mut label = Label::new("Centered")?;
    /// label.set_justification(Justification::Centred);
    /// ```
    pub fn set_justification(&mut self, justification: Justification) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::label_set_justification(ptr, justification as i32);
        }
    }
    
    /// Set whether the label is editable.
    ///
    /// When editable, the label can be clicked to edit its text.
    /// When not editable, the label is read-only.
    ///
    /// # Arguments
    ///
    /// * `editable` - true to make editable, false to make read-only
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut label = Label::new("Edit me")?;
    /// label.set_editable(true);
    /// ```
    pub fn set_editable(&mut self, editable: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::label_set_editable(ptr, editable);
        }
    }
    
    /// Set a callback to be invoked when the label text changes.
    ///
    /// The callback will be invoked on the message thread whenever the
    /// label text is changed by the user (when the label is editable).
    /// The callback receives the new text as a parameter.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to invoke when the text changes
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
    /// let mut label = Label::new("Edit me")?;
    /// label.set_editable(true);
    /// label.set_on_text_change(|text| {
    ///     println!("Label text changed to: {}", text);
    /// })?;
    /// ```
    pub fn set_on_text_change<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(&str) + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("Label pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, text_ptr: *const u8, text_len: usize)
        where
            F: Fn(&str),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Convert the C string to a Rust &str
            let text_slice = std::slice::from_raw_parts(text_ptr, text_len);
            if let Ok(text) = std::str::from_utf8(text_slice) {
                // Invoke the Rust closure with the text
                closure(text);
            }
        }
        
        // Define the drop function that will be called when the label is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(&str),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::label_set_on_text_change(
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
                    "Unknown error setting label text change callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for Label {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for Label {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure Label is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents labels from being moved or shared across threads,
// which is required by JUCE's threading model.
