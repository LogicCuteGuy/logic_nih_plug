//! Text rendering implementation.
//!
//! This module provides font rendering capabilities using the fontdue library.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_graphics::text::{Font, FontSettings};
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
    /// use logic_nih_plug_graphics::text::{Font, FontSettings};
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
    /// use logic_nih_plug_graphics::text::{Font, FontSettings};
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
    /// use logic_nih_plug_graphics::text::{Font, FontSettings};
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

    /// Measure the width of a string at a specific size.
    ///
    /// This is an alias for [`Font::measure_text`] matching JUCE's
    /// `Font::getStringWidthFloat` API name.
    pub fn get_string_width_float(&self, text: &str, size: f32) -> f32 {
        self.measure_text(text, size)
    }

    /// Get the typographic ascent for a specific font size.
    ///
    /// The ascent is the distance from the baseline to the top of the
    /// tallest glyph. This value is always positive.
    pub fn get_ascent(&self, size: f32) -> f32 {
        self.inner
            .horizontal_line_metrics(size)
            .map(|m| m.ascent)
            .unwrap_or(size * 0.8)
    }

    /// Get the typographic descent for a specific font size.
    ///
    /// The descent is the distance from the baseline to the bottom of the
    /// lowest glyph. This value is always returned as positive, matching
    /// JUCE's convention.
    pub fn get_descent(&self, size: f32) -> f32 {
        self.inner
            .horizontal_line_metrics(size)
            .map(|m| m.descent.abs())
            .unwrap_or(size * 0.2)
    }

    /// Get the total typographic height (ascent + descent) for a
    /// specific font size.
    pub fn get_height(&self, size: f32) -> f32 {
        self.get_ascent(size) + self.get_descent(size)
    }
}

/// A positioned glyph within a [`GlyphArrangement`].
#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    /// The character this glyph represents.
    pub character: char,
    /// X position of the glyph origin (baseline left) in pixels.
    pub x: f32,
    /// Y position of the glyph baseline in pixels.
    pub y: f32,
    /// The advance width of this glyph in pixels.
    pub advance: f32,
}

/// How much extra space to add between lines of text.
///
/// Mirrors `juce::LineSpacing`. The value is a multiplier of the
/// font's natural line height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineSpacing {
    /// Use the font's built-in line height (default).
    Single,
    /// Use a fixed multiplier of the font's line height.
    Multiple(f32),
    /// Use a fixed pixel distance between baselines of consecutive lines.
    Fixed(f32),
}

impl Default for LineSpacing {
    fn default() -> Self {
        Self::Single
    }
}

impl LineSpacing {
    /// Compute the pixel distance between baselines for the given font
    /// and size.
    pub fn line_distance(&self, font: &Font, size: f32) -> f32 {
        match *self {
            Self::Single => font.line_height(size),
            Self::Multiple(m) => font.line_height(size) * m,
            Self::Fixed(px) => px,
        }
    }
}

/// A positioned arrangement of glyphs for shaped text layout.
///
/// Mirrors `juce::GlyphArrangement`. Stores a list of
/// [`PositionedGlyph`] entries with a bounding box. Build one via
/// [`GlyphArrangement::from_text`], then query positions or iterate
/// glyphs.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_graphics::text::{Font, FontSettings, GlyphArrangement};
///
/// let font_data = include_bytes!("../tests/test_font.ttf");
/// let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
///
/// let arrangement = GlyphArrangement::from_text(&font, "Hello", 24.0, 0.0, 0.0);
/// assert_eq!(arrangement.len(), 5);
/// assert!(arrangement.width() > 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct GlyphArrangement {
    glyphs: Vec<PositionedGlyph>,
    width: f32,
}

impl GlyphArrangement {
    /// Lay out `text` using `font` at the given `size`, starting at
    /// `(origin_x, origin_y)`.
    pub fn from_text(
        font: &Font,
        text: &str,
        size: f32,
        origin_x: f32,
        origin_y: f32,
    ) -> Self {
        let mut glyphs = Vec::with_capacity(text.len());
        let mut cursor_x = origin_x;

        for ch in text.chars() {
            let advance = font.horizontal_advance(ch, size);
            glyphs.push(PositionedGlyph {
                character: ch,
                x: cursor_x,
                y: origin_y,
                advance,
            });
            cursor_x += advance;
        }

        let width = cursor_x - origin_x;
        Self { glyphs, width }
    }

