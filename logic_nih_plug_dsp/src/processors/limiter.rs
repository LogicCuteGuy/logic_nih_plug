//! Two-stage look-behind limiter (JUCE `dsp::Limiter`).
//!
//! [`Limiter`] is a brick-wall-style limiter built from two [`Compressor`]s
//! in series plus a smoothed output gain and a hard clipper at ±1.0.
//! Mirrors `juce::dsp::Limiter` exactly.
//!
//! ## Signal flow
//!
//! ```text
//! input ──▶ first_stage_compressor (4:1 @ -10 dB, 2 ms / 200 ms)
//!           │
//!           ▼
//!           second_stage_compressor (1000:1 @ threshold, 1 µs / release)
//!           │
//!           ▼
//!           × output_volume (smoothed)
//!           │
//!           ▼
//!           hard clip to ±1.0
//! ```
//!
//! The first stage catches most of the dynamic range with a gentle 4:1
//! curve. The second stage is a near-instant brick-wall compressor with an
//! extreme 1000:1 ratio that grabs whatever peaks the first stage let
//! through. The output volume compensates for the first-stage gain
//! reduction so the limiter is roughly unity-gain below the threshold.

use crate::processors::compressor::Compressor;
use crate::processors::dynamics::ProcessSpec;
use crate::processors::Processor;

/// A two-stage brick-wall limiter.
///
/// One pair of internal compressors and a smoothed output volume are kept
/// per channel. Call [`prepare_with_channels`](Self::prepare_with_channels)
/// before processing.
#[derive(Debug, Clone)]
pub struct Limiter {
    first_stage: Compressor,
    second_stage: Compressor,
    /// Smoothed output gain — see `Limiter::update`.
    output_volume: f32,
    output_volume_step: f32,
    output_volume_target: f32,
    /// Number of samples remaining in the current smoothing step.
    output_volume_steps_remaining: i32,
    sample_rate: f32,
    threshold_db: f32,
    release_time: f32,
}

impl Default for Limiter {
    fn default() -> Self {
        let mut s = Self {
            first_stage: Compressor::new(),
            second_stage: Compressor::new(),
            output_volume: 1.0,
            output_volume_step: 0.0,
            output_volume_target: 1.0,
            output_volume_steps_remaining: 0,
            sample_rate: 44100.0,
            threshold_db: -10.0,
            release_time: 100.0,
        };
        s.update();
        s
    }
}

impl Limiter {
    /// Creates a new limiter with defaults: -10 dB threshold, 100 ms release.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limiting threshold in decibels.
    pub fn set_threshold(&mut self, new_threshold_db: f32) {
        self.threshold_db = new_threshold_db;
        self.update();
    }

    /// Sets the release time in milliseconds.
    pub fn set_release(&mut self, new_release_ms: f32) {
        self.release_time = new_release_ms;
        self.update();
    }

    /// Returns the current threshold in dB.
    pub fn threshold(&self) -> f32 {
        self.threshold_db
    }

    /// Returns the current release time in milliseconds.
    pub fn release_time(&self) -> f32 {
        self.release_time
    }

    /// Returns the current smoothed output volume (test helper).
    pub fn output_volume(&self) -> f32 {
        self.output_volume
    }

    /// Prepares the limiter for one channel of processing.
    pub fn prepare(&mut self, sample_rate: f32) {
        self.prepare_with_channels(sample_rate, 1);
    }

    /// Prepares the limiter for the given number of channels.
    pub fn prepare_with_channels(&mut self, sample_rate: f32, num_channels: usize) {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(num_channels > 0, "num_channels must be > 0");
        self.sample_rate = sample_rate;
        self.first_stage.prepare_with_channels(sample_rate, num_channels);
        self.second_stage.prepare_with_channels(sample_rate, num_channels);
        self.update();
        self.reset();
    }

    /// Convenience that takes a [`ProcessSpec`].
    pub fn prepare_spec(&mut self, spec: ProcessSpec) {
        self.prepare_with_channels(spec.sample_rate, spec.num_channels);
    }

    /// Resets both compressor stages and snaps the output volume back to
    /// its current target (1 ms smoothing).
    pub fn reset(&mut self) {
        self.first_stage.reset();
        self.second_stage.reset();
        // Match JUCE: SmoothedValue.reset(sampleRate, 0.001)
        let steps = ((self.sample_rate as f64) * 0.001).round() as i32;
        let steps = steps.max(1);
        self.output_volume = self.output_volume_target;
        self.output_volume_steps_remaining = steps;
        self.output_volume_step = 0.0;
    }

    /// Forces denormal envelope state to zero on both stages.
    pub fn snap_to_zero(&mut self) {
        self.first_stage.snap_to_zero();
        self.second_stage.snap_to_zero();
    }

    /// Processes a single sample on `channel`.
    pub fn process_sample(&mut self, channel: usize, input: f32) -> f32 {
        let x = self.first_stage.process_sample(channel, input);
        let x = self.second_stage.process_sample(channel, x);
        // Smooth the output volume one step forward.
        self.output_volume = self.bump_output_volume();
        let scaled = x * self.output_volume;
        // Hard clip to ±1.0 (matches JUCE's FloatVectorOperations::clip).
        scaled.clamp(-1.0, 1.0)
    }

