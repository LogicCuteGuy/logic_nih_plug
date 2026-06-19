//! # Chorus example
//!
//! A stereo LFO-modulated chorus effect built on top of the [`Chorus`] processor
//! from `logic_nih_plug_dsp`. The [`Chorus`] processor itself is mono per channel,
//! so this example runs one instance per channel to get a true stereo effect.
//!
//! [`Chorus`]: logic_nih_plug_dsp::processors::chorus::Chorus

use logic_nih_plug::prelude::*;
use logic_nih_plug_dsp::processors::chorus::{Chorus, ChorusParameters};
use logic_nih_plug_dsp::processors::Processor;
use std::sync::Arc;

/// A stereo chorus / flanger / vibrato effect.
pub struct ChorusPlugin {
    params: Arc<ChorusParams>,
    chorus_l: Chorus,
    chorus_r: Chorus,
}

#[derive(Params)]
struct ChorusParams {
    /// LFO rate in Hz. Must be below 100 (clamped by the processor).
    #[id = "rate"]
    pub rate: FloatParam,

    /// Modulation depth in `[0.0, 1.0]`. 0 = no modulation, 1 = full modulation.
    #[id = "depth"]
    pub depth: FloatParam,

    /// Centre delay time in milliseconds. Lower values move toward flanger territory.
    #[id = "centre"]
    pub centre_delay: FloatParam,

    /// Feedback in `[-1.0, 1.0]`. Negative values invert the feedback phase.
    #[id = "fb"]
    pub feedback: FloatParam,

    /// Dry/wet mix in `[0.0, 1.0]`. Set to 1.0 for vibrato-only.
    #[id = "mix"]
    pub mix: FloatParam,
}

impl Default for ChorusPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ChorusParams::default()),
            chorus_l: Chorus::new(),
            chorus_r: Chorus::new(),
        }
    }
}

impl Default for ChorusParams {
    fn default() -> Self {
        Self {
            rate: FloatParam::new(
                "Rate",
                1.0,
                FloatRange::Skewed {
                    min: 0.01,
                    max: 50.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            depth: FloatParam::new(
                "Depth",
                0.25,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            centre_delay: FloatParam::new(
                "Centre Delay",
                7.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 50.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            feedback: FloatParam::new(
                "Feedback",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            mix: FloatParam::new(
                "Mix",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),
        }
    }
}

impl Plugin for ChorusPlugin {
    const NAME: &'static str = "Chorus";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let max_block = buffer_config.max_buffer_size as usize;
        self.chorus_l
            .prepare(buffer_config.sample_rate, max_block);
        self.chorus_r
            .prepare(buffer_config.sample_rate, max_block);
        self.apply_parameters();

        true
    }

    fn reset(&mut self) {
        // Re-applying the parameters also re-zeros the smoother state.
        self.apply_parameters();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.apply_parameters();

        let num_samples = buffer.samples();
        let slices = buffer.as_slice();

        if slices.len() >= 2 {
            let (left_channels, rest) = slices.split_at_mut(1);
            let (right_channels, _) = rest.split_at_mut(1);
            let left = &mut left_channels[0][..num_samples];
            let right = &mut right_channels[0][..num_samples];

            // The `Processor` trait takes a separate input and output slice, so we
            // copy into a scratch buffer first and write back. The input is dry at
            // this point because the wrapper copies inputs into outputs ahead of
            // `process`.
            let mut scratch_l = vec![0.0_f32; num_samples];
            let mut scratch_r = vec![0.0_f32; num_samples];
            self.chorus_l.process(left, &mut scratch_l);
            self.chorus_r.process(right, &mut scratch_r);
            left.copy_from_slice(&scratch_l);
            right.copy_from_slice(&scratch_r);
        } else {
            // Mono: just process the single channel.
            let mono = &mut slices[0][..num_samples];
            let mut scratch = vec![0.0_f32; num_samples];
            self.chorus_l.process(mono, &mut scratch);
            mono.copy_from_slice(&scratch);
        }

        ProcessStatus::Normal
    }
}

impl ChorusPlugin {
    fn apply_parameters(&mut self) {
        let parameters = ChorusParameters {
            rate: self.params.rate.smoothed.next(),
            depth: self.params.depth.smoothed.next(),
            centre_delay: self.params.centre_delay.smoothed.next(),
            feedback: self.params.feedback.smoothed.next(),
            mix: self.params.mix.smoothed.next(),
        };

        self.chorus_l.set_parameters(parameters);
        self.chorus_r.set_parameters(parameters);
    }
}

impl ClapPlugin for ChorusPlugin {
    const CLAP_ID: &'static str = "com.nih-plug.chorus";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Stereo LFO-modulated chorus example");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Chorus,
    ];
}

impl Vst3Plugin for ChorusPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"ChorusNihPlgExmp";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Modulation,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(ChorusPlugin);
nih_export_vst3!(ChorusPlugin);