//! Timer support for JUCE components.
//!
//! This module provides timer functionality for periodic callbacks on the
//! JUCE message thread. Timers are useful for animations, periodic UI updates,
//! and other time-based operations.
//!
//! # Thread Safety
//!
//! All timer operations must be performed on the JUCE message thread. The Timer
//! type does not implement `Send` or `Sync`, enforcing this requirement at
//! compile time.
//!
//! # Example
//!
//! ```no_run
//! use nih_plug_juce::events::Timer;
//!
//! // Create a timer with a callback
//! let mut timer = Timer::new(|| {
//!     println!("Timer fired!");
//! }).expect("Failed to create timer");
//!
//! // Start the timer with a 100ms interval
//! timer.start(100).expect("Failed to start timer");
//!
//! // Check if the timer is running
//! assert!(timer.is_running());
//!
//! // Stop the timer
//! timer.stop();
//! assert!(!timer.is_running());
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;

/// A timer that provides periodic callbacks on the JUCE message thread.
///
/// The Timer wraps a JUCE Timer object and provides safe access to timer
/// functionality. When the timer fires, it invokes a Rust closure on the
/// message thread.
///
/// # Thread Safety
///
/// Timer does not implement `Send` or `Sync`, ensuring that all timer
/// operations are performed on the JUCE message thread. This is enforced
/// at compile time by Rust's type system.
///
/// # Lifetime
///
/// The timer automatically stops and cleans up when dropped. The callback
/// closure is also properly cleaned up through the Drop implementation.
///
/// # Example
///
/// ```no_run
/// use nih_plug_juce::events::Timer;
///
/// let mut timer = Timer::new(|| {
///     println!("Tick!");
/// }).expect("Failed to create timer");
///
/// timer.start(1000).expect("Failed to start timer"); // Fire every second
/// ```
pub struct Timer {
    ptr: *mut ffi::JuceTimer,
    _phantom: PhantomData<*mut ()>, // !Send + !Sync
}

impl Timer {
    /// Create a new timer with the specified callback.
    ///
    /// The callback will be invoked on the JUCE message thread each time
    /// the timer fires. The timer is not started automatically - you must
    /// call `start()` to begin receiving callbacks.
    ///
    /// # Arguments
    ///
    /// * `callback` - A closure to be called when the timer fires
    ///
    /// # Returns
    ///
    /// Returns `Ok(Timer)` on success, or `Err(JuceError)` if timer creation fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nih_plug_juce::events::Timer;
    ///
    /// let timer = Timer::new(|| {
    ///     println!("Timer callback!");
    /// }).expect("Failed to create timer");
    /// ```
    pub fn new<F>(callback: F) -> Result<Self>
    where
        F: Fn() + 'static,
    {
        // Box the closure so we can pass it to C++
        let boxed = Box::new(callback);
        let raw = Box::into_raw(boxed);

        // Create trampoline function that will be called from C++
        extern "C" fn trampoline<F: Fn()>(ptr: usize) {
            let callback = unsafe { &*(ptr as *const F) };
            callback();
        }

        // Create drop function to clean up the closure
        extern "C" fn drop_closure<F>(ptr: usize) {
            unsafe {
                let _ = Box::from_raw(ptr as *mut F);
            }
        }

        // Call FFI to create the timer
        let mut error_buffer = [0i8; 256];
        let ptr = unsafe {
            ffi::create_timer(
                raw as usize,
                trampoline::<F> as usize,
                drop_closure::<F> as usize,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if ptr.is_null() {
            // Clean up the closure since timer creation failed
            unsafe {
                let _ = Box::from_raw(raw);
            }

            // Convert error buffer to string
            let error_msg = unsafe {
                let len = error_buffer.iter().position(|&c| c == 0).unwrap_or(error_buffer.len());
                String::from_utf8_lossy(&std::mem::transmute::<&[i8], &[u8]>(&error_buffer[..len]))
                    .into_owned()
            };

            return Err(JuceError::ComponentCreationFailed(format!("Timer creation failed: {}", error_msg)));
        }

        Ok(Timer {
            ptr,
            _phantom: PhantomData,
        })
    }

    /// Start the timer with the specified interval.
    ///
    /// The timer will fire repeatedly at the specified interval until
    /// `stop()` is called. If the timer is already running, this will
    /// restart it with the new interval.
    ///
    /// # Arguments
    ///
    /// * `interval_ms` - The interval in milliseconds between timer callbacks
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `Err(JuceError)` if starting the timer fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nih_plug_juce::events::Timer;
    ///
    /// let mut timer = Timer::new(|| {}).expect("Failed to create timer");
    /// timer.start(100).expect("Failed to start timer"); // Fire every 100ms
    /// ```
    pub fn start(&mut self, interval_ms: i32) -> Result<()> {
        let mut error_buffer = [0i8; 256];
        let result = unsafe {
            ffi::timer_start(
                self.ptr,
                interval_ms,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if result != 0 {
            let error_msg = unsafe {
                let len = error_buffer.iter().position(|&c| c == 0).unwrap_or(error_buffer.len());
                String::from_utf8_lossy(&std::mem::transmute::<&[i8], &[u8]>(&error_buffer[..len]))
                    .into_owned()
            };
            return Err(JuceError::OperationFailed(format!("Failed to start timer: {}", error_msg)));
        }

        Ok(())
    }

    /// Stop the timer.
    ///
    /// After calling this method, the timer will no longer fire until
    /// `start()` is called again. It is safe to call this method even
    /// if the timer is not currently running.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nih_plug_juce::events::Timer;
    ///
    /// let mut timer = Timer::new(|| {}).expect("Failed to create timer");
    /// timer.start(100).expect("Failed to start timer");
    /// timer.stop(); // Stop the timer
    /// ```
    pub fn stop(&mut self) {
        unsafe {
            ffi::timer_stop(self.ptr);
        }
    }

    /// Check if the timer is currently running.
    ///
    /// # Returns
    ///
    /// Returns `true` if the timer is running, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use nih_plug_juce::events::Timer;
    ///
    /// let mut timer = Timer::new(|| {}).expect("Failed to create timer");
    /// assert!(!timer.is_running());
    ///
    /// timer.start(100).expect("Failed to start timer");
    /// assert!(timer.is_running());
    ///
    /// timer.stop();
    /// assert!(!timer.is_running());
    /// ```
    pub fn is_running(&self) -> bool {
        unsafe { ffi::timer_is_running(self.ptr) }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Stop the timer first to ensure no more callbacks
        self.stop();

        // Delete the timer (this will also clean up the callback closure)
        unsafe {
            ffi::delete_timer(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        // This test just verifies that the Timer type compiles correctly
        // Actual timer functionality requires the JUCE message thread
        let _timer_fn = || {
            let _timer = Timer::new(|| {
                println!("Timer fired!");
            });
        };
    }
}
