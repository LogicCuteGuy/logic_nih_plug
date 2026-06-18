//! # LadderFilter — Moog ladder filter with drive and resonance
//!
//! A port of JUCE's [`juce::dsp::LadderFilter`](https://docs.juce.com/master/classjuce_1_1dsp_1_1LadderFilter.html).
//!
//! This is a multi-mode filter based on the Moog ladder topology, following
//! the paper *Valimaki (2006): Oscillator and Filter Algorithms for Virtual
//! Analog Synthesis*. It supports six filter modes:
//!
//! | Mode | Description |
//! |---|---|
//! | [`LadderFilterMode::LPF12`] | Low-pass  12 dB/octave |
//! | [`LadderFilterMode::HPF12`] | High-pass 12 dB/octave |
//! | [`LadderFilterMode::BPF12`] | Band-pass 12 dB/octave |
//! | [`LadderFilterMode::LPF24`] | Low-pass  24 dB/octave |
//! | [`LadderFilterMode::HPF24`] | High-pass 24 dB/octave |
//! | [`LadderFilterMode::BPF24`] | Band-pass 24 dB/octave |
//!
//! The filter uses five cascaded integrator stages with `tanh` saturation
//! at the input and feedback tap, a drive parameter controlling the
//! saturation amount, and smoothed cutoff / resonance parameters to
//! prevent clicks.
//!
//! # Example
//!
//! ```
//! use logic_nih_plug_dsp::processors::ladder_filter::{
//!     LadderFilter, LadderFilterMode,
//! };
//! use logic_nih_plug_dsp::processors::Processor;
//!
//! let mut filter = LadderFilter::new();
//! filter.prepare(44100.0, 512);
//! filter.set_mode(LadderFilterMode::LPF24);
//! filter.set_cutoff_frequency_hz(800.0);
//! filter.set_resonance(0.6);
//! filter.set_drive(2.0);
//!
//! let input = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 512];
//! filter.process(&input, &mut output);
//! ```

use std::f32::consts::PI;

use super::Processor;

// ---------------------------------------------------------------------------
// Filter mode
// ---------------------------------------------------------------------------

/// Multi-mode filter type for the Moog ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LadderFilterMode {
    /// Low-pass  12 dB/octave.
    LPF12,
    /// High-pass 12 dB/octave.
    HPF12,
    /// Band-pass 12 dB/octave.
    BPF12,
    /// Low-pass  24 dB/octave.
    LPF24,
    /// High-pass 24 dB/octave.
    HPF24,
    /// Band-pass 24 dB/octave.
    BPF24,
}

impl Default for LadderFilterMode {
    fn default() -> Self {
        Self::LPF12
    }
}

// ---------------------------------------------------------------------------
// Smoother
// ---------------------------------------------------------------------------

/// Linear ramp smoother for parameter changes.
#[derive(Debug, Clone)]
struct Smoother {
    current: f32,
    target: f32,
    step: f32,
    samples_remaining: usize,
}

impl Smoother {
    fn new() -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            step: 0.0,
            samples_remaining: 0,
        }
    }

    fn reset(&mut self, sample_rate: f32, ramp_time_secs: f32) {
        let num_steps = (sample_rate * ramp_time_secs) as usize;
        self.samples_remaining = 0;
        self.step = 0.0;
        let _ = num_steps;
    }

    fn snap_to(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.samples_remaining = 0;
    }

    fn set_target(&mut self, target: f32, sample_rate: f32, ramp_time_secs: f32) {
        if (self.target - target).abs() < 1e-10 {
            return;
        }
        self.target = target;
        let num_steps = (sample_rate * ramp_time_secs).max(1.0) as usize;
        let diff = target - self.current;
        self.step = diff / num_steps as f32;
        self.samples_remaining = num_steps;
    }

    #[inline]
    fn next(&mut self) -> f32 {
        if self.samples_remaining > 0 {
            self.current += self.step;
            self.samples_remaining -= 1;
            if self.samples_remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }
}

// ---------------------------------------------------------------------------
// LadderFilter
// ---------------------------------------------------------------------------

/// Number of integrator stages in the Moog ladder (5).
const NUM_STATES: usize = 5;

