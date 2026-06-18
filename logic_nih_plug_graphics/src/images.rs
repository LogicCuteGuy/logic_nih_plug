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
    /// The format is determined by the file extension. For JPEG output
    /// the alpha channel is discarded automatically (JPEG only supports
    /// opaque RGB).
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), GraphicsError> {
        let path = path.as_ref();
        let rgba = image::RgbaImage::from_raw(self.width, self.height, self.data.clone())
            .ok_or_else(|| GraphicsError::InvalidImageData {
                expected: (self.width * self.height * 4) as usize,
                actual: self.data.len(),
            })?;

        // JPEG only supports RGB — convert by dropping alpha.
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "jpg" || ext == "jpeg" {
            let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
            rgb.save(path)
                .map_err(|e| GraphicsError::ImageSaveError(e.to_string()))?;
        } else {
            rgba.save(path)
                .map_err(|e| GraphicsError::ImageSaveError(e.to_string()))?;
        }
        Ok(())
    }

    /// Create a rescaled copy of this image.
    ///
    /// Uses the `image` crate's built-in interpolation. Available filter
    /// modes: [`RescaleFilter::Nearest`] (fast, blocky) and
    /// [`RescaleFilter::Bilinear`] (smooth).
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::images::{Image, RescaleFilter};
    ///
    /// let img = Image::new(4, 4);
    /// let scaled = img.rescaled(8, 8, RescaleFilter::Nearest);
    /// assert_eq!(scaled.dimensions(), (8, 8));
    /// ```
    pub fn rescaled(&self, new_width: u32, new_height: u32, filter: RescaleFilter) -> Self {
        if new_width == self.width && new_height == self.height {
            return self.clone();
        }
        let src = image::RgbaImage::from_raw(self.width, self.height, self.data.clone())
            .expect("image data length always matches dimensions");
        let f = match filter {
            RescaleFilter::Nearest => image::imageops::FilterType::Nearest,
            RescaleFilter::Bilinear => image::imageops::FilterType::CatmullRom,
        };
        let resized = image::imageops::resize(&src, new_width, new_height, f);
        let (w, h) = resized.dimensions();
        Self {
            width: w,
            height: h,
            data: resized.into_raw(),
        }
    }

    /// Apply a convolution kernel to this image and return a new image.
    ///
    /// The kernel is a flat `f32` slice of length `ksize * ksize` stored
    /// in row-major order. The centre of the kernel is at index
    /// `ksize/2 + ksize/2 * ksize`.
    ///
    /// Edge pixels are clamped (repeated at the boundary).
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::images::Image;
    ///
    /// // 3x3 box blur
    /// let kernel = vec![
    ///     1.0/9.0, 1.0/9.0, 1.0/9.0,
    ///     1.0/9.0, 1.0/9.0, 1.0/9.0,
    ///     1.0/9.0, 1.0/9.0, 1.0/9.0,
    /// ];
    /// let img = Image::new(4, 4);
    /// let blurred = img.convolve(&kernel, 3);
    /// assert_eq!(blurred.dimensions(), (4, 4));
    /// ```
    pub fn convolve(&self, kernel: &[f32], ksize: u32) -> Self {
        let mut out = self.clone();
        out.convolve_in_place(kernel, ksize);
        out
    }

    /// Apply a convolution kernel in place.
    pub fn convolve_in_place(&mut self, kernel: &[f32], ksize: u32) {
        let expected = (ksize * ksize) as usize;
        if kernel.len() != expected || ksize == 0 || self.width == 0 || self.height == 0 {
            return;
        }
        let half = (ksize / 2) as i32;
        let w = self.width as i32;
        let h = self.height as i32;
        let orig = self.data.clone();

        for y in 0..self.height {
            for x in 0..self.width {
                let mut r_acc = 0.0f32;
                let mut g_acc = 0.0f32;
                let mut b_acc = 0.0f32;
                let mut a_acc = 0.0f32;

                for ky in 0..ksize as i32 {
                    for kx in 0..ksize as i32 {
                        let sx = (x as i32 + kx - half).max(0).min(w - 1) as u32;
                        let sy = (y as i32 + ky - half).max(0).min(h - 1) as u32;
                        let idx = ((sy * self.width + sx) * 4) as usize;
                        let wt = kernel[(ky * ksize as i32 + kx) as usize];
                        r_acc += orig[idx] as f32 * wt;
                        g_acc += orig[idx + 1] as f32 * wt;
                        b_acc += orig[idx + 2] as f32 * wt;
                        a_acc += orig[idx + 3] as f32 * wt;
                    }
                }

                let idx = ((y * self.width + x) * 4) as usize;
                self.data[idx] = r_acc.round().max(0.0).min(255.0) as u8;
                self.data[idx + 1] = g_acc.round().max(0.0).min(255.0) as u8;
                self.data[idx + 2] = b_acc.round().max(0.0).min(255.0) as u8;
                self.data[idx + 3] = a_acc.round().max(0.0).min(255.0) as u8;
            }
        }
    }
}

