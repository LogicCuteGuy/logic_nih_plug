//! Text rendering implementation.
//!
//! This module provides font rendering capabilities using the fontdue library.
//!
//! # Examples
//!
//! ```
//! use nih_plug_graphics::text::{Font, FontSettings};
//!
//! let font_data = include_bytes!("../tests/test_font.ttf");
//! let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
//!
//! let (metrics, bitmap) = font.rasterize('A', 24.0);
//! ```

use crate::error::GraphicsError;
use fontdue::{Font as FontdueFont, FontSettings as FontdueSettings, Metrics};

/// Font for text rendering.
///
/// This type wraps the fontdue Font and provides a safe interface for
/// loading fonts and rasterizing glyphs.
///
/// # Thread Safety
///
/// This type is `Send` and `Sync` and can be shared across threads.
pub struct Font {
    inner: FontdueFont,
}

/// Settings for font loading.
///
/// These settings control how the font is loaded and processed.
#[derive(Debug, Clone, Copy)]
pub struct FontSettings {
    /// The scale factor for the font (default: 1.0)
    pub scale: f32,
    /// The index of the font in a font collection (default: 0)
    pub collection_index: u32,
}

impl Default for FontSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            collection_index: 0,
        }
    }
}

impl Font {
    /// Create a new Font from byte data.
    ///
    /// # Arguments
    ///
    /// * `data` - The font file data (TTF, OTF, etc.)
    /// * `settings` - Font loading settings
    ///
    /// # Returns
    ///
    /// Returns a Result containing the Font or a GraphicsError if loading fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::text::{Font, FontSettings};
    ///
    /// let font_data = include_bytes!("../tests/test_font.ttf");
    /// let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
    /// ```
    pub fn from_bytes(data: &[u8], settings: FontSettings) -> Result<Self, GraphicsError> {
        let fontdue_settings = FontdueSettings {
            scale: settings.scale,
            collection_index: settings.collection_index,
            ..FontdueSettings::default()
        };

        let inner = FontdueFont::from_bytes(data, fontdue_settings)
            .map_err(|e| GraphicsError::FontLoadError(e.to_string()))?;

        Ok(Self { inner })
    }

    /// Rasterize a character at a specific size.
    ///
    /// # Arguments
    ///
    /// * `character` - The character to rasterize
    /// * `size` - The font size in pixels
    ///
    /// # Returns
    ///
    /// Returns a tuple of (Metrics, bitmap) where the bitmap is a grayscale
    /// image with values from 0 (transparent) to 255 (opaque).
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::text::{Font, FontSettings};
    ///
    /// let font_data = include_bytes!("../tests/test_font.ttf");
    /// let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
    ///
    /// let (metrics, bitmap) = font.rasterize('A', 24.0);
    /// assert!(metrics.width > 0);
    /// assert!(metrics.height > 0);
    /// ```
    pub fn rasterize(&self, character: char, size: f32) -> (Metrics, Vec<u8>) {
        self.inner.rasterize(character, size)
    }

    /// Rasterize a character at a specific size with subpixel positioning.
    ///
    /// # Arguments
    ///
    /// * `character` - The character to rasterize
    /// * `size` - The font size in pixels
    ///
    /// # Returns
    ///
    /// Returns a tuple of (Metrics, bitmap).
    pub fn rasterize_subpixel(
        &self,
        character: char,
        size: f32,
    ) -> (Metrics, Vec<u8>) {
        self.inner.rasterize_subpixel(character, size)
    }

    /// Get the horizontal advance for a character at a specific size.
    ///
    /// This is the amount to advance the cursor after drawing this character.
    ///
    /// # Arguments
    ///
    /// * `character` - The character to measure
    /// * `size` - The font size in pixels
    ///
    /// # Returns
    ///
    /// Returns the horizontal advance in pixels.
    pub fn horizontal_advance(&self, character: char, size: f32) -> f32 {
        self.inner.metrics(character, size).advance_width
    }

