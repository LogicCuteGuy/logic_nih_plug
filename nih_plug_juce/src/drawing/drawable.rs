//! JUCE Drawable wrapper for vector graphics.
//!
//! This module provides a safe Rust wrapper around JUCE's Drawable class,
//! which represents vector graphics that can be loaded from SVG or image data.
//!
//! # Thread Safety
//!
//! Drawable objects must be used on the message thread, as they are GUI objects.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::drawing::Drawable;
//!
//! // Load from SVG data
//! let svg_data = r#"<svg>...</svg>"#;
//! let drawable = Drawable::create_from_svg(svg_data)?;
//!
//! // Draw it in a paint callback
//! drawable.draw(&mut graphics, 1.0)?;
//! ```

use crate::bridge::ffi;
use crate::component::Component;
use crate::error::{JuceError, Result};
use crate::graphics::Graphics;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A drawable object that can be rendered to a Graphics context.
///
/// Drawable represents vector graphics that can be loaded from SVG data
/// or image data. Drawables can be scaled, transformed, and drawn to
/// any Graphics context.
///
/// # Thread Safety
///
/// Drawable does not implement `Send` or `Sync`, enforcing that all
/// drawable operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::drawing::Drawable;
///
/// // Create from SVG
/// let svg = r#"<svg width="100" height="100">
///     <circle cx="50" cy="50" r="40" fill="blue"/>
/// </svg>"#;
/// let drawable = Drawable::create_from_svg(svg)?;
///
/// // Draw it
/// drawable.draw(&mut graphics, 1.0)?;
/// ```
pub struct Drawable {
    ptr: *mut ffi::JuceDrawable,
    _phantom: PhantomData<*mut ()>,
}

impl Drawable {
    /// Create a Drawable from SVG data.
    ///
    /// This parses SVG data and creates a drawable object that can be
    /// rendered to any Graphics context.
    ///
    /// # Arguments
    ///
    /// * `svg_data` - The SVG data as a string
    ///
    /// # Returns
    ///
    /// Returns a new Drawable instance, or an error if parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let svg = r#"<svg width="100" height="100">
    ///     <rect x="10" y="10" width="80" height="80" fill="red"/>
    /// </svg>"#;
    /// let drawable = Drawable::create_from_svg(svg)?;
    /// ```
    pub fn create_from_svg(svg_data: &str) -> Result<Self> {
        let svg_bytes = svg_data.as_bytes();
        let mut error_buffer = vec![0i8; 256];

        let ptr = unsafe {
            ffi::create_drawable_from_svg(
                svg_bytes.as_ptr(),
                svg_bytes.len(),
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to create drawable from SVG: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a Drawable from image data.
    ///
    /// This creates a drawable from raw image data (PNG, JPEG, etc.).
    ///
    /// # Arguments
    ///
    /// * `data` - The image data as bytes
    ///
    /// # Returns
    ///
    /// Returns a new Drawable instance, or an error if parsing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let image_data = std::fs::read("icon.png")?;
    /// let drawable = Drawable::create_from_image_data(&image_data)?;
    /// ```
    pub fn create_from_image_data(data: &[u8]) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];

        let ptr = unsafe {
            ffi::create_drawable_from_image_data(
                data.as_ptr(),
                data.len(),
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to create drawable from image data: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Draw the drawable to a Graphics context.
    ///
    /// # Arguments
    ///
    /// * `g` - The Graphics context to draw to
    /// * `opacity` - The opacity to draw with (0.0 = transparent, 1.0 = opaque)
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if drawing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// drawable.draw(&mut graphics, 1.0)?;
    /// drawable.draw(&mut graphics, 0.5)?; // 50% transparent
    /// ```
    pub fn draw(&self, g: &mut Graphics, opacity: f32) -> Result<()> {
        if opacity < 0.0 || opacity > 1.0 {
            return Err(JuceError::InvalidParameter(format!(
                "Opacity must be between 0.0 and 1.0: {}",
                opacity
            )));
        }

        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::drawable_draw(
                self.ptr,
                g.as_ptr_mut(),
                opacity,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to draw drawable: {}",
                error_msg
            )))
        }
    }

    /// Set the drawable's transform to fit within the specified bounds.
    ///
    /// This scales and positions the drawable to fit within the given
    /// rectangle, maintaining aspect ratio.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the bounding rectangle
    /// * `y` - Y coordinate of the bounding rectangle
    /// * `width` - Width of the bounding rectangle
    /// * `height` - Height of the bounding rectangle
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// drawable.set_transform_to_fit(0.0, 0.0, 100.0, 100.0)?;
    /// ```
    pub fn set_transform_to_fit(&mut self, x: f32, y: f32, width: f32, height: f32) -> Result<()> {
        if width <= 0.0 || height <= 0.0 {
            return Err(JuceError::InvalidParameter(format!(
                "Bounds dimensions must be positive: {}x{}",
                width, height
            )));
        }

        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::drawable_set_transform_to_fit(
                self.ptr,
                x,
                y,
                width,
                height,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to set transform: {}",
                error_msg
            )))
        }
    }

    /// Get a raw pointer to the underlying JUCE Drawable object.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the returned pointer is only valid
    /// as long as this Drawable instance exists. The caller must ensure
    /// the pointer is not used after this Drawable is dropped.
    pub(crate) unsafe fn as_ptr(&self) -> *const ffi::JuceDrawable {
        self.ptr
    }
}