/// Creates a new blank (transparent) image.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_graphics::images::Image;
///
/// let img = Image::new(100, 100);
/// assert_eq!(img.dimensions(), (100, 100));
/// assert!(img.as_rgba8().iter().all(|&b| b == 0));
/// ```
impl Image {
    /// Create a new blank (fully transparent) image of the given size.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize],
        }
    }
}

/// Filter to use when rescaling an image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RescaleFilter {
    /// Nearest-neighbour (fast, blocky).
    Nearest,
    /// Bilinear / Catmull-Rom (smooth).
    Bilinear,
}

/// A reusable convolution engine.
///
/// `ImageConvolutionEngine` stores a kernel and its size so that the
/// same filter can be applied to multiple images without re-building the
/// kernel each time.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_graphics::images::{Image, ImageConvolutionEngine};
///
/// let mut engine = ImageConvolutionEngine::new(
///     vec![
///         0.0, 0.0, 0.0,
///         0.0, 1.0, 0.0,
///         0.0, 0.0, 0.0,
///     ],
///     3,
/// ).unwrap();
///
/// // Identity kernel — image should be unchanged.
/// let img = Image::new(8, 8);
/// let out = engine.apply(&img);
/// assert_eq!(out.as_rgba8(), img.as_rgba8());
/// ```
#[derive(Debug, Clone)]
pub struct ImageConvolutionEngine {
    kernel: Vec<f32>,
    ksize: u32,
}

impl ImageConvolutionEngine {
    /// Create a new convolution engine with the given square kernel.
    ///
    /// Returns `None` if the kernel length does not equal `ksize²` or
    /// if `ksize` is zero.
    pub fn new(kernel: Vec<f32>, ksize: u32) -> Option<Self> {
        if ksize == 0 || kernel.len() != (ksize * ksize) as usize {
            return None;
        }
        Some(Self { kernel, ksize })
    }

    /// Apply the stored kernel to `image` and return a new image.
    pub fn apply(&mut self, image: &Image) -> Image {
        image.convolve(&self.kernel, self.ksize)
    }

    /// Kernel size (width = height in cells).
    pub fn ksize(&self) -> u32 {
        self.ksize
    }

    /// Reference to the kernel data.
    pub fn kernel(&self) -> &[f32] {
        &self.kernel
    }

    /// Replace the kernel in place.
    pub fn set_kernel(&mut self, kernel: Vec<f32>, ksize: u32) -> Option<()> {
        if ksize == 0 || kernel.len() != (ksize * ksize) as usize {
            return None;
        }
        self.kernel = kernel;
        self.ksize = ksize;
        Some(())
    }

    /// Create a 3×3 box blur kernel.
    pub fn box_blur_3x3() -> Self {
        let v = 1.0 / 9.0;
        Self {
            kernel: vec![v; 9],
            ksize: 3,
        }
    }

    /// Create a 3×3 sharpen kernel.
    pub fn sharpen_3x3() -> Self {
        Self {
            kernel: vec![
                 0.0, -1.0,  0.0,
                -1.0,  5.0, -1.0,
                 0.0, -1.0,  0.0,
            ],
            ksize: 3,
        }
    }

