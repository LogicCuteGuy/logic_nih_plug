//! Image loading and rendering.
//!
//! This module provides functionality for loading and rendering images in various formats.
//!
//! # Supported Formats
//!
//! - PNG
//! - JPEG
//! - GIF
//!
//! # Examples
//!
//! ```no_run
//! use logic_nih_plug_graphics::images::Image;
//!
//! // Load an image from a file
//! let image = Image::load("path/to/image.png").unwrap();
//!
//! // Get image dimensions
//! let (width, height) = image.dimensions();
//!
//! // Access pixel data
//! let pixels = image.as_rgba8();
//! ```

use crate::error::GraphicsError;
use std::path::Path;

/// Represents a loaded image with RGBA pixel data.
///
/// Images are stored in RGBA format with 8 bits per channel.
/// The pixel data is stored in row-major order.
#[derive(Debug, Clone)]
pub struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Image {
    /// Loads an image from a file.
    ///
    /// Supports PNG, JPEG, and GIF formats. The format is automatically
    /// detected from the file contents.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the image file
    ///
    /// # Returns
    ///
    /// Returns `Ok(Image)` if the image was loaded successfully, or an error
    /// if the file could not be read or the format is unsupported.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_graphics::images::Image;
    ///
    /// let image = Image::load("logo.png").unwrap();
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, GraphicsError> {
        let img = image::open(path).map_err(|e| GraphicsError::ImageLoadError(e.to_string()))?;
        
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();
        
        Ok(Self {
            width,
            height,
            data,
        })
    }
    
    /// Loads an image from raw bytes.
    ///
    /// The format is automatically detected from the byte contents.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw image file data
    ///
    /// # Returns
    ///
    /// Returns `Ok(Image)` if the image was loaded successfully, or an error
    /// if the data is invalid or the format is unsupported.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use logic_nih_plug_graphics::images::Image;
    ///
    /// // Load from embedded bytes
    /// let png_data = include_bytes!("path/to/image.png");
    /// let image = Image::from_bytes(png_data).unwrap();
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, GraphicsError> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| GraphicsError::ImageLoadError(e.to_string()))?;
        
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();
        
        Ok(Self {
            width,
            height,
            data,
        })
    }
    
    /// Creates a new image with the specified dimensions and pixel data.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the image in pixels
    /// * `height` - Height of the image in pixels
    /// * `data` - RGBA pixel data (must be width * height * 4 bytes)
    ///
    /// # Returns
    ///
    /// Returns `Ok(Image)` if the dimensions and data are valid, or an error
    /// if the data length doesn't match the dimensions.
    pub fn from_rgba8(width: u32, height: u32, data: Vec<u8>) -> Result<Self, GraphicsError> {
        let expected_len = (width * height * 4) as usize;
        if data.len() != expected_len {
            return Err(GraphicsError::InvalidImageData {
                expected: expected_len,
                actual: data.len(),
            });
        }
        
        Ok(Self {
            width,
            height,
            data,
        })
    }
    
    /// Returns the dimensions of the image.
    ///
    /// # Returns
    ///
    /// A tuple of (width, height) in pixels.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    /// Returns the width of the image in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }
    
    /// Returns the height of the image in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
    
    /// Returns a reference to the raw RGBA pixel data.
    ///
    /// The data is in row-major order with 4 bytes per pixel (RGBA).
    pub fn as_rgba8(&self) -> &[u8] {
        &self.data
    }
    
    /// Returns a mutable reference to the raw RGBA pixel data.
    ///
    /// The data is in row-major order with 4 bytes per pixel (RGBA).
    pub fn as_rgba8_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
    
    /// Gets the pixel color at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (0 to width-1)
    /// * `y` - Y coordinate (0 to height-1)
    ///
    /// # Returns
    ///
    /// Returns `Some((r, g, b, a))` if the coordinates are valid, or `None` otherwise.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let index = ((y * self.width + x) * 4) as usize;
        Some((
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
            self.data[index + 3],
        ))
    }
    
    /// Sets the pixel color at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (0 to width-1)
    /// * `y` - Y coordinate (0 to height-1)
    /// * `r` - Red component (0-255)
    /// * `g` - Green component (0-255)
    /// * `b` - Blue component (0-255)
    /// * `a` - Alpha component (0-255)
    ///
    /// # Returns
    ///
    /// Returns `true` if the pixel was set, or `false` if the coordinates are out of bounds.
    pub fn set_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8, a: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let index = ((y * self.width + x) * 4) as usize;
        self.data[index] = r;
        self.data[index + 1] = g;
        self.data[index + 2] = b;
        self.data[index + 3] = a;
        true
    }
    
    /// Saves the image to a file.
    ///
    /// The format is determined by the file extension.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the image should be saved
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the image was saved successfully, or an error otherwise.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), GraphicsError> {
        let img = image::RgbaImage::from_raw(self.width, self.height, self.data.clone())
            .ok_or_else(|| GraphicsError::InvalidImageData {
                expected: (self.width * self.height * 4) as usize,
                actual: self.data.len(),
            })?;
        
        img.save(path).map_err(|e| GraphicsError::ImageSaveError(e.to_string()))?;
        Ok(())
    }
}
