//! 2D vector graphics primitives backed by [`tiny-skia`].
//!
//! This module provides a JUCE-style vector graphics API on top of the
//! [`tiny_skia`] crate. Where [`tiny_skia`] already has a Skia-compatible
//! type (e.g. [`tiny_skia::Path`], [`tiny_skia::Stroke`]) we re-export it
//! directly so callers get the full underlying API for free. JUCE-style
//! names that don't map 1:1 ([`Justification`], [`FillType`],
//! [`ColourGradient`], [`DropShadow`]) live in this module. The
//! [`Painter`] struct is a JUCE-style `Graphics`-equivalent for vector
//! primitives — fill / stroke paths, fill rects, render gradients.
//!
//! The pixel buffer returned by [`Painter::data`] is **premultiplied RGBA8**
//! (Skia's convention), unlike [`crate::primitives::Graphics::as_bytes`]
//! which returns straight RGBA8. Use [`Painter::data_straight`] if you
//! need to feed it to other code that expects straight alpha.
//!
//! # Example
//!
//! ```no_run
//! use logic_nih_plug_graphics::vector::{Painter, ColourGradient, GradientStop, SpreadMode};
//! use logic_nih_plug_graphics::vector::PathBuilder;
//! use logic_nih_plug_graphics::Color;
//!
//! let mut painter = Painter::new(256, 256).unwrap();
//!
//! // Build a rounded rectangle path
//! let path = PathBuilder::new()
//!     .move_to(10.0, 10.0)
//!     .line_to(246.0, 10.0)
//!     .quad_to(256.0, 10.0, 256.0, 20.0)
//!     .line_to(256.0, 246.0)
//!     .quad_to(256.0, 256.0, 246.0, 256.0)
//!     .line_to(10.0, 256.0)
//!     .quad_to(0.0, 256.0, 0.0, 246.0)
//!     .line_to(0.0, 20.0)
//!     .quad_to(0.0, 10.0, 10.0, 10.0)
//!     .close()
//!     .finish()
//!     .unwrap();
//!
//! // Fill with a linear gradient
//! let gradient = ColourGradient::linear(
//!     (0.0, 0.0),
//!     (256.0, 256.0),
//!     vec![
//!         GradientStop::new(0.0, tiny_skia::Color::from_rgba8(255, 0, 0, 255)),
//!         GradientStop::new(1.0, tiny_skia::Color::from_rgba8(0, 0, 255, 255)),
//!     ],
//!     SpreadMode::Pad,
//! ).unwrap();
//! painter.fill_path_with_gradient(&path, &gradient, Default::default());
//! ```

use crate::Color;

// Re-export tiny-skia types that map 1:1 to the JUCE API. Plugin authors
// who want full control over the underlying skia API can use these
// directly; those who want a more JUCE-flavoured surface can use the
// helpers below.
pub use tiny_skia::{
    BlendMode, FillRule, GradientStop, LineCap, LineJoin, Paint, Path,
    PathBuilder as SkPathBuilder, PathSegment, Shader, SpreadMode, Stroke,
    Transform as SkTransform,
};

bitflags::bitflags! {
    /// Horizontal/vertical alignment flags, mirroring `juce::Justification`.
    ///
    /// JUCE packs several flags into one integer; the most useful
    /// combinations are pre-defined as [`Justification::LEFT`],
    /// [`Justification::RIGHT`], [`Justification::TOP`],
    /// [`Justification::BOTTOM`], and [`Justification::CENTERED`]. Use
    /// [`Justification::contains`] to test individual flags:
    ///
    /// ```
    /// use logic_nih_plug_graphics::vector::Justification;
    ///
    /// let j = Justification::CENTERED;
    /// assert!(j.contains(Justification::HORIZONTALLY_CENTERED));
    /// assert!(j.contains(Justification::VERTICALLY_CENTERED));
    /// assert!(!j.contains(Justification::LEFT));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Justification: u32 {
        /// Place content against the left edge.
        const LEFT = 0x0001;
        /// Place content against the right edge.
        const RIGHT = 0x0002;
        /// Centre horizontally within the available space.
        const HORIZONTALLY_CENTERED = 0x0004;
        /// Place content against the top edge.
        const TOP = 0x0008;
        /// Place content against the bottom edge.
        const BOTTOM = 0x0010;
        /// Centre vertically within the available space.
        const VERTICALLY_CENTERED = 0x0020;
        /// Spread content horizontally so it fills the available space
        /// (text only).
        const HORIZONTALLY_JUSTIFIED = 0x0040;
        /// Convenience: centred horizontally and vertically.
        const CENTERED = Self::HORIZONTALLY_CENTERED.bits() | Self::VERTICALLY_CENTERED.bits();
    }
}

