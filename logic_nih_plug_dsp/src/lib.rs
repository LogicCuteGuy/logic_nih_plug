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
//! - **Dynamics**: Compressor, Limiter, NoiseGate, LookaheadLimiter
//! - **Reverb**: FreeVerb-style algorithmic reverb (Schroeder/Moorer)
//! - **Delay**: `DelayLine` (fractional, pluggable interpolation) and a
//!   feedback `Delay` effect (tempo-synced, ping-pong)
//! - **Modulation**: `Phaser`, `Chorus`, `WahWah`, `LadderFilter`
//! - **Mixer**: `Panner` (7 pan laws), `DryWetMixer` (7 mixing rules), `Gain`
//! - **Pitch**: `PhaseVocoder`, `PitchShift`, `TimeStretching`
//! - **Resampling**: `GenericInterpolator` with `ZeroOrderHold`, `Linear`, `CatmullRom`, `Lagrange`, `WindowedSinc`
//! - **Analysis**: `FFT` / `RealFFT` (real-only FFT path) and
//!   `STFT` (short-time Fourier transform with forward/inverse
//!   helpers), plus `WindowingFunction` (Hann, Hamming, Blackman,
//!   Kaiser, etc.) and metering/analysis tools (`LevelMeter`,
//!   `Follower`, `LoudnessMeter`, `Oscilloscope`)
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
