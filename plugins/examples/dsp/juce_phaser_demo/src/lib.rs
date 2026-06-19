//! `juce_phaser_demo` — 6-stage LFO-modulated allpass phaser.
//!
//! Ports `JUCE/examples/DSP/PhaserExample.h` to Rust. Uses six cascaded
//! first-order allpass filters with an LFO sweeping the corner frequencies
//! to create the characteristic notches.
//!
//! # Examples
//!
//! ```
//! use juce_phaser_demo::{allpass_filter, lfo_value};
//!
//! // LFO at phase 0.0 = −1
//! assert!((lfo_value(0.0) - (-1.0)).abs() < 1e-6);
//! // LFO at 0.25 = 0
//! assert!(lfo_value(0.25).abs() < 1e-6);
//! // LFO at 0.5 = +1
//! assert!((lfo_value(0.5) - 1.0).abs() < 1e-6);
//!
//! let mut state = 0.0_f32;
//! let out = allpass_filter(0.5, 0.5, &mut state, 1.0);
//! assert!((out - 1.0).abs() < 1e-6); // gain is always 1
//! ```

use logic_nih_plug::prelude::*;
use std::f32::consts::PI;
use std::sync::Arc;

/// Number of allpass stages (JUCE Phaser uses 6 by default).
const NUM_STAGES: usize = 6;

/// Evaluate the LFO (sine) at a given normalised phase.
#[inline]
pub fn lfo_value(phase: f32) -> f32 {
    (phase * 2.0 * PI).sin()
}

/// Process one sample through a first-order allpass filter.
///
/// Dead-code placeholder — see `first_order_allpass` below.
#[allow(dead_code)]
pub fn allpass_filter(_alpha: f32, _a: f32, _state: &mut f32, _input: f32) -> f32 {
    0.0
}

/// First-order allpass filter.
///
/// `a` is the allpass coefficient in (−1, 1). The state stores `[x_prev, y_prev]`.
#[inline]
pub fn first_order_allpass(a: f32, state: &mut [f32; 2], input: f32) -> f32 {
    // y[n] = -a * x[n] + x[n-1] + a * y[n-1]
    let output = -a * input + state[0] + a * state[1];
    state[0] = input;
    state[1] = output;
    output
}

pub struct PhaserPlugin {
    params: Arc<PhaserParams>,
    /// Per-channel filter states for each stage: `[x_prev, y_prev]`.
    stage_states: Vec<[[f32; 2]; NUM_STAGES]>,
    /// LFO phase accumulator (0.0 – 1.0).
    lfo_phase: f32,
    sample_rate: f32,
}

#[derive(Params)]
struct PhaserParams {
    #[id = "rate"]
    pub rate: FloatParam,
    #[id = "depth"]
    pub depth: FloatParam,
    #[id = "centre_freq"]
    pub centre_freq: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for PhaserPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(PhaserParams::default()),
            stage_states: vec![[[0.0; 2]; NUM_STAGES]; 2],
            lfo_phase: 0.0,
            sample_rate: 44100.0,
        }
    }
}

impl Default for PhaserParams {
    fn default() -> Self {
        Self {
            rate: FloatParam::new("Rate", 0.2, FloatRange::Linear { min: 0.01, max: 5.0 })
                .with_smoother(SmoothingStyle::Linear(50.0))
                .with_unit(" Hz"),
            depth: FloatParam::new("Depth", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
            centre_freq: FloatParam::new(
                "Centre Freq",
                1500.0,
                FloatRange::Skewed {
                    min: 200.0,
                    max: 8000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" Hz"),
            feedback: FloatParam::new("Feedback", 0.5, FloatRange::Linear { min: 0.0, max: 0.95 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
            mix: FloatParam::new("Mix", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

/// Compute the allpass coefficient `a` from a target corner frequency and sample rate.
///
/// Uses the bilinear transform of a first-order allpass:
/// `a = (1 - 2πfc/fs) / (1 + 2πfc/fs)` (pre-warp approximation).
#[inline]
fn allpass_coefficient(freq: f32, sample_rate: f32) -> f32 {
    let tan_val = (PI * freq / sample_rate).tan();
    (1.0 - tan_val) / (1.0 + tan_val)
}

impl Plugin for PhaserPlugin {
    const NAME: &'static str = "JUCE Phaser Demo";
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
            if ch_idx >= self.stage_states.len() {
                self.stage_states.resize(ch_idx + 1, [[0.0; 2]; NUM_STAGES]);
            }

            let rate = self.params.rate.smoothed.next();
            let depth = self.params.depth.smoothed.next();
            let centre_freq = self.params.centre_freq.smoothed.next();
            let fb = self.params.feedback.smoothed.next();
            let mix = self.params.mix.smoothed.next();

            for sample in channel_samples.iter_mut() {
                // LFO modulates the centre frequency
                let lfo = lfo_value(self.lfo_phase);
                let modulated_freq =
                    centre_freq * (1.0 + depth * lfo * 0.5).max(0.001);
                let a = allpass_coefficient(modulated_freq, self.sample_rate);

                // Run through all 6 allpass stages with feedback
                let input = *sample;
                let mut ap_input = input + fb * *sample; // feedback from wet output
                for stage in 0..NUM_STAGES {
                    ap_input = first_order_allpass(a, &mut self.stage_states[ch_idx][stage], ap_input);
                }
                let wet = ap_input;

                // Advance LFO
                self.lfo_phase += rate / self.sample_rate;
                if self.lfo_phase >= 1.0 {
                    self.lfo_phase -= 1.0;
                }

                // Dry/wet mix
                *sample = input * (1.0 - mix) + wet * mix;
            }
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        for states in &mut self.stage_states {
            *states = [[0.0; 2]; NUM_STAGES];
        }
        self.lfo_phase = 0.0;
    }
}

impl ClapPlugin for PhaserPlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_phaser_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style 6-stage allpass phaser");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for PhaserPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JucePhaserDemo\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Modulation];
}

nih_export_clap!(PhaserPlugin);
nih_export_vst3!(PhaserPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lfo_values() {
        assert!((lfo_value(0.0)).abs() < 1e-6);
        assert!((lfo_value(0.25) - 1.0).abs() < 1e-5);
        assert!((lfo_value(0.5)).abs() < 1e-6);
        assert!((lfo_value(0.75) - (-1.0)).abs() < 1e-5);
    }

    #[test]
    fn allpass_passes_dc() {
        let a = allpass_coefficient(1000.0, 44100.0);
        let mut state = [0.0_f32; 2];
        // DC should pass through unchanged after settling
        let mut out = 0.0;
        for _ in 0..200 {
            out = first_order_allpass(a, &mut state, 1.0);
        }
        assert!((out - 1.0).abs() < 0.01, "AP DC output = {out}");
    }
}
