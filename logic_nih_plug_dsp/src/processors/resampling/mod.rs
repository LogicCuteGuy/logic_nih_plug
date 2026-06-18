//! Sample-rate conversion and fractional-delay interpolation.
//!
//! This module provides five interpolation strategies matching
//! `juce::dsp::Interpolators` — from the cheapest (zero-order hold)
//! to the highest quality (windowed sinc).  Each strategy is wrapped
//! by [`GenericInterpolator`], which manages a circular sample buffer
//! and the fractional-position bookkeeping.
//!
//! # Interpolation strategies
//!
//! | Strategy | Latency (samples) | Circular buffer | Quality |
//! |---|---|---|---|
//! | [`ZeroOrderHold`] | 0 | 1 | Lo-fi (staircase) |
//! | [`Linear`] | 1 | 2 | Good for fast modulation |
//! | [`CatmullRom`] | 2 | 4 | Smooth cubic spline |
//! | [`Lagrange`] | 2 | 5 | 4th-order polynomial |
//! | [`WindowedSinc`] | 100 | 200 | Highest quality, Hann-windowed sinc |
//!
//! # Usage
//!
//! ```rust
//! use logic_nih_plug_dsp::processors::resampling::*;
//!
//! // Create one interpolator per channel
//! let mut interp = GenericInterpolator::<Linear>::new();
//!
//! // Resample: speed_ratio = 1.0 = same rate, 0.5 = half speed, 2.0 = double speed
//! let input = [0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
//! let mut output = [0.0f32; 16];
//! interp.process(0.5, &input, &mut output);
//! ```
//!
//! See also: the [`DelayLine`](super::delay::DelayLine) in the delay module,
//! which has its own interpolation traits tuned for fractional-delay reading.

mod generic;
mod zero_order_hold;
mod linear;
mod catmull_rom;
mod lagrange;
mod windowed_sinc;

pub use generic::{GenericInterpolator, Interpolator};
pub use zero_order_hold::ZeroOrderHold;
pub use linear::Linear;
pub use catmull_rom::CatmullRom;
pub use lagrange::Lagrange;
pub use windowed_sinc::WindowedSinc;