/// Output gain applied to all mode coefficient vectors (JUCE: `outputGain = 1.2`).
const OUTPUT_GAIN: f32 = 1.2;

/// Smoothing ramp time in seconds (JUCE: `smootherRampTimeSec = 0.05`).
const SMOOTHER_RAMP_TIME: f32 = 0.05;

/// A lookup-table approximation of `tanh` on `[-5, 5]` with 256 points.
/// On modern CPUs this is comparable to or faster than `libm::tanhf` and
/// avoids the transcendentals in tight loops.
#[derive(Debug)]
struct TanhLut {
    table: Vec<f32>,
    min: f32,
    max: f32,
    scale: f32,
}

impl TanhLut {
    fn new(min: f32, max: f32, num_points: usize) -> Self {
        let mut table = Vec::with_capacity(num_points);
        let step = (max - min) / (num_points as f32 - 1.0);
        for i in 0..num_points {
            table.push((min + step * i as f32).tanh());
        }
        Self {
            table,
            min,
            max,
            scale: (num_points as f32 - 1.0) / (max - min),
        }
    }

    #[inline]
    fn lookup(&self, x: f32) -> f32 {
        if x <= self.min {
            return self.table[0];
        }
        if x >= self.max {
            return *self.table.last().unwrap();
        }
        let idx = (x - self.min) * self.scale;
        let i = idx as usize;
        let frac = idx - i as f32;
        let a = self.table[i];
        let b = self.table[i + 1];
        a + frac * (b - a)
    }
}

/// A multi-mode Moog ladder filter with drive, resonance, and smoothed
/// cutoff / resonance parameters.
///
/// The algorithm follows Valimaki (2006) as implemented in JUCE's
/// `juce::dsp::LadderFilter`. Five cascaded integrator stages use
/// `tanh` saturation, and the output tap coefficients depend on the
/// selected mode.
#[derive(Debug)]
pub struct LadderFilter {
    /// Current filter mode.
    mode: LadderFilterMode,
    /// Output tap coefficients `[a, b, c, d, e]` for the current mode
    /// (already multiplied by `OUTPUT_GAIN`).
    a_coeff: [f32; NUM_STATES],
    /// Compensation factor per mode.
    comp: f32,
    /// Per-channel integrator state `[s[0..5]]`.
    state: Vec<[f32; NUM_STATES]>,
    /// Drive (saturation amount).  Must be >= 1.0.
    drive: f32,
    /// Precomputed `gain = pow(drive, -2.642) * 0.6103 + 0.3903`.
    gain: f32,
    /// Secondary drive: `drive2 = drive * 0.04 + 0.96`.
    drive2: f32,
    /// Precomputed `gain2 = pow(drive2, -2.642) * 0.6103 + 0.3903`.
    gain2: f32,
    /// Cutoff frequency in Hz.
    cutoff_freq_hz: f32,
    /// Cutoff frequency scaler: `-2π / fs`.
    cutoff_freq_scaler: f32,
    /// Resonance in `[0, 1]`.
    resonance: f32,
    /// Smoothed cutoff transform value: `exp(fc * cutoffFreqScaler)`.
    cutoff_transform_smoother: Smoother,
    /// Smoothed scaled resonance: `map(resonance, 0.1, 1.0)`.
    scaled_resonance_smoother: Smoother,
    /// Current smoothed cutoff transform value.
    cutoff_transform_value: f32,
    /// Current smoothed scaled resonance value.
    scaled_resonance_value: f32,
    /// Saturation LUT.
    saturation_lut: TanhLut,
    /// Whether the filter is enabled (passthrough when disabled).
    enabled: bool,
    /// Current sample rate.
    sample_rate: f32,
}

