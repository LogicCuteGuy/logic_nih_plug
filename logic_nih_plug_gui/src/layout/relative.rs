//! Relative coordinates and rectangles for percentage-based layout.
//!
//! `RelativeCoordinate` represents a position or size that can be either
//! an absolute pixel value or a percentage of the parent container's
//! dimension. `RelativeRectangle` combines four such coordinates into a
//! full rectangle description.
//!
//! This is the Rust equivalent of JUCE's `RelativeCoordinate` /
//! `RelativeRectangle`, commonly used in proportional layouts where a
//! child should always occupy, say, 25% of its parent's width.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_gui::layout::relative::{RelativeCoordinate, RelativeRectangle};
//!
//! // 50% across
//! let x = RelativeCoordinate::percent(50.0);
//! assert_eq!(x.resolve(400.0), 200.0);
//!
//! // 10px from the right edge
//! let right_margin = RelativeCoordinate::from_right(10.0);
//! assert_eq!(right_margin.resolve_horizontal(400.0), 390.0);
//!
//! // Rectangle: fill 25%-75% horizontally, 10%-90% vertically
//! let rect = RelativeRectangle::new(
//!     RelativeCoordinate::percent(25.0),
//!     RelativeCoordinate::percent(10.0),
//!     RelativeCoordinate::percent(50.0),
//!     RelativeCoordinate::percent(80.0),
//! );
//! let bounds = rect.resolve(400.0, 400.0);
//! assert_eq!(bounds, (100.0, 40.0, 200.0, 320.0));
//! ```

/// A coordinate that can be absolute (pixels) or relative (percentage).
///
/// Four modes are supported:
/// - **Absolute**: fixed pixel offset from the origin.
/// - **Percent**: percentage of the relevant parent dimension.
/// - **FromRight**: pixels from the right edge (parent width − value).
/// - **FromBottom**: pixels from the bottom edge (parent height − value).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelativeCoordinate {
    /// Absolute pixel offset.
    Absolute(f32),
    /// Percentage of the parent dimension (0–100+).
    Percent(f32),
    /// Pixels from the right edge of the parent.
    FromRight(f32),
    /// Pixels from the bottom edge of the parent.
    FromBottom(f32),
}

impl RelativeCoordinate {
    /// Create an absolute (pixel) coordinate.
    pub fn absolute(value: f32) -> Self {
        RelativeCoordinate::Absolute(value)
    }

    /// Create a percentage coordinate (0.0 = 0%, 100.0 = full size).
    pub fn percent(value: f32) -> Self {
        RelativeCoordinate::Percent(value)
    }

    /// Create a coordinate measured from the right edge.
    pub fn from_right(value: f32) -> Self {
        RelativeCoordinate::FromRight(value)
    }

    /// Create a coordinate measured from the bottom edge.
    pub fn from_bottom(value: f32) -> Self {
        RelativeCoordinate::FromBottom(value)
    }

    /// Resolve this coordinate against a horizontal parent dimension (width).
    pub fn resolve_horizontal(&self, parent_width: f32) -> f32 {
        match self {
            RelativeCoordinate::Absolute(v) => *v,
            RelativeCoordinate::Percent(p) => parent_width * p / 100.0,
            RelativeCoordinate::FromRight(v) => (parent_width - v).max(0.0),
            RelativeCoordinate::FromBottom(_) => 0.0, // not meaningful horizontally
        }
    }

    /// Resolve this coordinate against a vertical parent dimension (height).
    pub fn resolve_vertical(&self, parent_height: f32) -> f32 {
        match self {
            RelativeCoordinate::Absolute(v) => *v,
            RelativeCoordinate::Percent(p) => parent_height * p / 100.0,
            RelativeCoordinate::FromBottom(v) => (parent_height - v).max(0.0),
            RelativeCoordinate::FromRight(_) => 0.0, // not meaningful vertically
        }
    }

    /// Resolve against a single dimension (convenience for width or height).
    ///
    /// For `FromRight` / `FromBottom`, the caller must supply the *correct*
    /// dimension (width for `FromRight`, height for `FromBottom`).
    pub fn resolve(&self, parent_size: f32) -> f32 {
        match self {
            RelativeCoordinate::Absolute(v) => *v,
            RelativeCoordinate::Percent(p) => parent_size * p / 100.0,
            RelativeCoordinate::FromRight(v) => (parent_size - v).max(0.0),
            RelativeCoordinate::FromBottom(v) => (parent_size - v).max(0.0),
        }
    }
}

impl Default for RelativeCoordinate {
    fn default() -> Self {
        RelativeCoordinate::Absolute(0.0)
    }
}

/// A rectangle where each edge is a `RelativeCoordinate`.
///
/// Useful for defining child bounds proportionally relative to a parent.
#[derive(Debug, Clone, PartialEq)]
pub struct RelativeRectangle {
    /// X coordinate (horizontal origin).
    pub x: RelativeCoordinate,
    /// Y coordinate (vertical origin).
    pub y: RelativeCoordinate,
    /// Width.
    pub width: RelativeCoordinate,
    /// Height.
    pub height: RelativeCoordinate,
}

impl RelativeRectangle {
    /// Create a new relative rectangle.
    pub fn new(
        x: RelativeCoordinate,
        y: RelativeCoordinate,
        width: RelativeCoordinate,
        height: RelativeCoordinate,
    ) -> Self {
        Self { x, y, width, height }
    }

    /// Create a rectangle that fills the entire parent (0%, 0%, 100%, 100%).
    pub fn fill() -> Self {
        Self::new(
            RelativeCoordinate::percent(0.0),
            RelativeCoordinate::percent(0.0),
            RelativeCoordinate::percent(100.0),
            RelativeCoordinate::percent(100.0),
        )
    }

