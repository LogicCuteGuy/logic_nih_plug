//! JUCE ResizableWindow component.
//!
//! This module provides a safe Rust wrapper around JUCE's ResizableWindow class,
//! which represents a window that can be resized by the user with configurable
//! size constraints.
//!
//! # Thread Safety
//!
//! All ResizableWindow operations must be performed on the JUCE message thread.
//! This is enforced through the type system - ResizableWindow does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::containers::ResizableWindow;
//! use nih_plug_juce::Component;
//!
//! let mut window = ResizableWindow::new("My Plugin")?;
//! window.set_resizable(true);
//! window.set_resize_limits(400, 300, 1920, 1080);
//! window.set_visible(true);
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A JUCE ResizableWindow - a window that can be resized by the user.
///
/// ResizableWindow extends DocumentWindow to provide user-resizable windows
/// with configurable minimum and maximum size constraints. It's commonly used
/// for plugin editors that need to support different screen sizes.
///
/// # Inheritance
///
/// ResizableWindow inherits from Component through Deref/DerefMut, so all
/// Component methods are available on ResizableWindow instances.
///
/// # Thread Safety
///
/// ResizableWindow does not implement `Send` or `Sync`, enforcing that all
/// window operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::containers::ResizableWindow;
///
/// // Create a resizable window
/// let mut window = ResizableWindow::new("My Application")?;
///
/// // Enable resizing with constraints
/// window.set_resizable(true);
/// window.set_resize_limits(400, 300, 1920, 1080);
///
/// // Set up resize callback
/// window.set_on_resized(|width, height| {
///     println!("Window resized to {}x{}", width, height);
/// })?;
///
/// // Show the window
/// window.set_visible(true);
/// ```
pub struct ResizableWindow {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make ResizableWindow !Send + !Sync.
    /// This enforces that ResizableWindow can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl ResizableWindow {
    /// Create a new ResizableWindow with the specified title.
    ///
    /// This allocates a new juce::ResizableWindow in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Arguments
    ///
    /// * `title` - The window title to display in the title bar
    ///
    /// # Returns
    ///
    /// Returns `Ok(ResizableWindow)` on success, or an error if window
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::containers::ResizableWindow;
    ///
    /// let window = ResizableWindow::new("My Plugin")?;
    /// ```
    pub fn new(title: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_resizable_window(
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
                    "Unknown error creating ResizableWindow".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_resizable_window
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(ResizableWindow {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Enable or disable user resizing of the window.
    ///
    /// When enabled, the window will display resize handles that allow the user
    /// to change the window size. When disabled, the window size is fixed.
    ///
    /// # Arguments
    ///
    /// * `resizable` - Whether the window should be resizable
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut window = ResizableWindow::new("My Plugin")?;
    /// window.set_resizable(true);
    /// ```
    pub fn set_resizable(&mut self, resizable: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::resizable_window_set_resizable(ptr, resizable);
        }
    }
    
    /// Set the minimum and maximum size constraints for the window.
    ///
    /// These constraints limit how small or large the user can resize the window.
    /// The current window size will be clamped to these limits if necessary.
    ///
    /// # Arguments
    ///
    /// * `min_width` - Minimum window width in pixels
    /// * `min_height` - Minimum window height in pixels
    /// * `max_width` - Maximum window width in pixels
    /// * `max_height` - Maximum window height in pixels
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut window = ResizableWindow::new("My Plugin")?;
    /// // Set minimum size to 400x300, maximum to 1920x1080
    /// window.set_resize_limits(400, 300, 1920, 1080);
    /// ```
    pub fn set_resize_limits(&mut self, min_width: i32, min_height: i32, 
                            max_width: i32, max_height: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::resizable_window_set_resize_limits(ptr, min_width, min_height, 
                                                    max_width, max_height);
        }
    }
    
    /// Set a callback to be invoked when the window is resized.
    ///
    /// The callback receives the new width and height of the window.
    /// This is useful for updating layout or notifying other components
    /// of the size change.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure that receives the new width and height
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
    /// let mut window = ResizableWindow::new("My Plugin")?;
    /// window.set_on_resized(|width, height| {
    ///     println!("Window resized to {}x{}", width, height);
    /// })?;
    /// ```
    pub fn set_on_resized<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(i32, i32) + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("ResizableWindow pointer is null".to_string()));
        }
        
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, width: i32, height: i32)
        where
            F: Fn(i32, i32),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure
            closure(width, height);
        }
        
        // Define the drop function that will be called when the window is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(i32, i32),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::resizable_window_set_on_resized(
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
                    "Unknown error setting window resize callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for ResizableWindow {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for ResizableWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure ResizableWindow is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents windows from being moved or shared across threads,
// which is required by JUCE's threading model.
