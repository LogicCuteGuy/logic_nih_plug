//! Standard downward expander / noise gate (JUCE `dsp::NoiseGate`).
//!
//! [`NoiseGate`] attenuates the input whenever its RMS envelope drops below
//! a threshold. With low ratios it can be used as a gentle expander; with
//! high ratios it acts as a hard noise gate.
//!
//! ## Algorithm
//!
//! The JUCE implementation uses two [`BallisticsFilter`]s per channel:
//!
//! 1. An RMS-calculating filter (`attack = 0`, `release = 50 ms`) converts
//!    the raw input into a short-term RMS level.
//! 2. The attack/release envelope filter (driven by the configured
//!    `attack`/`release`) smooths the RMS level further.
//! 3. When the envelope is **above** the threshold, the signal passes
//!    through unchanged. When it is **below**, the gain is
//!    `pow(env · threshold⁻¹, ratio - 1)` — exactly mirroring
//!    `juce::dsp::NoiseGate::processSample`.

use crate::processors::ballistics_filter::{BallisticsFilter, LevelCalculationType};
use crate::processors::dynamics::{db_to_gain, ProcessSpec};
use crate::processors::Processor;

/// A downward expander / noise gate.
///
/// One RMS filter and one attack/release envelope filter are kept per
/// channel. Call [`prepare_with_channels`](Self::prepare_with_channels)
/// before processing.
#[derive(Debug, Clone)]
pub struct NoiseGate {
    rms_filter: BallisticsFilter,
    envelope: BallisticsFilter,
    threshold: f32,
    threshold_inverse: f32,
    current_ratio: f32,
    sample_rate: f32,
    threshold_db: f32,
    ratio: f32,
    attack_time: f32,
    release_time: f32,
}

impl Default for NoiseGate {
    fn default() -> Self {
        let mut s = Self {
            rms_filter: BallisticsFilter::new_rms(),
            envelope: BallisticsFilter::new(),
            threshold: db_to_gain(-100.0, -200.0),
            threshold_inverse: 1.0 / db_to_gain(-100.0, -200.0),
            current_ratio: 10.0,
            sample_rate: 44100.0,
            threshold_db: -100.0,
            ratio: 10.0,
            attack_time: 1.0,
            release_time: 100.0,
        };
        // JUCE: the RMS pre-filter is permanently 0 ms attack / 50 ms release.
        s.rms_filter.set_attack_time(0.0);
        s.rms_filter.set_release_time(50.0);
        s.update();
        s
    }
}

impl NoiseGate {
    /// Creates a new noise gate with defaults: -100 dB threshold, 10:1 ratio,
    /// 1 ms attack, 100 ms release.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the gating threshold in decibels. Signal whose RMS envelope is
    /// below this level is attenuated by the configured ratio.
    pub fn set_threshold(&mut self, new_threshold_db: f32) {
        self.threshold_db = new_threshold_db;
        self.update();
    }

