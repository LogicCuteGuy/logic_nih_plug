//! JUCE Colour wrapper for color manipulation.
//!
//! This module provides a safe Rust wrapper around JUCE's Colour class,
//! which represents RGBA colors and provides color manipulation methods.
//!
//! # Thread Safety
//!
//! Colour objects can be safely used across threads as they are simple value types.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::drawing::Colour;
//!
//! // Create a color from RGBA values
//! let red = Colour::from_rgba(255, 0, 0, 255);
//!
//! // Create a color from hex string
//! let blue = Colour::from_hex("#0000FF").unwrap();
//!
//! // Manipulate colors
//! let lighter_red = red.brighter(0.2);
//! let semi_transparent = red.with_alpha(0.5);
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;

/// A color represented in RGBA format.
///
/// This struct wraps a JUCE Colour object, providing methods for color
/// creation, conversion, and manipulation. Colors are represented internally
/// as RGBA values with 8 bits per channel.
///
/// # Thread Safety
///
/// Unlike most JUCE GUI types, Colour is a simple value type and can be
/// safely used across threads.
pub struct Colour {
    ptr: *mut ffi::JuceColour,
    _phantom: PhantomData<*mut ()>, // For consistency with other types
}

impl Colour {
    /// Create a new color from RGBA values.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    /// * `a` - Alpha component (0-255, where 255 is fully opaque)
    ///
    /// # Returns
    ///
    /// Returns a new Colour instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_rgba(255, 0, 0, 255);
    /// let semi_transparent_blue = Colour::from_rgba(0, 0, 255, 128);
    /// ```
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_colour_rgba(
                r,
                g,
                b,
                a,
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
                "Failed to create colour: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a new color from RGB values with full opacity.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    ///
    /// # Returns
    ///
    /// Returns a new Colour instance with alpha set to 255 (fully opaque).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let green = Colour::from_rgb(0, 255, 0);
    /// ```
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Result<Self> {
        Self::from_rgba(r, g, b, 255)
    }

    /// Create a color from a hexadecimal string.
    ///
    /// # Arguments
    ///
    /// * `hex` - Hexadecimal color string (e.g., "#FF0000", "FF0000", "#RGB", "RRGGBB")
    ///
    /// # Returns
    ///
    /// Returns a new Colour instance, or an error if the hex string is invalid.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_hex("#FF0000")?;
    /// let blue = Colour::from_hex("0000FF")?;
    /// let green = Colour::from_hex("#0F0")?;
    /// ```
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex_bytes = hex.as_bytes();
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_colour_from_hex(
                hex_bytes.as_ptr(),
                hex_bytes.len(),
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
            Err(JuceError::InvalidParameter(format!(
                "Invalid hex color string '{}': {}",
                hex, error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Convert the color to a hexadecimal string.
    ///
    /// # Returns
    ///
    /// Returns a hexadecimal string representation of the color in the format "RRGGBBAA".
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_rgb(255, 0, 0)?;
    /// assert_eq!(red.to_hex(), "FF0000FF");
    /// ```
    pub fn to_hex(&self) -> String {
        let mut buffer = vec![0u8; 16];
        let len = unsafe {
            ffi::colour_to_hex(self.ptr, buffer.as_mut_ptr(), buffer.len())
        };

        String::from_utf8_lossy(&buffer[..len]).to_string()
    }

    /// Create a new color with a different alpha value.
    ///
    /// # Arguments
    ///
    /// * `alpha` - New alpha value (0.0 = fully transparent, 1.0 = fully opaque)
    ///
    /// # Returns
    ///
    /// Returns a new Colour instance with the modified alpha value.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_rgb(255, 0, 0)?;
    /// let semi_transparent_red = red.with_alpha(0.5)?;
    /// ```
    pub fn with_alpha(&self, alpha: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::colour_with_alpha(
                self.ptr,
                alpha,
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
                "Failed to create colour with alpha: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a brighter version of this color.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to brighten (0.0 = no change, 1.0 = maximum brightening)
    ///
    /// # Returns
    ///
    /// Returns a new Colour instance that is brighter than the original.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_rgb(128, 0, 0)?;
    /// let brighter_red = red.brighter(0.5)?;
    /// ```
    pub fn brighter(&self, amount: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::colour_brighter(
                self.ptr,
                amount,
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
                "Failed to create brighter colour: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a darker version of this color.
    ///
    /// # Arguments
    ///
    /// * `amount` - Amount to darken (0.0 = no change, 1.0 = maximum darkening)
    ///
    /// # Returns
    ///
    /// Returns a new Colour instance that is darker than the original.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_rgb(255, 0, 0)?;
    /// let darker_red = red.darker(0.5)?;
    /// ```
    pub fn darker(&self, amount: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::colour_darker(
                self.ptr,
                amount,
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
                "Failed to create darker colour: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a color that is interpolated between this color and another.
    ///
    /// # Arguments
    ///
    /// * `other` - The other color to interpolate with
    /// * `proportion` - Interpolation amount (0.0 = this color, 1.0 = other color)
    ///
    /// # Returns
    ///
    /// Returns a new Colour instance that is a blend of the two colors.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_rgb(255, 0, 0)?;
    /// let blue = Colour::from_rgb(0, 0, 255)?;
    /// let purple = red.interpolated_with(&blue, 0.5)?;
    /// ```
    pub fn interpolated_with(&self, other: &Colour, proportion: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::colour_interpolated_with(
                self.ptr,
                other.ptr,
                proportion,
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
                "Failed to interpolate colours: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a Colour from a raw pointer.
    ///
    /// # Safety
    ///
    /// This method is unsafe because it takes ownership of the pointer.
    /// The pointer must be a valid JuceColour pointer created by JUCE FFI functions.
    /// The Colour will take ownership and free the pointer when dropped.
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::JuceColour) -> Self {
        Self {
            ptr: ptr as *mut ffi::JuceColour,
            _phantom: PhantomData,
        }
    }

    /// Get a raw pointer to the underlying JUCE Colour object.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the returned pointer is only valid
    /// as long as this Colour instance exists. The caller must ensure
    /// the pointer is not used after this Colour is dropped.
    pub(crate) unsafe fn as_ptr(&self) -> *const ffi::JuceColour {
        self.ptr
    }
}

impl Drop for Colour {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_colour(self.ptr);
        }
    }
}

// Colour is a simple value type and can be safely sent across threads
unsafe impl Send for Colour {}
unsafe impl Sync for Colour {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colour_from_rgba() {
        let colour = Colour::from_rgba(255, 0, 0, 255);
        assert!(colour.is_ok());
    }

    #[test]
    fn test_colour_from_rgb() {
        let colour = Colour::from_rgb(0, 255, 0);
        assert!(colour.is_ok());
    }

    #[test]
    fn test_colour_from_hex() {
        let colour = Colour::from_hex("#FF0000");
        assert!(colour.is_ok());
    }

    #[test]
    fn test_colour_to_hex() {
        let colour = Colour::from_rgb(255, 0, 0).unwrap();
        let hex = colour.to_hex();
        assert!(hex.starts_with("FF0000"));
    }

    #[test]
    fn test_colour_with_alpha() {
        let colour = Colour::from_rgb(255, 0, 0).unwrap();
        let transparent = colour.with_alpha(0.5);
        assert!(transparent.is_ok());
    }

    #[test]
    fn test_colour_brighter() {
        let colour = Colour::from_rgb(128, 0, 0).unwrap();
        let brighter = colour.brighter(0.5);
        assert!(brighter.is_ok());
    }

    #[test]
    fn test_colour_darker() {
        let colour = Colour::from_rgb(255, 0, 0).unwrap();
        let darker = colour.darker(0.5);
        assert!(darker.is_ok());
    }

    #[test]
    fn test_colour_interpolated() {
        let red = Colour::from_rgb(255, 0, 0).unwrap();
        let blue = Colour::from_rgb(0, 0, 255).unwrap();
        let purple = red.interpolated_with(&blue, 0.5);
        assert!(purple.is_ok());
    }
}
