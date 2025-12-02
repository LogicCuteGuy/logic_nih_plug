//! JUCE Font wrapper for text rendering.
//!
//! This module provides a safe Rust wrapper around JUCE's Font class,
//! which describes text rendering properties like typeface, size, and style.
//!
//! # Thread Safety
//!
//! Font objects can be safely used across threads as they are simple value types.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::drawing::Font;
//!
//! // Create a font with a specific size
//! let font = Font::new(14.0)?;
//!
//! // Create a font with a specific typeface
//! let arial = Font::with_typeface("Arial", 16.0)?;
//!
//! // Set font styles
//! let mut bold_font = Font::new(14.0)?;
//! bold_font.set_bold(true)?;
//! bold_font.set_italic(true)?;
//!
//! // Measure text
//! let width = font.get_string_width("Hello, World!")?;
//! let height = font.get_height()?;
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;

/// A font for text rendering.
///
/// This struct wraps a JUCE Font object, providing methods for font
/// creation, style manipulation, and text measurement. Fonts describe
/// how text should be rendered, including typeface, size, and style attributes.
///
/// # Thread Safety
///
/// Unlike most JUCE GUI types, Font is a simple value type and can be
/// safely used across threads.
pub struct Font {
    ptr: *mut ffi::JuceFont,
    _phantom: PhantomData<*mut ()>, // For consistency with other types
}

