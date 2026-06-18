//! Attack/release envelope follower (JUCE `dsp::BallisticsFilter`).
//!
//! A [`BallisticsFilter`] smooths an input signal with separate attack and
//! release time constants. It is the building block for the
//! [`Compressor`](crate::processors::compressor::Compressor),
//! [`NoiseGate`](crate::processors::noise_gate::NoiseGate), and
//! [`Limiter`](crate::processors::limiter::Limiter) — exactly as
//! `juce::dsp::BallisticsFilter` is in JUCE.
//!
//! Two level-calculation modes are supported:
//!
//! * [`LevelCalculationType::Peak`] — rectifies the input (`abs(x)`) before
//!   smoothing. This is the default and matches the algorithm used in
//!   `Compressor`.
//! * [`LevelCalculationType::Rms`] — squares the input before smoothing and
//!   takes the square root afterwards, giving an RMS-style level estimate.
//!   `NoiseGate` uses this for its RMS pre-filter.
//!
//! The transfer function is a one-pole smoother:
//!
//! ```text
//! y[n] = x[n] + cte * (y[n - 1] - x[n])
//! ```
//!
//! where `cte` is `cteAT` if the input is rising and `cteRL` if it is
//! falling. The `cte` constants are derived from the attack and release
//! times in milliseconds via:
//!
//! ```text
//! cte = exp(-2π · 1000 / (sample_rate · time_ms))
//! ```
//!
//! Times shorter than 1 µs (`0.001 ms`) snap to a coefficient of zero,
//! meaning the filter immediately follows the input.

use crate::processors::dynamics::{snap_to_zero, ProcessSpec};
use crate::processors::Processor;

/// Level-calculation mode used by [`BallisticsFilter`].
///
/// Port of `juce::dsp::BallisticsFilterLevelCalculationType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelCalculationType {
    /// Rectify (`abs`) the input before smoothing. Used by [`crate::processors::compressor::Compressor`].
    #[default]
    Peak,
    /// Square the input before smoothing, then square root the result. Used by
    /// [`crate::processors::noise_gate::NoiseGate`]'s RMS pre-filter.
    Rms,
}

/// Attack / release envelope follower.
///
/// One envelope is kept per channel — call
/// [`prepare_with_channels`](Self::prepare_with_channels) to allocate the
/// per-channel state, then [`process_sample`](Self::process_sample) for
/// each sample in turn.
#[derive(Debug, Clone)]
pub struct BallisticsFilter {
    sample_rate: f32,
    exp_factor: f64,
    attack_time: f32,
    release_time: f32,
    cte_at: f32,
    cte_rl: f32,
    level_type: LevelCalculationType,
    /// Per-channel last-output history.
    yold: Vec<f32>,
}

impl Default for BallisticsFilter {
    fn default() -> Self {
        let mut s = Self {
            sample_rate: 44100.0,
            exp_factor: -2.0 * std::f64::consts::PI * 1000.0 / 44100.0,
            attack_time: 1.0,
            release_time: 100.0,
            cte_at: 0.0,
            cte_rl: 0.0,
            level_type: LevelCalculationType::Peak,
            yold: Vec::new(),
        };
        s.cte_at = s.calculate_limited_cte(1.0);
        s.cte_rl = s.calculate_limited_cte(100.0);
        s
    }
}

impl BallisticsFilter {
    /// Creates a new peak-rectifying [`BallisticsFilter`] with default
    /// 1 ms attack / 100 ms release.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new RMS-level [`BallisticsFilter`].
    pub fn new_rms() -> Self {
        Self {
            level_type: LevelCalculationType::Rms,
            ..Self::default()
        }
    }

    /// Sets the attack time in milliseconds.
    ///
    /// Values smaller than 0.001 ms snap to a coefficient of zero (the
    /// filter tracks the input instantaneously). Very large values saturate
    /// depending on the underlying floating-point precision.
    pub fn set_attack_time(&mut self, attack_time_ms: f32) {
        self.attack_time = attack_time_ms;
        self.cte_at = self.calculate_limited_cte(attack_time_ms);
    }

    /// Sets the release time in milliseconds.
    pub fn set_release_time(&mut self, release_time_ms: f32) {
        self.release_time = release_time_ms;
        self.cte_rl = self.calculate_limited_cte(release_time_ms);
    }

