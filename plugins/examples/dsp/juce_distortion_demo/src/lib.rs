//! `juce_distortion_demo` — soft-clip overdrive distortion.
//!
//! Ports `JUCE/examples/DSP/OverdriveDemo.h` to Rust. The signal flow is:
//!
//! ```text
//! input -> drive gain -> soft-clip (tanh) -> output gain -> output
//! ```
//!
//! # Examples
//!
//! ```
//! use juce_distortion_demo::soft_clip;
//! let out = soft_clip(2.0);
//! assert!(out > 0.9 && out < 1.0);
//! ```

use logic_nih_plug::prelude::*;
use std::sync::Arc;

/// Tanh-based soft-clip. Maps `x` into `(-1, 1)` smoothly.
#[inline]
pub fn soft_clip(x: f32) -> f32 {
    x.tanh()
}

pub struct DistortionPlugin {
    params: Arc<DistortionParams>,
}

#[derive(Params)]
struct DistortionParams {
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "output"]
    pub output: FloatParam,
}

impl Default for DistortionPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DistortionParams::default()),
        }
    }
}

impl Default for DistortionParams {
    fn default() -> Self {
        Self {
            drive: FloatParam::new(
                "Drive",
                1.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 10.0,
                    factor: FloatRange::gain_skew_factor(1.0, 10.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0)),
            output: FloatParam::new(
                "Output",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

impl Plugin for DistortionPlugin {
    const NAME: &'static str = "JUCE Distortion Demo";
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
        for mut channel_samples in buffer.iter_samples() {
            let drive = self.params.drive.smoothed.next();
            let output = self.params.output.smoothed.next();
            for sample in channel_samples.iter_mut() {
                *sample = soft_clip(drive * *sample) * output;
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for DistortionPlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_distortion_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style soft-clip overdrive distortion");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Distortion,
    ];
}

impl Vst3Plugin for DistortionPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceDistortion!\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(DistortionPlugin);
nih_export_vst3!(DistortionPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_test_assertion() {
        let out = soft_clip(2.0);
        assert!(out > 0.9 && out < 1.0, "out = {out}");
    }

    #[test]
    fn soft_clip_zero_is_zero() {
        assert_eq!(soft_clip(0.0), 0.0);
    }

    #[test]
    fn soft_clip_is_symmetric() {
        assert!((soft_clip(1.0) + soft_clip(-1.0)).abs() < 1e-6);
    }
}
