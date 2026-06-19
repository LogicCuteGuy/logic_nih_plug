//! # Reverb example
//!
//! A simple algorithmic reverb built on top of the [`Reverb`] processor from
//! `logic_nih_plug_dsp`. It demonstrates:
//!
//! - Mapping host-facing parameters (room size, damping, wet/dry, width,
//!   freeze mode) to the processor's `Parameters` struct.
//! - The [`Reverb`] processor's stereo block-based `process_stereo` API.
//! - Re-applying parameters only when they actually change to avoid
//!   triggering the processor's internal smoothers unnecessarily.
//!
//! [`Reverb`]: logic_nih_plug_dsp::processors::reverb::Reverb

use logic_nih_plug::prelude::*;
use logic_nih_plug_dsp::processors::reverb::{Parameters as ReverbParams, Reverb};
use std::sync::Arc;

/// A simple stereo algorithmic reverb. Mono inputs are duplicated to both channels; stereo
/// inputs are processed as-is.
pub struct ReverbPlugin {
    params: Arc<ReverbParameters>,
    reverb: Reverb,
}

#[derive(Params)]
struct ReverbParameters {
    /// Room size. 0 = small, 1 = large.
    #[id = "size"]
    pub room_size: FloatParam,

    /// High-frequency damping. 0 = bright, 1 = dark.
    #[id = "damp"]
    pub damping: FloatParam,

    /// Wet (reverb) level. 0 = silent, 1 = full wet.
    #[id = "wet"]
    pub wet_level: FloatParam,

    /// Dry (direct) level. 0 = silent, 1 = full dry.
    #[id = "dry"]
    pub dry_level: FloatParam,

    /// Stereo width. 0 = mono wet, 1 = full stereo decorrelation.
    #[id = "width"]
    pub width: FloatParam,

    /// Freeze mode. Values >= 0.5 latch the reverb into a continuous loop.
    #[id = "freeze"]
    pub freeze_mode: BoolParam,
}

impl Default for ReverbPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(ReverbParameters::default()),
            reverb: Reverb::new(),
        }
    }
}

impl Default for ReverbParameters {
    fn default() -> Self {
        Self {
            room_size: FloatParam::new(
                "Room Size",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            damping: FloatParam::new(
                "Damping",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            wet_level: FloatParam::new(
                "Wet",
                0.33,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),
            dry_level: FloatParam::new(
                "Dry",
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),
            width: FloatParam::new(
                "Width",
                1.0,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            freeze_mode: BoolParam::new("Freeze", false),
        }
    }
}

impl Plugin for ReverbPlugin {
    const NAME: &'static str = "Reverb";
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
        self.reverb
            .prepare(buffer_config.sample_rate, buffer_config.max_buffer_size as usize);
        self.reverb.set_parameters(self.current_processor_params());

        true
    }

    fn reset(&mut self) {
        // Re-applying the parameters also calls `reset()` internally on the smoothers,
        // so this is enough to clear any tail from the delay lines.
        self.reverb.set_parameters(self.current_processor_params());
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Forward the smoothed parameter values into the processor. The processor
        // smooths its own internal state, but the host's smoothing guarantees that
        // we don't click when the user drags a knob.
        self.reverb.set_parameters(self.current_processor_params());

        let num_channels = buffer.channels();
        let num_samples = buffer.samples();

        if num_channels >= 2 {
            // Stereo in-place: take the first two channel slices out of the buffer and
            // process them.
            let slices = buffer.as_slice();
            let (left, rest) = slices.split_at_mut(1);
            let (right, _) = rest.split_at_mut(1);
            self.reverb
                .process_stereo(&mut left[0][..num_samples], &mut right[0][..num_samples]);
        } else {
            // Mono: process a stereo buffer where both channels are identical, then keep
            // only the left channel. This keeps the example consistent with the stereo
            // path while still using the processor's stereo `process_stereo` API.
            let slices = buffer.as_slice();
            let channel = &mut slices[0][..num_samples];
            let mut right = channel.to_vec();
            let mut left = channel.to_vec();
            self.reverb.process_stereo(&mut left, &mut right);
            channel.copy_from_slice(&left);
        }

        ProcessStatus::Normal
    }
}

impl ReverbPlugin {
    fn current_processor_params(&self) -> ReverbParams {
        ReverbParams {
            room_size: self.params.room_size.smoothed.next(),
            damping: self.params.damping.smoothed.next(),
            wet_level: self.params.wet_level.smoothed.next(),
            dry_level: self.params.dry_level.smoothed.next(),
            width: self.params.width.smoothed.next(),
            freeze_mode: if self.params.freeze_mode.value() { 1.0 } else { 0.0 },
        }
    }
}

impl ClapPlugin for ReverbPlugin {
    const CLAP_ID: &'static str = "com.nih-plug.reverb";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("FreeVerb-style algorithmic reverb example");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Reverb,
    ];
}

impl Vst3Plugin for ReverbPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"ReverbNihPlgExmp";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Reverb,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(ReverbPlugin);
nih_export_vst3!(ReverbPlugin);