/// Whether a [`Path`] should be filled or stroked.
///
/// Modelled as a tag enum (rather than tiny-skia's separate `fill_path`
/// vs `stroke_path` methods) so callers can write a single draw function
/// that branches on this at the call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillType {
    /// Fill the interior of the path. Used with [`Painter::fill_path`].
    #[default]
    Fill,
    /// Stroke the outline of the path with a [`Stroke`]. Used with
    /// [`Painter::stroke_path`].
    Stroke,
}

/// A colour gradient — linear or radial. Mirrors `juce::ColourGradient`.
///
/// Internally wraps a [`Shader`] from tiny-skia. Use
/// [`Painter::fill_path_with_gradient`] (or
/// [`Painter::stroke_path_with_gradient`]) to paint with one.
///
/// Constructed via [`ColourGradient::linear`] or
/// [`ColourGradient::radial`]. Both return `None` when the gradient is
/// degenerate (zero-length, single stop, etc.).
#[derive(Clone)]
pub struct ColourGradient {
    shader: Shader<'static>,
}

impl std::fmt::Debug for ColourGradient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColourGradient").finish()
    }
}

impl ColourGradient {
    /// Construct a linear gradient from `(start)` to `(end)` with the
    /// given colour stops. Returns `None` if the stops are empty or
    /// the gradient is degenerate.
    pub fn linear(
        start: (f32, f32),
        end: (f32, f32),
        stops: Vec<GradientStop>,
        spread: SpreadMode,
    ) -> Option<Self> {
        let shader = tiny_skia::LinearGradient::new(
            tiny_skia::Point::from_xy(start.0, start.1),
            tiny_skia::Point::from_xy(end.0, end.1),
            stops,
            spread,
            SkTransform::identity(),
        )?;
        Some(Self { shader })
    }

    /// Construct a two-point conical (radial) gradient. `start` and `end`
    /// are the centres of the inner and outer circles respectively; each
    /// has a radius of the same name. Returns `None` if the radii are
    /// negative, the stops are empty, or the gradient is degenerate.
    pub fn radial(
        start: (f32, f32),
        start_radius: f32,
        end: (f32, f32),
        end_radius: f32,
        stops: Vec<GradientStop>,
        spread: SpreadMode,
    ) -> Option<Self> {
        let shader = tiny_skia::RadialGradient::new(
            tiny_skia::Point::from_xy(start.0, start.1),
            start_radius,
            tiny_skia::Point::from_xy(end.0, end.1),
            end_radius,
            stops,
            spread,
            SkTransform::identity(),
        )?;
        Some(Self { shader })
    }

    /// Build a [`Shader`] suitable for assigning to [`Paint::shader`].
    pub fn as_shader(&self) -> Shader<'static> {
        self.shader.clone()
    }
}

/// A simple drop shadow — colour, offset, and (optional) blur radius.
///
/// Note: tiny-skia doesn't ship a Gaussian blur, so the [`Painter`] only
/// honours `offset_x` / `offset_y` and composites a solid offset copy of
/// the shape beneath the original. `blur_radius` is accepted for API
/// parity with JUCE and may be wired to a future blur path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropShadow {
    /// Horizontal offset of the shadow in pixels.
    pub offset_x: f32,
    /// Vertical offset of the shadow in pixels.
    pub offset_y: f32,
    /// Optional blur radius. Currently informational only — see the note
    /// on [`DropShadow`].
    pub blur_radius: f32,
    /// Shadow colour (alpha controls shadow opacity).
    pub color: Color,
}

