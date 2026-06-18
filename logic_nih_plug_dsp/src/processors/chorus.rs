//! # Chorus — LFO-modulated delay line chorus / flanger / vibrato
//!
//! A port of JUCE's [`juce::dsp::Chorus`](https://docs.juce.com/master/classjuce_1_1dsp_1_1Chorus.html).
//!
//! A sine LFO modulates the delay time of a delay line, creating sweeping
//! notches in the frequency response.  Classic chorus sounds use a centre
//! delay around 7–8 ms with low feedback; flanging uses shorter delays with
//! high feedback; vibrato uses mix = 1.0.
//!
//! ```text
//!   in ──┐
//!       ├──▶ Σ ──▶ delay line (LFO-modulated) ──▶ output ──┐
//!       │     ↑                                             │
//!       │     └────── feedback × volume ◄───────────────────┘
//!       │
//!       └──▶ dry ───────────────────────────── dry/wet mix ──▶ out
//! ```
//!
//! The delay line uses linear interpolation (matching JUCE).
//!
//! # Example
//!
//! ```
//! use logic_nih_plug_dsp::processors::chorus::{Chorus, ChorusParameters};
//! use logic_nih_plug_dsp::processors::Processor;
//!
//! let mut chorus = Chorus::new();
//! chorus.prepare(44100.0, 512);
//! chorus.set_parameters(ChorusParameters {
//!     rate: 1.0,
//!     depth: 0.25,
//!     centre_delay: 7.0,
//!     feedback: 0.0,
//!     mix: 0.5,
//! });
//!
//! let input = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 512];
//! chorus.process(&input, &mut output);
//! ```

use std::f32::consts::PI;

use super::delay::{DelayLine, LinearInterpolation};
use super::Processor;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum delay modulation depth in milliseconds (JUCE: `maximumDelayModulation = 20.0`).
const MAXIMUM_DELAY_MODULATION: f32 = 20.0;

/// Maximum allowed centre delay in ms (JUCE: `maxCentreDelayMs = 100.0`).
const MAX_CENTRE_DELAY_MS: f32 = 100.0;

/// Minimum allowed centre delay in ms.
const MIN_CENTRE_DELAY_MS: f32 = 1.0;

/// Oscillator volume multiplier (JUCE: `oscVolumeMultiplier = 0.5`).
const OSC_VOLUME_MULTIPLIER: f32 = 0.5;

/// Smoothing ramp time (seconds) for feedback volume.
const FEEDBACK_SMOOTH_TIME: f32 = 0.05;

/// Smoothing ramp time (seconds) for LFO volume.
const OSC_VOLUME_SMOOTH_TIME: f32 = 0.05;

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Parameters for the [`Chorus`] effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChorusParameters {
    /// LFO rate in Hz.  Must be < 100.  Default: 1.0.
    pub rate: f32,
    /// LFO depth in `[0, 1]`.  Default: 0.25.
    pub depth: f32,
    /// Centre delay time in milliseconds in `[1, 100]`.  Default: 7.0.
    pub centre_delay: f32,
    /// Feedback volume in `[-1, 1]`.  Default: 0.0.
    pub feedback: f32,
    /// Dry/wet mix in `[0, 1]`.  Default: 0.5.
    pub mix: f32,
}

