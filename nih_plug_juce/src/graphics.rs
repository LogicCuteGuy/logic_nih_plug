//! JUCE Graphics context wrapper.
//!
//! This module provides a safe Rust wrapper around JUCE's Graphics class,
//! which is used for all 2D drawing operations in JUCE.
//!
//! # Lifetime Management
//!
//! Graphics contexts are typically provided during paint callbacks and should
//! not be stored. The lifetime parameter ensures that a Graphics context
//! cannot outlive the paint callback that created it.
//!
//! # Thread Safety
//!
//! All Graphics operations must be performed on the JUCE message thread.
//! This is enforced through the type system - Graphics does not implement
//! `Send` or `Sync`.
//!
//! # Examples
//!
//! ```ignore
//! use nih_plug_juce::{Graphics, Colour};
//!
//! fn paint(g: &mut Graphics) {
//!     // Set drawing color
//!     let red = Colour::from_rgb(255, 0, 0);
//!     g.set_colour(&red);
//!     
//!     // Draw a filled rectangle
//!     g.fill_rect(10, 10, 100, 50);
//!     
//!     // Draw text
//!     g.draw_text("Hello, JUCE!", 10, 70, 200, 30, Justification::Centred);
//! }
//! ```

use crate::assert_message_thread;
use crate::bridge::ffi;
use crate::error::{JuceError, Result};
use std::marker::PhantomData;

/// Text justification options for drawing text.
///
/// These values match JUCE's Justification flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justification {
    /// Left-aligned text
    Left = 1,
    /// Right-aligned text
    Right = 2,
    /// Horizontally centered text
    HorizontallyCentred = 4,
    /// Top-aligned text
    Top = 8,
    /// Bottom-aligned text
    Bottom = 16,
    /// Vertically centered text
    VerticallyCentred = 32,
    /// Horizontally justified text
    HorizontallyJustified = 64,
    /// Centered both horizontally and vertically
    Centred = 36, // HorizontallyCentred | VerticallyCentred
    /// Centered horizontally, top-aligned
    CentredTop = 12, // HorizontallyCentred | Top
    /// Centered horizontally, bottom-aligned
    CentredBottom = 20, // HorizontallyCentred | Bottom
    /// Left-aligned, vertically centered
    CentredLeft = 33, // Left | VerticallyCentred
    /// Right-aligned, vertically centered
    CentredRight = 34, // Right | VerticallyCentred
    /// Top-left corner
    TopLeft = 9, // Left | Top
    /// Top-right corner
    TopRight = 10, // Right | Top
    /// Bottom-left corner
    BottomLeft = 17, // Left | Bottom
    /// Bottom-right corner
    BottomRight = 18, // Right | Bottom
}

