//! JUCE Image wrapper for image loading, saving, and manipulation.
//!
//! This module provides a safe Rust wrapper around JUCE's Image class,
//! which represents pixel data and provides image manipulation methods.
//!
//! # Thread Safety
//!
//! Image objects can be safely used across threads as they are value types
//! with internal reference counting.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::drawing::{Image, ImageFormat};
//! use std::path::Path;
//!
//! // Create a new image
//! let image = Image::new(ImageFormat::ARGB, 800, 600)?;
//!
//! // Load an image from file
//! let loaded = Image::load_from_file(Path::new("image.png"))?;
//!
//! // Save an image to file
//! loaded.save_to_file(Path::new("output.png"))?;
//!
//! // Apply blur effect
//! let mut blurred = loaded.clone();
//! blurred.apply_blur(5.0)?;
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use crate::graphics::Graphics;
use std::marker::PhantomData;
use std::path::Path;

/// Image pixel format.
///
/// This enum specifies the pixel format for image data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// RGB format (24-bit, no alpha channel)
    RGB,
    /// ARGB format (32-bit with alpha channel)
    ARGB,
    /// Single channel (8-bit grayscale)
    SingleChannel,
}

impl ImageFormat {
    /// Convert to JUCE format constant.
    fn to_juce_format(self) -> i32 {
        match self {
            ImageFormat::RGB => 1,
            ImageFormat::ARGB => 2,
            ImageFormat::SingleChannel => 3,
        }
    }
}

/// An image that can be loaded, saved, and manipulated.
///
/// This struct wraps a JUCE Image object, providing methods for image
/// creation, loading, saving, and manipulation. Images use internal
/// reference counting, so cloning is cheap.
///
/// # Thread Safety
///
/// Unlike most JUCE GUI types, Image is a value type with internal
/// reference counting and can be safely used across threads.
pub struct Image {
    ptr: *mut ffi::JuceImage,
    _phantom: PhantomData<*mut ()>,
}