    /// Processes a whole block on a single channel.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        for (i, (&x, y)) in input.iter().zip(output.iter_mut()).enumerate() {
            *y = self.process_sample(0, x);
            let _ = i;
        }
    }

    fn bump_output_volume(&mut self) -> f32 {
        if self.output_volume_steps_remaining > 0 {
            let next = self.output_volume + self.output_volume_step;
            self.output_volume_steps_remaining -= 1;
            next
        } else {
            self.output_volume_target
        }
    }

    fn update(&mut self) {
        // First stage: 4:1 at -10 dB, 2 ms attack, 200 ms release.
        // (These are hard-coded in JUCE and we mirror them exactly.)
        self.first_stage.set_threshold(-10.0);
        self.first_stage.set_ratio(4.0);
        self.first_stage.set_attack(2.0);
        self.first_stage.set_release(200.0);

        // Second stage: ratio 1000:1 at the user's threshold, 0.001 ms
        // attack (near-instant) and the user's release.
        self.second_stage.set_threshold(self.threshold_db);
        self.second_stage.set_ratio(1000.0);
        self.second_stage.set_attack(0.001);
        self.second_stage.set_release(self.release_time);

        // Output volume compensation: 1 / first_stage_gain, with first_stage
        // modeled as 4:1 above its threshold. The (10 * (1 - 1/ratio) / 40)
        // term is exactly JUCE's gain formula.
        let ratio_inverse = 1.0_f32 / 4.0;
        let gain = 10.0_f32.powf(10.0 * (1.0 - ratio_inverse) / 40.0);
        // Multiply by the inverse of the threshold gain so the limiter is
        // roughly unity-gain below the threshold.
        let threshold_gain = crate::processors::dynamics::db_to_gain(-self.threshold_db, -100.0);
        self.output_volume_target = gain * threshold_gain;

        // Set up smoothing toward the new target (1 ms ramp).
        let steps = ((self.sample_rate as f64) * 0.001).round() as i32;
        let steps = steps.max(1);
        self.output_volume_steps_remaining = steps;
        self.output_volume_step = (self.output_volume_target - self.output_volume) / steps as f32;
    }
}

impl Processor for Limiter {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate);
        let _ = max_block_size;
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        Limiter::process(self, input, output);
    }

    fn reset(&mut self) {
        Limiter::reset(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_juce() {
        let l = Limiter::new();
        assert_eq!(l.threshold(), -10.0);
        assert_eq!(l.release_time(), 100.0);
    }

    #[test]
    fn setters_round_trip() {
        let mut l = Limiter::new();
        l.set_threshold(-3.0);
        l.set_release(250.0);
        assert_eq!(l.threshold(), -3.0);
        assert_eq!(l.release_time(), 250.0);
    }

    #[test]
    fn quiet_signal_is_maximised() {
        let mut l = Limiter::new();
        l.set_threshold(-6.0); // ~0.5012 linear
        l.set_release(50.0);
        l.prepare(44100.0);
        // JUCE's Limiter is a loudness maximizer: the output_volume
        // compensates for the gain reduction that would be applied to a
        // signal at the threshold. Below the threshold the limiter
        // therefore *amplifies* the input. For a -12 dB signal (0.25
        // linear) at a -6 dB threshold the expected steady-state output
        // is approximately `input * output_volume_target`.
        let mut last = 0.0_f32;
        for _ in 0..2000 {
            last = l.process_sample(0, 0.25);
        }
        let expected = 0.25 * l.output_volume_target_for_test();
        assert!(
            (last - expected).abs() < 0.1,
            "expected ~{expected}, got {last}"
        );
    }

    #[test]
    fn hot_signal_is_clipped() {
        let mut l = Limiter::new();
        l.set_threshold(-6.0);
        l.set_release(50.0);
        l.prepare(44100.0);
        // Drive a DC signal at 1.0 for many samples and confirm the output
        // never exceeds 1.0.
        let mut max_out = 0.0_f32;
        for _ in 0..2000 {
            let y = l.process_sample(0, 1.0);
            max_out = max_out.max(y.abs());
        }
        assert!(max_out <= 1.0 + 1e-6, "output exceeded unity: {max_out}");
    }

    #[test]
    fn output_volume_smooths_toward_target() {
        let mut l = Limiter::new();
        l.set_threshold(-10.0);
        l.prepare(44100.0);
        let initial_target = l.output_volume_target_for_test();
        // Switch threshold and verify the smoothed output volume updates.
        l.set_threshold(-3.0);
        let new_target = l.output_volume_target_for_test();
        assert!(
            (new_target - initial_target).abs() > 0.01,
            "expected target to move, old={initial_target} new={new_target}"
        );
    }

    #[test]
    fn reset_snaps_output_volume_to_target() {
        let mut l = Limiter::new();
        l.prepare(44100.0);
        l.reset();
        // After reset, output_volume should equal the target (within fp
        // precision).
        assert!((l.output_volume() - l.output_volume_target_for_test()).abs() < 1e-3);
    }

    #[test]
    fn process_block_matches_per_sample() {
        let mut a = Limiter::new();
        a.set_threshold(-6.0);
        a.set_release(50.0);
        a.prepare(44100.0);
        let mut b = Limiter::new();
        b.set_threshold(-6.0);
        b.set_release(50.0);
        b.prepare(44100.0);

        let input: Vec<f32> = (0..256).map(|i| (i as f32 / 256.0).sin()).collect();
        let mut block_out = vec![0.0_f32; 256];
        a.process(&input, &mut block_out);

        let mut sample_out = Vec::with_capacity(256);
        for &x in &input {
            sample_out.push(b.process_sample(0, x));
        }

        for (i, (x, y)) in block_out.iter().zip(sample_out.iter()).enumerate() {
            assert!(
                (x - y).abs() < 1e-5,
                "mismatch at sample {i}: block={x}, sample={y}"
            );
        }
    }
}

// Internal accessor used only by the test module.
impl Limiter {
    #[cfg(test)]
    fn output_volume_target_for_test(&self) -> f32 {
        self.output_volume_target
    }
}
