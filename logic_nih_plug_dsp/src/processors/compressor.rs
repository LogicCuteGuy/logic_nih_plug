//! Standard feed-forward compressor (JUCE `dsp::Compressor`).
//!
//! [`Compressor`] is a standard feed-forward compressor with threshold,
//! ratio, attack time and release time controls. It is the building block
//! used by [`crate::processors::limiter::Limiter`].
//!
//! ## Algorithm
//!
//! On each sample:
//!
//! 1. The signal's envelope is tracked by a
//!    [`BallisticsFilter`](crate::processors::ballistics_filter::BallisticsFilter)
//!    in peak-rectifier mode (one envelope per channel).
//! 2. When the envelope is below the threshold, the VCA passes the input
//!    unchanged.
//! 3. When the envelope is at or above the threshold, the VCA gain is
//!    `pow(env · threshold⁻¹, 1/ratio - 1)` — exactly the soft-knee
//!    formula used by `juce::dsp::Compressor::processSample`.

use crate::processors::ballistics_filter::BallisticsFilter;
use crate::processors::dynamics::{db_to_gain, ProcessSpec};
use crate::processors::Processor;

/// A standard feed-forward compressor.
///
/// The compressor maintains a separate envelope follower per channel.
/// Call [`prepare_with_channels`](Self::prepare_with_channels) (or
/// [`prepare_spec`](Self::prepare_spec)) before processing.
#[derive(Debug, Clone)]
pub struct Compressor {
    envelope: BallisticsFilter,
    /// Linear threshold (set from `threshold_db`).
    threshold: f32,
    /// `1.0 / threshold`, precomputed for the gain formula.
    threshold_inverse: f32,
    /// `1.0 / ratio`, precomputed for the gain formula.
    ratio_inverse: f32,
    sample_rate: f32,
    threshold_db: f32,
    ratio: f32,
    attack_time: f32,
    release_time: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        let mut s = Self {
            envelope: BallisticsFilter::new(),
            threshold: db_to_gain(0.0, -200.0),
            threshold_inverse: 1.0 / db_to_gain(0.0, -200.0),
            ratio_inverse: 1.0,
            sample_rate: 44100.0,
            threshold_db: 0.0,
            ratio: 1.0,
            attack_time: 1.0,
            release_time: 100.0,
        };
        s.update();
        s
    }
}

impl Compressor {
    /// Creates a new compressor with defaults: 0 dB threshold, 1:1 ratio,
    /// 1 ms attack, 100 ms release.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the threshold in decibels. Any signal above this level is
    /// reduced by the configured ratio.
    pub fn set_threshold(&mut self, new_threshold_db: f32) {
        self.threshold_db = new_threshold_db;
        self.update();
    }

    /// Sets the compression ratio (must be ≥ 1.0). A ratio of `1.0` makes
    /// the compressor a no-op; `4.0` means each additional 4 dB above the
    /// threshold only raises the output by 1 dB.
    ///
    /// # Panics
    ///
    /// Panics if `new_ratio < 1.0`.
    pub fn set_ratio(&mut self, new_ratio: f32) {
        assert!(
            new_ratio >= 1.0,
            "compressor ratio must be >= 1.0 (got {new_ratio})"
        );
        self.ratio = new_ratio;
        self.update();
    }

    /// Sets the attack time in milliseconds.
    pub fn set_attack(&mut self, new_attack_ms: f32) {
        self.attack_time = new_attack_ms;
        self.update();
    }

    /// Sets the release time in milliseconds.
    pub fn set_release(&mut self, new_release_ms: f32) {
        self.release_time = new_release_ms;
        self.update();
    }

    /// Returns the current threshold in dB.
    #[allow(clippy::misnamed_getters)]
    pub fn threshold(&self) -> f32 {
        self.threshold_db
    }

    /// Returns the current compression ratio.
    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    /// Returns the attack time in milliseconds.
    pub fn attack_time(&self) -> f32 {
        self.attack_time
    }

    /// Returns the release time in milliseconds.
    pub fn release_time(&self) -> f32 {
        self.release_time
    }

    /// Prepares the compressor for one channel of processing.
    pub fn prepare(&mut self, sample_rate: f32) {
        self.prepare_with_channels(sample_rate, 1);
    }

    /// Prepares the compressor for the given number of channels.
    pub fn prepare_with_channels(&mut self, sample_rate: f32, num_channels: usize) {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(num_channels > 0, "num_channels must be > 0");
        self.sample_rate = sample_rate;
        self.envelope.prepare_with_channels(sample_rate, num_channels);
        self.update();
        self.reset();
    }

    /// Convenience that takes a [`ProcessSpec`].
    pub fn prepare_spec(&mut self, spec: ProcessSpec) {
        self.prepare_with_channels(spec.sample_rate, spec.num_channels);
    }

    /// Resets all per-channel envelope state.
    pub fn reset(&mut self) {
        self.envelope.reset();
    }

    /// Forces denormal envelope state to zero.
    pub fn snap_to_zero(&mut self) {
        self.envelope.snap_to_zero();
    }