    /// Create a rectangle with absolute pixel values.
    pub fn from_pixels(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(
            RelativeCoordinate::absolute(x),
            RelativeCoordinate::absolute(y),
            RelativeCoordinate::absolute(width),
            RelativeCoordinate::absolute(height),
        )
    }

    /// Create a rectangle with percentage values.
    pub fn from_percent(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self::new(
            RelativeCoordinate::percent(x),
            RelativeCoordinate::percent(y),
            RelativeCoordinate::percent(width),
            RelativeCoordinate::percent(height),
        )
    }

    /// Resolve this relative rectangle against a parent size.
    ///
    /// Returns `(x, y, width, height)` in absolute pixels.
    pub fn resolve(&self, parent_width: f32, parent_height: f32) -> (f32, f32, f32, f32) {
        let x = self.x.resolve_horizontal(parent_width);
        let y = self.y.resolve_vertical(parent_height);
        let w = self.width.resolve_horizontal(parent_width);
        let h = self.height.resolve_vertical(parent_height);
        (x, y, w, h)
    }
}

impl Default for RelativeRectangle {
    fn default() -> Self {
        Self::fill()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- RelativeCoordinate tests --

    #[test]
    fn absolute_resolve() {
        let c = RelativeCoordinate::absolute(42.0);
        assert_eq!(c.resolve(1000.0), 42.0);
    }

    #[test]
    fn percent_resolve() {
        let c = RelativeCoordinate::percent(25.0);
        assert_eq!(c.resolve(400.0), 100.0);
    }

    #[test]
    fn percent_zero() {
        let c = RelativeCoordinate::percent(0.0);
        assert_eq!(c.resolve(500.0), 0.0);
    }

    #[test]
    fn percent_over_100() {
        let c = RelativeCoordinate::percent(150.0);
        assert_eq!(c.resolve(200.0), 300.0);
    }

    #[test]
    fn from_right_resolve_horizontal() {
        let c = RelativeCoordinate::from_right(10.0);
        assert_eq!(c.resolve_horizontal(400.0), 390.0);
    }

    #[test]
    fn from_right_does_not_clamp_negative() {
        let c = RelativeCoordinate::from_right(500.0);
        assert_eq!(c.resolve_horizontal(400.0), 0.0);
    }

    #[test]
    fn from_bottom_resolve_vertical() {
        let c = RelativeCoordinate::from_bottom(20.0);
        assert_eq!(c.resolve_vertical(300.0), 280.0);
    }

    #[test]
    fn from_bottom_clamps() {
        let c = RelativeCoordinate::from_bottom(500.0);
        assert_eq!(c.resolve_vertical(300.0), 0.0);
    }

    #[test]
    fn from_right_horizontal_returns_zero() {
        let c = RelativeCoordinate::from_right(10.0);
        assert_eq!(c.resolve_vertical(100.0), 0.0);
    }

    #[test]
    fn from_bottom_vertical_returns_zero() {
        let c = RelativeCoordinate::from_bottom(10.0);
        assert_eq!(c.resolve_horizontal(100.0), 0.0);
    }

    #[test]
    fn default_is_absolute_zero() {
        let c = RelativeCoordinate::default();
        assert_eq!(c, RelativeCoordinate::Absolute(0.0));
        assert_eq!(c.resolve(999.0), 0.0);
    }

    #[test]
    fn percent_negative() {
        let c = RelativeCoordinate::percent(-10.0);
        assert_eq!(c.resolve(100.0), -10.0);
    }

    // -- RelativeRectangle tests --

    #[test]
    fn resolve_absolute_rect() {
        let rect = RelativeRectangle::from_pixels(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.resolve(400.0, 300.0), (10.0, 20.0, 100.0, 50.0));
    }

    #[test]
    fn resolve_percent_rect() {
        let rect = RelativeRectangle::from_percent(10.0, 20.0, 50.0, 40.0);
        // x=400*10%=40, y=300*20%=60, w=400*50%=200, h=300*40%=120
        assert_eq!(rect.resolve(400.0, 300.0), (40.0, 60.0, 200.0, 120.0));
    }

    #[test]
    fn fill_rect() {
        let rect = RelativeRectangle::fill();
        assert_eq!(rect.resolve(800.0, 600.0), (0.0, 0.0, 800.0, 600.0));
    }

    #[test]
    fn mixed_coordinates() {
        let rect = RelativeRectangle::new(
            RelativeCoordinate::absolute(10.0),
            RelativeCoordinate::percent(50.0),
            RelativeCoordinate::from_right(20.0),  // width: 400 - 20 = 380
            RelativeCoordinate::from_bottom(30.0), // height: 300 - 30 = 270
        );
        assert_eq!(rect.resolve(400.0, 300.0), (10.0, 150.0, 380.0, 270.0));
    }

    #[test]
    fn default_rect_is_fill() {
        let rect = RelativeRectangle::default();
        assert_eq!(rect.resolve(100.0, 100.0), (0.0, 0.0, 100.0, 100.0));
    }

    #[test]
    fn zero_size_parent() {
        let rect = RelativeRectangle::fill();
        assert_eq!(rect.resolve(0.0, 0.0), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn large_percent() {
        let rect = RelativeRectangle::from_percent(0.0, 0.0, 200.0, 200.0);
        // width/height > 100% is allowed (overflow)
        assert_eq!(rect.resolve(300.0, 300.0), (0.0, 0.0, 600.0, 600.0));
    }
}