impl Font {
    /// Create a new font with the specified size.
    ///
    /// # Arguments
    ///
    /// * `size` - Font size in points
    ///
    /// # Returns
    ///
    /// Returns a new Font instance with the default typeface.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let font = Font::new(14.0)?;
    /// ```
    pub fn new(size: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_font(
                size,
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
                "Failed to create font: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a new font with a specific typeface and size.
    ///
    /// # Arguments
    ///
    /// * `typeface` - Name of the typeface (e.g., "Arial", "Times New Roman")
    /// * `size` - Font size in points
    ///
    /// # Returns
    ///
    /// Returns a new Font instance with the specified typeface, or an error
    /// if the typeface is not available.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let arial = Font::with_typeface("Arial", 16.0)?;
    /// let times = Font::with_typeface("Times New Roman", 12.0)?;
    /// ```
    pub fn with_typeface(typeface: &str, size: f32) -> Result<Self> {
        let typeface_bytes = typeface.as_bytes();
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_font_with_typeface(
                typeface_bytes.as_ptr(),
                typeface_bytes.len(),
                size,
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
                "Failed to create font with typeface '{}': {}",
                typeface, error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Set whether the font is bold.
    ///
    /// # Arguments
    ///
    /// * `bold` - true for bold, false for normal weight
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut font = Font::new(14.0)?;
    /// font.set_bold(true)?;
    /// ```
    pub fn set_bold(&mut self, bold: bool) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::font_set_bold(
                self.ptr,
                bold,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if result != 0 {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to set font bold: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Set whether the font is italic.
    ///
    /// # Arguments
    ///
    /// * `italic` - true for italic, false for normal style
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut font = Font::new(14.0)?;
    /// font.set_italic(true)?;
    /// ```
    pub fn set_italic(&mut self, italic: bool) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::font_set_italic(
                self.ptr,
                italic,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if result != 0 {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to set font italic: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Set whether the font is underlined.
    ///
    /// # Arguments
    ///
    /// * `underline` - true for underlined, false for no underline
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut font = Font::new(14.0)?;
    /// font.set_underline(true)?;
    /// ```
    pub fn set_underline(&mut self, underline: bool) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::font_set_underline(
                self.ptr,
                underline,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };

        if result != 0 {
            let error_msg = String::from_utf8_lossy(
                &error_buffer
                    .iter()
                    .map(|&b| b as u8)
                    .take_while(|&b| b != 0)
                    .collect::<Vec<u8>>(),
            )
            .to_string();
            Err(JuceError::OperationFailed(format!(
                "Failed to set font underline: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Get the width of a string when rendered with this font.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to measure
    ///
    /// # Returns
    ///
    /// Returns the width in pixels, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let font = Font::new(14.0)?;
    /// let width = font.get_string_width("Hello, World!")?;
    /// println!("Text width: {} pixels", width);
    /// ```
    pub fn get_string_width(&self, text: &str) -> Result<i32> {
        let text_bytes = text.as_bytes();
        let mut error_buffer = vec![0i8; 256];
        let width = unsafe {
            ffi::font_get_string_width(
                self.ptr,
                text_bytes.as_ptr(),
                text_bytes.len(),
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
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
                "Failed to get string width: {}",
                error_msg
            )))
        } else {
            Ok(width)
        }
    }

    /// Get the height of this font.
    ///
    /// # Returns
    ///
    /// Returns the height in pixels, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let font = Font::new(14.0)?;
    /// let height = font.get_height()?;
    /// println!("Font height: {} pixels", height);
    /// ```
    pub fn get_height(&self) -> Result<i32> {
        let mut error_buffer = vec![0i8; 256];
        let height = unsafe {
            ffi::font_get_height(
                self.ptr,
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
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
                "Failed to get font height: {}",
                error_msg
            )))
        } else {
            Ok(height)
        }
    }

    /// Find all available typeface names on the system.
    ///
    /// # Returns
    ///
    /// Returns a vector of typeface names, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let typefaces = Font::find_all_typeface_names()?;
    /// for name in typefaces {
    ///     println!("Available font: {}", name);
    /// }
    /// ```
    pub fn find_all_typeface_names() -> Result<Vec<String>> {
        // First, get the count of typefaces
        let count = unsafe { ffi::font_get_typeface_count() };
        
        if count < 0 {
            return Err(JuceError::OperationFailed(
                "Failed to get typeface count".to_string()
            ));
        }

        let mut typefaces = Vec::new();
        let mut buffer = vec![0u8; 256];

        for i in 0..count {
            let len = unsafe {
                ffi::font_get_typeface_name(
                    i,
                    buffer.as_mut_ptr(),
                    buffer.len(),
                )
            };

            if len > 0 {
                let name = String::from_utf8_lossy(&buffer[..len as usize]).to_string();
                typefaces.push(name);
            }
        }

        Ok(typefaces)
    }

    /// Get a raw pointer to the underlying JUCE Font object.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the returned pointer is only valid
    /// as long as this Font instance exists. The caller must ensure
    /// the pointer is not used after this Font is dropped.
    pub(crate) unsafe fn as_ptr(&self) -> *const ffi::JuceFont {
        self.ptr
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_font(self.ptr);
        }
    }
}

// Font is a simple value type and can be safely sent across threads
unsafe impl Send for Font {}
unsafe impl Sync for Font {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_new() {
        let font = Font::new(14.0);
        assert!(font.is_ok());
    }

    #[test]
    fn test_font_with_typeface() {
        // Use a common system font that should be available
        let font = Font::with_typeface("Arial", 16.0);
        // This might fail if Arial is not available, so we just check it doesn't panic
        let _ = font;
    }

    #[test]
    fn test_font_set_bold() {
        let mut font = Font::new(14.0).unwrap();
        let result = font.set_bold(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_font_set_italic() {
        let mut font = Font::new(14.0).unwrap();
        let result = font.set_italic(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_font_set_underline() {
        let mut font = Font::new(14.0).unwrap();
        let result = font.set_underline(true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_font_get_string_width() {
        let font = Font::new(14.0).unwrap();
        let width = font.get_string_width("Hello");
        assert!(width.is_ok());
        if let Ok(w) = width {
            assert!(w > 0);
        }
    }

    #[test]
    fn test_font_get_height() {
        let font = Font::new(14.0).unwrap();
        let height = font.get_height();
        assert!(height.is_ok());
        if let Ok(h) = height {
            assert!(h > 0);
        }
    }

    #[test]
    fn test_find_all_typeface_names() {
        let typefaces = Font::find_all_typeface_names();
        assert!(typefaces.is_ok());
        if let Ok(names) = typefaces {
            // Should have at least some fonts on any system
            assert!(!names.is_empty());
        }
    }
}
