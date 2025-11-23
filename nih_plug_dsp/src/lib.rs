//! # nih_plug_dsp
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

#[cfg(feature = "oscillators")]
pub mod oscillators;

#[cfg(feature = "convolution")]
pub mod convolution;

#[cfg(feature = "envelopes")]
pub mod envelopes;

#[cfg(feature = "smoothing")]
pub mod smoothing;

pub use error::DspError;
