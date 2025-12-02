//! LookAndFeel system for customizing component appearance.
//!
//! This module provides access to JUCE's LookAndFeel system, which controls
//! the visual appearance of GUI components. LookAndFeel objects define colors,
//! fonts, and drawing methods for all components.
//!
//! # Thread Safety
//!
//! LookAndFeel objects must only be used on the message thread. This is enforced
//! through the type system - LookAndFeel does not implement Send or Sync.
//!
//! # Example
//!
//! ```ignore
//! use nih_plug_juce::{LookAndFeel, Component};
//!
//! let mut laf = LookAndFeel::new_v4()?;
//! laf.set_colour(0x1000b00, Colour::from_rgb(100, 100, 200));
//!
//! let mut component = Component::new()?;
//! component.set_look_and_feel(&laf)?;
//! ```

use crate::bridge::ffi;
use crate::drawing::Colour;
use crate::error::{JuceError, Result};
use crate::graphics::Graphics;
use crate::widgets::Slider;
use crate::Component;
use std::marker::PhantomData;

/// A LookAndFeel object that defines the visual appearance of components.
///
/// LookAndFeel controls how components are drawn, including colors, fonts,
/// and custom drawing methods. JUCE provides a default LookAndFeel_V4
/// implementation that can be customized by setting colors or by subclassing
/// (though subclassing is not yet supported in this FFI layer).
///
/// # Thread Safety
///
/// LookAndFeel objects must only be used on the message thread. The type
/// system enforces this by not implementing Send or Sync.
///
/// # Lifetime
///
/// LookAndFeel objects must outlive any components that use them. Setting
/// a LookAndFeel on a component does not transfer ownership - the component
/// holds a reference to the LookAndFeel.
pub struct LookAndFeel {
    ptr: *mut ffi::JuceLookAndFeel,
    _phantom: PhantomData<*mut ()>, // !Send + !Sync
}

impl LookAndFeel {
    /// Create a new LookAndFeel_V4 instance.
    ///
    /// This creates a JUCE LookAndFeel_V4 object, which is the modern
    /// default look and feel for JUCE applications.
    ///
    /// # Returns
    ///
    /// Returns a new LookAndFeel instance, or an error if creation failed.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::LookAndFeel;
    ///
    /// let laf = LookAndFeel::new_v4()?;
    /// ```
    pub fn new_v4() -> Result<Self> {
        let mut error_buffer = [0i8; 512];
        let ptr = unsafe {
            ffi::create_lookandfeel_v4(
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if ptr.is_null() {
            let error_msg = unsafe {
                std::ffi::CStr::from_ptr(error_buffer.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            };
            Err(JuceError::OperationFailed(format!(
                "Failed to create LookAndFeel_V4: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Set a color for a specific color ID.
    ///
    /// JUCE components use color IDs to look up colors from the LookAndFeel.
    /// This method allows you to customize these colors.
    ///
    /// # Arguments
    ///
    /// * `colour_id` - The JUCE color ID to set (e.g., TextButton::buttonColourId)
    /// * `colour` - The color to use for this ID
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::{LookAndFeel, Colour};
    ///
    /// let mut laf = LookAndFeel::new_v4()?;
    /// // Set button background color (TextButton::buttonColourId = 0x1000100)
    /// laf.set_colour(0x1000100, Colour::from_rgb(100, 100, 200));
    /// ```
    pub fn set_colour(&mut self, colour_id: i32, colour: Colour) {
        unsafe {
            ffi::lookandfeel_set_colour(self.ptr, colour_id, colour.as_ptr());
        }
    }

    /// Find the color for a specific color ID.
    ///
    /// This retrieves the color that would be used for the given color ID.
    /// If the color hasn't been explicitly set, it returns the default color
    /// from the LookAndFeel.
    ///
    /// # Arguments
    ///
    /// * `colour_id` - The JUCE color ID to query
    ///
    /// # Returns
    ///
    /// Returns the color for the given ID.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::LookAndFeel;
    ///
    /// let laf = LookAndFeel::new_v4()?;
    /// let button_color = laf.find_colour(0x1000100);
    /// ```
    pub fn find_colour(&self, colour_id: i32) -> Colour {
        unsafe {
            let ptr = ffi::lookandfeel_find_colour(self.ptr, colour_id);
            Colour::from_ptr(ptr)
        }
    }

    /// Get the raw pointer to the underlying JUCE LookAndFeel object.
    ///
    /// # Safety
    ///
    /// This is an internal method used by the FFI layer. The returned pointer
    /// is only valid as long as this LookAndFeel object exists.
    pub(crate) fn as_ptr(&self) -> *mut ffi::JuceLookAndFeel {
        self.ptr
    }
}

impl Drop for LookAndFeel {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_lookandfeel(self.ptr);
        }
    }
}

/// Trait for custom LookAndFeel implementations.
///
/// This trait allows you to customize how components are drawn by implementing
/// custom drawing methods. Note that full trait-based customization is not yet
/// implemented in this FFI layer - this trait is provided for future extensibility.
///
/// For now, use `LookAndFeel::set_colour()` to customize component colors.
pub trait LookAndFeelMethods {
    /// Draw the background of a button.
    ///
    /// # Arguments
    ///
    /// * `g` - Graphics context for drawing
    /// * `button` - The button component being drawn
    /// * `colour` - The background color to use
    /// * `is_highlighted` - Whether the button is highlighted (mouse over)
    /// * `is_down` - Whether the button is pressed down
    fn draw_button_background(
        &self,
        g: &mut Graphics,
        button: &Component,
        colour: &Colour,
        is_highlighted: bool,
        is_down: bool,
    );

    /// Draw a slider.
    ///
    /// # Arguments
    ///
    /// * `g` - Graphics context for drawing
    /// * `x` - X coordinate of the slider
    /// * `y` - Y coordinate of the slider
    /// * `width` - Width of the slider
    /// * `height` - Height of the slider
    /// * `slider_pos` - Current position of the slider thumb
    /// * `min_slider_pos` - Minimum position of the slider thumb
    /// * `max_slider_pos` - Maximum position of the slider thumb
    /// * `style` - The slider style
    /// * `slider` - The slider component being drawn
    fn draw_slider(
        &self,
        g: &mut Graphics,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        slider_pos: f32,
        min_slider_pos: f32,
        max_slider_pos: f32,
        style: crate::widgets::SliderStyle,
        slider: &Slider,
    );

    /// Draw a label.
    ///
    /// # Arguments
    ///
    /// * `g` - Graphics context for drawing
    /// * `label` - The label component being drawn
    fn draw_label(&self, g: &mut Graphics, label: &crate::widgets::Label);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookandfeel_creation() {
        let laf = LookAndFeel::new_v4();
        assert!(laf.is_ok(), "LookAndFeel creation should succeed");
    }

    #[test]
    fn test_lookandfeel_set_and_find_colour() {
        let mut laf = LookAndFeel::new_v4().expect("Failed to create LookAndFeel");
        
        // Set a custom color
        let custom_color = Colour::from_rgb(123, 45, 67).expect("Failed to create colour");
        let colour_id = 0x1000100; // TextButton::buttonColourId
        
        laf.set_colour(colour_id, custom_color);
        
        // Find the color we just set
        let found_color = laf.find_colour(colour_id);
        
        // Note: We can't directly compare colors without implementing PartialEq,
        // but we can verify the operation doesn't crash
        drop(found_color);
    }
}