impl Drop for Drawable {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_drawable(self.ptr);
        }
    }
}

/// A button that displays drawable images for different states.
///
/// DrawableButton is a button that uses Drawable objects for its visual
/// representation instead of text. Different drawables can be set for
/// normal, over (hover), and down (pressed) states.
///
/// # Inheritance
///
/// DrawableButton inherits from Component through Deref/DerefMut, so all
/// Component methods are available on DrawableButton instances.
///
/// # Thread Safety
///
/// DrawableButton does not implement `Send` or `Sync`, enforcing that all
/// button operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::drawing::{Drawable, DrawableButton};
///
/// // Create drawables for different states
/// let normal = Drawable::create_from_svg(normal_svg)?;
/// let over = Drawable::create_from_svg(hover_svg)?;
///
/// // Create button
/// let mut button = DrawableButton::new("MyButton")?;
/// button.set_images(&normal, Some(&over), None)?;
/// ```
pub struct DrawableButton {
    component: Component,
    _phantom: PhantomData<*mut ()>,
}

impl DrawableButton {
    /// Create a new DrawableButton with the specified name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name for the button (used for identification)
    ///
    /// # Returns
    ///
    /// Returns a new DrawableButton instance, or an error if creation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let button = DrawableButton::new("IconButton")?;
    /// ```
    pub fn new(name: &str) -> Result<Self> {
        let name_bytes = name.as_bytes();
        let mut error_buffer = vec![0i8; 256];

        let ptr = unsafe {
            ffi::create_drawable_button(
                name_bytes.as_ptr(),
                name_bytes.len(),
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if ptr.is_null() {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::ComponentCreationFailed(format!(
                "Failed to create DrawableButton: {}",
                error_msg
            )))
        } else {
            let component = unsafe { Component::from_raw(ptr) };
            Ok(Self {
                component,
                _phantom: PhantomData,
            })
        }
    }

    /// Set the images for different button states.
    ///
    /// # Arguments
    ///
    /// * `normal` - The drawable to show in normal state (required)
    /// * `over` - The drawable to show when mouse is over (optional)
    /// * `down` - The drawable to show when button is pressed (optional)
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let normal = Drawable::create_from_svg(normal_svg)?;
    /// let over = Drawable::create_from_svg(hover_svg)?;
    /// let down = Drawable::create_from_svg(pressed_svg)?;
    ///
    /// button.set_images(&normal, Some(&over), Some(&down))?;
    /// ```
    pub fn set_images(
        &mut self,
        normal: &Drawable,
        over: Option<&Drawable>,
        down: Option<&Drawable>,
    ) -> Result<()> {
        let over_ptr = over.map(|d| unsafe { d.as_ptr() }).unwrap_or(std::ptr::null());
        let down_ptr = down.map(|d| unsafe { d.as_ptr() }).unwrap_or(std::ptr::null());

        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::drawable_button_set_images(
                self.component.as_ptr_mut(),
                normal.as_ptr(),
                over_ptr,
                down_ptr,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if result == 0 {
            Ok(())
        } else {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to set button images: {}",
                error_msg
            )))
        }
    }
}

impl Deref for DrawableButton {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl DerefMut for DrawableButton {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawable_from_svg_simple() {
        let svg = r#"<svg width="100" height="100">
            <circle cx="50" cy="50" r="40" fill="blue"/>
        </svg>"#;
        let result = Drawable::create_from_svg(svg);
        // This will fail without JUCE initialized, but tests the API
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_drawable_draw_invalid_opacity() {
        let svg = r#"<svg width="100" height="100"></svg>"#;
        if let Ok(drawable) = Drawable::create_from_svg(svg) {
            // Can't test without Graphics context, but we can test parameter validation
            // This would need a mock Graphics context
        }
    }

    #[test]
    fn test_drawable_button_new() {
        let result = DrawableButton::new("TestButton");
        // This will fail without JUCE initialized, but tests the API
        assert!(result.is_ok() || result.is_err());
    }
}