impl DropShadow {
    /// Build a new drop shadow.
    pub fn new(offset_x: f32, offset_y: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius: 0.0,
            color,
        }
    }

    /// Set the blur radius. Currently informational only — see the note
    /// on [`DropShadow`].
    pub fn with_blur(mut self, radius: f32) -> Self {
        self.blur_radius = radius;
        self
    }
}

impl Default for DropShadow {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 2.0,
            blur_radius: 4.0,
            color: Color::rgba(0, 0, 0, 128),
        }
    }
}

/// A JUCE-style builder for [`Path`] objects.
///
/// thin wrapper around [`SkPathBuilder`] with a chainable API matching
/// JUCE's `Path` interface:
/// [`PathBuilder::move_to`], [`PathBuilder::line_to`],
/// [`PathBuilder::quad_to`], [`PathBuilder::cubic_to`],
/// [`PathBuilder::close`], and [`PathBuilder::start_new_sub_path`].
///
/// Note: tiny-skia has no explicit "subpath" concept — calling
/// [`PathBuilder::move_to`] begins a fresh contour. We expose the JUCE
/// name as an alias for symmetry with the rest of the framework.
#[derive(Clone, Debug, Default)]
pub struct PathBuilder(SkPathBuilder);

impl PathBuilder {
    /// Construct an empty path builder.
    pub fn new() -> Self {
        Self(SkPathBuilder::new())
    }

    /// Move the pen to `(x, y)` without drawing, starting a new contour.
    /// Calling `move_to` while a contour is open begins a new subpath
    /// (matching `juce::Path::startNewSubPath`).
    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.0.move_to(x, y);
        self
    }

    /// Alias for [`PathBuilder::move_to`] — matches JUCE's
    /// `Path::startNewSubPath`.
    pub fn start_new_sub_path(&mut self, x: f32, y: f32) -> &mut Self {
        self.move_to(x, y)
    }

    /// Draw a straight line to `(x, y)`.
    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.0.line_to(x, y);
        self
    }

    /// Draw a quadratic bezier curve to `(x, y)` using `(cx, cy)` as the
    /// single control point.
    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> &mut Self {
        self.0.quad_to(cx, cy, x, y);
        self
    }

    /// Draw a cubic bezier curve to `(x, y)` using `(cx1, cy1)` and
    /// `(cx2, cy2)` as the two control points.
    pub fn cubic_to(
        &mut self,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    ) -> &mut Self {
        self.0.cubic_to(cx1, cy1, cx2, cy2, x, y);
        self
    }

    /// Close the current contour by drawing a line back to the most
    /// recent `move_to`.
    pub fn close(&mut self) -> &mut Self {
        self.0.close();
        self
    }

    /// Finalise the path. Returns `None` if the builder has no contours
    /// (matching `tiny_skia::PathBuilder::finish`).
    pub fn finish(&mut self) -> Option<Path> {
        let inner = std::mem::replace(&mut self.0, SkPathBuilder::new());
        inner.finish()
    }
}

/// A CPU paint target backed by a `tiny_skia::Pixmap`.
///
/// Use [`Painter::new`] to allocate a premultiplied-RGBA8 surface of the
/// given size, then issue draw calls. The pixel buffer is owned by the
/// [`Painter`] and reachable via [`Painter::data`] (premultiplied) or
/// [`Painter::data_straight`] (un-premultiplied).
#[derive(Clone)]
pub struct Painter {
    pixmap: tiny_skia::Pixmap,
    current_color: Color,
}

impl std::fmt::Debug for Painter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Painter")
            .field("width", &self.pixmap.width())
            .field("height", &self.pixmap.height())
            .field("current_color", &self.current_color)
            .finish()
    }
}

impl Painter {
    /// Allocate a new paint surface of the given size. Returns `None` if
    /// either dimension is zero or the allocation fails.
    pub fn new(width: u32, height: u32) -> Option<Self> {
        tiny_skia::Pixmap::new(width, height).map(|pixmap| Self {
            pixmap,
            current_color: Color::rgba(0, 0, 0, 255),
        })
    }

    /// Paint surface width in pixels.
    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    /// Paint surface height in pixels.
    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    /// Raw pixel buffer (premultiplied RGBA8, row-major). Skia's
    /// convention.
    pub fn data(&self) -> &[u8] {
        self.pixmap.data()
    }

