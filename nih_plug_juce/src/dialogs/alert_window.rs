//! JUCE AlertWindow for displaying dialogs and alerts.
//!
//! This module provides a safe Rust wrapper around JUCE's AlertWindow class,
//! which is used to display message boxes, confirmation dialogs, and custom
//! alert windows.
//!
//! # Thread Safety
//!
//! All AlertWindow operations must be performed on the JUCE message thread.
//! The synchronous methods will block the message thread until the user responds,
//! while the async methods return immediately and invoke callbacks when the user
//! responds.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::dialogs::AlertWindow;
//!
//! // Show a simple message
//! AlertWindow::show_message_box("Info", "Operation completed successfully");
//!
//! // Show an async message with callback
//! AlertWindow::show_message_box_async("Info", "Processing...", || {
//!     println!("User dismissed the message");
//! });
//!
//! // Show OK/Cancel dialog
//! AlertWindow::show_ok_cancel_box("Confirm", "Are you sure?", |confirmed| {
//!     if confirmed {
//!         println!("User clicked OK");
//!     } else {
//!         println!("User clicked Cancel");
//!     }
//! });
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};

/// A JUCE AlertWindow for displaying dialogs and alerts.
///
/// AlertWindow provides static methods for showing common dialog types:
/// - Simple message boxes
/// - Asynchronous message boxes with callbacks
/// - OK/Cancel confirmation dialogs
///
/// # Thread Safety
///
/// All AlertWindow methods must be called on the JUCE message thread.
/// Synchronous methods will block until the user responds, while asynchronous
/// methods return immediately and invoke callbacks on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::dialogs::AlertWindow;
///
/// // Synchronous message (blocks until dismissed)
/// AlertWindow::show_message_box("Title", "Message");
///
/// // Asynchronous message (returns immediately)
/// AlertWindow::show_message_box_async("Title", "Message", || {
///     println!("Dialog dismissed");
/// });
///
/// // Confirmation dialog
/// AlertWindow::show_ok_cancel_box("Confirm", "Proceed?", |ok| {
///     if ok {
///         // User clicked OK
///     }
/// });
/// ```
pub struct AlertWindow;

impl AlertWindow {
    /// Show a synchronous message box.
    ///
    /// This displays a simple message box with an OK button and blocks the
    /// message thread until the user dismisses it. For non-blocking behavior,
    /// use [`show_message_box_async`](Self::show_message_box_async) instead.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display in the dialog's title bar
    /// * `message` - The message text to display
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread and will block
    /// until the user dismisses the dialog.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::dialogs::AlertWindow;
    ///
    /// AlertWindow::show_message_box("Success", "File saved successfully");
    /// ```
    pub fn show_message_box(title: &str, message: &str) {
        unsafe {
            ffi::alert_window_show_message_box(
                title.as_ptr(),
                title.len(),
                message.as_ptr(),
                message.len(),
            );
        }
    }
    
    /// Show an asynchronous message box with a callback.
    ///
    /// This displays a message box with an OK button and returns immediately.
    /// When the user dismisses the dialog, the provided callback is invoked
    /// on the message thread.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display in the dialog's title bar
    /// * `message` - The message text to display
    /// * `callback` - A closure to invoke when the dialog is dismissed
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting up the callback failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread. The callback
    /// will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::dialogs::AlertWindow;
    ///
    /// AlertWindow::show_message_box_async("Info", "Processing complete", || {
    ///     println!("User acknowledged the message");
    /// })?;
    /// ```
    pub fn show_message_box_async<F>(title: &str, message: &str, callback: F) -> Result<()>
    where
        F: Fn() + 'static,
    {
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
        
        // Define the drop function that will be called when the callback is no longer needed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::alert_window_show_message_box_async(
                title.as_ptr(),
                title.len(),
                message.as_ptr(),
                message.len(),
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
                    "Unknown error showing async message box".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
    
    /// Show an OK/Cancel confirmation dialog with a callback.
    ///
    /// This displays a dialog with OK and Cancel buttons and returns immediately.
    /// When the user clicks a button, the provided callback is invoked on the
    /// message thread with `true` for OK or `false` for Cancel.
    ///
    /// # Arguments
    ///
    /// * `title` - The title to display in the dialog's title bar
    /// * `message` - The message text to display
    /// * `callback` - A closure to invoke with the user's choice (true=OK, false=Cancel)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if setting up the callback failed.
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread. The callback
    /// will also be invoked on the message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::dialogs::AlertWindow;
    ///
    /// AlertWindow::show_ok_cancel_box("Confirm", "Delete this file?", |confirmed| {
    ///     if confirmed {
    ///         // User clicked OK - proceed with deletion
    ///         delete_file();
    ///     } else {
    ///         // User clicked Cancel - do nothing
    ///     }
    /// })?;
    /// ```
    pub fn show_ok_cancel_box<F>(title: &str, message: &str, callback: F) -> Result<()>
    where
        F: Fn(bool) + 'static,
    {
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);
        
        // Define the trampoline function that will be called from C++
        unsafe extern "C" fn trampoline<F>(closure_ptr: usize, result: bool)
        where
            F: Fn(bool),
        {
            // Reconstruct the closure reference (without taking ownership)
            let closure = &*(closure_ptr as *const F);
            
            // Invoke the Rust closure with the result
            closure(result);
        }
        
        // Define the drop function that will be called when the callback is no longer needed
        unsafe extern "C" fn drop_closure<F>(closure_ptr: usize)
        where
            F: Fn(bool),
        {
            // Take ownership and drop the closure
            let _ = Box::from_raw(closure_ptr as *mut F);
        }
        
        let mut error_buf = vec![0u8; 256];
        
        let result = unsafe {
            ffi::alert_window_show_ok_cancel_box(
                title.as_ptr(),
                title.len(),
                message.as_ptr(),
                message.len(),
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
                    "Unknown error showing OK/Cancel box".to_string()
                ))
            } else {
                Err(JuceError::CallbackError(error_msg))
            }
        }
    }
}

