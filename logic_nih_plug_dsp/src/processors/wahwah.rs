//! # WahWah — LFO-modulated resonant bandpass filter
//!
//! An auto-wah effect that sweeps a resonant bandpass filter's centre
//! frequency with a sine LFO.  This is similar to an auto-wah pedal
//! (Cry Baby style) but with a constant-rate LFO instead of envelope
//! following.
//!
//! ```text
//!   in ──┐
//!       │
//!       ├──▶ resonant bandpass (TPT SVF) ──▶ wet ──┐
//!       │        ↑                                  │
//!       │        └── centre freq modulated by LFO   │
//!       │                                           │
//!       └──▶ dry ──────────────────── dry/wet mix ──▶ out
//! ```
//!
//! The filter uses the same TPT (Topology-Preserving Transform) state
//! variable filter as [`crate::state_variable::StateVariableFilter`],
//! inlined here for self-containment.
//!
//! # Example
//!
//! ```
//! use logic_nih_plug_dsp::processors::wahwah::{WahWah, WahWahParameters};
//! use logic_nih_plug_dsp::processors::Processor;
//!
//! let mut wah = WahWah::new();
//! wah.prepare(44100.0, 512);
//! wah.set_parameters(WahWahParameters {
//!     rate: 2.0,
//!     depth: 1.0,
//!     min_frequency: 300.0,
//!     max_frequency: 3000.0,
//!     resonance: 0.6,
//!     mix: 0.8,
//! });
//!
//! let input = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 512];
//! wah.process(&input, &mut output);
//! ```

use std::f32::consts::PI;

use super::Processor;

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Parameters for the [`WahWah`] effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WahWahParameters {
    /// LFO rate in Hz.  Default: 2.0.
    pub rate: f32,
    /// LFO depth in `[0, 1]`.  0 = no sweep, 1 = full sweep.  Default: 1.0.
    pub depth: f32,
    /// Minimum centre frequency in Hz.  Default: 300.0.
    pub min_frequency: f32,
    /// Maximum centre frequency in Hz.  Default: 3000.0.
    pub max_frequency: f32,
    /// Resonance / Q in `[0, 1]`.  Higher values produce a sharper peak.
    /// Default: 0.6.
    pub resonance: f32,
    /// Dry/wet mix in `[0, 1]`.  Default: 0.8.
    pub mix: f32,
}

impl Default for WahWahParameters {
    fn default() -> Self {
        Self {
            rate: 2.0,
            depth: 1.0,
            min_frequency: 300.0,
            max_frequency: 3000.0,
            resonance: 0.6,
            mix: 0.8,
        }
    }
}

// ---------------------------------------------------------------------------
// Simple sine LFO
// ---------------------------------------------------------------------------

/// Phase-accumulator sine oscillator.
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
// TPT State Variable Filter (inlined, bandpass only)
// ---------------------------------------------------------------------------

/// Minimal TPT state variable filter producing a bandpass output.
///
/// Uses the Cytomic / Andrew Simper formulation:
///
/// ```text
///   g  = tan(π·fc/fs)
///   k  = 2 − 2·resonance
///   a1 = 1 / (1 + g·(g + k))
///   a2 = g · a1
///   a3 = g · a2
///
///   v0 = input
///   v1 = a1·s1 + a2·(v0 − s2)     ← bandpass
///   v2 = s2 + a2·s1 + a3·(v0 − s2) ← lowpass
///
///   s1 = 2·v1 − s1
///   s2 = 2·v2 − s2
/// ```
#[derive(Debug, Clone)]
struct TptBandpass {
    g: f32,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    s1: f32,
    s2: f32,
    sample_rate: f32,
}

impl TptBandpass {
    fn new() -> Self {
        Self {
            g: 0.0,
            k: 2.0,
            a1: 0.5,
            a2: 0.0,
            a3: 0.0,
            s1: 0.0,
            s2: 0.0,
            sample_rate: 44100.0,
        }
    }

    fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    /// Update cutoff and resonance.  Resonance is in `[0, 1]`.
    #[inline]
    fn set_params(&mut self, fc: f32, resonance: f32) {
        let nyquist = self.sample_rate * 0.49;
        let fc = fc.clamp(20.0, nyquist);
        let r = resonance.clamp(0.0, 0.999);

        self.g = (PI * fc / self.sample_rate).tan();
        self.k = 2.0 - 2.0 * r;
        let denom = 1.0 + self.g * (self.g + self.k);
        self.a1 = 1.0 / denom;
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
    }

