//! # logic_nih_plug_graphics
//!
//! 2D graphics primitives ported from JUCE.
//!
//! This crate provides:
//!
//! - **Primitives**: Rectangle, line, circle drawing
//! - **Images**: PNG, JPEG, GIF loading and rendering
//! - **Text**: Font rendering and text layout
//!
//! ## Examples
//!
//! ```
//! use logic_nih_plug_graphics::{Graphics, Color};
//!
//! let mut graphics = Graphics::new(800, 600).unwrap();
//! graphics.set_color(Color::rgb(255, 0, 0));
//! graphics.fill_rect(10, 10, 100, 100);
//! ```

#![warn(missing_docs)]

pub mod error;
pub mod color;
pub mod transform;

#[cfg(feature = "primitives")]
pub mod primitives;

#[cfg(feature = "images")]
pub mod images;

#[cfg(feature = "text")]
pub mod text;

#[cfg(feature = "vector")]
pub mod vector;

pub use error::GraphicsError;
pub use color::Color;
pub use transform::Transform;

#[cfg(feature = "primitives")]
pub use primitives::Graphics;

#[cfg(feature = "images")]
pub use images::{Image, ImageConvolutionEngine, RescaleFilter};

#[cfg(feature = "text")]
pub use text::{Font, FontSettings, GlyphArrangement, LineSpacing, PositionedGlyph};

// Re-export the most common vector types at the crate root for ergonomic
// `use logic_nih_plug_graphics::Path` rather than
// `use logic_nih_plug_graphics::vector::Path`.
#[cfg(feature = "vector")]
pub use vector::{
    ColourGradient, DropShadow, FillType, GradientStop, Justification, Paint, Path, PathBuilder,
    Painter, Shader, SpreadMode, Stroke,
};
