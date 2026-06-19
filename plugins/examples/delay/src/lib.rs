//! # Delay example
//!
//! A simple feedback delay effect built on top of the [`Delay`] processor
//! from `logic_nih_plug_dsp`. It demonstrates:
//!
//! - Tempo-synced delay time using [`NoteDivision`].
//! - Switching between free-running time and tempo-synced time.
//! - Feedback and dry/wet mix.
//! - Stereo ping-pong mode.
//!
//! [`Delay`]: logic_nih_plug_dsp::processors::delay::Delay
//! [`NoteDivision`]: logic_nih_plug_dsp::processors::delay::NoteDivision

use logic_nih_plug::prelude::*;
use logic_nih_plug_dsp::processors::delay::{Delay, DelayParameters, NoteDivision};
use std::sync::Arc;

/// The note divisions we expose in the UI, paired with their `NoteDivision` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum DelayNoteValue {
    #[name = "1/1"]
    Whole,
    #[name = "1/2"]
    Half,
    #[name = "1/4"]
    Quarter,
    #[name = "1/8"]
    Eighth,
    #[name = "1/16"]
    Sixteenth,
    #[name = "1/32"]
    ThirtySecond,
    #[name = "1/4."]
    DottedQuarter,
    #[name = "1/8."]
    DottedEighth,
    #[name = "1/4T"]
    TripletQuarter,
    #[name = "1/8T"]
    TripletEighth,
}

impl From<DelayNoteValue> for NoteDivision {
    fn from(v: DelayNoteValue) -> Self {
        match v {
            DelayNoteValue::Whole => NoteDivision::Whole,
            DelayNoteValue::Half => NoteDivision::Half,
            DelayNoteValue::Quarter => NoteDivision::Quarter,
            DelayNoteValue::Eighth => NoteDivision::Eighth,
            DelayNoteValue::Sixteenth => NoteDivision::Sixteenth,
            DelayNoteValue::ThirtySecond => NoteDivision::ThirtySecond,
            DelayNoteValue::DottedQuarter => NoteDivision::DottedQuarter,
            DelayNoteValue::DottedEighth => NoteDivision::DottedEighth,
            DelayNoteValue::TripletQuarter => NoteDivision::TripletQuarter,
            DelayNoteValue::TripletEighth => NoteDivision::TripletEighth,
        }
    }
}

/// A stereo delay effect with tempo-sync and ping-pong.
pub struct DelayPlugin {
    params: Arc<DelayParams>,
    delay: Delay,
}

#[derive(Params)]
struct DelayParams {
    /// Whether to lock the delay time to the host's tempo.
    #[id = "sync"]
    pub tempo_sync: BoolParam,

    /// The note value used when `tempo_sync` is enabled.
    #[id = "note"]
    pub note_value: EnumParam<DelayNoteValue>,

    /// The delay time in seconds, used when `tempo_sync` is disabled.
    #[id = "time"]
    pub time_seconds: FloatParam,

    /// Feedback in `[0.0, 1.2]`. Values above 1.0 self-oscillate.
    #[id = "fb"]
    pub feedback: FloatParam,

    /// Dry/wet mix in `[0.0, 1.0]`.
    #[id = "mix"]
    pub mix: FloatParam,

    /// Ping-pong mode: each channel's delay output is fed back into the opposite channel.
    #[id = "pp"]
    pub ping_pong: BoolParam,
}

impl Default for DelayPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DelayParams::default()),
            delay: Delay::new(),
        }
    }
}

impl Default for DelayParams {
    fn default() -> Self {
        Self {
            tempo_sync: BoolParam::new("Tempo Sync", true),
            note_value: EnumParam::new("Note Value", DelayNoteValue::Quarter),
            time_seconds: FloatParam::new(
                "Time",
                0.375,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            feedback: FloatParam::new(
                "Feedback",
                0.35,
                FloatRange::Linear { min: 0.0, max: 1.2 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            mix: FloatParam::new(
                "Mix",
                0.4,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0)),
            ping_pong: BoolParam::new("Ping Pong", false),
        }
    }
}

impl Plugin for DelayPlugin {
    const NAME: &'static str = "Delay";
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
        // The delay processor needs the sample rate and the maximum block size so it can
        // allocate the per-channel delay-line buffers. The max delay time is configured
        // via `DelayParameters::max_delay_seconds` (2.0s by default, which is plenty for
        // a tempo-synced delay).
        self.delay
            .prepare(buffer_config.sample_rate, buffer_config.max_buffer_size as usize);
        // Use a sensible default tempo for the very first block; `process()` will
        // refresh it from the host transport on the next call.
        self.update_parameters_with_tempo(120.0);

        true
    }

    fn reset(&mut self) {
        // Re-applying the parameters re-zeros the processor's internal smoother state and
        // clears the delay-line buffers. Using a placeholder 120 BPM here is fine because
        // `process()` will refresh the tempo from the transport on the very next block.
        self.update_parameters_with_tempo(120.0);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Pull fresh tempo information from the transport each block so that tempo-synced
        // delay times track any changes to the host's BPM.
        let tempo_bpm = context
            .transport()
            .tempo
            .map(|t| t as f32)
            .unwrap_or(120.0);
        self.update_parameters_with_tempo(tempo_bpm);

        let num_samples = buffer.samples();
        let slices = buffer.as_slice();

        if slices.len() >= 2 {
            // Stereo: process the left and right channels independently. The
            // `process_stereo` function handles the cross-feedback when `ping_pong` is
            // enabled.
            let (left_channels, rest) = slices.split_at_mut(1);
            let (right_channels, _) = rest.split_at_mut(1);
            let left = &mut left_channels[0][..num_samples];
            let right = &mut right_channels[0][..num_samples];

            for i in 0..num_samples {
                let mut out_l = 0.0;
                let mut out_r = 0.0;
                self.delay
                    .process_stereo(left[i], right[i], &mut out_l, &mut out_r);
                left[i] = out_l;
                right[i] = out_r;
            }
        } else {
            // Mono: process the same signal on both internal delay lines so the
            // processor's smoother behaviour stays consistent with the stereo path.
            let mono = &mut slices[0][..num_samples];
            for i in 0..num_samples {
                let mut out_l = 0.0;
                let mut out_r = 0.0;
                self.delay
                    .process_stereo(mono[i], mono[i], &mut out_l, &mut out_r);
                mono[i] = out_l;
            }
        }

        ProcessStatus::Normal
    }
}

impl DelayPlugin {
    fn update_parameters_with_tempo(&mut self, tempo_bpm: f32) {
        let feedback = self.params.feedback.smoothed.next();
        let mix = self.params.mix.smoothed.next();

        let (tempo_sync, delay_time_seconds) = if self.params.tempo_sync.value() {
            (true, 0.0) // the processor ignores `delay_time_seconds` when `tempo_sync` is true
        } else {
            (false, self.params.time_seconds.smoothed.next())
        };

        self.delay.set_parameters(DelayParameters {
            delay_time_seconds,
            feedback,
            mix,
            ping_pong: self.params.ping_pong.value(),
            tempo_sync,
            tempo_bpm,
            note_division: self.params.note_value.value().into(),
            max_delay_seconds: 2.0,
            enabled: true,
        });
    }
}

impl ClapPlugin for DelayPlugin {
    const CLAP_ID: &'static str = "com.nih-plug.delay";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Tempo-synced feedback delay example");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Delay,
    ];
}

impl Vst3Plugin for DelayPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"DelayNihPlugExam";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Delay,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(DelayPlugin);
nih_export_vst3!(DelayPlugin);