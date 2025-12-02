//! Message thread utilities for safe cross-thread UI updates.
//!
//! JUCE requires all GUI operations to be performed on the message thread.
//! This module provides utilities to check if code is running on the message
//! thread and to safely post callbacks to the message thread from other threads.
//!
//! # Thread Safety Enforcement
//!
//! This crate enforces JUCE's message thread requirement through multiple layers:
//!
//! ## 1. Type System Enforcement (!Send + !Sync)
//!
//! All JUCE GUI types use `PhantomData<*mut ()>` to prevent them from implementing
//! `Send` or `Sync`. This means the compiler will reject any attempt to move or
//! share GUI objects across threads:
//!
//! ```compile_fail
//! use nih_plug_juce::Component;
//! use std::thread;
//!
//! let component = Component::new().unwrap();
//! thread::spawn(move || {
//!     // This will not compile! Component is !Send
//!     component.set_visible(true);
//! });
//! ```
//!
//! ## 2. Runtime Assertions (Debug Mode)
//!
//! All public methods on GUI types include `assert_message_thread!()` debug
//! assertions. These verify at runtime (in debug builds) that methods are
//! called on the message thread:
//!
//! ```ignore
//! pub fn set_visible(&mut self, visible: bool) {
//!     assert_message_thread!(); // Panics in debug if not on message thread
//!     // ... actual implementation
//! }
//! ```
//!
//! ## 3. Safe Cross-Thread Updates
//!
//! When you need to update the UI from another thread (e.g., the audio thread),
//! use `MessageManager::call_async()` to safely post a callback to the message
//! thread:
//!
//! ```ignore
//! use nih_plug_juce::MessageManager;
//!
//! // From audio thread or any other thread
//! let value = 0.5;
//! MessageManager::call_async(move || {
//!     // This closure runs on the message thread
//!     // Safe to update UI here
//!     println!("Value: {}", value);
//! }).expect("Failed to post callback");
//! ```
//!
//! # Thread Safety
//!
//! All JUCE GUI types enforce message thread usage through the type system
//! by not implementing `Send` or `Sync`. However, sometimes you need to update
//! the UI from another thread (e.g., the audio processing thread). The
//! `MessageManager::call_async()` function provides a safe way to do this.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::{MessageManager, assert_message_thread};
//!
//! // Check if we're on the message thread
//! if MessageManager::is_message_thread() {
//!     // Safe to update UI directly
//! }
//!
//! // Update UI from another thread
//! let value = 0.5;
//! MessageManager::call_async(move || {
//!     // This closure runs on the message thread
//!     // Safe to update UI here
//!     println!("Value: {}", value);
//! });
//!
//! // Use debug assertion to catch thread violations
//! fn update_ui() {
//!     assert_message_thread!();
//!     // ... UI update code
//! }
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::ffi::c_void;

/// Message manager for thread-safe UI updates.
///
/// The `MessageManager` provides utilities for working with JUCE's message
/// thread. All GUI operations in JUCE must be performed on the message thread,
/// and this struct provides methods to check the current thread and post
/// callbacks to the message thread.
///
/// # Thread Safety
///
/// The `call_async()` method is the only safe way to update the UI from
/// another thread. It queues a closure for execution on the message thread,
/// ensuring that GUI operations are always performed in the correct context.
pub struct MessageManager;

impl MessageManager {
    /// Check if the current thread is the message thread.
    ///
    /// This function queries JUCE to determine if the calling thread is
    /// the message thread. All GUI operations must be performed on the
    /// message thread.
    ///
    /// # Returns
    ///
    /// Returns `true` if the current thread is the message thread,
    /// `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::MessageManager;
    ///
    /// if MessageManager::is_message_thread() {
    ///     // Safe to update UI
    /// } else {
    ///     // Must use call_async to update UI
    /// }
    /// ```
    pub fn is_message_thread() -> bool {
        unsafe { ffi::message_manager_is_message_thread() }
    }

    /// Post a callback to be executed on the message thread.
    ///
    /// This function queues a closure for execution on the message thread.
    /// It's the safe way to update the UI from another thread (e.g., the
    /// audio processing thread).
    ///
    /// The closure must be `Send + 'static` because it will be moved to
    /// the message thread. The closure will be executed asynchronously -
    /// this function returns immediately without waiting for the closure
    /// to execute.
    ///
    /// # Arguments
    ///
    /// * `callback` - The closure to execute on the message thread
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the callback was successfully queued, or an
    /// error if the operation failed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::MessageManager;
    ///
    /// // Update UI from audio thread
    /// let value = 0.5;
    /// MessageManager::call_async(move || {
    ///     // This runs on the message thread
    ///     println!("Value: {}", value);
    /// }).expect("Failed to post callback");
    /// ```
    ///
    /// # Safety
    ///
    /// The closure will be executed on the message thread at some point
    /// in the future. Make sure any captured data is still valid when
    /// the closure executes. Using `move` closures is recommended to
    /// ensure captured data is owned by the closure.
    pub fn call_async<F>(callback: F) -> Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        // Box the closure and convert to raw pointer
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);

        // Trampoline function that will be called from C++
        // This function takes the raw pointer, converts it back to a Box,
        // calls the closure, and then drops the Box to free memory
        unsafe extern "C" fn trampoline<F: FnOnce()>(ptr: *mut c_void) {
            if !ptr.is_null() {
                let boxed = Box::from_raw(ptr as *mut F);
                boxed();
            }
        }

        // Call the FFI function to post the callback
        let mut error_buffer = vec![0u8; 256];
        let result = unsafe {
            ffi::message_manager_call_async(
                raw as usize,
                trampoline::<F> as usize,
                error_buffer.as_mut_ptr() as *mut i8,
                error_buffer.len(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            // If posting failed, we need to clean up the boxed closure
            unsafe {
                let _ = Box::from_raw(raw);
            }

            let error_msg = String::from_utf8_lossy(&error_buffer)
                .trim_end_matches('\0')
                .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to post callback to message thread: {}",
                error_msg
            )))
        }
    }
}

/// Assert that the current code is running on the message thread.
///
/// This macro provides a debug assertion that checks if the current thread
/// is the message thread. It's useful for catching thread safety violations
/// during development.
///
/// The assertion is only active in debug builds. In release builds, it
/// compiles to nothing, so there's no runtime overhead.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::assert_message_thread;
///
/// fn update_ui() {
///     assert_message_thread!();
///     // ... UI update code
/// }
/// ```
///
/// # Panics
///
/// Panics in debug builds if the current thread is not the message thread.
#[macro_export]
macro_rules! assert_message_thread {
    () => {
        debug_assert!(
            $crate::MessageManager::is_message_thread(),
            "This operation must be called on the message thread"
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_message_thread() {
        // This test runs on the main thread, which should be the message thread
        // in a test environment
        let is_message_thread = MessageManager::is_message_thread();
        // We can't assert true/false here because it depends on JUCE initialization
        // Just verify the function doesn't crash
        println!("Is message thread: {}", is_message_thread);
    }

    #[test]
    fn test_assert_message_thread_macro() {
        // Test that the macro compiles
        // Note: In debug builds, this will panic if not on the message thread
        // In release builds, it compiles to nothing
        // We can't reliably test the assertion itself without initializing JUCE's message thread
        
        // Just verify the macro compiles by using it in a conditional
        if MessageManager::is_message_thread() {
            assert_message_thread!();
        }
    }
}
