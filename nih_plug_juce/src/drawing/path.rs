//! JUCE Path wrapper for vector graphics.
//!
//! This module provides a safe Rust wrapper around JUCE's Path class,
//! which describes vector graphics paths for drawing complex shapes.
//!
//! # Thread Safety
//!
//! Path objects can be safely used across threads as they are value types.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::drawing::Path;
//!
//! // Create a new path
//! let mut path = Path::new()?;
//!
//! // Draw a triangle
//! path.start_new_sub_path(100.0, 50.0)?;
//! path.line_to(150.0, 150.0)?;
//! path.line_to(50.0, 150.0)?;
//! path.close_sub_path()?;
//!
//! // Add shapes
//! path.add_rectangle(200.0, 200.0, 100.0, 50.0)?;
//! path.add_ellipse(350.0, 200.0, 80.0, 80.0)?;
//! ```

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;

/// A vector graphics path for drawing complex shapes.
///
/// This struct wraps a JUCE Path object, providing methods for creating
/// and manipulating vector graphics paths. Paths can contain lines, curves,
/// and shapes, and can be stroked or filled using a Graphics context.
///
/// # Thread Safety
///
/// Unlike most JUCE GUI types, Path is a value type and can be
/// safely used across threads.
pub struct Path {
    ptr: *mut ffi::JucePath,
    _phantom: PhantomData<*mut ()>,
}