    /// Mutable raw pixel buffer (premultiplied RGBA8, row-major).
    pub fn data_mut(&mut self) -> &mut [u8] {
        self.pixmap.data_mut()
    }

    /// Pixel buffer with premultiplication undone, in straight RGBA8
    /// order. This is what [`crate::primitives::Graphics::as_bytes`]
    /// produces; use it when feeding painter output into the
    /// pixel-pushing `Graphics` API or other code that expects straight
    /// alpha.
    pub fn data_straight(&self) -> Vec<u8> {
        unpremultiply_rgba8(self.pixmap.data())
    }

    /// Set the colour used by subsequent fill / stroke operations that
    /// don't supply their own paint or gradient.
    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    /// Current colour used by subsequent fill / stroke operations.
    pub fn current_color(&self) -> Color {
        self.current_color
    }

    /// Fill the entire surface with transparent black.
    pub fn clear(&mut self) {
        self.pixmap.fill(tiny_skia::Color::TRANSPARENT);
    }

    /// Fill the entire surface with the given colour.
    pub fn fill_all(&mut self, color: Color) {
        self.pixmap
            .fill(tiny_skia::Color::from_rgba8(color.r, color.g, color.b, color.a));
    }

    /// Fill an axis-aligned rectangle with the current colour.
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, w, h) {
            let color = self.current_color;
            self.pixmap.fill_rect(rect, &solid_paint(color), SkTransform::identity(), None);
        }
    }

    /// Stroke the outline of an axis-aligned rectangle.
    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32, stroke: &Stroke) {
        let mut builder = SkPathBuilder::new();
        builder.move_to(x, y);
        builder.line_to(x + w, y);
        builder.line_to(x + w, y + h);
        builder.line_to(x, y + h);
        builder.close();
        if let Some(path) = builder.finish() {
            let color = self.current_color;
            self.pixmap.stroke_path(
                &path,
                &solid_paint(color),
                stroke,
                SkTransform::identity(),
                None,
            );
        }
    }

    /// Fill a [`Path`] with the current colour using the given fill
    /// rule.
    pub fn fill_path(&mut self, path: &Path, fill_rule: FillRule) {
        let color = self.current_color;
        self.pixmap
            .fill_path(path, &solid_paint(color), fill_rule, SkTransform::identity(), None);
    }

    /// Stroke a [`Path`] with the current colour and the given stroke
    /// properties.
    pub fn stroke_path(&mut self, path: &Path, stroke: &Stroke) {
        let color = self.current_color;
        self.pixmap
            .stroke_path(path, &solid_paint(color), stroke, SkTransform::identity(), None);
    }

    /// Stroke a [`Path`] using a [`SkTransform`] for positioning /
    /// rotation / scaling.
    pub fn stroke_path_transformed(
        &mut self,
        path: &Path,
        stroke: &Stroke,
        transform: SkTransform,
    ) {
        let color = self.current_color;
        self.pixmap
            .stroke_path(path, &solid_paint(color), stroke, transform, None);
    }

    /// Fill a [`Path`] with the given [`ColourGradient`]. Useful for
    /// linear / radial gradient fills.
    pub fn fill_path_with_gradient(
        &mut self,
        path: &Path,
        gradient: &ColourGradient,
        fill_rule: FillRule,
    ) {
        let mut paint = Paint::default();
        paint.shader = gradient.as_shader();
        paint.anti_alias = true;
        self.pixmap
            .fill_path(path, &paint, fill_rule, SkTransform::identity(), None);
    }

    /// Stroke a [`Path`] with the given [`ColourGradient`].
    pub fn stroke_path_with_gradient(
        &mut self,
        path: &Path,
        gradient: &ColourGradient,
        stroke: &Stroke,
    ) {
        let mut paint = Paint::default();
        paint.shader = gradient.as_shader();
        paint.anti_alias = true;
        self.pixmap
            .stroke_path(path, &paint, stroke, SkTransform::identity(), None);
    }

    /// Fill a [`Path`] with the current colour plus a (non-blurred) drop
    /// shadow beneath it.
    pub fn fill_path_with_shadow(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        shadow: &DropShadow,
    ) {
        // 1. Shadow pass: render the path filled with the shadow colour
        //    at the configured offset, then offset back.
        let mut shadow_paint = Paint::default();
        shadow_paint.shader = Shader::SolidColor(tiny_skia::Color::from_rgba8(
            shadow.color.r,
            shadow.color.g,
            shadow.color.b,
            shadow.color.a,
        ));
        shadow_paint.anti_alias = true;
        let shift = SkTransform::from_translate(shadow.offset_x, shadow.offset_y);
        self.pixmap.fill_path(
            path,
            &shadow_paint,
            fill_rule,
            shift,
            None,
        );
        // 2. Foreground pass on top.
        let color = self.current_color;
        self.pixmap
            .fill_path(path, &solid_paint(color), fill_rule, SkTransform::identity(), None);
    }

}

