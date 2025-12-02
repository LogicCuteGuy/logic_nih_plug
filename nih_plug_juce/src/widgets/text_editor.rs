//! JUCE TextEditor components.
//!
//! This module provides safe Rust wrappers around JUCE's TextEditor component,
//! which is used for text input and editing.
//!
//! # Thread Safety
//!
//! All text editor operations must be performed on the JUCE message thread.
//! This is enforced through the type system - text editors do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::widgets::TextEditor;
//!
//! let mut editor = TextEditor::new()?;
//! editor.set_bounds(10, 10, 200, 100);
//! editor.set_multiline(true);
//! editor.set_text("Enter text here...");
//! editor.set_on_text_change(|text| {
//!     println!("Text changed: {}", text);
//! })?;
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A JUCE TextEditor - a component for text input and editing.
///
/// TextEditor provides a text input field that can be single-line or multiline.
/// It supports text change callbacks and can be made read-only. This is useful
/// for user input, displaying editable text, or creating text-based interfaces.
///
/// # Inheritance
///
/// TextEditor inherits from Component through Deref/DerefMut, so all
/// Component methods are available on TextEditor instances.
///
/// # Thread Safety
///
/// TextEditor does not implement `Send` or `Sync`, enforcing that all
/// text editor operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::widgets::TextEditor;
///
/// // Create a text editor
/// let mut editor = TextEditor::new()?;
///
/// // Set its position and size (inherited from Component)
/// editor.set_bounds(10, 10, 200, 100);
///
/// // Configure as multiline
/// editor.set_multiline(true);
///
/// // Set initial text
/// editor.set_text("Enter your text here...");
///
/// // Set a text change callback
/// editor.set_on_text_change(|text| {
///     println!("Text changed to: {}", text);
/// })?;
/// ```
pub struct TextEditor {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make TextEditor !Send + !Sync.
    /// This enforces that TextEditor can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl TextEditor {
    /// Create a new TextEditor.
    ///
    /// This allocates a new juce::TextEditor in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Returns
    ///
    /// Returns `Ok(TextEditor)` on success, or an error if text editor
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::widgets::TextEditor;
    ///
    /// let editor = TextEditor::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_text_editor(
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
                    "Unknown error creating TextEditor".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_text_editor
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(TextEditor {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the text in the text editor.
    ///
    /// This updates the text editor's content. This will not trigger
    /// the text change callback.
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
    /// let mut editor = TextEditor::new()?;
    /// editor.set_text("Hello, world!");
    /// ```
    pub fn set_text(&mut self, text: &str) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::text_editor_set_text(ptr, text.as_ptr(), text.len());
        }
    }
    
    /// Get the current text from the text editor.
    ///
    /// # Returns
    ///
    /// Returns the current text as a String.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let editor = TextEditor::new()?;
    /// let text = editor.get_text();
    /// println!("Current text: {}", text);
    /// ```
    pub fn get_text(&self) -> String {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return String::new();
        }
        
        // Allocate a buffer for the text
        // Start with a reasonable size and grow if needed
        let mut buffer = vec![0u8; 1024];
        
        let bytes_written = unsafe {
            ffi::text_editor_get_text(ptr, buffer.as_mut_ptr(), buffer.len())
        };
        
        // If the text was truncated, allocate a larger buffer and try again
        if bytes_written >= buffer.len() {
            buffer.resize(bytes_written + 1, 0);
            let bytes_written = unsafe {
                ffi::text_editor_get_text(ptr, buffer.as_mut_ptr(), buffer.len())
            };
            buffer.truncate(bytes_written);
        } else {
            buffer.truncate(bytes_written);
        }
        
        // Convert UTF-8 bytes to String
        String::from_utf8_lossy(&buffer).to_string()
    }
    
    /// Set whether the text editor is multiline.
    ///
    /// Multiline text editors allow multiple lines of text with line breaks.
    /// Single-line text editors only allow one line of text.
    ///
    /// # Arguments
    ///
    /// * `multiline` - true for multiline, false for single line
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut editor = TextEditor::new()?;
    /// editor.set_multiline(true); // Enable multiline mode
    /// ```
    pub fn set_multiline(&mut self, multiline: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::text_editor_set_multiline(ptr, multiline);
        }
    }
    
    /// Set whether the text editor is read-only.
    ///
    /// Read-only text editors display text but do not allow editing.
    /// This is useful for displaying information that should not be modified.
    ///
    /// # Arguments
    ///
    /// * `readonly` - true for read-only, false for editable
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut editor = TextEditor::new()?;
    /// editor.set_readonly(true); // Make read-only
    /// ```
    pub fn set_readonly(&mut self, readonly: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::text_editor_set_readonly(ptr, readonly);
        }
    }
    
    /// Set a callback to be invoked when the text changes.
    ///
    /// The callback will be invoked on the message thread whenever the
    /// text editor's content changes. The callback receives the new text
    /// as a parameter.
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
    /// let mut editor = TextEditor::new()?;
    /// editor.set_on_text_change(|text| {
    ///     println!("Text changed to: {}", text);
    /// })?;
    /// ```
    pub fn set_on_text_change<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(&str) + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("TextEditor pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, text: *const u8, text_len: usize)
        where
            F: Fn(&str),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Convert the text to a Rust string slice
            let text_slice = std::slice::from_raw_parts(text, text_len);
            let text_str = std::str::from_utf8_unchecked(text_slice);
            
            // Invoke the Rust closure with the text
            closure(text_str);
        }
        
        // Define the drop function that will be called when the text editor is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(&str),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::text_editor_set_on_text_change(
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
                    "Unknown error setting text editor text change callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for TextEditor {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for TextEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure TextEditor is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents text editors from being moved or shared across threads,
// which is required by JUCE's threading model.