    /// Get metrics for a character without rasterizing.
    ///
    /// # Arguments
    ///
    /// * `character` - The character to measure
    /// * `size` - The font size in pixels
    ///
    /// # Returns
    ///
    /// Returns the Metrics for the character.
    pub fn metrics(&self, character: char, size: f32) -> Metrics {
        self.inner.metrics(character, size)
    }

    /// Get the line height for a specific font size.
    ///
    /// # Arguments
    ///
    /// * `size` - The font size in pixels
    ///
    /// # Returns
    ///
    /// Returns the line height in pixels.
    pub fn line_height(&self, size: f32) -> f32 {
        self.inner
            .horizontal_line_metrics(size)
            .map(|m| m.new_line_size)
            .unwrap_or(size * 1.2) // Default to 120% of font size if metrics unavailable
    }

    /// Measure the width of a string at a specific size.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to measure
    /// * `size` - The font size in pixels
    ///
    /// # Returns
    ///
    /// Returns the total width in pixels.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_graphics::text::{Font, FontSettings};
    ///
    /// let font_data = include_bytes!("../tests/test_font.ttf");
    /// let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
    ///
    /// let width = font.measure_text("Hello", 24.0);
    /// assert!(width > 0.0);
    /// ```
    pub fn measure_text(&self, text: &str, size: f32) -> f32 {
        text.chars()
            .map(|c| self.horizontal_advance(c, size))
            .sum()
    }
}

// Safety: Font is thread-safe as fontdue::Font is thread-safe
unsafe impl Send for Font {}
unsafe impl Sync for Font {}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal valid TTF font data for testing
    // This is a very basic font with just a few glyphs
    // Generated using a minimal TTF structure
    const TEST_FONT_DATA: &[u8] = include_bytes!("../tests/test_font.ttf");

    // Helper to create a test font
    fn create_test_font() -> Result<Font, GraphicsError> {
        Font::from_bytes(TEST_FONT_DATA, FontSettings::default())
    }

    #[test]
    fn test_font_creation() {
        let font = create_test_font();
        assert!(font.is_ok(), "Font creation should succeed");
    }

    #[test]
    fn test_rasterize_character() {
        let font = create_test_font().unwrap();
        let (metrics, bitmap) = font.rasterize('A', 24.0);

        // Check that we got some output
        assert!(metrics.width > 0);
        assert!(metrics.height > 0);
        assert_eq!(bitmap.len(), metrics.width * metrics.height);
    }

    #[test]
    fn test_measure_text() {
        let font = create_test_font().unwrap();
        let width = font.measure_text("Hello", 24.0);

        // Width should be positive
        assert!(width > 0.0);

        // Longer text should be wider
        let longer_width = font.measure_text("Hello World", 24.0);
        assert!(longer_width > width);
    }

    #[test]
    fn test_font_size_scaling() {
        let font = create_test_font().unwrap();
        let small = font.measure_text("A", 12.0);
        let large = font.measure_text("A", 24.0);

        // Larger font size should produce larger measurements
        assert!(large > small);
    }

    #[test]
    fn test_line_height() {
        let font = create_test_font().unwrap();
        let height = font.line_height(24.0);

        // Line height should be positive
        assert!(height > 0.0);
    }

    #[test]
    fn test_metrics() {
        let font = create_test_font().unwrap();
        let metrics = font.metrics('A', 24.0);

        // Metrics should have reasonable values
        assert!(metrics.width > 0);
        assert!(metrics.height > 0);
        assert!(metrics.advance_width > 0.0);
    }

    #[test]
    fn test_invalid_font_data() {
        let invalid_data = b"not a font";
        let result = Font::from_bytes(invalid_data, FontSettings::default());
        assert!(result.is_err(), "Invalid font data should return an error");
    }

    #[test]
    fn test_font_settings() {
        let settings = FontSettings {
            scale: 2.0,
            collection_index: 0,
        };
        
        let font = Font::from_bytes(TEST_FONT_DATA, settings);
        assert!(font.is_ok(), "Font with custom settings should load");
    }
}
