//! Common types and helpers for dynamics processors.
//!
//! This module provides the [`ProcessSpec`] struct (a Rust equivalent of
//! JUCE's `juce::dsp::ProcessSpec`) plus shared decibel conversion helpers
//! used by [`crate::processors::compressor::Compressor`],
//! [`crate::processors::noise_gate::NoiseGate`],
//! [`crate::processors::limiter::Limiter`] and
//! [`crate::processors::lookahead_limiter::LookaheadLimiter`].
//!
//! All algorithms here mirror the JUCE `dsp::Dynamics` and
//! `dsp::BallisticsFilter` reference implementations but are written
//! idiomatic Rust with `f32` sample type.

use crate::error::DspError;

/// Specification passed to a dynamics processor's [`prepare`](Processor::prepare) call.
///
/// This is the Rust analogue of JUCE's `juce::dsp::ProcessSpec`. It bundles
/// the three pieces of information that every dynamics processor needs:
/// the sample rate, the maximum number of channels it will be asked to
/// process, and the maximum block size it should expect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProcessSpec {
    /// Sample rate in Hz (must be > 0).
    pub sample_rate: f32,
    /// Number of channels that will be processed (must be > 0).
    pub num_channels: usize,
    /// Maximum block size that will be processed (must be > 0).
    pub maximum_block_size: usize,
}

impl ProcessSpec {
    /// Constructs a new `ProcessSpec`.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz
    /// * `num_channels` - Number of channels
    /// * `maximum_block_size` - Maximum block size in samples
    pub fn new(sample_rate: f32, num_channels: usize, maximum_block_size: usize) -> Self {
        Self {
            sample_rate,
            num_channels,
            maximum_block_size,
        }
    }

    /// Validates that all fields are within acceptable ranges.
    pub fn validate(&self) -> Result<(), DspError> {
        crate::util::validate_sample_rate(self.sample_rate)?;
        if self.num_channels == 0 {
            return Err(DspError::InvalidBufferSize(0));
        }
        crate::util::validate_buffer_size(self.maximum_block_size)?;
        Ok(())
    }
}

impl Default for ProcessSpec {
    fn default() -> Self {
        Self {
            sample_rate: 44100.0,
            num_channels: 1,
            maximum_block_size: 512,
        }
    }
}

/// Converts a decibel value to a linear gain factor.
///
/// Uses the standard amplitude conversion `linear = 10^(dB / 20)`.
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Converts a linear gain factor to a decibel value.
///
/// Uses `dB = 20 * log10(linear)`. Returns `-200.0` for non-positive input
/// (mirroring JUCE's `Decibels::gainToDecibels` which floors at -100 dB;
/// we use -200 dB to match JUCE's `Compressor::update` floor).
#[inline]
pub fn linear_to_db(linear: f32) -> f32 {
    if linear > 0.0 {
        20.0 * linear.log10()
    } else {
        -200.0
    }
}

/// Convenience: same as [`db_to_linear`] but with a floor to avoid `-Inf`.
#[inline]
pub fn db_to_gain(db: f32, min_db: f32) -> f32 {
    db_to_linear(db.max(min_db))
}

/// Returns `true` if the value is a sub-normal float. JUCE uses this to
/// "snap to zero" denormals on the envelope state in
/// `BallisticsFilter::snapToZero`.
#[inline]
pub fn is_denormal(x: f32) -> bool {
    x.is_subnormal()
}

/// Forces a denormal value to zero. Used by
/// [`crate::processors::ballistics_filter::BallisticsFilter::snap_to_zero`].
#[inline]
pub fn snap_to_zero(x: &mut f32) {
    if is_denormal(*x) {
        *x = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_roundtrip_is_stable() {
        for db in [-60.0_f32, -12.0, -6.0, 0.0, 6.0, 12.0, 60.0] {
            let linear = db_to_linear(db);
            let back = linear_to_db(linear);
            assert!((db - back).abs() < 1e-3, "db={db}, back={back}");
        }
    }

    #[test]
    fn db_to_gain_floors() {
        assert!((db_to_gain(-1000.0, -200.0) - db_to_linear(-200.0)).abs() < 1e-6);
    }

    #[test]
    fn linear_to_db_handles_non_positive() {
        assert_eq!(linear_to_db(0.0), -200.0);
        assert_eq!(linear_to_db(-1.0), -200.0);
    }

    #[test]
    fn process_spec_validates() {
        assert!(ProcessSpec::new(44100.0, 2, 512).validate().is_ok());
        assert!(ProcessSpec::new(0.0, 1, 512).validate().is_err());
        assert!(ProcessSpec::new(44100.0, 0, 512).validate().is_err());
        assert!(ProcessSpec::new(44100.0, 1, 0).validate().is_err());
    }

    #[test]
    fn process_spec_default_is_sane() {
        let spec = ProcessSpec::default();
        assert_eq!(spec.sample_rate, 44100.0);
        assert_eq!(spec.num_channels, 1);
        assert_eq!(spec.maximum_block_size, 512);
    }

    #[test]
    fn snap_to_zero_clears_denormal() {
        let mut x: f32 = 1.0e-40; // denormal
        snap_to_zero(&mut x);
        assert_eq!(x, 0.0);
    }

    #[test]
    fn snap_to_zero_preserves_normal() {
        let mut x: f32 = 0.5;
        snap_to_zero(&mut x);
        assert_eq!(x, 0.5);
    }
}
