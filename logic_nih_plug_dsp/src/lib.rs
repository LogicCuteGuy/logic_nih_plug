//! # logic_nih_plug_dsp
//!
//! Digital signal processing algorithms ported from JUCE.
//!
//! This crate provides pure Rust implementations of common DSP algorithms:
//!
//! - **Filters**: IIR and FIR filters
//! - **Oscillators**: Sine, saw, square, triangle waveforms
//! - **Convolution**: FFT-based convolution for reverb
//! - **Envelopes**: ADSR envelope generators
//! - **Smoothing**: Parameter smoothing utilities
//!
//! ## Examples
//!
//! See the `examples/` directory for complete plugin examples.

#![warn(missing_docs)]

pub mod error;
pub mod util;

#[cfg(feature = "filters")]
pub mod filters;

#[cfg(feature = "filters")]
pub mod fir;

#[cfg(feature = "filters")]
pub mod state_variable;

#[cfg(feature = "oscillators")]
pub mod oscillators;

#[cfg(feature = "convolution")]
pub mod convolution;

#[cfg(feature = "envelopes")]
pub mod envelopes;

#[cfg(feature = "smoothing")]
pub mod smoothing;

#[cfg(feature = "processors")]
pub mod processors;

#[cfg(feature = "analysis")]
pub mod analysis;

#[cfg(feature = "simd")]
pub mod simd;

pub use error::DspError;