    /// Process one sample, returning the bandpass output.
    #[inline]
    fn process_sample(&mut self, input: f32) -> f32 {
        let v0 = input;
        let v1 = self.a1 * self.s1 + self.a2 * (v0 - self.s2);
        let v2 = self.s2 + self.a2 * self.s1 + self.a3 * (v0 - self.s2);

        self.s1 = 2.0 * v1 - self.s1;
        self.s2 = 2.0 * v2 - self.s2;

        // Snap denormals.
        if self.s1.abs() < 1e-15 {
            self.s1 = 0.0;
        }
        if self.s2.abs() < 1e-15 {
            self.s2 = 0.0;
        }

        v1 // bandpass output
    }
}

// ---------------------------------------------------------------------------
// WahWah
// ---------------------------------------------------------------------------

/// An auto-wah effect: resonant bandpass filter with LFO-modulated centre
/// frequency.
#[derive(Debug)]
pub struct WahWah {
    /// TPT bandpass filter.
    filter: TptBandpass,
    /// Sine LFO.
    lfo: SineLfo,
    /// Current sample rate.
    sample_rate: f32,

    // Parameters
    rate: f32,
    depth: f32,
    min_frequency: f32,
    max_frequency: f32,
    resonance: f32,
    mix: f32,
}

impl WahWah {
    /// Creates a new `WahWah` with default parameters.
    pub fn new() -> Self {
        Self {
            filter: TptBandpass::new(),
            lfo: SineLfo::new(),
            sample_rate: 44100.0,
            rate: 2.0,
            depth: 1.0,
            min_frequency: 300.0,
            max_frequency: 3000.0,
            resonance: 0.6,
            mix: 0.8,
        }
    }

    /// Returns the current parameters.
    pub fn parameters(&self) -> WahWahParameters {
        WahWahParameters {
            rate: self.rate,
            depth: self.depth,
            min_frequency: self.min_frequency,
            max_frequency: self.max_frequency,
            resonance: self.resonance,
            mix: self.mix,
        }
    }

    /// Sets all parameters at once.
    pub fn set_parameters(&mut self, params: WahWahParameters) {
        self.rate = params.rate;
        self.depth = params.depth.clamp(0.0, 1.0);
        self.min_frequency = params.min_frequency.max(20.0);
        self.max_frequency = params.max_frequency;
        self.resonance = params.resonance;
        self.mix = params.mix.clamp(0.0, 1.0);
        self.lfo
            .set_frequency(self.rate, self.sample_rate);
    }

    /// Sets the LFO rate in Hz.
    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate.max(0.0);
        self.lfo.set_frequency(self.rate, self.sample_rate);
    }

    /// Sets the modulation depth in `[0, 1]`.
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth.clamp(0.0, 1.0);
    }

    /// Sets the centre frequency range.
    pub fn set_frequency_range(&mut self, min_hz: f32, max_hz: f32) {
        self.min_frequency = min_hz.max(20.0);
        self.max_frequency = max_hz;
    }

    /// Sets the resonance in `[0, 1]`.
    pub fn set_resonance(&mut self, q: f32) {
        self.resonance = q.clamp(0.0, 1.0);
    }

    /// Sets the dry/wet mix in `[0, 1]`.
    pub fn set_mix(&mut self, mix: f32) {
        self.mix = mix.clamp(0.0, 1.0);
    }

    /// Prepares the processor for the given sample rate and block size.
    pub fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.filter.set_sample_rate(sample_rate);
        self.lfo.set_frequency(self.rate, sample_rate);
        self.reset_internal();
    }

    fn reset_internal(&mut self) {
        self.filter.reset();
        self.lfo.reset();
    }
}

impl Default for WahWah {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for WahWah {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        let num_samples = input.len().min(output.len());
        let min_freq = self.min_frequency;
        let max_freq = self.max_frequency;
        let depth = self.depth;
        let resonance = self.resonance;
        let mix = self.mix;