    /// Create a 3×3 edge-detection (Laplacian) kernel.
    pub fn edge_detect_3x3() -> Self {
        Self {
            kernel: vec![
                -1.0, -1.0, -1.0,
                -1.0,  8.0, -1.0,
                -1.0, -1.0, -1.0,
            ],
            ksize: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_image_is_transparent() {
        let img = Image::new(4, 4);
        assert_eq!(img.dimensions(), (4, 4));
        assert!(img.as_rgba8().iter().all(|&b| b == 0));
    }

    #[test]
    fn rescale_nearest_same_size_is_clone() {
        let img = Image::new(10, 10);
        let scaled = img.rescaled(10, 10, RescaleFilter::Nearest);
        assert_eq!(scaled.as_rgba8(), img.as_rgba8());
    }

    #[test]
    fn rescale_nearest_upscales() {
        let mut img = Image::new(2, 2);
        // Top-left pixel is red.
        img.set_pixel(0, 0, 255, 0, 0, 255);
        let scaled = img.rescaled(4, 4, RescaleFilter::Nearest);
        assert_eq!(scaled.dimensions(), (4, 4));
        // Top-left quadrant should be red.
        let (r, g, b, a) = scaled.get_pixel(1, 1).unwrap();
        assert_eq!((r, g, b, a), (255, 0, 0, 255));
    }

    #[test]
    fn rescale_bilinear_upscales() {
        let mut img = Image::new(2, 2);
        img.set_pixel(0, 0, 255, 0, 0, 255);
        let scaled = img.rescaled(4, 4, RescaleFilter::Bilinear);
        assert_eq!(scaled.dimensions(), (4, 4));
        // Top-left should be mostly red.
        let (r, _g, _b, _a) = scaled.get_pixel(0, 0).unwrap();
        assert!(r > 100, "expected red channel > 100, got {}", r);
    }

    #[test]
    fn identity_convolution_preserves_image() {
        let mut img = Image::new(4, 4);
        img.set_pixel(1, 1, 100, 150, 200, 255);
        let kernel = vec![
            0.0, 0.0, 0.0,
            0.0, 1.0, 0.0,
            0.0, 0.0, 0.0,
        ];
        let out = img.convolve(&kernel, 3);
        assert_eq!(out.as_rgba8(), img.as_rgba8());
    }

    #[test]
    fn box_blur_smoothes() {
        let mut img = Image::new(6, 6);
        // A single bright pixel.
        img.set_pixel(3, 3, 255, 255, 255, 255);
        let mut engine = ImageConvolutionEngine::box_blur_3x3();
        let blurred = engine.apply(&img);
        // The bright pixel should spread to neighbours.
        let (r, _g, _b, _a) = blurred.get_pixel(3, 3).unwrap();
        assert!(r < 255, "central pixel should be dimmer after blur, got {}", r);
        let (r2, _g2, _b2, _a2) = blurred.get_pixel(2, 3).unwrap();
        assert!(r2 > 0, "neighbour should have some brightness, got {}", r2);
    }

    #[test]
    fn sharpen_on_uniform_image_is_identity() {
        let img = Image::new(4, 4); // all zeros
        let mut engine = ImageConvolutionEngine::sharpen_3x3();
        let out = engine.apply(&img);
        assert_eq!(out.as_rgba8(), img.as_rgba8());
    }

    #[test]
    fn edge_detect_on_uniform_image_is_zero() {
        let img = Image::new(4, 4); // all zeros
        let mut engine = ImageConvolutionEngine::edge_detect_3x3();
        let out = engine.apply(&img);
        assert!(out.as_rgba8().iter().all(|&b| b == 0));
    }

    #[test]
    fn convolution_engine_new_validates() {
        assert!(ImageConvolutionEngine::new(vec![1.0; 9], 3).is_some());
        assert!(ImageConvolutionEngine::new(vec![1.0; 8], 3).is_none());
        assert!(ImageConvolutionEngine::new(vec![], 0).is_none());
    }

    #[test]
    fn convolution_engine_set_kernel() {
        let mut engine = ImageConvolutionEngine::new(vec![0.0; 9], 3).unwrap();
        assert!(engine.set_kernel(vec![0.0; 16], 4).is_some());
        assert_eq!(engine.ksize(), 4);
        assert!(engine.set_kernel(vec![0.0; 5], 3).is_none());
    }

    #[test]
    fn convolve_in_place_does_not_panic_on_empty_image() {
        let mut img = Image::new(0, 0);
        img.convolve_in_place(&[1.0; 9], 3);
    }

    #[test]
    fn convolve_in_place_does_not_panic_on_zero_kernel() {
        let mut img = Image::new(4, 4);
        img.convolve_in_place(&[], 0);
    }
}
