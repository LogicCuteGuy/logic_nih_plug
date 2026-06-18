//! # Phaser — 6-stage LFO-modulated allpass phaser
//!
//! A port of JUCE's [`juce::dsp::Phaser`](https://docs.juce.com/master/classjuce_1_1dsp_1_1Phaser.html).
//!
//! The phaser sweeps six first-order allpass filters whose cutoff frequencies
//! are modulated by a sine LFO, creating moving notches in the frequency
//! response. A feedback path feeds the output back into the input for a
//! more pronounced effect.
//!
//! ```text
//!   in ──┐
//!       ├──▶ Σ ──▶ 6× first-order allpass ──▶ output ──┐
//!       │     ↑                                        │
//!       │     └────── feedback × volume ◄──────────────┘
//!       │
//!       └──▶ dry ──────────────────────────── dry/wet mix ──▶ out
//! ```
//!
//! The LFO is subsampled by a factor of 4 for efficiency (matching JUCE).
//!
//! # Example
//!
//! ```
//! use logic_nih_plug_dsp::processors::phaser::{Phaser, PhaserParameters};
//! use logic_nih_plug_dsp::processors::Processor;
//!
//! let mut phaser = Phaser::new();
//! phaser.prepare(44100.0, 512);
//! phaser.set_parameters(PhaserParameters {
//!     rate: 0.5,
//!     depth: 0.7,
//!     centre_frequency: 1300.0,
//!     feedback: 0.3,
//!     mix: 0.5,
//! });
//!
//! let input = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 512];
//! phaser.process(&input, &mut output);
//! ```

use std::f32::consts::PI;