impl LadderFilter {
    /// Creates a new `LadderFilter` with default parameters.
    pub fn new() -> Self {
        let mut filter = Self {
            mode: LadderFilterMode::LPF24, // intentionally different so set_mode(LPF12) in new() runs
            a_coeff: [0.0; NUM_STATES],
            comp: 0.5,
            state: Vec::new(),
            drive: 1.2,
            gain: 0.0,
            drive2: 0.0,
            gain2: 0.0,
            cutoff_freq_hz: 200.0,
            cutoff_freq_scaler: 0.0,
            resonance: 0.0,
            cutoff_transform_smoother: Smoother::new(),
            scaled_resonance_smoother: Smoother::new(),
            cutoff_transform_value: 0.0,
            scaled_resonance_value: 0.0,
            saturation_lut: TanhLut::new(-5.0, 5.0, 256),
            enabled: true,
            sample_rate: 44100.0,
        };
        filter.set_mode(LadderFilterMode::LPF12);
        filter
    }

    /// Sets the filter mode.
    pub fn set_mode(&mut self, mode: LadderFilterMode) {
        if mode == self.mode {
            return;
        }
        match mode {
            LadderFilterMode::LPF12 => {
                self.a_coeff = [0.0, 0.0, 1.0, 0.0, 0.0];
                self.comp = 0.5;
            }
            LadderFilterMode::HPF12 => {
                self.a_coeff = [1.0, -2.0, 1.0, 0.0, 0.0];
                self.comp = 0.0;
            }
            LadderFilterMode::BPF12 => {
                self.a_coeff = [0.0, 1.0, -1.0, 0.0, 0.0];
                self.comp = 0.5;
            }
            LadderFilterMode::LPF24 => {
                self.a_coeff = [0.0, 0.0, 0.0, 0.0, 1.0];
                self.comp = 0.5;
            }
            LadderFilterMode::HPF24 => {
                self.a_coeff = [1.0, -4.0, 6.0, -4.0, 1.0];
                self.comp = 0.0;
            }
            LadderFilterMode::BPF24 => {
                self.a_coeff = [0.0, 0.0, 1.0, -2.0, 1.0];
                self.comp = 0.5;
            }
        }
        // Apply output gain.
        for a in &mut self.a_coeff {
            *a *= OUTPUT_GAIN;
        }
        self.mode = mode;
        self.reset_states();
    }

    /// Returns the current filter mode.
    pub fn mode(&self) -> LadderFilterMode {
        self.mode
    }

    /// Enables or disables the filter.  When disabled, the input is
    /// passed through unchanged.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the filter is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the cutoff frequency in Hz.
    pub fn set_cutoff_frequency_hz(&mut self, freq: f32) {
        self.cutoff_freq_hz = freq.max(1.0);
        self.update_cutoff_freq();
    }

    /// Returns the cutoff frequency in Hz.
    pub fn cutoff_frequency_hz(&self) -> f32 {
        self.cutoff_freq_hz
    }

    /// Sets the resonance in `[0, 1]`.  Higher values increase resonance;
    /// values near 1.0 approach self-oscillation.
    pub fn set_resonance(&mut self, res: f32) {
        self.resonance = res.clamp(0.0, 1.0);
        self.update_resonance();
    }

    /// Returns the resonance value.
    pub fn resonance(&self) -> f32 {
        self.resonance
    }

    /// Sets the drive (saturation amount).  Must be >= 1.0.
    pub fn set_drive(&mut self, drive: f32) {
        self.drive = drive.max(1.0);
        self.gain = self.drive.powf(-2.642) * 0.6103 + 0.3903;
        self.drive2 = self.drive * 0.04 + 0.96;
        self.gain2 = self.drive2.powf(-2.642) * 0.6103 + 0.3903;
    }

    /// Returns the current drive value.
    pub fn drive(&self) -> f32 {
        self.drive
    }

    /// Prepares the filter for the given sample rate and block size.
    pub fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.cutoff_freq_scaler = -2.0 * PI / sample_rate;

        // Ensure state arrays are allocated for at least 1 channel.
        if self.state.is_empty() {
            self.state.resize(2, [0.0; NUM_STATES]);
        }

        self.cutoff_transform_smoother
            .reset(sample_rate, SMOOTHER_RAMP_TIME);
        self.scaled_resonance_smoother
            .reset(sample_rate, SMOOTHER_RAMP_TIME);