    /// The number of positioned glyphs.
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// Whether the arrangement contains no glyphs.
    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }

    /// The total width of the arranged text in pixels.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// A slice of all positioned glyphs.
    pub fn glyphs(&self) -> &[PositionedGlyph] {
        &self.glyphs
    }

    /// Shift every glyph by `(dx, dy)`.
    pub fn translate(&mut self, dx: f32, dy: f32) {
        for g in &mut self.glyphs {
            g.x += dx;
            g.y += dy;
        }
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

    #[test]
    fn test_get_string_width_float_matches_measure_text() {
        let font = create_test_font().unwrap();
        let text = "Hello World";
        let size = 24.0;
        assert_eq!(font.measure_text(text, size), font.get_string_width_float(text, size));
    }

    #[test]
    fn test_get_ascent_positive() {
        let font = create_test_font().unwrap();
        let a = font.get_ascent(24.0);
        assert!(a > 0.0, "ascent should be positive, got {}", a);
    }

    #[test]
    fn test_get_descent_positive() {
        let font = create_test_font().unwrap();
        let d = font.get_descent(24.0);
        assert!(d > 0.0, "descent should be positive, got {}", d);
    }

    #[test]
    fn test_get_height_equals_ascent_plus_descent() {
        let font = create_test_font().unwrap();
        let size = 24.0;
        let h = font.get_height(size);
        let a = font.get_ascent(size);
        let d = font.get_descent(size);
        assert!((h - (a + d)).abs() < 0.001);
    }

    #[test]
    fn test_glyph_arrangement_from_text() {
        let font = create_test_font().unwrap();
        let arr = GlyphArrangement::from_text(&font, "Hi", 24.0, 0.0, 0.0);
        assert_eq!(arr.len(), 2);
        assert!(!arr.is_empty());
        assert!(arr.width() > 0.0);
        assert_eq!(arr.glyphs()[0].character, 'H');
        assert_eq!(arr.glyphs()[1].character, 'i');
        // Second glyph should be after the first.
        assert!(arr.glyphs()[1].x > arr.glyphs()[0].x);
    }

    #[test]
    fn test_glyph_arrangement_translate() {
        let font = create_test_font().unwrap();
        let mut arr = GlyphArrangement::from_text(&font, "X", 24.0, 10.0, 20.0);
        assert!((arr.glyphs()[0].x - 10.0).abs() < 0.001);
        assert!((arr.glyphs()[0].y - 20.0).abs() < 0.001);
        arr.translate(5.0, 3.0);
        assert!((arr.glyphs()[0].x - 15.0).abs() < 0.001);
        assert!((arr.glyphs()[0].y - 23.0).abs() < 0.001);
    }

    #[test]
    fn test_glyph_arrangement_empty_string() {
        let font = create_test_font().unwrap();
        let arr = GlyphArrangement::from_text(&font, "", 24.0, 0.0, 0.0);
        assert!(arr.is_empty());
        assert_eq!(arr.width(), 0.0);
    }

    #[test]
    fn test_line_spacing_default_is_single() {
        let ls = LineSpacing::default();
        assert_eq!(ls, LineSpacing::Single);
    }

    #[test]
    fn test_line_spacing_single() {
        let font = create_test_font().unwrap();
        let ls = LineSpacing::Single;
        assert_eq!(ls.line_distance(&font, 24.0), font.line_height(24.0));
    }

    #[test]
    fn test_line_spacing_multiple() {
        let font = create_test_font().unwrap();
        let ls = LineSpacing::Multiple(1.5);
        let expected = font.line_height(24.0) * 1.5;
        assert!((ls.line_distance(&font, 24.0) - expected).abs() < 0.001);
    }

    #[test]
    fn test_line_spacing_fixed() {
        let font = create_test_font().unwrap();
        let ls = LineSpacing::Fixed(40.0);
        assert_eq!(ls.line_distance(&font, 24.0), 40.0);
    }
}
