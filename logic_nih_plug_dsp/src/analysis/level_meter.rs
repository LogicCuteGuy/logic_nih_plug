//! Peak and RMS level metering with configurable ballistics.
//!
//! A [`LevelMeter`] tracks the peak amplitude and root-mean-square (RMS) level
//! of a multi-channel audio signal using first-order IIR smoothing with
//! separate attack and release time constants — the same topology used in
//! professional hardware meters and in JUCE's
//! [`AudioDeviceManager::LevelMeter`](https://docs.juce.com/master/classAudioDeviceManager_1_1LevelMeter.html).
//!
//! # Quick start
//!
//! ```
//! use logic_nih_plug_dsp::analysis::level_meter::LevelMeter;
//!
//! let mut meter = LevelMeter::new();
//! meter.prepare(44100.0, 2);
//!
//! // Feed a block of interleaved stereo samples:
//! let left  = vec![0.5f32; 512];
//! let right = vec![0.3f32; 512];
//! meter.process(&[&left, &right]);
//!
//! assert!(meter.peak_db() > f32::NEG_INFINITY);
//! assert!(meter.rms_db() > f32::NEG_INFINITY);
//! ```

/// Calculation mode for the meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelMeterMode {
    /// Track peak (absolute) level.
    #[default]
    Peak,
    /// Track root-mean-square level.
    Rms,
}

/// A real-time peak / RMS level meter with smoothed ballistics.
///
/// One meter instance covers any number of channels (set at
/// [`prepare`](Self::prepare) time). Peak and RMS values are
/// maintained independently and can be queried in linear or dB
/// domain.
///
/// The smoothing filter is a simple one-pole IIR:
///
/// ```text
/// y[n] = x[n] + c * (y[n-1] - x[n])
/// ```
///
/// where `c = exp(-1 / (tau * sample_rate))` and `tau` is the
/// attack or release time constant in seconds.
#[derive(Debug, Clone)]
pub struct LevelMeter {
    sample_rate: f32,

    /// Attack time in seconds.
    attack_time: f32,
    /// Release time in seconds.
    release_time: f32,

    /// Smoothing coefficient for attack (rising signal).
    cte_attack: f32,
    /// Smoothing coefficient for release (falling signal).
    cte_release: f32,

    /// Per-channel smoothed peak value.
    peak_channels: Vec<f32>,
    /// Per-channel smoothed RMS accumulator.
    rms_channels: Vec<f32>,

    /// Global peak across all channels (smoothed).
    peak: f32,
    /// Global RMS across all channels (smoothed).
    rms: f32,
}

impl Default for LevelMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelMeter {
    /// Creates a new level meter with default 15 ms attack / 150 ms release
    /// time constants.
    pub fn new() -> Self {
        Self {
            sample_rate: 44100.0,
            attack_time: 0.015,
            release_time: 0.15,
            cte_attack: 0.0,
            cte_release: 0.0,
            peak_channels: Vec::new(),
            rms_channels: Vec::new(),
            peak: 0.0,
            rms: 0.0,
        }
    }

    /// Sets the attack time constant in seconds.
    ///
    /// Typical values: 10–20 ms for a responsive meter, 50–100 ms for
    /// a slower VU-style ballistics.
    pub fn set_attack_time(&mut self, seconds: f32) {
        self.attack_time = seconds.max(0.0);
        self.recompute_coefficients();
    }

    /// Sets the release time constant in seconds.
    ///
    /// Typical values: 100–300 ms for standard meter ballistics.
    pub fn set_release_time(&mut self, seconds: f32) {
        self.release_time = seconds.max(0.0);
        self.recompute_coefficients();
    }

    /// Returns the current peak level (linear, 0.0–1.0 for unclipped signals).
    pub fn peak(&self) -> f32 {
        self.peak
    }

    /// Returns the current peak level in decibels (relative to 0 dBFS).
    pub fn peak_db(&self) -> f32 {
        linear_to_db(self.peak)
    }

    /// Returns the current RMS level (linear, 0.0–1.0 for unclipped signals).
    pub fn rms(&self) -> f32 {
        self.rms
    }

    /// Returns the current RMS level in decibels (relative to 0 dBFS).
    pub fn rms_db(&self) -> f32 {
        linear_to_db(self.rms)
    }

    /// Prepares the meter for processing at the given sample rate and channel
    /// count.
    ///
    /// All internal state is reset to zero.
    pub fn prepare(&mut self, sample_rate: f32, num_channels: usize) {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(num_channels > 0, "num_channels must be > 0");

        self.sample_rate = sample_rate;
        self.recompute_coefficients();

        self.peak_channels.resize(num_channels, 0.0);
        self.rms_channels.resize(num_channels, 0.0);
        self.reset();
    }

    /// Resets all accumulated levels to zero.
    pub fn reset(&mut self) {
        self.peak = 0.0;
        self.rms = 0.0;
        for v in &mut self.peak_channels {
            *v = 0.0;
        }
        for v in &mut self.rms_channels {
            *v = 0.0;
        }
    }