impl Default for ChorusParameters {
    fn default() -> Self {
        Self {
            rate: 1.0,
            depth: 0.25,
            centre_delay: 7.0,
            feedback: 0.0,
            mix: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Simple sine LFO
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
// Linear smoother
// ---------------------------------------------------------------------------

/// Tiny linear ramp smoother.
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

    fn reset(&mut self, _sample_rate: f32, _ramp_time_secs: f32) {
        self.samples_remaining = 0;
        self.step = 0.0;
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
// Chorus
// ---------------------------------------------------------------------------

/// A chorus / flanger / vibrato effect that modulates a delay line with
/// a sine LFO.
///
/// Classic chorus: centre_delay ≈ 7–8 ms, low feedback, moderate depth.
/// Flanging: short centre delay, high feedback.
/// Vibrato: mix = 1.0.
#[derive(Debug)]
pub struct Chorus {
    /// Delay line with linear interpolation (mono, one per channel).
    delay: DelayLine<LinearInterpolation>,
    /// Sine LFO.
    lfo: SineLfo,
    /// LFO volume smoother.
    osc_volume: Smoother,
    /// Per-channel feedback volume smoother.
    feedback_volume: Vec<Smoother>,
    /// Per-channel last output value (for feedback path).
    last_output: Vec<f32>,
    /// Delay-time buffer (subsampled LFO values converted to samples).
    delay_buffer: Vec<f32>,
    /// Current sample rate.
    sample_rate: f32,

    // Parameters
    rate: f32,
    depth: f32,
    centre_delay: f32,
    feedback: f32,
    mix: f32,
}

impl Chorus {
    /// Creates a new `Chorus` with default parameters.
    pub fn new() -> Self {
        // Allocate a generous delay line — will be recreated in prepare().
        let delay = DelayLine::<LinearInterpolation>::with_maximum_delay(2048);

        Self {
            delay,
            lfo: SineLfo::new(),
            osc_volume: Smoother::new(),
            feedback_volume: Vec::new(),
            last_output: Vec::new(),
            delay_buffer: Vec::new(),
            sample_rate: 44100.0,
            rate: 1.0,
            depth: 0.25,
            centre_delay: 7.0,
            feedback: 0.0,
            mix: 0.5,
        }
    }

    /// Returns the current parameters.
    pub fn parameters(&self) -> ChorusParameters {
        ChorusParameters {
            rate: self.rate,
            depth: self.depth,
            centre_delay: self.centre_delay,
            feedback: self.feedback,
            mix: self.mix,
        }
    }

    /// Sets all parameters at once.
    pub fn set_parameters(&mut self, params: ChorusParameters) {
        self.rate = params.rate;
        self.depth = params.depth;
        self.centre_delay = params.centre_delay.clamp(MIN_CENTRE_DELAY_MS, MAX_CENTRE_DELAY_MS);
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

    /// Sets the centre delay in milliseconds in `[1, 100]`.
    pub fn set_centre_delay(&mut self, delay_ms: f32) {
        self.centre_delay = delay_ms.clamp(MIN_CENTRE_DELAY_MS, MAX_CENTRE_DELAY_MS);
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

        // Compute the maximum delay line length in samples.
        let max_possible_delay =
            ((MAXIMUM_DELAY_MODULATION * 1.0 * OSC_VOLUME_MULTIPLIER + MAX_CENTRE_DELAY_MS)
                * sample_rate
                / 1000.0)
                .ceil() as usize;
        self.delay = DelayLine::<LinearInterpolation>::with_maximum_delay(max_possible_delay.max(4));
        self.delay.prepare(sample_rate, 2); // stereo

        let num_channels = 2;
        self.feedback_volume.resize_with(num_channels, Smoother::new);
        self.last_output.resize(num_channels, 0.0);
        self.delay_buffer.resize(max_block_size, 0.0);

        self.lfo
            .set_frequency(self.rate, sample_rate);
        self.osc_volume.reset(sample_rate, OSC_VOLUME_SMOOTH_TIME);
        self.osc_volume.snap_to(self.depth * OSC_VOLUME_MULTIPLIER);

        for vol in &mut self.feedback_volume {
            vol.reset(sample_rate, FEEDBACK_SMOOTH_TIME);
            vol.snap_to(self.feedback);
        }

        self.update_internal();
        self.reset_internal();
    }

    fn reset_internal(&mut self) {
        self.delay.reset();
        self.lfo.reset();
        for out in &mut self.last_output {
            *out = 0.0;
        }
    }

    fn update_internal(&mut self) {
        self.lfo.set_frequency(self.rate, self.sample_rate);
        self.osc_volume.set_target(
            self.depth * OSC_VOLUME_MULTIPLIER,
            self.sample_rate,
            OSC_VOLUME_SMOOTH_TIME,
        );
        for vol in &mut self.feedback_volume {
            vol.set_target(self.feedback, self.sample_rate, FEEDBACK_SMOOTH_TIME);
        }
    }
}

impl Default for Chorus {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for Chorus {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let num_samples = input.len().min(output.len());

        // Step 1: generate LFO values and compute delay times in samples.
        let osc_vol = self.osc_volume.next();

        for i in 0..num_samples {
            let lfo = self.lfo.tick();
            let delay_ms =
                (MAXIMUM_DELAY_MODULATION * lfo * osc_vol + self.centre_delay)
                    .max(MIN_CENTRE_DELAY_MS);
            self.delay_buffer[i] = delay_ms * self.sample_rate / 1000.0;
        }

        // Step 2: process each sample.
        for i in 0..num_samples {
            let input_sample = input[i];

            // feedback path: input - lastOutput (JUCE pattern)
            let fb_vol = self.feedback_volume[0].next();
            let to_delay = input_sample - self.last_output[0] * fb_vol;

            self.delay.push_sample(0, to_delay);
            self.delay.set_delay(self.delay_buffer[i]);
            let wet = self.delay.pop_sample(0);

            output[i] = wet;
            self.last_output[0] = wet;
        }

        // Apply dry/wet mix.
        for i in 0..num_samples {
            output[i] = (1.0 - self.mix) * input[i] + self.mix * output[i];
        }
    }

    fn reset(&mut self) {
        self.reset_internal();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chorus_defaults() {
        let c = Chorus::new();
        let p = c.parameters();
        assert!((p.rate - 1.0).abs() < 1e-6);
        assert!((p.depth - 0.25).abs() < 1e-6);
        assert!((p.centre_delay - 7.0).abs() < 0.1);
        assert!((p.feedback).abs() < 1e-6);
        assert!((p.mix - 0.5).abs() < 1e-6);
    }

    #[test]
    fn chorus_passthrough_at_zero_mix() {
        let mut c = Chorus::new();
        c.prepare(44100.0, 512);
        c.set_parameters(ChorusParameters {
            mix: 0.0,
            ..Default::default()
        });

        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin()).collect();
        let mut output = vec![0.0_f32; 512];
        c.process(&input, &mut output);

        for i in 0..512 {
            assert!(
                (input[i] - output[i]).abs() < 1e-6,
                "sample {}: expected {} got {}",
                i,
                input[i],
                output[i]
            );
        }
    }

    #[test]
    fn chorus_full_wet_modifies_signal() {
        let mut c = Chorus::new();
        c.prepare(44100.0, 1024);
        c.set_parameters(ChorusParameters {
            mix: 1.0,
            depth: 0.5,
            rate: 2.0,
            centre_delay: 8.0,
            feedback: 0.0,
        });

        let input: Vec<f32> = (0..1024)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0_f32; 1024];
        c.process(&input, &mut output);

        // The delayed signal should differ from the input.
        let diff: f32 = input
            .iter()
            .zip(output.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 1.0, "chorus should modify signal (diff={})", diff);
    }

    #[test]
    fn chorus_vibrato_mode() {
        // Mix = 1.0 means pure wet (vibrato). Output should still be non-zero.
        // Need enough samples for the delay line to read back pushed values
        // (centre delay ~7 ms = ~309 samples at 44.1 kHz).
        let mut c = Chorus::new();
        c.prepare(44100.0, 1024);
        c.set_parameters(ChorusParameters {
            mix: 1.0,
            depth: 0.5,
            rate: 5.0,
            feedback: 0.0,
            ..Default::default()
        });

        let input = vec![0.5_f32; 1024];
        let mut output = vec![0.0_f32; 1024];
        c.process(&input, &mut output);

        let non_zero = output.iter().filter(|v| v.abs() > 1e-6).count();
        assert!(non_zero > 0, "vibrato should produce non-zero output");
    }

    #[test]
    fn chorus_flanger_high_feedback() {
        // Flanger: short delay, high feedback.
        let mut c = Chorus::new();
        c.prepare(44100.0, 512);
        c.set_parameters(ChorusParameters {
            rate: 0.5,
            depth: 0.8,
            centre_delay: 2.0,
            feedback: 0.8,
            mix: 0.7,
        });

        let input: Vec<f32> = (0..512)
            .map(|i| (2.0 * PI * 200.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0_f32; 512];
        c.process(&input, &mut output);

        let non_zero = output.iter().filter(|v| v.abs() > 1e-6).count();
        assert!(non_zero > 0, "flanger should produce non-zero output");
    }

    #[test]
    fn chorus_centre_delay_clamped() {
        let mut c = Chorus::new();
        c.prepare(44100.0, 256);

        // Try setting delay outside allowed range.
        c.set_centre_delay(0.1); // below MIN
        assert!((c.centre_delay - MIN_CENTRE_DELAY_MS).abs() < 1e-6);

        c.set_centre_delay(200.0); // above MAX
        assert!((c.centre_delay - MAX_CENTRE_DELAY_MS).abs() < 1e-6);
    }

    #[test]
    fn chorus_depth_zero_no_modulation() {
        let mut c = Chorus::new();
        c.prepare(44100.0, 512);
        c.set_parameters(ChorusParameters {
            depth: 0.0,
            mix: 1.0,
            feedback: 0.0,
            ..Default::default()
        });

        let input: Vec<f32> = (0..512)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0_f32; 512];
        c.process(&input, &mut output);

        // With depth=0 and mix=1, the output should be a pure delayed copy.
        // It should not be all zeros but should differ from input by the delay.
        let energy: f32 = output.iter().map(|x| x * x).sum();
        assert!(energy > 0.01, "output should have energy even with depth=0");
    }
}
