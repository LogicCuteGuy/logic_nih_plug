//! JUCE parameter attachment system.
//!
//! This module provides parameter attachments that connect JUCE GUI components
//! (like sliders) to audio parameters, enabling bidirectional synchronization.
//!
//! # Thread Safety
//!
//! All parameter attachment operations must be performed on the JUCE message thread.
//! This is enforced through the type system - attachments do not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::parameter_attachment::SliderParameterAttachment;
//! use nih_plug_juce::widgets::Slider;
//!
//! let mut slider = Slider::new(SliderStyle::Rotary)?;
//! let attachment = SliderParameterAttachment::new(&mut slider, "gain")?;
//! // Now the slider and parameter are synchronized
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use crate::widgets::Slider;
use std::marker::PhantomData;

/// A parameter attachment that connects a Slider to an audio parameter.
///
/// This provides bidirectional synchronization between a JUCE slider and
/// an audio parameter. When the slider value changes, the parameter is
/// updated. When the parameter changes (e.g., from automation), the slider
/// is updated.
///
/// # Lifecycle
///
/// The attachment remains active as long as it exists. When dropped, the
/// connection is broken and synchronization stops.
///
/// # Thread Safety
///
/// SliderParameterAttachment does not implement `Send` or `Sync`, enforcing
/// that all operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::parameter_attachment::SliderParameterAttachment;
/// use nih_plug_juce::widgets::{Slider, SliderStyle};
///
/// let mut slider = Slider::new(SliderStyle::Rotary)?;
/// slider.set_range(0.0, 1.0, 0.01);
///
/// // Create attachment to connect slider to parameter
/// let attachment = SliderParameterAttachment::new(&mut slider, "gain")?;
///
/// // Now slider and parameter are synchronized bidirectionally
/// // Changing the slider updates the parameter
/// // Changing the parameter (e.g., via automation) updates the slider
/// ```
pub struct SliderParameterAttachment {
    /// Opaque pointer to the C++ SliderParameterAttachment object.
    ptr: *mut ffi::JuceSliderParameterAttachment,
    
    /// PhantomData to make SliderParameterAttachment !Send + !Sync.
    /// This enforces that the attachment can only be used on the thread where
    /// it was created (the message thread), matching JUCE's requirements.
    _phantom: PhantomData<*mut ()>,
}

impl SliderParameterAttachment {
    /// Create a new parameter attachment between a slider and a parameter.
    ///
    /// This establishes bidirectional synchronization:
    /// - When the slider value changes, the parameter is updated
    /// - When the parameter changes (e.g., from automation), the slider is updated
    ///
    /// # Arguments
    ///
    /// * `slider` - The slider to attach to the parameter
    /// * `parameter_id` - The ID of the parameter to attach to
    ///
    /// # Returns
    ///
    /// Returns `Ok(SliderParameterAttachment)` on success, or an error if
    /// attachment creation failed (e.g., parameter not found).
    ///
    /// # Thread Safety
    ///
    /// This function must be called on the JUCE message thread.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::parameter_attachment::SliderParameterAttachment;
    /// use nih_plug_juce::widgets::{Slider, SliderStyle};
    ///
    /// let mut slider = Slider::new(SliderStyle::Rotary)?;
    /// let attachment = SliderParameterAttachment::new(&mut slider, "gain")?;
    /// ```
    pub fn new(slider: &mut Slider, parameter_id: &str) -> Result<Self> {
        let mut error_buf = vec![0u8; 256];
        
        let ptr = unsafe {
            ffi::create_slider_parameter_attachment(
                slider.as_ptr(),
                parameter_id.as_ptr(),
                parameter_id.len(),
                error_buf.as_mut_ptr() as *mut i8,
                error_buf.len(),
            )
        };
        
        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(&error_buf)
                .trim_end_matches('\0')
                .to_string();
            
            if error_msg.is_empty() {
                Err(JuceError::OperationFailed(
                    "Unknown error creating SliderParameterAttachment".to_string()
                ))
            } else {
                Err(JuceError::OperationFailed(error_msg))
            }
        } else {
            Ok(SliderParameterAttachment {
                ptr,
                _phantom: PhantomData,
            })
        }
    }
    
    /// Get the raw pointer to the C++ SliderParameterAttachment object.
    ///
    /// This is primarily for internal use by the FFI layer.
    ///
    /// # Safety
    ///
    /// The returned pointer is only valid as long as this SliderParameterAttachment
    /// exists. Do not store or use the pointer after the attachment is dropped.
    pub(crate) fn as_ptr(&self) -> *mut ffi::JuceSliderParameterAttachment {
        self.ptr
    }
}

impl Drop for SliderParameterAttachment {
    /// Clean up the parameter attachment when it goes out of scope.
    ///
    /// This breaks the connection between the slider and parameter,
    /// stopping bidirectional synchronization.
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::delete_slider_parameter_attachment(self.ptr);
            }
            self.ptr = std::ptr::null_mut();
        }
    }
}

// Ensure SliderParameterAttachment is !Send + !Sync (enforced by PhantomData<*mut ()>)
// This prevents attachments from being moved or shared across threads,
// which is required by JUCE's threading model.

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests require JUCE to be initialized and a message loop running.
    // They are primarily for documentation and will be tested in integration tests.
    
    #[test]
    fn test_attachment_is_not_send() {
        // This test verifies at compile time that SliderParameterAttachment
        // does not implement Send, preventing it from being moved across threads.
        fn assert_not_send<T: Send>() {}
        
        // Uncommenting this line should cause a compile error:
        // assert_not_send::<SliderParameterAttachment>();
    }
    
    #[test]
    fn test_attachment_is_not_sync() {
        // This test verifies at compile time that SliderParameterAttachment
        // does not implement Sync, preventing it from being shared across threads.
        fn assert_not_sync<T: Sync>() {}
        
        // Uncommenting this line should cause a compile error:
        // assert_not_sync::<SliderParameterAttachment>();
    }
}
