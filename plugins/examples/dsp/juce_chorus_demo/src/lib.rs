//! `juce_chorus_demo` — Modulated delay-line chorus / flanger / vibrato.
//!
//! Ports `JUCE/examples/DSP/ChorusExample.h` to Rust. A sine LFO sweeps the
//! delay time of a circular delay buffer, creating the classic sweeping notches
//! of chorus (longer delay, low feedback) or flanger (short delay, high feedback).
//!
//! # Examples
//!
//! ```
//! use juce_chorus_demo::lfo_sine;
//!
//! // LFO at phase 0.5 should be ≈ 1.0
//! let v = lfo_sine(0.5);
//! assert!((v - 1.0).abs() < 0.01);
//! ```

use logic_nih_plug::prelude::*;
use std::f32::consts::PI;
use std::sync::Arc;

/// Maximum delay in samples (20 ms at 96 kHz worst case).
const MAX_DELAY_SAMPLES: usize = 1920;

/// Sine LFO at normalised phase.
#[inline]
pub fn lfo_sine(phase: f32) -> f32 {
    (phase * 2.0 * PI).sin()
}

/// Circular delay line with linear interpolation.
#[derive(Clone)]
struct DelayLine {
    buffer: [f32; MAX_DELAY_SAMPLES],
    write_pos: usize,
}

impl DelayLine {
    fn new() -> Self {
        Self {
            buffer: [0.0; MAX_DELAY_SAMPLES],
            write_pos: 0,
        }
    }

    fn write(&mut self, sample: f32) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % MAX_DELAY_SAMPLES;
    }

    /// Read from the delay line at `delay_samples` (fractional, linear interp).
    fn read(&self, delay_samples: f32) -> f32 {
        let delay_samples = delay_samples.min(MAX_DELAY_SAMPLES as f32 - 1.0).max(0.0);
        let int_delay = delay_samples as usize;
        let frac = delay_samples - int_delay as f32;
        let read_pos = (self.write_pos + MAX_DELAY_SAMPLES - 1 - int_delay) % MAX_DELAY_SAMPLES;
        let next_pos = (read_pos + MAX_DELAY_SAMPLES - 1) % MAX_DELAY_SAMPLES;
        self.buffer[read_pos] * (1.0 - frac) + self.buffer[next_pos] * frac
    }

    fn clear(&mut self) {
        self.buffer = [0.0; MAX_DELAY_SAMPLES];
        self.write_pos = 0;
    }
}

pub struct ChorusPlugin {
    params: Arc<ChorusParams>,
    delay_lines: Vec<DelayLine>,
    /// Per-channel LFO phase.
    lfo_phases: Vec<f32>,
    /// Previous wet output for feedback.
    prev_wet: Vec<f32>,
    sample_rate: f32,
}

#[derive(Params)]
struct ChorusParams {
    #[id = "rate"]
    pub rate: FloatParam,
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "centre_delay"]
    pub centre_delay: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for ChorusPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ChorusParams::default()),
            delay_lines: vec![DelayLine::new(); 2],
            lfo_phases: vec![0.0; 2],
            prev_wet: vec![0.0; 2],
            sample_rate: 44100.0,
        }
    }
}

impl Default for ChorusParams {
    fn default() -> Self {
        Self {
            rate: FloatParam::new("Rate", 1.0, FloatRange::Linear { min: 0.05, max: 10.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" Hz"),
            depth: FloatParam::new("Depth", 0.25, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
            centre_delay: FloatParam::new(
                "Centre Delay",
                7.0,
                FloatRange::Linear { min: 1.0, max: 50.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms"),
            feedback: FloatParam::new("Feedback", 0.0, FloatRange::Linear { min: -0.95, max: 0.95 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

impl Plugin for ChorusPlugin {
    const NAME: &'static str = "JUCE Chorus Demo";
    const VENDOR: &'static str = "LogicCuteGuy";
    const URL: &'static str = "https://github.com/LogicCuteGuy/logic_nih_plug";
    const EMAIL: &'static str = "contact@logiccuteguy.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    type SysExMessage = ();
    type BackgroundTask = ();

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = _context.transport().sample_rate;
        if (sample_rate - self.sample_rate).abs() > 0.1 {
            self.sample_rate = sample_rate;
        }

        for (ch_idx, mut channel_samples) in buffer.iter_samples().enumerate() {
            // Ensure per-channel vectors are large enough
            if ch_idx >= self.delay_lines.len() {
                self.delay_lines.resize(ch_idx + 1, DelayLine::new());
                self.lfo_phases.resize(ch_idx + 1, 0.0);
                self.prev_wet.resize(ch_idx + 1, 0.0);
            }

            let rate = self.params.rate.smoothed.next();
            let depth = self.params.depth.smoothed.next();
            let centre_ms = self.params.centre_delay.smoothed.next();
            let fb = self.params.feedback.smoothed.next();
            let mix = self.params.mix.smoothed.next();

            let centre_samples = centre_ms * 0.001 * self.sample_rate;
            let max_mod = 20.0 * 0.001 * self.sample_rate; // 20 ms max

            for sample in channel_samples.iter_mut() {
                let input = *sample;

                // Write input + feedback into delay line
                let dl = &mut self.delay_lines[ch_idx];
                dl.write(input + fb * self.prev_wet[ch_idx]);

                // LFO-modulated delay time
                let lfo = lfo_sine(self.lfo_phases[ch_idx]);
                let delay_time = centre_samples + depth * lfo * max_mod;
                let delay_time = delay_time.max(0.5); // minimum to avoid DC

                let wet = dl.read(delay_time);
                self.prev_wet[ch_idx] = wet;

                // Advance LFO
                self.lfo_phases[ch_idx] += rate / self.sample_rate;
                if self.lfo_phases[ch_idx] >= 1.0 {
                    self.lfo_phases[ch_idx] -= 1.0;
                }

                *sample = input * (1.0 - mix) + wet * mix;
            }
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        for dl in &mut self.delay_lines {
            dl.clear();
        }
        for phase in &mut self.lfo_phases {
            *phase = 0.0;
        }
        for w in &mut self.prev_wet {
            *w = 0.0;
        }
    }
}

impl ClapPlugin for ChorusPlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_chorus_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style modulated delay chorus / flanger");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for ChorusPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceChorusDem\0\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

nih_export_clap!(ChorusPlugin);
nih_export_vst3!(ChorusPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delay_line_passthrough() {
        let mut dl = DelayLine::new();
        for i in 0..100 {
            dl.write(i as f32);
        }
        // Read at 0 delay = most recent sample
        let val = dl.read(0.5);
        assert!((val - 99.0).abs() < 1.0, "got {val}");
    }

    #[test]
    fn lfo_sine_values() {
        assert!((lfo_sine(0.0)).abs() < 1e-6);
        assert!((lfo_sine(0.25) - 1.0).abs() < 1e-5);
    }
}