/// Build a [`Paint`] that fills with a solid colour.
fn solid_paint(color: Color) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.shader = Shader::SolidColor(tiny_skia::Color::from_rgba8(
        color.r, color.g, color.b, color.a,
    ));
    paint.anti_alias = true;
    paint
}

/// Convert our [`Color`] to a tiny-skia color. Both are RGBA u8.
#[cfg(test)]
#[allow(dead_code)]
fn to_sk_color(c: Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(c.r, c.g, c.b, c.a)
}

/// Convert our [`Color`] to a tiny-skia premultiplied color.
#[cfg(test)]
#[allow(dead_code)]
fn to_sk_color_premul(c: Color) -> tiny_skia::PremultipliedColor {
    to_sk_color(c).premultiply()
}

/// Unpremultiply a premultiplied-RGBA8 buffer in place.
///
/// Each pixel `c ∈ [0, 255]` is converted back to straight alpha using
/// `c' = c * 255 / a`, with the convention that `c' = 0` when `a == 0`.
fn unpremultiply_rgba8(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for chunk in input.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];
        if a == 0 {
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            // Round-half-up division (matches Skia's helper).
            let r = ((r as u16 * 255 + (a as u16 / 2)) / a as u16) as u8;
            let g = ((g as u16 * 255 + (a as u16 / 2)) / a as u16) as u8;
            let b = ((b as u16 * 255 + (a as u16 / 2)) / a as u16) as u8;
            out.extend_from_slice(&[r, g, b, a]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_some_for_positive_dimensions() {
        let p = Painter::new(64, 64);
        assert!(p.is_some());
        let p = p.unwrap();
        assert_eq!(p.width(), 64);
        assert_eq!(p.height(), 64);
    }

    #[test]
    fn new_returns_none_for_zero_dimensions() {
        assert!(Painter::new(0, 64).is_none());
        assert!(Painter::new(64, 0).is_none());
        assert!(Painter::new(0, 0).is_none());
    }

    #[test]
    fn clear_yields_transparent_buffer() {
        let mut p = Painter::new(4, 4).unwrap();
        p.clear();
        assert!(p.data().iter().all(|&b| b == 0));
    }

    #[test]
    fn fill_all_sets_every_pixel() {
        let mut p = Painter::new(2, 2).unwrap();
        p.fill_all(Color::rgb(255, 128, 64));
        // Premultiplied: alpha is 255, so the colour passes through.
        for chunk in p.data().chunks_exact(4) {
            assert_eq!(chunk, &[255, 128, 64, 255]);
        }
    }

    #[test]
    fn fill_rect_paints_region() {
        let mut p = Painter::new(10, 10).unwrap();
        p.set_color(Color::rgb(0, 0, 255));
        p.fill_rect(2.0, 3.0, 4.0, 5.0);
        // The top-left pixel of the rect should now be blue.
        let stride = p.width() as usize * 4;
        let idx = (3 * stride) + (2 * 4);
        assert_eq!(&p.data()[idx..idx + 4], &[0, 0, 255, 255]);
    }

    #[test]
    fn fill_rect_ignores_zero_size() {
        let mut p = Painter::new(4, 4).unwrap();
        // Should not panic or fill anything.
        p.fill_rect(0.0, 0.0, 0.0, 4.0);
        p.fill_rect(0.0, 0.0, 4.0, 0.0);
    }

    #[test]
    fn path_builder_builds_rectangle() {
        let path = PathBuilder::new()
            .move_to(0.0, 0.0)
            .line_to(10.0, 0.0)
            .line_to(10.0, 10.0)
            .line_to(0.0, 10.0)
            .close()
            .finish();
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.bounds().width() > 0.0 && path.bounds().height() > 0.0);
    }

    #[test]
    fn empty_builder_returns_none() {
        let path = PathBuilder::new().finish();
        assert!(path.is_none());
    }

    #[test]
    fn start_new_sub_path_starts_fresh_contour() {
        // Two disjoint rectangles in one path.
        let path = PathBuilder::new()
            .move_to(0.0, 0.0)
            .line_to(1.0, 0.0)
            .line_to(1.0, 1.0)
            .close()
            .start_new_sub_path(10.0, 10.0)
            .line_to(11.0, 10.0)
            .line_to(11.0, 11.0)
            .close()
            .finish();
        assert!(path.is_some());
    }

    #[test]
    fn fill_path_paints_under_current_color() {
        let mut p = Painter::new(20, 20).unwrap();
        p.set_color(Color::rgb(0, 200, 0));
        let path = PathBuilder::new()
            .move_to(2.0, 2.0)
            .line_to(18.0, 2.0)
            .line_to(18.0, 18.0)
            .line_to(2.0, 18.0)
            .close()
            .finish()
            .unwrap();
        p.fill_path(&path, FillRule::Winding);
        // Sample a pixel that should be inside the path.
        let stride = p.width() as usize * 4;
        let idx = (10 * stride) + (10 * 4);
        assert_eq!(&p.data()[idx..idx + 4], &[0, 200, 0, 255]);
    }

    #[test]
    fn fill_path_with_gradient_renders_stops() {
        let mut p = Painter::new(20, 20).unwrap();
        let gradient = ColourGradient::linear(
            (0.0, 0.0),
            (20.0, 0.0),
            vec![
                GradientStop::new(0.0, to_sk_color(Color::rgb(255, 0, 0))),
                GradientStop::new(1.0, to_sk_color(Color::rgb(0, 0, 255))),
            ],
            SpreadMode::Pad,
        )
        .unwrap();
        let path = PathBuilder::new()
            .move_to(0.0, 0.0)
            .line_to(20.0, 0.0)
            .line_to(20.0, 20.0)
            .line_to(0.0, 20.0)
            .close()
            .finish()
            .unwrap();
        p.fill_path_with_gradient(&path, &gradient, FillRule::Winding);
        // The left edge should be reddish, the right edge bluish.
        let stride = p.width() as usize * 4;
        let left = &p.data()[(10 * stride)..(10 * stride + 4)].to_vec();
        let right = &p.data()[(10 * stride) + (19 * 4)..(10 * stride) + (19 * 4) + 4].to_vec();
        assert!(left[0] > left[2], "left side should be redder: {:?}", left);
        assert!(right[2] > right[0], "right side should be bluer: {:?}", right);
    }

    #[test]
    fn stroke_path_paints_outline() {
        let mut p = Painter::new(20, 20).unwrap();
        p.set_color(Color::rgb(255, 0, 0));
        let path = PathBuilder::new()
            .move_to(2.0, 2.0)
            .line_to(18.0, 2.0)
            .line_to(18.0, 18.0)
            .line_to(2.0, 18.0)
            .close()
            .finish()
            .unwrap();
        let stroke = Stroke::default();
        p.stroke_path(&path, &stroke);
        // Anti-aliased stroke of a (2,2)..(18,18) rect should produce
        // red pixels along the top edge. Sample a range to allow for
        // anti-aliasing sub-pixel shifts.
        let stride = p.width() as usize * 4;
        let mut red_sum = 0u32;
        for x in 2..18 {
            let idx = (2 * stride) + (x * 4);
            red_sum += p.data()[idx] as u32; // R channel
        }
        assert!(red_sum > 255 * 8, "expected substantial red along top edge, got sum={}", red_sum);
    }

    #[test]
    fn fill_path_with_shadow_offsets_under_shape() {
        let mut p = Painter::new(20, 20).unwrap();
        p.set_color(Color::rgb(255, 255, 255));
        let path = PathBuilder::new()
            .move_to(2.0, 2.0)
            .line_to(10.0, 2.0)
            .line_to(10.0, 10.0)
            .line_to(2.0, 10.0)
            .close()
            .finish()
            .unwrap();
        let shadow = DropShadow::new(3.0, 3.0, Color::rgba(255, 0, 0, 255));
        p.fill_path_with_shadow(&path, FillRule::Winding, &shadow);
        // The pixel at (5, 5) is inside the foreground (white); the pixel
        // at (2+3+8, 2+3+8) = (13, 13) is shadow-only (red) because the
        // foreground only covers (2,2)..(10,10) while the shadow is
        // offset by (3,3), covering (5,5)..(13,13).
        let stride = p.width() as usize * 4;
        let foreground = &p.data()[(5 * stride) + (5 * 4)..(5 * stride) + (5 * 4) + 4].to_vec();
        let shadow_only = &p.data()[(11 * stride) + (11 * 4)..(11 * stride) + (11 * 4) + 4].to_vec();
        assert!(foreground[0] >= 250 && foreground[1] >= 250 && foreground[2] >= 250);
        assert!(shadow_only[0] >= 250 && shadow_only[1] <= 5 && shadow_only[2] <= 5);
    }

    #[test]
    fn justification_flags_compose() {
        let j = Justification::HORIZONTALLY_CENTERED | Justification::VERTICALLY_CENTERED;
        assert!(j.contains(Justification::CENTERED));
        let j2 = Justification::LEFT | Justification::TOP;
        assert!(j2.contains(Justification::LEFT));
        assert!(!j2.contains(Justification::RIGHT));
    }

    #[test]
    fn fill_type_default_is_fill() {
        assert_eq!(FillType::default(), FillType::Fill);
    }

    #[test]
    fn drop_shadow_default_is_dark_offset_down() {
        let s = DropShadow::default();
        assert_eq!(s.offset_x, 0.0);
        assert_eq!(s.offset_y, 2.0);
        assert_eq!(s.color, Color::rgba(0, 0, 0, 128));
    }

    #[test]
    fn colour_gradient_linear_returns_some_for_valid_stops() {
        let g = ColourGradient::linear(
            (0.0, 0.0),
            (10.0, 0.0),
            vec![
                GradientStop::new(0.0, to_sk_color(Color::rgb(0, 0, 0))),
                GradientStop::new(1.0, to_sk_color(Color::rgb(255, 255, 255))),
            ],
            SpreadMode::Pad,
        );
        assert!(g.is_some());
    }

    #[test]
    fn colour_gradient_linear_single_stop_returns_solid() {
        // tiny-skia collapses a single-stop gradient to SolidColor.
        let g = ColourGradient::linear(
            (0.0, 0.0),
            (10.0, 0.0),
            vec![GradientStop::new(0.5, to_sk_color(Color::rgb(128, 64, 32)))],
            SpreadMode::Pad,
        );
        assert!(g.is_some());
    }

    #[test]
    fn colour_gradient_radial_returns_some_for_valid_stops() {
        let g = ColourGradient::radial(
            (5.0, 5.0),
            0.0,
            (5.0, 5.0),
            5.0,
            vec![
                GradientStop::new(0.0, to_sk_color(Color::rgb(255, 255, 255))),
                GradientStop::new(1.0, to_sk_color(Color::rgb(0, 0, 0))),
            ],
            SpreadMode::Pad,
        );
        assert!(g.is_some());
    }

    #[test]
    fn data_straight_unpremultiplies_alpha() {
        let mut p = Painter::new(1, 1).unwrap();
        p.fill_all(Color::rgba(255, 0, 0, 128));
        let premul = p.data();
        // Premultiplied: r = 255 * 128 / 255 = 128
        assert_eq!(premul, &[128, 0, 0, 128]);
        let straight = p.data_straight();
        // Un-premultiplied: r = 128 * 255 / 128 = 255
        assert_eq!(straight, &[255, 0, 0, 128]);
    }
}