impl Image {
    /// Create a new image with the specified format and dimensions.
    ///
    /// # Arguments
    ///
    /// * `format` - The pixel format for the image
    /// * `width` - Width of the image in pixels
    /// * `height` - Height of the image in pixels
    ///
    /// # Returns
    ///
    /// Returns a new Image instance, or an error if creation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let image = Image::new(ImageFormat::ARGB, 800, 600)?;
    /// ```
    pub fn new(format: ImageFormat, width: i32, height: i32) -> Result<Self> {
        if width <= 0 || height <= 0 {
            return Err(JuceError::InvalidParameter(format!(
                "Image dimensions must be positive: {}x{}",
                width, height
            )));
        }

        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_image(
                format.to_juce_format(),
                width,
                height,
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
                "Failed to create image: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Load an image from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file
    ///
    /// # Returns
    ///
    /// Returns a new Image instance with the loaded image data, or an error
    /// if the file cannot be loaded.
    ///
    /// # Supported Formats
    ///
    /// JUCE supports common image formats including PNG, JPEG, GIF, and BMP.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let image = Image::load_from_file(Path::new("photo.jpg"))?;
    /// ```
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let path_str = path
            .to_str()
            .ok_or_else(|| JuceError::InvalidParameter("Invalid file path".to_string()))?;
        let path_bytes = path_str.as_bytes();

        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::load_image_from_file(
                path_bytes.as_ptr(),
                path_bytes.len(),
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
                "Failed to load image from '{}': {}",
                path_str, error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Save the image to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the image should be saved
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the file cannot be saved.
    ///
    /// # Format Detection
    ///
    /// The file format is determined by the file extension. Supported formats
    /// include PNG, JPEG, GIF, and BMP.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// image.save_to_file(Path::new("output.png"))?;
    /// ```
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let path_str = path
            .to_str()
            .ok_or_else(|| JuceError::InvalidParameter("Invalid file path".to_string()))?;
        let path_bytes = path_str.as_bytes();

        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::save_image_to_file(
                self.ptr,
                path_bytes.as_ptr(),
                path_bytes.len(),
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
                "Failed to save image to '{}': {}",
                path_str, error_msg
            )))
        }
    }

    /// Get a graphics context for drawing to this image.
    ///
    /// # Returns
    ///
    /// Returns a Graphics context that can be used to draw on the image.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut image = Image::new(ImageFormat::ARGB, 800, 600)?;
    /// let mut g = image.get_graphics_context()?;
    /// g.fill_rect(0, 0, 800, 600);
    /// ```
    pub fn get_graphics_context(&mut self) -> Result<Graphics<'_>> {
        let mut error_buffer = vec![0i8; 256];
        let graphics_ptr = unsafe {
            ffi::image_get_graphics_context(
                self.ptr,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if graphics_ptr.is_null() {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to get graphics context: {}",
                error_msg
            )))
        } else {
            Ok(Graphics::from_ptr(graphics_ptr))
        }
    }

    /// Apply a blur effect to the image.
    ///
    /// # Arguments
    ///
    /// * `radius` - Blur radius in pixels (larger values = more blur)
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut image = Image::load_from_file(Path::new("photo.jpg"))?;
    /// image.apply_blur(5.0)?;
    /// ```
    pub fn apply_blur(&mut self, radius: f32) -> Result<()> {
        if radius < 0.0 {
            return Err(JuceError::InvalidParameter(format!(
                "Blur radius must be non-negative: {}",
                radius
            )));
        }

        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::image_apply_blur(
                self.ptr,
                radius,
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
                "Failed to apply blur: {}",
                error_msg
            )))
        }
    }

    /// Get the width of the image in pixels.
    ///
    /// # Returns
    ///
    /// Returns the image width, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let width = image.get_width()?;
    /// ```
    pub fn get_width(&self) -> Result<i32> {
        let mut error_buffer = vec![0i8; 256];
        let width = unsafe {
            ffi::image_get_width(self.ptr, error_buffer.as_mut_ptr(), error_buffer.len())
        };

        if width < 0 {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to get image width: {}",
                error_msg
            )))
        } else {
            Ok(width)
        }
    }

    /// Get the height of the image in pixels.
    ///
    /// # Returns
    ///
    /// Returns the image height, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let height = image.get_height()?;
    /// ```
    pub fn get_height(&self) -> Result<i32> {
        let mut error_buffer = vec![0i8; 256];
        let height = unsafe {
            ffi::image_get_height(self.ptr, error_buffer.as_mut_ptr(), error_buffer.len())
        };

        if height < 0 {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to get image height: {}",
                error_msg
            )))
        } else {
            Ok(height)
        }
    }

    /// Get a raw pointer to the underlying JUCE Image object.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the returned pointer is only valid
    /// as long as this Image instance exists. The caller must ensure
    /// the pointer is not used after this Image is dropped.
    pub(crate) unsafe fn as_ptr(&self) -> *const ffi::JuceImage {
        self.ptr
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_image(self.ptr);
        }
    }
}

// Image is a value type with internal reference counting and can be safely sent across threads
unsafe impl Send for Image {}
unsafe impl Sync for Image {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_new() {
        let image = Image::new(ImageFormat::ARGB, 100, 100);
        assert!(image.is_ok());
    }

    #[test]
    fn test_image_new_invalid_dimensions() {
        let image = Image::new(ImageFormat::ARGB, -100, 100);
        assert!(image.is_err());

        let image = Image::new(ImageFormat::ARGB, 100, 0);
        assert!(image.is_err());
    }

    #[test]
    fn test_image_dimensions() {
        let image = Image::new(ImageFormat::ARGB, 200, 150).unwrap();
        assert_eq!(image.get_width().unwrap(), 200);
        assert_eq!(image.get_height().unwrap(), 150);
    }

    #[test]
    fn test_image_blur_invalid_radius() {
        let mut image = Image::new(ImageFormat::ARGB, 100, 100).unwrap();
        let result = image.apply_blur(-1.0);
        assert!(result.is_err());
    }
}