    /// Processes one block of audio and updates the meter.
    ///
    /// `channel_data` is a slice of `&[f32]` — one inner slice per
    /// channel. All slices **must** have the same length.
    pub fn process(&mut self, channel_data: &[&[f32]]) {
        if channel_data.is_empty() {
            return;
        }

        let num_samples = channel_data[0].len();
        let num_channels = channel_data.len();

        // Make sure per-channel vectors are sized.
        self.peak_channels.resize(num_channels, 0.0);
        self.rms_channels.resize(num_channels, 0.0);

        for (ch, samples) in channel_data.iter().enumerate() {
            debug_assert_eq!(
                samples.len(),
                num_samples,
                "all channels must have the same length"
            );

            let mut peak_acc = self.peak_channels[ch];
            let mut rms_acc = self.rms_channels[ch];

            for &s in samples.iter() {
                let abs_s = s.abs();

                // Peak ballistics
                let c = if abs_s > peak_acc {
                    self.cte_attack
                } else {
                    self.cte_release
                };
                peak_acc = abs_s + c * (peak_acc - abs_s);

                // RMS ballistics (smooth the squared value, sqrt at the end)
                let sq = s * s;
                let c_rms = if sq > rms_acc {
                    self.cte_attack
                } else {
                    self.cte_release
                };
                rms_acc = sq + c_rms * (rms_acc - sq);
            }

            self.peak_channels[ch] = peak_acc;
            self.rms_channels[ch] = rms_acc;
        }

        // Compute the global peak and RMS as the max across channels.
        self.peak = 0.0;
        self.rms = 0.0;
        for &p in &self.peak_channels {
            if p > self.peak {
                self.peak = p;
            }
        }
        for &r in &self.rms_channels {
            let r_linear = r.sqrt();
            if r_linear > self.rms {
                self.rms = r_linear;
            }
        }
    }

    fn recompute_coefficients(&mut self) {
        self.cte_attack = compute_smooth(self.sample_rate, self.attack_time);
        self.cte_release = compute_smooth(self.sample_rate, self.release_time);
    }
}

/// Converts a linear amplitude to decibels (relative to 0 dBFS).
pub fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0 {
        f32::NEG_INFINITY
    } else {
        20.0 * linear.log10()
    }
}

/// Converts decibels (relative to 0 dBFS) to linear amplitude.
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
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
    fn test_peak_sine() {
        let mut meter = LevelMeter::new();
        meter.set_attack_time(0.0);
        meter.set_release_time(0.0);
        meter.prepare(44100.0, 1);

        let mut buffer = vec![0.0f32; 1024];
        for (i, s) in buffer.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin();
        }

        meter.process(&[&buffer]);

        // With zero attack/release, peak should track the instantaneous
        // peak, which for a discrete sine may not be exactly 1.0.
        let peak = meter.peak();
        assert!(
            peak > 0.95,
            "peak should be close to 1.0, got {peak}"
        );
    }

    #[test]
    fn test_rms_sine() {
        let mut meter = LevelMeter::new();
        // Use small smoothing so the RMS accumulator averages properly.
        meter.set_attack_time(0.001);
        meter.set_release_time(0.001);
        meter.prepare(44100.0, 1);

        // Process many blocks so the RMS converges.
        for _ in 0..50 {
            let mut buffer = vec![0.0f32; 1024];
            for (i, s) in buffer.iter_mut().enumerate() {
                *s = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin();
            }
            meter.process(&[&buffer]);
        }

        // RMS of a sine is 1/sqrt(2) ≈ 0.707
        let rms = meter.rms();
        assert!(
            (rms - 0.707).abs() < 0.05,
            "RMS of sine should be ~0.707, got {rms}"
        );
    }

    #[test]
    fn test_peak_db() {
        let mut meter = LevelMeter::new();
        meter.set_attack_time(0.0);
        meter.set_release_time(0.0);
        meter.prepare(44100.0, 1);

        let mut buffer = vec![0.0f32; 1024];
        for (i, s) in buffer.iter_mut().enumerate() {
            *s = (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin();
        }

        meter.process(&[&buffer]);
        let db = meter.peak_db();
        // Sine peaks may not hit exactly 0 dBFS on the sample grid
        assert!(
            db > -1.0,
            "peak dB of unity sine should be close to 0, got {db}"
        );
    }

    #[test]
    fn test_quiet_signal() {
        let mut meter = LevelMeter::new();
        meter.set_attack_time(0.0);
        meter.set_release_time(0.0);
        meter.prepare(44100.0, 1);

        let buffer = vec![0.001f32; 1024];
        meter.process(&[&buffer]);

        let peak = meter.peak();
        assert!(
            (peak - 0.001).abs() < 0.0001,
            "peak should be ~0.001, got {peak}"
        );

        let db = meter.peak_db();
        assert!(
            db < -50.0,
            "dB of 0.001 should be below -50, got {db}"
        );
    }

    #[test]
    fn test_multi_channel() {
        let mut meter = LevelMeter::new();
        meter.set_attack_time(0.0);
        meter.set_release_time(0.0);
        meter.prepare(44100.0, 2);

        let left = vec![0.8f32; 256];
        let right = vec![0.2f32; 256];
        meter.process(&[&left, &right]);

        // Global peak should be the max across channels.
        let peak = meter.peak();
        assert!(
            (peak - 0.8).abs() < 0.01,
            "peak should be ~0.8, got {peak}"
        );
    }

    #[test]
    fn test_linear_to_db_conversion() {
        assert_eq!(linear_to_db(1.0), 0.0);
        assert_eq!(linear_to_db(0.0), f32::NEG_INFINITY);
        assert!(linear_to_db(0.5) < -5.0);
        assert!(linear_to_db(0.5) > -7.0);
    }

    #[test]
    fn test_db_to_linear_conversion() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 1e-6);
        assert!((db_to_linear(20.0) - 10.0).abs() < 1e-4);
        assert!((db_to_linear(-20.0) - 0.1).abs() < 1e-4);
    }
}
