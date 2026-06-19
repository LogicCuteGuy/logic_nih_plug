//! `juce_oscillator_demo` — 4-waveform oscillator (sine / saw / square / triangle).
//!
//! Ports `JUCE/examples/DSP/OscillatorDemo.h` to Rust. The plugin generates
//! a steady tone whose waveform is selected by the user via a parameter.
//!
//! # Examples
//!
//! ```rust,no_run
//! use juce_oscillator_demo::{Waveform, oscillator_value};
//!
//! // Sine wave at phase 0.0
//! assert!((oscillator_value(Waveform::Sine, 0.0) - 0.0).abs() < 1e-6);
//! // Sine at π/2 ≈ 1.0
//! assert!((oscillator_value(Waveform::Sine, 0.5) - 1.0).abs() < 1e-5);
//! // Square at 0.25 (positive half-cycle) = 1.0
//! assert!((oscillator_value(Waveform::Square, 0.25) - 1.0).abs() < 1e-6);
//! // Saw at 0.0 = 0.0
//! assert!((oscillator_value(Waveform::Saw, 0.0)).abs() < 1e-6);
//! ```

use logic_nih_plug::prelude::*;
use std::f32::consts::PI;
use std::sync::Arc;

/// Waveform selector matching JUCE's `OscillatorDemo` modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

/// Returns the oscillator output for a given waveform at the given normalised
/// phase (0.0–1.0 = one full cycle).
#[inline]
pub fn oscillator_value(waveform: Waveform, phase: f32) -> f32 {
    match waveform {
        Waveform::Sine => (phase * 2.0 * PI).sin(),
        // Saw: goes from −1 to +1 over one cycle
        Waveform::Saw => 2.0 * phase - 1.0,
        // Square: +1 for first half, −1 for second half
        Waveform::Square => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        // Triangle: −1 to +1 in first half, +1 to −1 in second half
        Waveform::Triangle => {
            if phase < 0.25 {
                4.0 * phase - 1.0
            } else if phase < 0.75 {
                2.0 - 4.0 * phase
            } else {
                4.0 * phase - 5.0
            }
        }
    }
}

pub struct OscillatorPlugin {
    params: Arc<OscillatorParams>,
    phase: f32,
}

#[derive(Params)]
struct OscillatorParams {
    #[id = "frequency"]
    pub frequency: FloatParam,
    #[id = "waveform"]
    pub waveform: IntParam,
}

impl Default for OscillatorPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(OscillatorParams::default()),
            phase: 0.0,
        }
    }
}

impl Default for OscillatorParams {
    fn default() -> Self {
        Self {
            frequency: FloatParam::new(
                "Frequency",
                440.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" Hz"),
            waveform: IntParam::new(
                "Waveform",
                0,
                IntRange::Linear {
                    min: 0,
                    max: 3,
                },
            ),
        }
    }
}

impl Plugin for OscillatorPlugin {
    const NAME: &'static str = "JUCE Oscillator Demo";
    const VENDOR: &'static str = "LogicCuteGuy";
    const URL: &'static str = "https://github.com/LogicCuteGuy/logic_nih_plug";
    const EMAIL: &'static str = "contact@logiccuteguy.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    type SysExMessage = ();
    type BackgroundTask = ();

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(0),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(0),
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
        let wf_idx = self.params.waveform.value();
        let waveform = match wf_idx {
            0 => Waveform::Sine,
            1 => Waveform::Saw,
            2 => Waveform::Square,
            _ => Waveform::Triangle,
        };

        for mut channel_samples in buffer.iter_samples() {
            let freq = self.params.frequency.smoothed.next();
            let value = oscillator_value(waveform, self.phase);

            // Advance phase
            self.phase += freq / sample_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }

            for sample in channel_samples.iter_mut() {
                *sample = value;
            }
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

impl ClapPlugin for OscillatorPlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_oscillator_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style 4-waveform oscillator");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for OscillatorPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceOscDemo\0\0\0\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}

nih_export_clap!(OscillatorPlugin);
nih_export_vst3!(OscillatorPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_at_zero_is_zero() {
        assert!(oscillator_value(Waveform::Sine, 0.0).abs() < 1e-6);
    }

    #[test]
    fn sine_at_quarter_is_one() {
        assert!((oscillator_value(Waveform::Sine, 0.25) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn square_symmetry() {
        let pos = oscillator_value(Waveform::Square, 0.25);
        let neg = oscillator_value(Waveform::Square, 0.75);
        assert!((pos + neg).abs() < 1e-6);
    }

    #[test]
    fn saw_range() {
        for i in 0..100 {
            let phase = i as f32 / 100.0;
            let v = oscillator_value(Waveform::Saw, phase);
            assert!(v >= -1.0 && v <= 1.0);
        }
    }
}