        for i in 0..num_samples {
            let lfo = self.lfo.tick(); // [-1, 1]

            // Map LFO to centre frequency.
            // At depth=0, frequency stays at the geometric mean.
            // At depth=1, frequency sweeps from min to max.
            let lfo_norm = (lfo + 1.0) * 0.5; // [0, 1]
            let target_freq = if depth > 0.0 {
                (min_freq.ln() * (1.0 - lfo_norm * depth)
                    + max_freq.ln() * (lfo_norm * depth)
                    + (min_freq * max_freq).sqrt().ln() * (1.0 - depth))
                    .exp()
            } else {
                (min_freq * max_freq).sqrt()
            };

            self.filter.set_params(target_freq, resonance);
            let wet = self.filter.process_sample(input[i]);
            output[i] = (1.0 - mix) * input[i] + mix * wet;
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
    fn wahwah_defaults() {
        let w = WahWah::new();
        let p = w.parameters();
        assert!((p.rate - 2.0).abs() < 1e-6);
        assert!((p.depth - 1.0).abs() < 1e-6);
        assert!((p.min_frequency - 300.0).abs() < 1.0);
        assert!((p.max_frequency - 3000.0).abs() < 1.0);
        assert!((p.resonance - 0.6).abs() < 1e-6);
        assert!((p.mix - 0.8).abs() < 1e-6);
    }

    #[test]
    fn wahwah_passthrough_at_zero_mix() {
        let mut w = WahWah::new();
        w.prepare(44100.0, 512);
        w.set_parameters(WahWahParameters {
            mix: 0.0,
            ..Default::default()
        });

        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin()).collect();
        let mut output = vec![0.0_f32; 512];
        w.process(&input, &mut output);

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
    fn wahwah_full_wet_modifies_signal() {
        let mut w = WahWah::new();
        w.prepare(44100.0, 1024);
        w.set_parameters(WahWahParameters {
            mix: 1.0,
            depth: 1.0,
            rate: 2.0,
            resonance: 0.7,
            ..Default::default()
        });

        let input: Vec<f32> = (0..1024)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0_f32; 1024];
        w.process(&input, &mut output);

        let diff: f32 = input
            .iter()
            .zip(output.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 0.5, "wahwah should modify signal (diff={})", diff);
    }

    #[test]
    fn wahwah_depth_zero_produces_constant_filter() {
        let mut w = WahWah::new();
        w.prepare(44100.0, 256);
        w.set_parameters(WahWahParameters {
            depth: 0.0,
            mix: 1.0,
            ..Default::default()
        });

        let input = vec![0.5_f32; 256];
        let mut output = vec![0.0_f32; 256];
        w.process(&input, &mut output);

        // With depth=0 the filter should settle to a constant frequency.
        // After transient, the output should approach a steady state.
        let non_zero = output.iter().filter(|v| v.abs() > 1e-6).count();
        assert!(non_zero > 0, "should produce non-zero output");
    }

    #[test]
    fn wahwah_resonance_changes_output() {
        let mut w_low = WahWah::new();
        w_low.prepare(44100.0, 256);
        w_low.set_parameters(WahWahParameters {
            resonance: 0.1,
            mix: 1.0,
            depth: 0.0,
            ..Default::default()
        });

        let mut w_high = WahWah::new();
        w_high.prepare(44100.0, 256);
        w_high.set_parameters(WahWahParameters {
            resonance: 0.9,
            mix: 1.0,
            depth: 0.0,
            ..Default::default()
        });

        let input: Vec<f32> = (0..256)
            .map(|i| (2.0 * PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut out_low = vec![0.0_f32; 256];
        let mut out_high = vec![0.0_f32; 256];
        w_low.process(&input, &mut out_low);
        w_high.process(&input, &mut out_high);

        let diff: f32 = out_low
            .iter()
            .zip(out_high.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(
            diff > 0.001,
            "resonance should affect output (diff={})",
            diff
        );
    }

    #[test]
    fn tpt_bandpass_passes_center_frequency() {
        // A resonant bandpass at 1000 Hz should pass 1000 Hz and attenuate 100 Hz.
        let mut bp = TptBandpass::new();
        bp.set_sample_rate(44100.0);
        bp.set_params(1000.0, 0.8);

        // 1000 Hz signal (22 samples per cycle at 44100 Hz)
        let mut energy_1k = 0.0f32;
        for i in 0..4096 {
            let x = (2.0 * PI * 1000.0 * i as f32 / 44100.0).sin();
            let y = bp.process_sample(x);
            energy_1k += y * y;
        }

        bp.reset();
        bp.set_params(1000.0, 0.8);

        // 100 Hz signal
        let mut energy_100 = 0.0f32;
        for i in 0..4096 {
            let x = (2.0 * PI * 100.0 * i as f32 / 44100.0).sin();
            let y = bp.process_sample(x);
            energy_100 += y * y;
        }

        assert!(
            energy_1k > energy_100 * 2.0,
            "bandpass at 1000 Hz should pass 1k more than 100 Hz: 1k={} 100={}",
            energy_1k,
            energy_100
        );
    }
}