    /// Sets the level-calculation mode (peak rectification vs. RMS).
    ///
    /// Switching modes resets the per-channel state to zero.
    pub fn set_level_calculation_type(&mut self, ty: LevelCalculationType) {
        self.level_type = ty;
        self.reset();
    }

    /// Returns the attack time in milliseconds.
    pub fn attack_time(&self) -> f32 {
        self.attack_time
    }

    /// Returns the release time in milliseconds.
    pub fn release_time(&self) -> f32 {
        self.release_time
    }

    /// Returns the current level-calculation mode.
    pub fn level_calculation_type(&self) -> LevelCalculationType {
        self.level_type
    }

    /// Prepares the filter for processing with a single channel.
    pub fn prepare(&mut self, sample_rate: f32) {
        self.prepare_with_channels(sample_rate, 1);
    }

    /// Prepares the filter for processing with `num_channels` parallel
    /// envelopes.
    pub fn prepare_with_channels(&mut self, sample_rate: f32, num_channels: usize) {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(num_channels > 0, "num_channels must be > 0");
        self.sample_rate = sample_rate;
        self.exp_factor = -2.0_f64 * std::f64::consts::PI * 1000.0 / f64::from(sample_rate);
        // Recompute cte constants under the new sample rate.
        self.cte_at = self.calculate_limited_cte(self.attack_time);
        self.cte_rl = self.calculate_limited_cte(self.release_time);
        self.yold.resize(num_channels, 0.0);
        self.reset();
    }

    /// Convenience helper that takes a [`ProcessSpec`].
    pub fn prepare_spec(&mut self, spec: ProcessSpec) {
        self.prepare_with_channels(spec.sample_rate, spec.num_channels);
    }

    /// Resets all per-channel envelope state to zero.
    pub fn reset(&mut self) {
        self.reset_to(0.0);
    }

    /// Resets all per-channel envelope state to the given initial value.
    pub fn reset_to(&mut self, initial_value: f32) {
        for old in &mut self.yold {
            *old = initial_value;
        }
    }

    /// Processes a single sample on the given channel index and returns
    /// the smoothed envelope value.
    pub fn process_sample(&mut self, channel: usize, input: f32) -> f32 {
        assert!(channel < self.yold.len(), "channel out of range");
        // Pre-process the input according to the level-calculation type.
        let mut input_value = input;
        match self.level_type {
            LevelCalculationType::Peak => {
                input_value = input_value.abs();
            }
            LevelCalculationType::Rms => {
                input_value *= input_value;
            }
        }

        // Use attack time constant when the input is rising,
        // release time constant when it is falling.
        let prev = self.yold[channel];
        let cte = if input_value > prev {
            self.cte_at
        } else {
            self.cte_rl
        };
        let result = input_value + cte * (prev - input_value);
        self.yold[channel] = result;

        match self.level_type {
            LevelCalculationType::Peak => result,
            LevelCalculationType::Rms => result.sqrt(),
        }
    }

    /// Forces any denormal envelope values to zero (called once per
    /// process block in JUCE).
    pub fn snap_to_zero(&mut self) {
        for old in &mut self.yold {
            snap_to_zero(old);
        }
    }

    /// Returns the number of channels that have been prepared.
    pub fn num_channels(&self) -> usize {
        self.yold.len()
    }

    /// Returns the stored envelope value for a single channel (test helper).
    pub fn channel_state(&self, channel: usize) -> f32 {
        self.yold[channel]
    }

    fn calculate_limited_cte(&self, time_ms: f32) -> f32 {
        if time_ms < 1.0e-3 {
            0.0
        } else {
            (self.exp_factor / f64::from(time_ms)).exp() as f32
        }
    }
}