/// A JUCE Graphics context for 2D drawing operations.
///
/// Graphics provides methods for drawing shapes, text, and images. It is
/// typically provided during paint callbacks and should not be stored.
///
/// # Lifetime
///
/// The lifetime parameter `'a` ensures that the Graphics context cannot
/// outlive the paint callback that created it. This prevents use-after-free
/// bugs.
///
/// # Thread Safety
///
/// Graphics does not implement `Send` or `Sync`, enforcing that all drawing
/// operations occur on the message thread.
///
/// # Examples
///
/// ```ignore
/// use nih_plug_juce::{Graphics, Colour};
///
/// fn paint(g: &mut Graphics) {
///     let blue = Colour::from_rgb(0, 0, 255);
///     g.set_colour(&blue);
///     g.fill_ellipse(50.0, 50.0, 100.0, 100.0);
/// }
/// ```
pub struct Graphics<'a> {
    /// Opaque pointer to the C++ juce::Graphics object.
    /// This pointer is NOT owned by this struct - it's managed by JUCE.
    ptr: *mut ffi::JuceGraphics,
    
    /// Lifetime parameter to prevent Graphics from outliving its source.
    /// This ensures Graphics cannot be stored beyond the paint callback.
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Graphics<'a> {
    /// Create a Graphics wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// This is an internal method used by the FFI layer. The pointer must
    /// be valid for the lifetime 'a and must point to a valid juce::Graphics.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Raw pointer to a juce::Graphics object
    pub(crate) unsafe fn from_raw(ptr: *mut ffi::JuceGraphics) -> Self {
        Graphics {
            ptr,
            _lifetime: PhantomData,
        }
    }
    
    /// Create a Graphics wrapper from a pointer (for owned graphics contexts).
    ///
    /// # Safety
    ///
    /// This is an internal method used by the FFI layer. The pointer must
    /// be valid and point to a valid juce::Graphics object.
    ///
    /// # Arguments
    ///
    /// * `ptr` - Raw pointer to a juce::Graphics object
    pub(crate) fn from_ptr(ptr: *mut ffi::JuceGraphics) -> Self {
        Graphics {
            ptr,
            _lifetime: PhantomData,
        }
    }
    
    /// Fill a rectangle with the current color.
    ///
    /// Draws a filled rectangle at the specified position and size.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the top-left corner
    /// * `y` - Y coordinate of the top-left corner
    /// * `width` - Width of the rectangle
    /// * `height` - Height of the rectangle
    ///
    /// # Examples
    ///
    /// ```ignore
    /// g.fill_rect(10, 10, 100, 50);
    /// ```
    pub fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        assert_message_thread!();
        
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_fill_rect(self.ptr, x, y, width, height);
        }
    }
    
    /// Draw a rectangle outline with the current color.
    ///
    /// Draws the outline of a rectangle at the specified position and size.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the top-left corner
    /// * `y` - Y coordinate of the top-left corner
    /// * `width` - Width of the rectangle
    /// * `height` - Height of the rectangle
    ///
    /// # Examples
    ///
    /// ```ignore
    /// g.draw_rect(10, 10, 100, 50);
    /// ```
    pub fn draw_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_draw_rect(self.ptr, x, y, width, height);
        }
    }
    
    /// Fill an ellipse with the current color.
    ///
    /// Draws a filled ellipse within the specified bounding rectangle.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate of the bounding rectangle's top-left corner
    /// * `y` - Y coordinate of the bounding rectangle's top-left corner
    /// * `width` - Width of the bounding rectangle
    /// * `height` - Height of the bounding rectangle
    ///
    /// # Examples
    ///
    /// ```ignore
    /// g.fill_ellipse(50.0, 50.0, 100.0, 100.0);
    /// ```
    pub fn fill_ellipse(&mut self, x: f32, y: f32, width: f32, height: f32) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_fill_ellipse(self.ptr, x, y, width, height);
        }
    }
    
    /// Draw a line with the current color.
    ///
    /// Draws a line from (x1, y1) to (x2, y2).
    ///
    /// # Arguments
    ///
    /// * `x1` - X coordinate of the start point
    /// * `y1` - Y coordinate of the start point
    /// * `x2` - X coordinate of the end point
    /// * `y2` - Y coordinate of the end point
    ///
    /// # Examples
    ///
    /// ```ignore
    /// g.draw_line(10.0, 10.0, 100.0, 100.0);
    /// ```
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_draw_line(self.ptr, x1, y1, x2, y2);
        }
    }
    
    /// Set the current drawing color.
    ///
    /// All subsequent drawing operations will use this color until it is
    /// changed again.
    ///
    /// # Arguments
    ///
    /// * `colour` - The color to use for drawing
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let red = Colour::from_rgb(255, 0, 0);
    /// g.set_colour(&red);
    /// g.fill_rect(10, 10, 100, 50);
    /// ```
    pub fn set_colour(&mut self, colour: &crate::drawing::Colour) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            let colour_ptr = colour.as_ptr();
            ffi::graphics_set_colour(self.ptr, colour_ptr);
        }
    }
    
    /// Draw text within a rectangle.
    ///
    /// Draws the specified text within the given rectangle, using the
    /// specified justification.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to draw
    /// * `x` - X coordinate of the text rectangle's top-left corner
    /// * `y` - Y coordinate of the text rectangle's top-left corner
    /// * `width` - Width of the text rectangle
    /// * `height` - Height of the text rectangle
    /// * `justification` - How to align the text within the rectangle
    ///
    /// # Examples
    ///
    /// ```ignore
    /// g.draw_text("Hello!", 10, 10, 200, 30, Justification::Centred);
    /// ```
    pub fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        justification: Justification,
    ) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_draw_text(
                self.ptr,
                text.as_ptr(),
                text.len(),
                x,
                y,
                width,
                height,
                justification as i32,
            );
        }
    }
    
    /// Draw an image at the specified position.
    ///
    /// Draws the image at its original size at the given coordinates.
    ///
    /// # Arguments
    ///
    /// * `image` - The image to draw
    /// * `x` - X coordinate where the image should be drawn
    /// * `y` - Y coordinate where the image should be drawn
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let image = Image::load_from_file("icon.png")?;
    /// g.draw_image_at(&image, 10, 10);
    /// ```
    pub fn draw_image_at(&mut self, image: &ffi::JuceImage, x: i32, y: i32) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_draw_image_at(self.ptr, image, x, y);
        }
    }
    
    /// Stroke (outline) a path with the current color.
    ///
    /// Draws the outline of the specified path using the current color
    /// and stroke settings.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to stroke
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new();
    /// path.start_new_sub_path(10.0, 10.0);
    /// path.line_to(100.0, 100.0);
    /// g.stroke_path(&path);
    /// ```
    pub fn stroke_path(&mut self, path: &ffi::JucePath) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_stroke_path(self.ptr, path);
        }
    }
    
    /// Fill a path with the current color.
    ///
    /// Fills the interior of the specified path with the current color.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to fill
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut path = Path::new();
    /// path.add_rectangle(10.0, 10.0, 100.0, 50.0);
    /// g.fill_path(&path);
    /// ```
    pub fn fill_path(&mut self, path: &ffi::JucePath) {
        if self.ptr.is_null() {
            return;
        }
        
        unsafe {
            ffi::graphics_fill_path(self.ptr, path);
        }
    }
    
    /// Get the raw pointer to the underlying C++ Graphics object.
    ///
    /// # Safety
    ///
    /// This is an internal method used by other parts of the FFI layer.
    /// The returned pointer is only valid for the lifetime 'a.
    #[doc(hidden)]
    pub(crate) fn as_ptr(&self) -> *mut ffi::JuceGraphics {
        self.ptr
    }
    
    /// Get a mutable raw pointer to the underlying JUCE Graphics object.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the returned pointer is only valid
    /// as long as this Graphics instance exists. The caller must ensure
    /// the pointer is not used after this Graphics is dropped.
    #[doc(hidden)]
    pub(crate) unsafe fn as_ptr_mut(&mut self) -> *mut ffi::JuceGraphics {
        self.ptr
    }
}

// Explicitly do NOT implement Send or Sync for Graphics.
// This enforces that Graphics can only be used on the thread where
// it was created (the message thread), matching JUCE's requirements.
//
// The PhantomData<&'a ()> field doesn't make Graphics !Send + !Sync by itself,
// so we need to ensure the underlying pointer type does this.

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: These tests cannot actually create a Graphics context without
    // a full JUCE application running. They are here as documentation of
    // the expected API. Real testing will be done through integration tests
    // with actual JUCE components.
    
    #[test]
    fn test_justification_values() {
        // Verify that justification enum values match JUCE
        assert_eq!(Justification::Left as i32, 1);
        assert_eq!(Justification::Right as i32, 2);
        assert_eq!(Justification::HorizontallyCentred as i32, 4);
        assert_eq!(Justification::Centred as i32, 36);
    }
}