        // Pre-set smoothers to current targets.
        self.update_cutoff_freq();
        self.update_resonance();
        self.cutoff_transform_smoother
            .snap_to(self.cutoff_transform_smoother.target);
        self.scaled_resonance_smoother
            .snap_to(self.scaled_resonance_smoother.target);
        self.cutoff_transform_value = self.cutoff_transform_smoother.current;
        self.scaled_resonance_value = self.scaled_resonance_smoother.current;
    }

    fn reset_states(&mut self) {
        for s in &mut self.state {
            *s = [0.0; NUM_STATES];
        }
    }

    fn update_cutoff_freq(&mut self) {
        let target = (self.cutoff_freq_hz * self.cutoff_freq_scaler).exp();
        self.cutoff_transform_smoother
            .set_target(target, self.sample_rate, SMOOTHER_RAMP_TIME);
    }

    fn update_resonance(&mut self) {
        let target = self.resonance * 0.9 + 0.1;
        self.scaled_resonance_smoother
            .set_target(target, self.sample_rate, SMOOTHER_RAMP_TIME);
    }

    /// Process one sample on the given channel (internal, no smoothing update).
    #[inline]
    fn process_sample_internal(&mut self, input: f32, channel: usize) -> f32 {
        let s = &mut self.state[channel];

        let a1 = self.cutoff_transform_value;
        let g = 1.0 - a1;
        let b0 = g * 0.76923076923;
        let b1 = g * 0.23076923076;

        // Input with drive and tanh saturation.
        let dx = self.gain * self.saturation_lut.lookup(self.drive * input);

        // Feedback from the last integrator stage.
        let a = dx
            + self.scaled_resonance_value
                * (-4.0)
                * (self.gain2 * self.saturation_lut.lookup(self.drive2 * s[4])
                    - dx * self.comp);

        // Five cascaded integrators (trapezoidal integration).
        let b = b1 * s[0] + a1 * s[1] + b0 * a;
        let c = b1 * s[1] + a1 * s[2] + b0 * b;
        let d = b1 * s[2] + a1 * s[3] + b0 * c;
        let e = b1 * s[3] + a1 * s[4] + b0 * d;

        s[0] = a;
        s[1] = b;
        s[2] = c;
        s[3] = d;
        s[4] = e;

        // Output tap: weighted sum of integrator outputs.
        a * self.a_coeff[0]
            + b * self.a_coeff[1]
            + c * self.a_coeff[2]
            + d * self.a_coeff[3]
            + e * self.a_coeff[4]
    }
}

impl Default for LadderFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for LadderFilter {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let num_samples = input.len().min(output.len());

        if !self.enabled {
            output[..num_samples].copy_from_slice(&input[..num_samples]);
            return;
        }

