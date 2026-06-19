//! `juce_noise_gate_demo` — Noise gate / downward expander with envelope follower.
//!
//! Ports `JUCE/examples/DSP/NoiseGateExample.h` to Rust. An envelope follower
//! detects the signal level and attenuates when below the threshold, creating
//! a gate or expander effect.
//!
//! # Examples
//!
//! ```
//! use juce_noise_gate_demo::envelope_follower;
//!
//! // Envelope should rise toward the input level
//! let mut env = 0.0_f32;
//! for _ in 0..1000 {
//!     env = envelope_follower(env, 1.0, 0.999);
//! }
//! assert!((env - 1.0).abs() < 0.01, "env = {env}");
//!
//! // Gate gain for signal above threshold
//! use juce_noise_gate_demo::gate_gain;
//! let g = gate_gain(0.5, -6.0, 1.0);
//! assert!((g - 1.0).abs() < 1e-6); // above threshold = unity gain
//! ```

use logic_nih_plug::prelude::*;
use std::sync::Arc;

/// Convert decibels to linear gain.
#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Simple first-order envelope follower (attack/release).
///
/// `current_env` is the previous envelope value, `input_level` is the
/// instantaneous |x| or RMS value, `smoothing` is the per-sample coefficient.
#[inline]
pub fn envelope_follower(current_env: f32, input_level: f32, smoothing: f32) -> f32 {
    // Exponential smoothing: env = env * alpha + level * (1 - alpha)
    // For attack: low alpha (fast); for release: high alpha (slow)
    current_env * smoothing + input_level * (1.0 - smoothing)
}

/// Compute the gate gain given the envelope level, threshold, and ratio.
///
/// When `env >= threshold`, returns 1.0 (open gate).
/// When `env < threshold`, returns `(env / threshold)^(ratio - 1)`.
#[inline]
pub fn gate_gain(env: f32, threshold_db: f32, ratio: f32) -> f32 {
    let threshold = db_to_gain(threshold_db);
    if env >= threshold || threshold <= 0.0 {
        1.0
    } else {
        (env / threshold).powf(ratio - 1.0).clamp(0.0, 1.0)
    }
}

pub struct NoiseGatePlugin {
    params: Arc<NoiseGateParams>,
    /// Per-channel envelope values.
    envelopes: Vec<f32>,
    sample_rate: f32,
}

#[derive(Params)]
struct NoiseGateParams {
    #[id = "threshold"]
    pub threshold: FloatParam,
    #[id = "ratio"]
    pub ratio: FloatParam,
    #[id = "attack"]
    pub attack: FloatParam,
    #[id = "release"]
    pub release: FloatParam,
}

impl Default for NoiseGatePlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(NoiseGateParams::default()),
            envelopes: vec![0.0; 2],
            sample_rate: 44100.0,
        }
    }
}

impl Default for NoiseGateParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                -40.0,
                FloatRange::Linear {
                    min: -80.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB"),
            ratio: FloatParam::new(
                "Ratio",
                10.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0)),
            attack: FloatParam::new(
                "Attack",
                1.0,
                FloatRange::Linear {
                    min: 0.1,
                    max: 100.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms"),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Linear {
                    min: 10.0,
                    max: 1000.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms"),
        }
    }
}

impl Plugin for NoiseGatePlugin {
    const NAME: &'static str = "JUCE Noise Gate Demo";
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

        let threshold_db = self.params.threshold.smoothed.next();
        let ratio = self.params.ratio.smoothed.next();

        // Attack: 0.1 ms → 100 ms. Convert to coefficient.
        let attack_ms = self.params.attack.smoothed.next();
        let attack_coeff = (-1.0 / (attack_ms * 0.001 * self.sample_rate)).exp();

        // Release: 10 ms → 1000 ms
        let release_ms = self.params.release.smoothed.next();
        let release_coeff = (-1.0 / (release_ms * 0.001 * self.sample_rate)).exp();

        for (ch_idx, mut channel_samples) in buffer.iter_samples().enumerate() {
            if ch_idx >= self.envelopes.len() {
                self.envelopes.resize(ch_idx + 1, 0.0);
            }

            for sample in channel_samples.iter_mut() {
                let abs_level = sample.abs();

                // Envelope follower: attack/release
                let target = abs_level;
                let coeff = if target > self.envelopes[ch_idx] {
                    attack_coeff
                } else {
                    release_coeff
                };
                self.envelopes[ch_idx] =
                    envelope_follower(self.envelopes[ch_idx], target, coeff);

                // Compute gate gain
                let gain = gate_gain(self.envelopes[ch_idx], threshold_db, ratio);
                *sample *= gain;
            }
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        for env in &mut self.envelopes {
            *env = 0.0;
        }
    }
}

impl ClapPlugin for NoiseGatePlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_noise_gate_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style noise gate / downward expander");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for NoiseGatePlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceNoiseGate\0\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(NoiseGatePlugin);
nih_export_vst3!(NoiseGatePlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_converges_to_level() {
        let mut env = 0.0_f32;
        for _ in 0..5000 {
            env = envelope_follower(env, 1.0, 0.999);
        }
        assert!((env - 1.0).abs() < 0.01, "env = {env}");
    }

    #[test]
    fn gate_open_above_threshold() {
        let g = gate_gain(1.0, -6.0, 10.0);
        assert!((g - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gate_closed_below_threshold() {
        let g = gate_gain(0.001, -20.0, 100.0);
        assert!(g < 0.1, "gate gain = {g}");
    }
}