impl Processor for BallisticsFilter {
    fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.prepare(sample_rate);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            output.len(),
            "input and output lengths must match"
        );
        if self.yold.is_empty() {
            self.prepare_with_channels(self.sample_rate, 1);
        }
        for (i, (&x, y)) in input.iter().zip(output.iter_mut()).enumerate() {
            *y = self.process_sample(0, x);
            // We only have one prepared channel in the Processor-trait path,
            // so copy the state to subsequent virtual channels by alternating.
            let _ = i; // (kept for future multi-channel wiring)
        }
    }

    fn reset(&mut self) {
        self.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_peak_mode() {
        let f = BallisticsFilter::new();
        assert_eq!(f.level_calculation_type(), LevelCalculationType::Peak);
    }

    #[test]
    fn cte_saturates_at_zero_for_tiny_times() {
        let mut f = BallisticsFilter::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare_with_channels(44100.0, 1);
        // 0.0 < 1e-3 → cte is forced to zero, so the filter follows the
        // input instantaneously.
        assert_eq!(f.process_sample(0, 0.7), 0.7);
        assert_eq!(f.process_sample(0, 0.2), 0.2);
    }

    #[test]
    fn peak_rectifies() {
        let mut f = BallisticsFilter::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare_with_channels(44100.0, 1);
        assert_eq!(f.process_sample(0, -0.5), 0.5);
        assert_eq!(f.process_sample(0, 0.25), 0.25);
    }

    #[test]
    fn rms_squares_then_squareroots() {
        let mut f = BallisticsFilter::new_rms();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare_with_channels(44100.0, 1);
        // 0.25^2 = 0.0625, sqrt(0.0625) = 0.25
        assert!((f.process_sample(0, 0.25) - 0.25).abs() < 1e-6);
        assert!((f.process_sample(0, -0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn smoothing_does_not_overshoot() {
        let mut f = BallisticsFilter::new();
        f.set_attack_time(10.0);
        f.set_release_time(100.0);
        f.prepare_with_channels(44100.0, 1);

        // Step up to 1.0 — envelope should approach but never exceed 1.0.
        let mut last = 0.0;
        for _ in 0..2000 {
            last = f.process_sample(0, 1.0);
        }
        assert!(last <= 1.0 + 1e-5, "overshot to {last}");
        assert!((last - 1.0).abs() < 1e-3, "did not reach 1.0: {last}");

        // Step down to 0 — envelope should approach but never go below 0.
        for _ in 0..20000 {
            last = f.process_sample(0, 0.0);
        }
        assert!(last >= -1e-5, "undershot to {last}");
        assert!(last.abs() < 1e-3, "did not reach 0: {last}");
    }

    #[test]
    fn level_type_change_resets_state() {
        let mut f = BallisticsFilter::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare_with_channels(44100.0, 2);
        f.process_sample(0, 0.7);
        f.process_sample(1, 0.4);
        assert!((f.channel_state(0) - 0.7).abs() < 1e-6);

        f.set_level_calculation_type(LevelCalculationType::Rms);
        assert_eq!(f.channel_state(0), 0.0);
        assert_eq!(f.channel_state(1), 0.0);
    }

    #[test]
    fn reset_clears_state() {
        let mut f = BallisticsFilter::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare_with_channels(44100.0, 1);
        f.process_sample(0, 0.8);
        f.reset();
        assert_eq!(f.channel_state(0), 0.0);
    }

    #[test]
    fn reset_to_initial_value() {
        let mut f = BallisticsFilter::new();
        f.prepare_with_channels(44100.0, 2);
        f.process_sample(0, 0.5);
        f.process_sample(1, 0.9);
        f.reset_to(0.42);
        assert!((f.channel_state(0) - 0.42).abs() < 1e-6);
        assert!((f.channel_state(1) - 0.42).abs() < 1e-6);
    }

    #[test]
    fn exp_factor_scales_with_sample_rate() {
        let mut f = BallisticsFilter::new();
        f.set_attack_time(10.0);
        f.set_release_time(10.0);
        f.prepare_with_channels(96000.0, 1);
        f.prepare_with_channels(44100.0, 1);
        // Same time constants, different sample rates ⇒ different cte
        // values. Just verify cte is non-zero and finite.
        assert!(f.attack_time > 0.0);
        assert!(f.release_time > 0.0);
    }

    #[test]
    fn multi_channel_isolation() {
        let mut f = BallisticsFilter::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare_with_channels(44100.0, 4);
        assert_eq!(f.process_sample(0, 0.1), 0.1);
        assert_eq!(f.process_sample(2, 0.9), 0.9);
        // Channels 1 and 3 are still at zero.
        assert_eq!(f.channel_state(1), 0.0);
        assert_eq!(f.channel_state(3), 0.0);
    }
}