    /// Processes a single sample on `channel`.
    ///
    /// Mirrors `juce::dsp::Compressor<SampleType>::processSample` exactly:
    /// the envelope is computed first, then a gain is computed from the
    /// difference between the envelope and the threshold.
    pub fn process_sample(&mut self, channel: usize, input: f32) -> f32 {
        let env = self.envelope.process_sample(channel, input);
        let gain = if env < self.threshold {
            1.0
        } else {
            // gain = (env · threshold⁻¹)^(1/ratio - 1)
            (env * self.threshold_inverse).powf(self.ratio_inverse - 1.0)
        };
        gain * input
    }

    /// Processes a whole block on a single channel.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        for (i, (&x, y)) in input.iter().zip(output.iter_mut()).enumerate() {
            *y = self.process_sample(0, x);
            // Mirror JUCE: the Processor-trait path is mono-only, but we
            // accept any length so the dyn Processor trait works.
            let _ = i;
        }
    }

    fn update(&mut self) {
        self.threshold = db_to_gain(self.threshold_db, -200.0);
        self.threshold_inverse = if self.threshold > 0.0 {
            1.0 / self.threshold
        } else {
            0.0
        };
        self.ratio_inverse = 1.0 / self.ratio;
        self.envelope.set_attack_time(self.attack_time);
        self.envelope.set_release_time(self.release_time);
    }
}

impl Processor for Compressor {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate);
        let _ = max_block_size;
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        Compressor::process(self, input, output);
    }

    fn reset(&mut self) {
        Compressor::reset(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_ratio_is_a_no_op() {
        let mut c = Compressor::new();
        c.set_ratio(1.0);
        c.set_threshold(-6.0);
        c.prepare(44100.0);
        // Anything above the threshold with a 1:1 ratio must come through
        // untouched.
        let input = vec![0.5_f32; 64];
        let mut output = vec![0.0_f32; 64];
        c.process(&input, &mut output);
        for (x, y) in input.iter().zip(output.iter()) {
            assert!(
                (x - y).abs() < 1e-3,
                "1:1 ratio should pass signal through: x={x}, y={y}"
            );
        }
    }

    #[test]
    fn below_threshold_is_unity_gain() {
        let mut c = Compressor::new();
        c.set_ratio(4.0);
        c.set_threshold(-6.0);
        c.prepare(44100.0);
        // -12 dB amplitude (well below the -6 dB threshold).
        let input = vec![0.25_f32; 256];
        let mut output = vec![0.0_f32; 256];
        c.process(&input, &mut output);
        for (x, y) in input.iter().zip(output.iter()) {
            assert!(
                (x - y).abs() < 1e-4,
                "below-threshold signal should be unchanged: x={x}, y={y}"
            );
        }
    }

    #[test]
    fn above_threshold_compresses() {
        let mut c = Compressor::new();
        c.set_threshold(-6.0);
        c.set_ratio(4.0);
        // Disable envelope smoothing so the steady-state is reached
        // immediately.
        c.set_attack(0.0);
        c.set_release(0.0);
        c.prepare(44100.0);

        // 0 dB amplitude (well above -6 dB threshold).
        // With a 4:1 ratio and a -6 dB threshold: gain should be
        //   pow(1.0 / 0.5012, 1/4 - 1) ≈ 0.7079  (-3 dB)
        let mut last = 0.0_f32;
        for _ in 0..1024 {
            last = c.process_sample(0, 1.0);
        }
        // 6 dB above threshold compressed 4:1 should produce 6/4 = 1.5 dB
        // above threshold — i.e. output gain ≈ 10^(-4.5/20) ≈ 0.5957
        // (this is the steady-state formula derived from the gain curve).
        assert!(
            (last - 0.5957).abs() < 0.01,
            "expected ~-4.5 dB gain, got {last}"
        );
    }

    #[test]
    fn ratio_panics_when_below_one() {
        let mut c = Compressor::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            c.set_ratio(0.5);
        }));
        assert!(result.is_err(), "ratio < 1.0 must panic");
    }

    #[test]
    fn setters_round_trip() {
        let mut c = Compressor::new();
        c.set_threshold(-12.0);
        c.set_ratio(8.0);
        c.set_attack(5.0);
        c.set_release(250.0);
        assert_eq!(c.threshold(), -12.0);
        assert_eq!(c.ratio(), 8.0);
        assert_eq!(c.attack_time(), 5.0);
        assert_eq!(c.release_time(), 250.0);
    }

    #[test]
    fn reset_clears_envelope_state() {
        let mut c = Compressor::new();
        c.prepare_with_channels(44100.0, 2);
        c.process_sample(0, 0.9);
        c.process_sample(1, 0.7);
        c.reset();
        // After reset, the envelope is zero — the first sample should
        // pass through at unity gain (env < threshold).
        assert!((c.process_sample(0, 0.5) - 0.5).abs() < 1e-4);
        assert!((c.process_sample(1, 0.5) - 0.5).abs() < 1e-4);
    }
}
