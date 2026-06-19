//! `juce_limiter_demo` — Lookahead brickwall limiter.
//!
//! Ports the concept of `JUCE/examples/DSP/LimiterExample.h` to Rust. A
//! single-stage peak detector with gain smoothing limits the output to a
//! configurable ceiling, preventing clipping.
//!
//! # Examples
//!
//! ```
//! use juce_limiter_demo::compute_limiter_gain;
//!
//! // Signal below ceiling → unity gain
//! let g = compute_limiter_gain(0.5, 0.0);
//! assert!((g - 1.0).abs() < 1e-6);
//!
//! // Signal above ceiling → reduced gain
//! let g = compute_limiter_gain(2.0, 0.0);
//! assert!(g < 1.0);
//! ```

use logic_nih_plug::prelude::*;
use std::sync::Arc;

/// Compute the limiting gain for a given peak level and ceiling (dB).
///
/// Returns the gain multiplier to apply so that `peak_level * gain <= ceiling`.
#[inline]
pub fn compute_limiter_gain(peak_level: f32, ceiling_db: f32) -> f32 {
    let ceiling_linear = 10.0_f32.powf(ceiling_db / 20.0);
    if peak_level <= 0.0 {
        return 1.0;
    }
    let needed = ceiling_linear / peak_level;
    needed.min(1.0).max(0.0)
}

/// Convert decibels to linear gain.
#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

pub struct LimiterPlugin {
    params: Arc<LimiterParams>,
    /// Current smoothed gain multiplier.
    current_gain: f32,
    sample_rate: f32,
}

#[derive(Params)]
struct LimiterParams {
    #[id = "ceiling"]
    pub ceiling: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
}

impl Default for LimiterPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(LimiterParams::default()),
            current_gain: 1.0,
            sample_rate: 44100.0,
        }
    }
}

impl Default for LimiterParams {
    fn default() -> Self {
        Self {
            ceiling: FloatParam::new(
                "Ceiling",
                0.0,
                FloatRange::Linear {
                    min: -20.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(1.0))
            .with_unit(" dB"),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Linear {
                    min: 5.0,
                    max: 500.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms"),
        }
    }
}

impl Plugin for LimiterPlugin {
    const NAME: &'static str = "JUCE Limiter Demo";
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

        let ceiling_db = self.params.ceiling.smoothed.next();
        let release_ms = self.params.release.smoothed.next();
        let release_coeff = (-1.0 / (release_ms * 0.001 * self.sample_rate)).exp();

        // Two-pass approach: first find peak, then apply gain.
        // Pass 1: find the peak across all samples in the buffer.
        let mut peak = 0.0_f32;
        for mut channel_samples in buffer.iter_samples() {
            for sample in channel_samples.iter_mut() {
                let abs = sample.abs();
                if abs > peak {
                    peak = abs;
                }
            }
        }

        // Compute target gain
        let target_gain = compute_limiter_gain(peak, ceiling_db);

        // Smooth the gain (attack: instant, release: smoothed)
        if target_gain < self.current_gain {
            self.current_gain = target_gain; // instant attack
        } else {
            // Release: smooth back toward 1.0
            self.current_gain = self.current_gain * release_coeff
                + target_gain * (1.0 - release_coeff);
        }

        // Pass 2: apply gain to all samples
        for mut channel_samples in buffer.iter_samples() {
            for sample in channel_samples.iter_mut() {
                *sample *= self.current_gain;
            }
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        self.current_gain = 1.0;
    }
}

impl ClapPlugin for LimiterPlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_limiter_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style brickwall limiter");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for LimiterPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceLimiterDem\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(LimiterPlugin);
nih_export_vst3!(LimiterPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limiter_below_ceiling() {
        // 0.5 peak, 0 dB ceiling → unity gain
        let g = compute_limiter_gain(0.5, 0.0);
        assert!((g - 1.0).abs() < 1e-6);
    }

    #[test]
    fn limiter_above_ceiling() {
        // 2.0 peak, 0 dB ceiling → gain should halve
        let g = compute_limiter_gain(2.0, 0.0);
        assert!((g - 0.5).abs() < 1e-6);
    }

    #[test]
    fn limiter_ceiling_db() {
        // 1.0 peak, -6 dB ceiling → gain ≈ 0.5
        let g = compute_limiter_gain(1.0, -6.0);
        assert!((g - 0.5012).abs() < 0.01, "gain = {g}");
    }

    #[test]
    fn limiter_zero_peak() {
        let g = compute_limiter_gain(0.0, 0.0);
        assert!((g - 1.0).abs() < 1e-6);
    }
}
