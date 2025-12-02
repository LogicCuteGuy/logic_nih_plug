//! JUCE AffineTransform wrapper for 2D transformations.
//!
//! This module provides a safe Rust wrapper around JUCE's AffineTransform class,
//! which describes 2D transformations (translation, rotation, scaling).
//!
//! # Thread Safety
//!
//! AffineTransform objects can be safely used across threads as they are value types.

use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;

/// A 2D affine transformation.
///
/// This struct wraps a JUCE AffineTransform object, providing methods for
/// creating and composing 2D transformations including translation, rotation,
/// and scaling.
///
/// # Thread Safety
///
/// Unlike most JUCE GUI types, AffineTransform is a value type and can be
/// safely used across threads.
pub struct AffineTransform {
    ptr: *mut ffi::JuceAffineTransform,
    _phantom: PhantomData<*mut ()>,
}

impl AffineTransform {
    /// Create an identity transformation (no transformation).
    ///
    /// # Returns
    ///
    /// Returns a new AffineTransform instance representing the identity transformation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let transform = AffineTransform::identity()?;
    /// ```
    pub fn identity() -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_affine_transform_identity(
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
                "Failed to create identity transform: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a translation transformation.
    ///
    /// # Arguments
    ///
    /// * `dx` - Translation distance in X direction
    /// * `dy` - Translation distance in Y direction
    ///
    /// # Returns
    ///
    /// Returns a new AffineTransform instance representing the translation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let transform = AffineTransform::translation(100.0, 50.0)?;
    /// ```
    pub fn translation(dx: f32, dy: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_affine_transform_translation(
                dx,
                dy,
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
                "Failed to create translation transform: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a rotation transformation.
    ///
    /// # Arguments
    ///
    /// * `angle_radians` - Rotation angle in radians
    ///
    /// # Returns
    ///
    /// Returns a new AffineTransform instance representing the rotation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::f32::consts::PI;
    /// let transform = AffineTransform::rotation(PI / 4.0)?; // 45 degrees
    /// ```
    pub fn rotation(angle_radians: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_affine_transform_rotation(
                angle_radians,
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
                "Failed to create rotation transform: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Create a scaling transformation.
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor in X direction
    /// * `sy` - Scale factor in Y direction
    ///
    /// # Returns
    ///
    /// Returns a new AffineTransform instance representing the scaling.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let transform = AffineTransform::scale(2.0, 2.0)?; // Double size
    /// ```
    pub fn scale(sx: f32, sy: f32) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::create_affine_transform_scale(
                sx,
                sy,
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
                "Failed to create scale transform: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Compose this transformation with another (this followed by other).
    ///
    /// Returns a new transformation that represents applying this transformation
    /// first, then applying the other transformation.
    ///
    /// # Arguments
    ///
    /// * `other` - The transformation to apply after this one
    ///
    /// # Returns
    ///
    /// Returns a new AffineTransform instance representing the composed transformation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let translate = AffineTransform::translation(100.0, 50.0)?;
    /// let rotate = AffineTransform::rotation(std::f32::consts::PI / 4.0)?;
    /// let combined = translate.followed_by(&rotate)?;
    /// ```
    pub fn followed_by(&self, other: &AffineTransform) -> Result<Self> {
        let mut error_buffer = vec![0i8; 256];
        let ptr = unsafe {
            ffi::affine_transform_followed_by(
                self.ptr,
                other.ptr,
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
                "Failed to compose transforms: {}",
                error_msg
            )))
        } else {
            Ok(Self {
                ptr,
                _phantom: PhantomData,
            })
        }
    }

    /// Get a raw pointer to the underlying JUCE AffineTransform object.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the returned pointer is only valid
    /// as long as this AffineTransform instance exists. The caller must ensure
    /// the pointer is not used after this AffineTransform is dropped.
    pub(crate) unsafe fn as_ptr(&self) -> *const ffi::JuceAffineTransform {
        self.ptr
    }
}

impl Drop for AffineTransform {
    fn drop(&mut self) {
        unsafe {
            ffi::delete_affine_transform(self.ptr);
        }
    }
}

// AffineTransform is a value type and can be safely sent across threads
unsafe impl Send for AffineTransform {}
unsafe impl Sync for AffineTransform {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_affine_transform_identity() {
        let transform = AffineTransform::identity();
        assert!(transform.is_ok());
    }

    #[test]
    fn test_affine_transform_translation() {
        let transform = AffineTransform::translation(100.0, 50.0);
        assert!(transform.is_ok());
    }

    #[test]
    fn test_affine_transform_rotation() {
        let transform = AffineTransform::rotation(std::f32::consts::PI / 4.0);
        assert!(transform.is_ok());
    }

    #[test]
    fn test_affine_transform_scale() {
        let transform = AffineTransform::scale(2.0, 2.0);
        assert!(transform.is_ok());
    }

    #[test]
    fn test_affine_transform_followed_by() {
        let translate = AffineTransform::translation(100.0, 50.0);
        assert!(translate.is_ok());
        let translate = translate.unwrap();

        let rotate = AffineTransform::rotation(std::f32::consts::PI / 4.0);
        assert!(rotate.is_ok());
        let rotate = rotate.unwrap();

        let combined = translate.followed_by(&rotate);
        assert!(combined.is_ok());
    }
}
