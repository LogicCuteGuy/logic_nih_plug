//! JUCE Viewport component.
//!
//! This module provides a safe Rust wrapper around JUCE's Viewport class,
//! which provides a scrollable view of a larger component.
//!
//! # Thread Safety
//!
//! All Viewport operations must be performed on the JUCE message thread.
//! This is enforced through the type system - Viewport does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::containers::Viewport;
//! use nih_plug_juce::Component;
//!
//! let mut viewport = Viewport::new()?;
//! let content = Component::new()?;
//! viewport.set_viewed_component(content)?;
//! viewport.set_scrollbars_shown(true, true);
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A JUCE Viewport - a scrollable view of a larger component.
///
/// Viewport provides a scrollable window into a larger component, with
/// optional horizontal and vertical scrollbars. It's commonly used for
/// displaying content that doesn't fit in the available space.
///
/// # Inheritance
///
/// Viewport inherits from Component through Deref/DerefMut, so all
/// Component methods are available on Viewport instances.
///
/// # Thread Safety
///
/// Viewport does not implement `Send` or `Sync`, enforcing that all
/// viewport operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::containers::Viewport;
/// use nih_plug_juce::Component;
///
/// // Create a viewport
/// let mut viewport = Viewport::new()?;
///
/// // Create and set content
/// let mut content = Component::new()?;
/// content.set_bounds(0, 0, 800, 600);
/// viewport.set_viewed_component(content)?;
///
/// // Enable scrollbars
/// viewport.set_scrollbars_shown(true, true);
///
/// // Set initial scroll position
/// viewport.set_view_position(0, 0);
/// ```
pub struct Viewport {
    /// The underlying Component.
    /// This is wrapped to provide the inheritance pattern through Deref/DerefMut.
    component: Component,
    
    /// PhantomData to make Viewport !Send + !Sync.
    /// This enforces that Viewport can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl Viewport {
    /// Create a new Viewport.
    ///
    /// This allocates a new juce::Viewport in C++ and returns a safe
    /// Rust wrapper around it.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Viewport)` on success, or an error if viewport
    /// creation failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::containers::Viewport;
    ///
    /// let viewport = Viewport::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_viewport(
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
                    "Unknown error creating Viewport".to_string()
                ))
            } else {
                Err(JuceError::ComponentCreationFailed(error_msg))
            }
        } else {
            // Wrap the raw pointer in a Component
            // Safety: We know this is a valid JuceComponent pointer from create_viewport
            let component = unsafe { Component::from_raw(ptr) };
            
            Ok(Viewport {
                component,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Set the component to be viewed in this viewport, transferring ownership.
    ///
    /// The viewport takes ownership of the viewed component and will manage
    /// its lifetime. The component will be displayed in the viewport's
    /// scrollable area.
    ///
    /// # Arguments
    ///
    /// * `component` - The component to view in the viewport
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
    /// let mut viewport = Viewport::new()?;
    /// let content = Component::new()?;
    /// viewport.set_viewed_component(content)?;
    /// ```
    pub fn set_viewed_component(&mut self, component: Component) -> Result<()> {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("Viewport pointer is null".to_string()));
        }
        
        let component_ptr = component.as_ptr();
        if component_ptr.is_null() {
            return Err(JuceError::NullPointer("Viewed component pointer is null".to_string()));
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::viewport_set_viewed_component(
                ptr,
                component_ptr,
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len()
            )
        };
        
        if result == 0 {
            // Prevent the component from being dropped since JUCE now owns it
            std::mem::forget(component);
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::CppException(error_msg))
        }
    }
    
    /// Set the scroll position of the viewport.
    ///
    /// This updates the position of the viewed component within the viewport,
    /// effectively scrolling the view.
    ///
    /// # Arguments
    ///
    /// * `x` - The X coordinate of the top-left corner of the visible area
    /// * `y` - The Y coordinate of the top-left corner of the visible area
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut viewport = Viewport::new()?;
    /// viewport.set_view_position(100, 50);
    /// ```
    pub fn set_view_position(&mut self, x: i32, y: i32) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::viewport_set_view_position(ptr, x, y);
        }
    }
    
    /// Set whether scrollbars are shown.
    ///
    /// This controls the visibility of the horizontal and vertical scrollbars.
    /// Scrollbars can be independently enabled or disabled.
    ///
    /// # Arguments
    ///
    /// * `vertical` - Whether to show the vertical scrollbar
    /// * `horizontal` - Whether to show the horizontal scrollbar
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut viewport = Viewport::new()?;
    /// // Show both scrollbars
    /// viewport.set_scrollbars_shown(true, true);
    /// // Show only vertical scrollbar
    /// viewport.set_scrollbars_shown(true, false);
    /// ```
    pub fn set_scrollbars_shown(&mut self, vertical: bool, horizontal: bool) {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::viewport_set_scrollbars_shown(ptr, vertical, horizontal);
        }
    }
    
    /// Set a callback to be invoked when the visible area changes.
    ///
    /// The callback is invoked whenever the viewport is scrolled or resized,
    /// changing which part of the viewed component is visible.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to be invoked when the visible area changes
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
    /// let mut viewport = Viewport::new()?;
    /// viewport.set_on_visible_area_changed(|| {
    ///     println!("Viewport scrolled or resized");
    /// })?;
    /// ```
    pub fn set_on_visible_area_changed<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn() + 'static,
    {
        let ptr = self.component.as_ptr();
        if ptr.is_null() {
            return Err(JuceError::NullPointer("Viewport pointer is null".to_string()));
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
        
        // Define the drop function that will be called when the viewport is destroyed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::viewport_set_on_visible_area_changed(
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
                    "Unknown error setting viewport visible area changed callback".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

impl Deref for Viewport {
    type Target = Component;
    
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for Viewport {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

// Ensure Viewport is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents viewports from being moved or shared across threads,
// which is required by JUCE's threading model.