impl Path {
    /// Create a new empty path.
    ///
    /// # Returns
    ///
    /// Returns a new Path instance.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let path = Path::new()?;
    /// ```
    pub fn new() -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_path(
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
                "Failed to create path: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Start a new sub-path at the specified position.
    ///
    /// This begins a new disconnected section of the path. Any subsequent
    /// line or curve operations will extend from this point.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the starting point
    /// * `y` - Y coordinate of the starting point
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new()?;
    /// path.start_new_sub_path(100.0, 100.0)?;
    /// ```
    pub fn start_new_sub_path(&mut self, x: f32, y: f32) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_start_new_sub_path(
                self.ptr,
                x,
                y,
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
                "Failed to start new sub-path: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Add a line from the current position to the specified point.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the end point
    /// * `y` - Y coordinate of the end point
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new()?;
    /// path.start_new_sub_path(0.0, 0.0)?;
    /// path.line_to(100.0, 100.0)?;
    /// ```
    pub fn line_to(&mut self, x: f32, y: f32) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_line_to(
                self.ptr,
                x,
                y,
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
                "Failed to add line: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Add a quadratic bezier curve from the current position.
    ///
    /// # Arguments
    ///
    /// * `cx` - X coordinate of the control point
    /// * `cy` - Y coordinate of the control point
    /// * `x` - X coordinate of the end point
    /// * `y` - Y coordinate of the end point
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new()?;
    /// path.start_new_sub_path(0.0, 0.0)?;
    /// path.quadratic_to(50.0, 100.0, 100.0, 0.0)?;
    /// ```
    pub fn quadratic_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_quadratic_to(
                self.ptr,
                cx,
                cy,
                x,
                y,
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
                "Failed to add quadratic curve: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Add a cubic bezier curve from the current position.
    ///
    /// # Arguments
    ///
    /// * `cx1` - X coordinate of the first control point
    /// * `cy1` - Y coordinate of the first control point
    /// * `cx2` - X coordinate of the second control point
    /// * `cy2` - Y coordinate of the second control point
    /// * `x` - X coordinate of the end point
    /// * `y` - Y coordinate of the end point
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new()?;
    /// path.start_new_sub_path(0.0, 0.0)?;
    /// path.cubic_to(33.0, 100.0, 66.0, 100.0, 100.0, 0.0)?;
    /// ```
    pub fn cubic_to(&mut self, cx1: f32, cy1: f32, cx2: f32, cy2: f32, x: f32, y: f32) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_cubic_to(
                self.ptr,
                cx1,
                cy1,
                cx2,
                cy2,
                x,
                y,
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
                "Failed to add cubic curve: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Add a rectangle to the path.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the top-left corner
    /// * `y` - Y coordinate of the top-left corner
    /// * `width` - Width of the rectangle
    /// * `height` - Height of the rectangle
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new()?;
    /// path.add_rectangle(10.0, 10.0, 100.0, 50.0)?;
    /// ```
    pub fn add_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_add_rectangle(
                self.ptr,
                x,
                y,
                width,
                height,
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
                "Failed to add rectangle: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Add an ellipse to the path.
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
    /// let mut path = Path::new()?;
    /// path.add_ellipse(50.0, 50.0, 100.0, 100.0)?;
    /// ```
    pub fn add_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_add_ellipse(
                self.ptr,
                x,
                y,
                width,
                height,
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
                "Failed to add ellipse: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Add an arc to the path.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the bounding rectangle
    /// * `y` - Y coordinate of the bounding rectangle
    /// * `width` - Width of the bounding rectangle
    /// * `height` - Height of the bounding rectangle
    /// * `start_angle` - Starting angle in radians
    /// * `end_angle` - Ending angle in radians
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::f32::consts::PI;
    /// let mut path = Path::new()?;
    /// // Draw a quarter circle
    /// path.add_arc(50.0, 50.0, 100.0, 100.0, 0.0, PI / 2.0)?;
    /// ```
    pub fn add_arc(&mut self, x: f32, y: f32, width: f32, height: f32, 
                   start_angle: f32, end_angle: f32) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_add_arc(
                self.ptr,
                x,
                y,
                width,
                height,
                start_angle,
                end_angle,
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
                "Failed to add arc: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Close the current sub-path by adding a line back to its starting point.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new()?;
    /// path.start_new_sub_path(0.0, 0.0)?;
    /// path.line_to(100.0, 0.0)?;
    /// path.line_to(50.0, 100.0)?;
    /// path.close_sub_path()?; // Completes the triangle
    /// ```
    pub fn close_sub_path(&mut self) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_close_sub_path(
                self.ptr,
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
                "Failed to close sub-path: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Apply a transformation to the path.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transformation to apply
    ///
    /// # Returns
    ///
    /// Returns Ok(()) on success, or an error if the operation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use nih_plug_juce::drawing::{Path, AffineTransform};
    ///
    /// let mut path = Path::new()?;
    /// path.add_rectangle(0.0, 0.0, 100.0, 100.0)?;
    ///
    /// let transform = AffineTransform::rotation(std::f32::consts::PI / 4.0)?;
    /// path.apply_transform(&transform)?;
    /// ```
    pub fn apply_transform(&mut self, transform: &crate::drawing::AffineTransform) -> Result<()> {
        let mut error_buffer = vec![0i8; 256];
        let result = unsafe {
            ffi::path_apply_transform(
                self.ptr,
                transform.as_ptr(),
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
                "Failed to apply transform: {}",
                error_msg
            )))
        } else {
            Ok(())
        }
    }

    /// Get a raw pointer to the underlying JUCE Path object.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the returned pointer is only valid
    /// as long as this Path instance exists. The caller must ensure
    /// the pointer is not used after this Path is dropped.
    pub(crate) unsafe fn as_ptr(&self) -> *const ffi::JucePath {
        self.ptr
    }
}

impl Drop for Path {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_path(self.ptr);
        }
    }
}

// Path is a value type and can be safely sent across threads
unsafe impl Send for Path {}
unsafe impl Sync for Path {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_new() {
        let path = Path::new();
        assert!(path.is_ok());
    }

    #[test]
    fn test_path_start_new_sub_path() {
        let mut path = Path::new().unwrap();
        let result = path.start_new_sub_path(100.0, 100.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_line_to() {
        let mut path = Path::new().unwrap();
        path.start_new_sub_path(0.0, 0.0).unwrap();
        let result = path.line_to(100.0, 100.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_quadratic_to() {
        let mut path = Path::new().unwrap();
        path.start_new_sub_path(0.0, 0.0).unwrap();
        let result = path.quadratic_to(50.0, 100.0, 100.0, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_cubic_to() {
        let mut path = Path::new().unwrap();
        path.start_new_sub_path(0.0, 0.0).unwrap();
        let result = path.cubic_to(33.0, 100.0, 66.0, 100.0, 100.0, 0.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_add_rectangle() {
        let mut path = Path::new().unwrap();
        let result = path.add_rectangle(10.0, 10.0, 100.0, 50.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_add_ellipse() {
        let mut path = Path::new().unwrap();
        let result = path.add_ellipse(50.0, 50.0, 100.0, 100.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_add_arc() {
        let mut path = Path::new().unwrap();
        let result = path.add_arc(50.0, 50.0, 100.0, 100.0, 0.0, std::f32::consts::PI / 2.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_close_sub_path() {
        let mut path = Path::new().unwrap();
        path.start_new_sub_path(0.0, 0.0).unwrap();
        path.line_to(100.0, 0.0).unwrap();
        path.line_to(50.0, 100.0).unwrap();
        let result = path.close_sub_path();
        assert!(result.is_ok());
    }
}
