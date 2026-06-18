//! Smoothed envelope follower for extracting amplitude envelopes.
//!
//! A [`Follower`] tracks the amplitude of a signal over time using
//! first-order IIR smoothing with separate attack and release time
//! constants. Unlike the [`LevelMeter`](super::level_meter::LevelMeter),
//! which provides peak *and* RMS readings across multiple channels,
//! a `Follower` is a lighter, single-path tool that outputs a
//! smoothed amplitude value suitable for modulation sources, VU-style
//! metering, or sidechain detection.
//!
//! Two rectification modes are supported:
//!
//! * [`Rectification::Absolute`] — `abs(x)` (default, full-wave).
//! * [`Rectification::Squared`]  — `x²` then `sqrt` after smoothing
//!   (RMS-like).
//!
//! # Quick start
//!
//! ```
//! use logic_nih_plug_dsp::analysis::follower::Follower;
//!
//! let mut f = Follower::new();
//! f.set_attack_time(0.0);
//! f.set_release_time(0.0);
//! f.prepare(44100.0);
//!
//! let input = vec![0.5f32; 512];
//! let mut output = vec![0.0f32; 512];
//! f.process(&input, &mut output);
//!
//! // output[0] is 0.5 with zero attack/release (instant tracking).
//! assert!((output[0] - 0.5).abs() < 0.01);
//! ```

/// Rectification mode for the envelope follower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rectification {
    /// Full-wave rectification (`abs(x)`).
    #[default]
    Absolute,
    /// Squared rectification (`x²` before smoothing, `sqrt` after).
    Squared,
}

/// A first-order envelope follower with configurable attack/release
/// ballistics.
///
/// The smoothing transfer function is:
///
/// ```text
/// y[n] = x_rect[n] + c * (y[n-1] - x_rect[n])
/// ```
///
/// where `c = exp(-1 / (tau * sample_rate))` and `tau` is the attack
/// or release time constant.
#[derive(Debug, Clone)]
pub struct Follower {
    sample_rate: f32,
    attack_time: f32,
    release_time: f32,
    cte_attack: f32,
    cte_release: f32,
    rectification: Rectification,
    /// Previous smoothed output.
    state: f32,
}

impl Default for Follower {
    fn default() -> Self {
        Self::new()
    }
}

impl Follower {
    /// Creates a new follower with 10 ms attack and 100 ms release.
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            attack_time: 0.01,
            release_time: 0.1,
            cte_attack: 0.0,
            cte_release: 0.0,
            rectification: Rectification::default(),
            state: 0.0,
        }
    }

    /// Sets the attack time constant in seconds.
    pub fn set_attack_time(&mut self, seconds: f32) {
        self.attack_time = seconds.max(0.0);
        self.recompute_coefficients();
    }

    /// Sets the release time constant in seconds.
    pub fn set_release_time(&mut self, seconds: f32) {
        self.release_time = seconds.max(0.0);
        self.recompute_coefficients();
    }

    /// Sets the rectification mode.
    pub fn set_rectification(&mut self, rect: Rectification) {
        self.rectification = rect;
    }

    /// Returns the current rectification mode.
    pub fn rectification(&self) -> Rectification {
        self.rectification
    }

    /// Prepares the follower for processing at the given sample rate.
    pub fn prepare(&mut self, sample_rate: f32) {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        self.sample_rate = sample_rate;
        self.recompute_coefficients();
        self.reset();
    }

    /// Resets the internal state to zero.
    pub fn reset(&mut self) {
        self.state = 0.0;
    }

    /// Returns the current smoothed output value.
    pub fn value(&self) -> f32 {
        self.state
    }

    /// Processes a block of audio and writes the smoothed envelope into
    /// `output`.
    ///
    /// `input` and `output` must have the same length.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            output.len(),
            "input and output must have the same length"
        );

        let mut state = self.state;

        for (i, &s) in input.iter().enumerate() {
            let rectified = match self.rectification {
                Rectification::Absolute => s.abs(),
                Rectification::Squared => s * s,
            };

            let c = if rectified > state {
                self.cte_attack
            } else {
                self.cte_release
            };

            state = rectified + c * (state - rectified);

            output[i] = match self.rectification {
                Rectification::Absolute => state,
                Rectification::Squared => state.sqrt(),
            };
        }

        self.state = state;
    }

    fn recompute_coefficients(&mut self) {
        self.cte_attack = compute_smooth(self.sample_rate, self.attack_time);
        self.cte_release = compute_smooth(self.sample_rate, self.release_time);
    }
}

fn compute_smooth(sample_rate: f32, time_secs: f32) -> f32 {
    if time_secs <= 0.0 {
        return 0.0;
    }
    (-1.0 / (sample_rate * time_secs)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instant_tracking() {
        let mut f = Follower::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare(44100.0);

        let input = vec![0.75f32; 256];
        let mut output = vec![0.0f32; 256];
        f.process(&input, &mut output);

        assert!(
            (output[0] - 0.75).abs() < 0.001,
            "should track instantly, got {}",
            output[0]
        );
    }

    #[test]
    fn test_attack_ramp() {
        let mut f = Follower::new();
        f.set_attack_time(0.0);
        f.set_release_time(1.0); // very slow release
        f.prepare(44100.0);

        let input = vec![0.0f32; 1024];
        let mut output = vec![0.0f32; 1024];

        // First: jump to 1.0
        let input_up = vec![1.0f32; 64];
        let mut output_up = vec![0.0f32; 64];
        f.process(&input_up, &mut output_up);
        assert!(
            (output_up[0] - 1.0).abs() < 0.001,
            "attack should be instant"
        );

        // Now: release should be very slow
        f.process(&input, &mut output);
        assert!(
            output[100] > 0.9,
            "slow release should keep level high, got {}",
            output[100]
        );
    }

    #[test]
    fn test_squared_rectification() {
        let mut f = Follower::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.set_rectification(Rectification::Squared);
        f.prepare(44100.0);

        let input = vec![0.5f32; 256];
        let mut output = vec![0.0f32; 256];
        f.process(&input, &mut output);

        // With squared: rectified = 0.25, sqrt(0.25) = 0.5
        assert!(
            (output[0] - 0.5).abs() < 0.01,
            "squared rectification should give ~0.5, got {}",
            output[0]
        );
    }

    #[test]
    fn test_negative_input() {
        let mut f = Follower::new();
        f.set_attack_time(0.0);
        f.set_release_time(0.0);
        f.prepare(44100.0);

        let input = vec![-0.8f32; 256];
        let mut output = vec![0.0f32; 256];
        f.process(&input, &mut output);

        assert!(
            (output[0] - 0.8).abs() < 0.01,
            "should rectify negative to positive, got {}",
            output[0]
        );
    }

    #[test]
    fn test_sine_rms_envelope() {
        let mut f = Follower::new();
        f.set_attack_time(0.001);
        f.set_release_time(0.001);
        f.set_rectification(Rectification::Squared);
        f.prepare(44100.0);

        let num_samples = 4096;
        let input: Vec<f32> = (0..num_samples)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; num_samples];
        f.process(&input, &mut output);

        // After settling, the envelope should approximate the RMS of a sine
        // which is 1/sqrt(2) ≈ 0.707
        let final_val = output[num_samples - 1];
        assert!(
            (final_val - 0.707).abs() < 0.1,
            "RMS envelope of sine should be ~0.707, got {final_val}"
        );
    }
}
