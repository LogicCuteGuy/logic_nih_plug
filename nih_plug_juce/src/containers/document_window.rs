//! JUCE DocumentWindow component.
//!
//! This module provides a safe Rust wrapper around JUCE's DocumentWindow class,
//! which represents a top-level window with a title bar and optional close button.
//!
//! # Thread Safety
//!
//! All DocumentWindow operations must be performed on the JUCE message thread.
//! This is enforced through the type system - DocumentWindow does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::containers::DocumentWindow;
//! use nih_plug_juce::Component;
//!
//! let mut window = DocumentWindow::new("My Plugin")?;
//! let content = Component::new()?;
//! window.set_content_owned(content)?;
//! window.set_visible(true);
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A JUCE DocumentWindow - a top-level window with title bar.
///
/// DocumentWindow is used to create standalone windows with a title bar,
/// close button, and content area. It's commonly used for plugin editors
/// and standalone applications.
///
/// # Inheritance
///
/// DocumentWindow inherits from Component through Deref/DerefMut, so all
/// Component methods are available on DocumentWindow instances.
///
/// # Thread Safety
///
/// DocumentWindow does not implement `Send` or `Sync`, enforcing that all
/// window operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::containers::DocumentWindow;
/// use nih_plug_juce::Component;
///
/// // Create a window
/// let mut window = DocumentWindow::new("My Application")?;
///
/// // Create and set content
/// let mut content = Component::new()?;
/// content.set_bounds(0, 0, 400, 300);
/// window.set_content_owned(content)?;
///
/// // Show the window
/// window.set_visible(true);
/// ```
pub struct DocumentWindow {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make DocumentWindow !Send + !Sync.
    /// This enforces that DocumentWindow can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl DocumentWindow {
    /// Create a new DocumentWindow with the specified title.
    ///
    /// This allocates a new juce::DocumentWindow in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Arguments
    ///
    /// * `title` - The window title to display in the title bar
    ///
    /// # Returns
    ///
    /// Returns `Ok(DocumentWindow)` on success, or an error if window
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::containers::DocumentWindow;
    ///
    /// let window = DocumentWindow::new("My Plugin")?;
    /// ```
    pub fn new(title: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_document_window(
                title.as_ptr(),
                title.len(),
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
                    "Unknown error creating DocumentWindow".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_document_window
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(DocumentWindow {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the content component for this window, transferring ownership.
    ///
    /// The window takes ownership of the content component and will manage
    /// its lifetime. The content component will be displayed in the window's
    /// content area.
    ///
    /// # Arguments
    ///
    /// * `content` - The component to use as window content
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the operation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut window = DocumentWindow::new("My Plugin")?;
    /// let content = Component::new()?;
    /// window.set_content_owned(content)?;
    /// ```
    pub fn set_content_owned(&mut self, content: Component) -> Result<()> {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("DocumentWindow pointer is null".to_string()));
        }
        
        let content_ptr = content.as_ptr();
        if content_ptr.is_null() {
            return Err(JuceError::NullPointer("Content component pointer is null".to_string()));
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::document_window_set_content_owned(
                ptr,
                content_ptr,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if result == 0 {
            // Prevent the content component from being dropped since JUCE now owns it
            std::mem::forget(content);
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Set the window title.
    ///
    /// This updates the text displayed in the window's title bar.
    ///
    /// # Arguments
    ///
    /// * `name` - The new window title
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut window = DocumentWindow::new("Initial Title")?;
    /// window.set_name("Updated Title");
    /// ```
    pub fn set_name(&mut self, name: &str) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::document_window_set_name(ptr, name.as_ptr(), name.len());
        }
    }
    
    /// Set a callback to be invoked when the window's close button is clicked.
    ///
    /// The callback should return `true` to allow the window to close, or
    /// `false` to prevent closing. This allows implementing custom close
    /// behavior or confirmation dialogs.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure that returns whether the window should close
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
    /// let mut window = DocumentWindow::new("My Plugin")?;
    /// window.set_on_close(|| {
    ///     println!("Window is closing");
    ///     true // Allow close
    /// })?;
    /// ```
    pub fn set_on_close<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn() -> bool + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("DocumentWindow pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize) -> bool
        where
            F: Fn() -> bool,
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure
            closure()
        }
        
        // Define the drop function that will be called when the window is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn() -> bool,
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::document_window_set_on_close(
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
                    "Unknown error setting window close callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for DocumentWindow {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for DocumentWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure DocumentWindow is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents windows from being moved or shared across threads,
// which is required by JUCE's threading model.
