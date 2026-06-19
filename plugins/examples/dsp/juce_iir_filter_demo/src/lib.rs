//! `juce_iir_filter_demo` — IIR filter (low-pass / high-pass / band-pass).
//!
//! Ports `JUCE/examples/DSP/FilterExample.h` to Rust. Uses a second-order
//! (biquad) direct-form I implementation with pre-computed coefficients for
//! Butterworth response.
//!
//! # Examples
//!
//! ```
//! use juce_iir_filter_demo::{biquad_coefficients, FilterType, process_biquad};
//!
//! // 1 kHz low-pass at 44.1 kHz sample rate
//! let (b, a) = biquad_coefficients(FilterType::LowPass, 44100.0, 1000.0, 0.7071);
//! let mut state = [0.0_f32; 4];
//! // DC passes through a low-pass
//! let out = process_biquad(&b, &a, &mut state, 1.0);
//! assert!((out - 1.0).abs() < 0.01);
//! ```

use logic_nih_plug::prelude::*;
use std::f32::consts::PI;
use std::sync::Arc;

/// Filter type selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
}

/// Compute second-order Butterworth biquad coefficients.
///
/// Returns `(b, a)` where `b = [b0, b1, b2]` and `a = [1.0, a1, a2]`.
pub fn biquad_coefficients(ftype: FilterType, sample_rate: f32, freq: f32, q: f32) -> ([f32; 3], [f32; 3]) {
    let omega = 2.0 * PI * freq / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q);

    let (b0, b1, b2, a0, a1, a2) = match ftype {
        FilterType::LowPass => {
            let b0 = (1.0 - cos_omega) / 2.0;
            let b1 = 1.0 - cos_omega;
            let b2 = (1.0 - cos_omega) / 2.0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::HighPass => {
            let b0 = (1.0 + cos_omega) / 2.0;
            let b1 = -(1.0 + cos_omega);
            let b2 = (1.0 + cos_omega) / 2.0;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
        FilterType::BandPass => {
            let b0 = alpha;
            let b1 = 0.0;
            let b2 = -alpha;
            let a0 = 1.0 + alpha;
            let a1 = -2.0 * cos_omega;
            let a2 = 1.0 - alpha;
            (b0, b1, b2, a0, a1, a2)
        }
    };

    let inv_a0 = 1.0 / a0;
    (
        [b0 * inv_a0, b1 * inv_a0, b2 * inv_a0],
        [1.0, a1 * inv_a0, a2 * inv_a0],
    )
}

/// Process one sample through a biquad filter (direct-form I).
///
/// `state` must be `[x1, x2, y1, y2]`.
#[inline]
pub fn process_biquad(b: &[f32; 3], a: &[f32; 3], state: &mut [f32; 4], input: f32) -> f32 {
    let output = b[0] * input + b[1] * state[0] + b[2] * state[1]
        - a[1] * state[2] - a[2] * state[3];
    state[1] = state[0];
    state[0] = input;
    state[3] = state[2];
    state[2] = output;
    output
}

pub struct IirFilterPlugin {
    params: Arc<IirFilterParams>,
    // Per-channel filter state: [x1, x2, y1, y2]
    states: Vec<[f32; 4]>,
    sample_rate: f32,
}

#[derive(Params)]
struct IirFilterParams {
    #[id = "cutoff"]
    pub cutoff: FloatParam,
    #[id = "resonance"]
    pub resonance: FloatParam,
    #[id = "filter_type"]
    pub filter_type: IntParam,
}

impl Default for IirFilterPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(IirFilterParams::default()),
            states: vec![[0.0; 4]; 2],
            sample_rate: 44100.0,
        }
    }
}

impl Default for IirFilterParams {
    fn default() -> Self {
        Self {
            cutoff: FloatParam::new(
                "Cutoff",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" Hz"),
            resonance: FloatParam::new(
                "Resonance",
                0.7071,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 20.0,
                    factor: FloatRange::skew_factor(0.5),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
            filter_type: IntParam::new("Type", 0, IntRange::Linear { min: 0, max: 2 }),
        }
    }
}

impl Plugin for IirFilterPlugin {
    const NAME: &'static str = "JUCE IIR Filter Demo";
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

        // Recompute coefficients per sample (smoothed cutoff/resonance)
        let ftype = match self.params.filter_type.value() {
            0 => FilterType::LowPass,
            1 => FilterType::HighPass,
            _ => FilterType::BandPass,
        };

        for (ch_idx, mut channel_samples) in buffer.iter_samples().enumerate() {
            // Ensure state array is large enough
            if ch_idx >= self.states.len() {
                self.states.resize(ch_idx + 1, [0.0; 4]);
            }

            let cutoff = self.params.cutoff.smoothed.next();
            let q = self.params.resonance.smoothed.next();
            let (b, a) = biquad_coefficients(ftype, self.sample_rate, cutoff, q);

            for sample in channel_samples.iter_mut() {
                let out = process_biquad(&b, &a, &mut self.states[ch_idx], *sample);
                *sample = out;
            }
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        for state in &mut self.states {
            *state = [0.0; 4];
        }
    }
}

impl ClapPlugin for IirFilterPlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_iir_filter_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style IIR biquad filter (LP/HP/BP)");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Filter,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for IirFilterPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceIIRFilter\0\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Filter];
}

nih_export_clap!(IirFilterPlugin);
nih_export_vst3!(IirFilterPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowpass_passes_dc() {
        let (b, a) = biquad_coefficients(FilterType::LowPass, 44100.0, 1000.0, 0.7071);
        let mut state = [0.0; 4];
        // DC input should pass through ~1.0 after settling
        let mut last = 0.0;
        for _ in 0..1000 {
            last = process_biquad(&b, &a, &mut state, 1.0);
        }
        assert!((last - 1.0).abs() < 0.01, "DC output = {last}");
    }

    #[test]
    fn highpass_blocks_dc() {
        let (b, a) = biquad_coefficients(FilterType::HighPass, 44100.0, 1000.0, 0.7071);
        let mut state = [0.0; 4];
        let mut last = 0.0;
        for _ in 0..1000 {
            last = process_biquad(&b, &a, &mut state, 1.0);
        }
        assert!(last.abs() < 0.01, "HP DC output = {last}");
    }
}