        for i in 0..num_samples {
            // Update smoothed parameters.
            self.cutoff_transform_value = self.cutoff_transform_smoother.next();
            self.scaled_resonance_value = self.scaled_resonance_smoother.next();

            output[i] = self.process_sample_internal(input[i], 0);
        }
    }

    fn reset(&mut self) {
        self.reset_states();
        self.cutoff_transform_value = self.cutoff_transform_smoother.target;
        self.scaled_resonance_value = self.scaled_resonance_smoother.target;
        self.cutoff_transform_smoother.snap_to(self.cutoff_transform_value);
        self.scaled_resonance_smoother.snap_to(self.scaled_resonance_value);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_filter_defaults() {
        let f = LadderFilter::new();
        assert_eq!(f.mode(), LadderFilterMode::LPF12);
        assert!(f.is_enabled());
        assert!((f.drive() - 1.2).abs() < 1e-6);
        assert!((f.resonance()).abs() < 1e-6);
        assert!((f.cutoff_frequency_hz() - 200.0).abs() < 1.0);
    }

    #[test]
    fn ladder_filter_passthrough_when_disabled() {
        let mut f = LadderFilter::new();
        f.prepare(44100.0, 256);
        f.set_enabled(false);

        let input: Vec<f32> = (0..256).map(|i| (i as f32 * 0.01).sin()).collect();
        let mut output = vec![0.0_f32; 256];
        f.process(&input, &mut output);

        for i in 0..256 {
            assert!(
                (input[i] - output[i]).abs() < 1e-6,
                "disabled filter should pass through: sample {}",
                i
            );
        }
    }

    #[test]
    fn ladder_filter_modifies_signal() {
        let mut f = LadderFilter::new();
        f.prepare(44100.0, 1024);
        f.set_mode(LadderFilterMode::LPF24);
        f.set_cutoff_frequency_hz(500.0);
        f.set_resonance(0.5);
        f.set_drive(2.0);

        // White-ish signal with lots of high-frequency content.
        let input: Vec<f32> = (0..1024)
            .map(|i| {
                (2.0 * PI * 100.0 * i as f32 / 44100.0).sin() * 0.3
                    + (2.0 * PI * 5000.0 * i as f32 / 44100.0).sin() * 0.3
            })
            .collect();
        let mut output = vec![0.0_f32; 1024];
        f.process(&input, &mut output);

        // The output should differ from input (LPF should attenuate 5 kHz).
        let diff: f32 = input
            .iter()
            .zip(output.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 0.1, "LPF24 should modify signal (diff={})", diff);
    }

    #[test]
    fn ladder_filter_highpass_rejects_low_freq() {
        let mut f = LadderFilter::new();
        f.prepare(44100.0, 4096);
        f.set_mode(LadderFilterMode::HPF12);
        f.set_cutoff_frequency_hz(1000.0);
        f.set_resonance(0.1);
        f.set_drive(1.0);

        // 100 Hz signal — should be attenuated by HPF at 1000 Hz.
        let input: Vec<f32> = (0..4096)
            .map(|i| (2.0 * PI * 100.0 * i as f32 / 44100.0).sin() * 0.5)
            .collect();
        let mut output = vec![0.0_f32; 4096];
        f.process(&input, &mut output);

        // Measure energy of first half (steady state after settling).
        let start = 2048;
        let energy_in: f32 = input[start..].iter().map(|x| x * x).sum();
        let energy_out: f32 = output[start..].iter().map(|x| x * x).sum();

        assert!(
            energy_out < energy_in * 0.5,
            "HPF12 at 1000 Hz should attenuate 100 Hz: in={} out={}",
            energy_in,
            energy_out
        );
    }

    #[test]
    fn ladder_filter_mode_coefficients() {
        let mut f = LadderFilter::new();

        f.set_mode(LadderFilterMode::LPF12);
        assert!((f.a_coeff[2] - 1.2).abs() < 1e-6); // only tap 2 is non-zero

        f.set_mode(LadderFilterMode::HPF24);
        assert!((f.a_coeff[0] - 1.2).abs() < 1e-6);
        assert!((f.a_coeff[1] - (-4.8)).abs() < 1e-5);
        assert!((f.a_coeff[2] - 7.2).abs() < 1e-5);
        assert!((f.a_coeff[3] - (-4.8)).abs() < 1e-5);
        assert!((f.a_coeff[4] - 1.2).abs() < 1e-6);
    }

    #[test]
    fn ladder_filter_drive_affects_output() {
        let mut f_low = LadderFilter::new();
        f_low.prepare(44100.0, 256);
        f_low.set_drive(1.0);
        f_low.set_cutoff_frequency_hz(1000.0);
        f_low.set_resonance(0.3);

        let mut f_high = LadderFilter::new();
        f_high.prepare(44100.0, 256);
        f_high.set_drive(10.0);
        f_high.set_cutoff_frequency_hz(1000.0);
        f_high.set_resonance(0.3);

        let input: Vec<f32> = (0..256)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / 44100.0).sin() * 0.5)
            .collect();
        let mut out_low = vec![0.0_f32; 256];
        let mut out_high = vec![0.0_f32; 256];
        f_low.process(&input, &mut out_low);
        f_high.process(&input, &mut out_high);

        let diff: f32 = out_low
            .iter()
            .zip(out_high.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            diff > 0.01,
            "drive should affect output (diff={})",
            diff
        );
    }

    #[test]
    fn tanh_lut_accuracy() {
        let lut = TanhLut::new(-5.0, 5.0, 256);
        for i in 0..100 {
            let x = -5.0 + 10.0 * i as f32 / 99.0;
            let expected = x.tanh();
            let got = lut.lookup(x);
            assert!(
                (expected - got).abs() < 0.001,
                "tanh LUT at {}: expected {} got {}",
                x,
                expected,
                got
            );
        }
    }
}
