//! Audio processors for signal manipulation.
//!
//! This module contains various audio processors that can be chained together
//! to create complex audio processing pipelines.

pub mod bias;
pub mod chain;
pub mod dc_filter;
pub mod gain;
pub mod waveshaper;

#[cfg(feature = "dynamics")]
pub mod ballistics_filter;
#[cfg(feature = "dynamics")]
pub mod compressor;
#[cfg(feature = "dynamics")]
pub mod dynamics;
#[cfg(feature = "dynamics")]
pub mod limiter;
#[cfg(feature = "dynamics")]
pub mod lookahead_limiter;
#[cfg(feature = "dynamics")]
pub mod noise_gate;

#[cfg(feature = "reverb")]
pub mod reverb;

#[cfg(feature = "delay")]
pub mod delay;

#[cfg(feature = "modulation")]
pub mod chorus;
#[cfg(feature = "modulation")]
pub mod ladder_filter;
#[cfg(feature = "modulation")]
pub mod phaser;
#[cfg(feature = "modulation")]
pub mod wahwah;

#[cfg(feature = "mixer")]
pub mod dry_wet;
#[cfg(feature = "mixer")]
pub mod panner;

#[cfg(feature = "pitch")]
pub mod pitch;

#[cfg(feature = "resampling")]
pub mod resampling;

// Re-export the Processor trait for convenience
pub use chain::Processor;