    /// Sets the gating ratio (must be ≥ 1.0). A ratio of `1.0` is a no-op;
    /// higher ratios give a deeper gate / expander.
    ///
    /// # Panics
    ///
    /// Panics if `new_ratio < 1.0`.
    pub fn set_ratio(&mut self, new_ratio: f32) {
        assert!(
            new_ratio >= 1.0,
            "noise gate ratio must be >= 1.0 (got {new_ratio})"
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

    /// Returns the current ratio.
    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    /// Returns the current attack time in milliseconds.
    pub fn attack_time(&self) -> f32 {
        self.attack_time
    }

    /// Returns the current release time in milliseconds.
    pub fn release_time(&self) -> f32 {
        self.release_time
    }

    /// Prepares the noise gate for one channel.
    pub fn prepare(&mut self, sample_rate: f32) {
        self.prepare_with_channels(sample_rate, 1);
    }

    /// Prepares the noise gate for the given number of channels.
    pub fn prepare_with_channels(&mut self, sample_rate: f32, num_channels: usize) {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(num_channels > 0, "num_channels must be > 0");
        self.sample_rate = sample_rate;
        self.rms_filter.prepare_with_channels(sample_rate, num_channels);
        self.envelope.prepare_with_channels(sample_rate, num_channels);
        self.update();
        self.reset();
    }

    /// Convenience that takes a [`ProcessSpec`].
    pub fn prepare_spec(&mut self, spec: ProcessSpec) {
        self.prepare_with_channels(spec.sample_rate, spec.num_channels);
    }

    /// Resets all per-channel envelope and RMS state.
    pub fn reset(&mut self) {
        self.rms_filter.reset();
        self.envelope.reset();
    }

    /// Forces denormal envelope state to zero.
    pub fn snap_to_zero(&mut self) {
        self.rms_filter.snap_to_zero();
        self.envelope.snap_to_zero();
    }

    /// Returns the [`LevelCalculationType`] of the RMS pre-filter.
    pub fn level_type(&self) -> LevelCalculationType {
        self.rms_filter.level_calculation_type()
    }

    /// Processes a single sample on `channel`.
    ///
    /// Mirrors `juce::dsp::NoiseGate<SampleType>::processSample` exactly:
    /// RMS is computed, smoothed, and a downward VCA is applied whenever
    /// the envelope falls below the threshold.
    pub fn process_sample(&mut self, channel: usize, input: f32) -> f32 {
        let mut env = self.rms_filter.process_sample(channel, input);
        env = self.envelope.process_sample(channel, env);

        let gain = if env > self.threshold {
            1.0
        } else {
            // gain = (env · threshold⁻¹)^(ratio - 1)
            (env * self.threshold_inverse).powf(self.current_ratio - 1.0)
        };
        gain * input
    }

    /// Processes a whole block on a single channel.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        for (i, (&x, y)) in input.iter().zip(output.iter_mut()).enumerate() {
            *y = self.process_sample(0, x);
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
        self.current_ratio = self.ratio;
        self.envelope.set_attack_time(self.attack_time);
        self.envelope.set_release_time(self.release_time);
    }
}

impl Processor for NoiseGate {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate);
        let _ = max_block_size;
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        NoiseGate::process(self, input, output);
    }

    fn reset(&mut self) {
        NoiseGate::reset(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_filter_uses_rms_mode() {
        let g = NoiseGate::new();
        assert_eq!(g.level_type(), LevelCalculationType::Rms);
    }

    #[test]
    fn above_threshold_passes_through() {
        let mut g = NoiseGate::new();
        g.set_threshold(-30.0); // ≈ 0.0316 linear
        g.set_ratio(10.0);
        g.prepare(44100.0);
        // Use a DC input well above the threshold. RMS filter will produce
        // a steady value once it settles.
        let mut last = 0.0_f32;
        for _ in 0..2000 {
            last = g.process_sample(0, 0.5);
        }
        // Once the envelope exceeds the threshold, gain should be 1.0.
        assert!(
            (last - 0.5).abs() < 1e-2,
            "above-threshold signal should pass through, got {last}"
        );
    }

    #[test]
    fn below_threshold_is_attenuated() {
        let mut g = NoiseGate::new();
        g.set_threshold(-6.0); // ≈ 0.5012 linear
        g.set_ratio(10.0);
        g.set_attack(0.0);
        g.set_release(0.0);
        g.prepare(44100.0);

        // DC at 0.01 → RMS = 0.01 → envelope will be ~0.01 → way below
        // the 0.5012 threshold, so the gate attenuates aggressively.
        let mut last = 1.0_f32;
        for _ in 0..2000 {
            last = g.process_sample(0, 0.01);
        }
        assert!(last < 0.01, "deep gate should attenuate to ~0, got {last}");
    }

    #[test]
    fn ratio_panics_when_below_one() {
        let mut g = NoiseGate::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            g.set_ratio(0.5);
        }));
        assert!(result.is_err(), "ratio < 1.0 must panic");
    }

    #[test]
    fn setters_round_trip() {
        let mut g = NoiseGate::new();
        g.set_threshold(-50.0);
        g.set_ratio(4.0);
        g.set_attack(2.5);
        g.set_release(120.0);
        assert_eq!(g.threshold(), -50.0);
        assert_eq!(g.ratio(), 4.0);
        assert_eq!(g.attack_time(), 2.5);
        assert_eq!(g.release_time(), 120.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut g = NoiseGate::new();
        g.set_threshold(-6.0);
        g.prepare_with_channels(44100.0, 2);
        g.process_sample(0, 0.9);
        g.process_sample(1, 0.8);
        g.reset();
        // RMS state and envelope state should both be zero after reset.
        // The next call sees envelope = 0 (below threshold) and the gain
        // formula must produce 0 for `env * threshold_inverse = 0`.
        let out = g.process_sample(0, 0.5);
        // gain = 0^positive = 0 → output = 0
        assert!(out.abs() < 1e-4, "expected attenuation, got {out}");
    }

    #[test]
    fn multi_channel_processing() {
        let mut g = NoiseGate::new();
        g.set_threshold(-6.0);
        g.set_ratio(4.0);
        g.set_attack(0.0);
        g.set_release(0.0);
        g.prepare_with_channels(44100.0, 2);
        // Hot channel passes through, cold channel stays at zero.
        for _ in 0..1024 {
            let hot = g.process_sample(0, 1.0);
            let cold = g.process_sample(1, 0.001);
            assert!(hot > 0.9, "hot channel should pass, got {hot}");
            assert!(cold < 0.01, "cold channel should be gated, got {cold}");
        }
    }
}