use super::Processor;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of allpass filter stages (matching JUCE's `numStages = 6`).
const NUM_STAGES: usize = 6;

/// LFO subsampling factor.  The LFO only computes new values every
/// `MAX_UPDATE_COUNTER` samples; the value is held for the intervening
/// samples.  Matches JUCE.
const MAX_UPDATE_COUNTER: usize = 4;

/// Smoothing ramp time (seconds) for feedback volume.
const FEEDBACK_SMOOTH_TIME: f32 = 0.05;

/// Smoothing ramp time (seconds) for LFO volume.
const OSC_VOLUME_SMOOTH_TIME: f32 = 0.05;

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Parameters for the [`Phaser`] effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhaserParameters {
    /// LFO rate in Hz.  Must be < 100 Hz.  Default: 1.0.
    pub rate: f32,
    /// LFO depth in `[0, 1]`.  Default: 0.5.
    pub depth: f32,
    /// Centre frequency in Hz for the allpass modulation.
    /// Default: 1300.0.
    pub centre_frequency: f32,
    /// Feedback volume in `[-1, 1]`.  Default: 0.0.
    pub feedback: f32,
    /// Dry/wet mix in `[0, 1]`.  Default: 0.5.
    pub mix: f32,
}

impl Default for PhaserParameters {
    fn default() -> Self {
        Self {
            rate: 1.0,
            depth: 0.5,
            centre_frequency: 1300.0,
            feedback: 0.0,
            mix: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// First-order TPT allpass filter (inlined, per-channel)
// ---------------------------------------------------------------------------

/// Minimal first-order TPT allpass filter.
///
/// ```text
///   v  = G * (x - s)
///   y  = v + s
///   s' = y + v
///   out = 2*y - x        (allpass)
/// ```
///
/// where `G = tan(π·fc/fs) / (1 + tan(π·fc/fs))`.
#[derive(Debug, Clone)]
struct FirstOrderAllpass {
    /// Precomputed `G = tan(π·fc/fs) / (1 + tan(π·fc/fs))`.
    g: f32,
    /// Single state variable per channel.
    s1: Vec<f32>,
}

impl FirstOrderAllpass {
    fn new() -> Self {
        Self {
            g: 0.0,
            s1: Vec::new(),
        }
    }

    fn prepare(&mut self, num_channels: usize) {
        self.s1.resize(num_channels, 0.0);
    }

    fn reset(&mut self) {
        for s in &mut self.s1 {
            *s = 0.0;
        }
    }

    /// Update G from a cutoff frequency.
    #[inline]
    fn set_cutoff(&mut self, fc: f32, sample_rate: f32) {
        let g_raw = (PI * fc / sample_rate).tan();
        self.g = g_raw / (1.0 + g_raw);
    }

    /// Process one sample on the given channel.
    #[inline]
    fn process_sample(&mut self, channel: usize, input: f32) -> f32 {
        let s = &mut self.s1[channel];
        let v = self.g * (input - *s);
        let y = v + *s;
        *s = y + v;
        2.0 * y - input
    }
}

// ---------------------------------------------------------------------------
// Simple sine LFO (inline, no dependency on oscillators crate)
// ---------------------------------------------------------------------------

/// Phase-accumulator sine oscillator used as an LFO.
#[derive(Debug, Clone)]
struct SineLfo {
    phase: f32,
    phase_inc: f32,
}

impl SineLfo {
    fn new() -> Self {
        Self {
            phase: 0.0,
            phase_inc: 0.0,
        }
    }

    #[inline]
    fn set_frequency(&mut self, freq: f32, sample_rate: f32) {
        self.phase_inc = freq / sample_rate;
    }

    #[inline]
    fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Returns a sine sample in `[-1, 1]` and advances the phase.
    #[inline]
    fn tick(&mut self) -> f32 {
        let out = (2.0 * PI * self.phase).sin();
        self.phase += self.phase_inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Linear smoother (simple ramp)
// ---------------------------------------------------------------------------

/// Tiny linear ramp smoother for parameter smoothing.
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
        let _ = num_steps; // used in set_target_value
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

    /// Snap current value immediately (for reset).
    fn snap_to(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.samples_remaining = 0;
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
// Phaser
// ---------------------------------------------------------------------------

/// A 6-stage phaser that modulates first-order allpass filters to create
/// sweeping notches in the magnitude frequency response.
///
/// See the [module-level](self) documentation for a signal-flow diagram.
#[derive(Debug)]
pub struct Phaser {
    /// Six allpass filter stages.
    filters: [FirstOrderAllpass; NUM_STAGES],
    /// Sine LFO.
    lfo: SineLfo,
    /// LFO volume smoother (maps depth to amplitude).
    osc_volume: Smoother,
    /// Per-channel feedback volume smoother.
    feedback_volume: Vec<Smoother>,
    /// Per-channel last output (for feedback path).
    last_output: Vec<f32>,
    /// Normalised centre frequency (mapped from Hz to [0, 1] log scale).
    norm_centre_frequency: f32,
    /// Current sample rate.
    sample_rate: f32,
    /// LFO subsample counter.
    update_counter: usize,
    /// Cached LFO buffer (subsampled, `MAX_UPDATE_COUNTER` size).
    lfo_buffer: Vec<f32>,

    // Parameters
    rate: f32,
    depth: f32,
    centre_frequency: f32,
    feedback: f32,
    mix: f32,
}

impl Phaser {
    /// Creates a new `Phaser` with default parameters.
    pub fn new() -> Self {
        let filters = std::array::from_fn(|_| FirstOrderAllpass::new());
        Self {
            filters,
            lfo: SineLfo::new(),
            osc_volume: Smoother::new(),
            feedback_volume: Vec::new(),
            last_output: Vec::new(),
            norm_centre_frequency: 0.5,
            sample_rate: 44100.0,
            update_counter: 0,
            lfo_buffer: vec![0.0; MAX_UPDATE_COUNTER],
            rate: 1.0,
            depth: 0.5,
            centre_frequency: 1300.0,
            feedback: 0.0,
            mix: 0.5,
        }
    }

    /// Returns the current parameters.
    pub fn parameters(&self) -> PhaserParameters {
        PhaserParameters {
            rate: self.rate,
            depth: self.depth,
            centre_frequency: self.centre_frequency,
            feedback: self.feedback,
            mix: self.mix,
        }
    }

    /// Sets all parameters at once.
    pub fn set_parameters(&mut self, params: PhaserParameters) {
        self.rate = params.rate;
        self.depth = params.depth;
        self.centre_frequency = params.centre_frequency;
        self.feedback = params.feedback;
        self.mix = params.mix;
        self.update_internal();
    }

    /// Sets the LFO rate in Hz (must be < 100).
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.clamp(0.0, 99.9);
        self.update_internal();
    }

    /// Sets the LFO depth in `[0, 1]`.
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
        self.update_internal();
    }

    /// Sets the centre frequency in Hz.
    pub fn set_centre_frequency(&mut self, fc: f32) {
        self.centre_frequency = fc;
        self.norm_centre_frequency = map_from_log10(fc, 20.0, (20000.0_f32).min(0.49 * self.sample_rate));
        // Immediately update filter cutoffs.
        for filter in &mut self.filters {
            let freq = map_to_log10(
                self.depth * 0.0 + self.norm_centre_frequency, // static centre
                20.0,
                (20000.0_f32).min(0.49 * self.sample_rate),
            );
            filter.set_cutoff(freq, self.sample_rate);
        }
    }

    /// Sets the feedback volume in `[-1, 1]`.
    pub fn set_feedback(&mut self, fb: f32) {
        self.feedback = fb.clamp(-1.0, 1.0);
        self.update_internal();
    }

    /// Sets the dry/wet mix in `[0, 1]`.
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
        self.update_internal();
    }

    /// Prepares the processor for the given sample rate and block size.
    pub fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.sample_rate = sample_rate;

        let num_channels = 2; // stereo
        for filter in &mut self.filters {
            filter.prepare(num_channels);
        }

        self.feedback_volume.resize_with(num_channels, Smoother::new);
        self.last_output.resize(num_channels, 0.0);
        self.lfo_buffer.resize(max_block_size.div_ceil(MAX_UPDATE_COUNTER), 0.0);

        self.lfo
            .set_frequency(self.rate, sample_rate / MAX_UPDATE_COUNTER as f32);

        self.osc_volume
            .reset(sample_rate / MAX_UPDATE_COUNTER as f32, OSC_VOLUME_SMOOTH_TIME);
        self.osc_volume.snap_to(self.depth * 0.5);

        for vol in &mut self.feedback_volume {
            vol.reset(sample_rate, FEEDBACK_SMOOTH_TIME);
            vol.snap_to(self.feedback);
        }

        self.update_internal();
        self.reset_internal();
    }

    fn reset_internal(&mut self) {
        for out in &mut self.last_output {
            *out = 0.0;
        }
        for filter in &mut self.filters {
            filter.reset();
        }
        self.lfo.reset();
        self.update_counter = 0;
    }

    fn update_internal(&mut self) {
        self.lfo
            .set_frequency(self.rate, self.sample_rate / MAX_UPDATE_COUNTER as f32);
        self.osc_volume.set_target(
            self.depth * 0.5,
            self.sample_rate / MAX_UPDATE_COUNTER as f32,
            OSC_VOLUME_SMOOTH_TIME,
        );
        for vol in &mut self.feedback_volume {
            vol.set_target(self.feedback, self.sample_rate, FEEDBACK_SMOOTH_TIME);
        }
    }
}

impl Default for Phaser {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for Phaser {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let num_samples = input.len().min(output.len());

        // Step 1: generate subsampled LFO values and map to cutoff frequencies.
        let mut counter = self.update_counter;
        let mut num_down = 0usize;
        for i in 0..num_samples {
            if counter == 0 {
                num_down += 1;
            }
            counter += 1;
            if counter >= MAX_UPDATE_COUNTER {
                counter = 0;
            }
        }

        let lfo_vol = self.osc_volume.next();
        for k in 0..num_down {
            self.lfo_buffer[k] = self.lfo.tick() * lfo_vol;
        }

        // Map LFO values to cutoff frequencies via log10 mapping.
        let nyquist = (20000.0_f32).min(0.49 * self.sample_rate);
        for k in 0..num_down {
            let lfo = self.lfo_buffer[k]
                .clamp(0.0, 1.0)
                .max(0.0) + self.norm_centre_frequency;
            self.lfo_buffer[k] = map_to_log10(lfo, 20.0, nyquist);
        }

        // Step 2: process each sample.
        let mut counter = self.update_counter;
        let mut k = 0usize;

        for i in 0..num_samples {
            let input_sample = input[i];

            // Feedback: input - lastOutput (JUCE: input - lastOutput[channel])
            let feedback_vol = self.feedback_volume[0].next();
            let output_sample = input_sample - self.last_output[0] * feedback_vol;

            // Update allpass cutoffs at subsample rate.
            if counter == 0 && k < num_down {
                for filter in &mut self.filters {
                    filter.set_cutoff(self.lfo_buffer[k], self.sample_rate);
                }
                k += 1;
            }

            // Run through all 6 allpass stages.
            let mut y = output_sample;
            for filter in &mut self.filters {
                y = filter.process_sample(0, y);
            }

            output[i] = y;
            self.last_output[0] = y;

            counter += 1;
            if counter >= MAX_UPDATE_COUNTER {
                counter = 0;
            }
        }

        // Apply dry/wet mix: out = (1-mix)*dry + mix*wet
        for i in 0..num_samples {
            output[i] = (1.0 - self.mix) * input[i] + self.mix * output[i];
        }

        self.update_counter = counter;
    }

    fn reset(&mut self) {
        self.reset_internal();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Maps a value from `[in_min, in_max]` to `[0, 1]` on a log10 scale.
/// Mirrors JUCE's `mapFromLog10`.
#[inline]
fn map_from_log10(value: f32, in_min: f32, in_max: f32) -> f32 {
    let log_min = in_min.max(1e-20).log10();
    let log_max = in_max.max(1e-20).log10();
    let log_val = value.max(1e-20).log10();
    if (log_max - log_min).abs() < 1e-20 {
        return 0.5;
    }
    (log_val - log_min) / (log_max - log_min)
}

/// Maps a value from `[0, 1]` to `[out_min, out_max]` on a log10 scale.
/// Mirrors JUCE's `mapToLog10`.
#[inline]
fn map_to_log10(value: f32, out_min: f32, out_max: f32) -> f32 {
    let log_min = out_min.max(1e-20).log10();
    let log_max = out_max.max(1e-20).log10();
    let t = value.clamp(0.0, 1.0);
    10.0_f32.powf(log_min + t * (log_max - log_min))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phaser_defaults() {
        let p = Phaser::new();
        let params = p.parameters();
        assert!((params.rate - 1.0).abs() < 1e-6);
        assert!((params.depth - 0.5).abs() < 1e-6);
        assert!((params.centre_frequency - 1300.0).abs() < 1.0);
        assert!((params.feedback).abs() < 1e-6);
        assert!((params.mix - 0.5).abs() < 1e-6);
    }

    #[test]
    fn phaser_passthrough_at_zero_mix() {
        let mut p = Phaser::new();
        p.prepare(44100.0, 512);
        p.set_parameters(PhaserParameters {
            mix: 0.0,
            ..Default::default()
        });

        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin()).collect();
        let mut output = vec![0.0_f32; 512];
        p.process(&input, &mut output);

        for i in 0..512 {
            assert!(
                (input[i] - output[i]).abs() < 1e-6,
                "sample {}: input={} output={}",
                i,
                input[i],
                output[i]
            );
        }
    }

    #[test]
    fn phaser_full_wet_modifies_signal() {
        let mut p = Phaser::new();
        p.prepare(44100.0, 512);
        p.set_parameters(PhaserParameters {
            mix: 1.0,
            depth: 0.7,
            rate: 2.0,
            feedback: 0.3,
            ..Default::default()
        });

        let input: Vec<f32> = (0..512)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0_f32; 512];
        p.process(&input, &mut output);

        // With allpass stages, output should differ from input.
        let diff: f32 = input
            .iter()
            .zip(output.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 0.1, "phaser should modify the signal (diff={})", diff);
    }

    #[test]
    fn phaser_produces_nonzero_output() {
        let mut p = Phaser::new();
        p.prepare(44100.0, 256);
        p.set_parameters(PhaserParameters {
            mix: 0.5,
            depth: 0.5,
            rate: 1.0,
            feedback: 0.0,
            ..Default::default()
        });

        let input = vec![0.5_f32; 256];
        let mut output = vec![0.0_f32; 256];
        p.process(&input, &mut output);

        let non_zero = output.iter().filter(|v| v.abs() > 1e-6).count();
        assert!(non_zero > 0, "phaser should produce non-zero output");
    }

    #[test]
    fn phaser_feedback_affects_output() {
        let mut p_no_fb = Phaser::new();
        p_no_fb.prepare(44100.0, 256);
        p_no_fb.set_parameters(PhaserParameters {
            feedback: 0.0,
            mix: 1.0,
            depth: 0.5,
            rate: 1.0,
            ..Default::default()
        });

        let mut p_with_fb = Phaser::new();
        p_with_fb.prepare(44100.0, 256);
        p_with_fb.set_parameters(PhaserParameters {
            feedback: 0.8,
            mix: 1.0,
            depth: 0.5,
            rate: 1.0,
            ..Default::default()
        });

        let input: Vec<f32> = (0..256)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut out_no = vec![0.0_f32; 256];
        let mut out_yes = vec![0.0_f32; 256];
        p_no_fb.process(&input, &mut out_no);
        p_with_fb.process(&input, &mut out_yes);

        // With feedback the outputs should be different.
        let diff: f32 = out_no
            .iter()
            .zip(out_yes.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            diff > 0.01,
            "feedback should change output (diff={})",
            diff
        );
    }

    #[test]
    fn map_to_log10_roundtrip() {
        let original = 0.5_f32;
        let mapped = map_from_log10(map_to_log10(original, 20.0, 20000.0), 20.0, 20000.0);
        assert!(
            (original - mapped).abs() < 1e-5,
            "roundtrip failed: {} vs {}",
            original,
            mapped
        );
    }

    #[test]
    fn allpass_is_allpass() {
        // A first-order allpass should preserve the magnitude of each frequency.
        // Test with a simple impulse through a single stage.
        let mut ap = FirstOrderAllpass::new();
        ap.prepare(1);
        ap.set_cutoff(1000.0, 44100.0);

        let mut energy_in = 0.0f32;
        let mut energy_out = 0.0f32;

        // Feed a short burst of white-ish signal.
        for i in 0..1024 {
            let x = (i as f32 * 0.1).sin() * 0.7 + (i as f32 * 0.37).cos() * 0.3;
            energy_in += x * x;
            let y = ap.process_sample(0, x);
            energy_out += y * y;
        }

        let ratio = energy_out / energy_in;
        assert!(
            (ratio - 1.0).abs() < 0.05,
            "allpass energy ratio should be ~1.0, got {}",
            ratio
        );
    }
}
