//! # Overdrive Effect Example
//!
//! This example demonstrates processor chain composition by creating an overdrive
//! effect using the following signal chain:
//!
//! Input -> Gain (drive) -> Bias -> WaveShaper -> DC Filter -> Gain (output) -> Output
//!
//! The chain demonstrates:
//! - Gain processor for input drive control
//! - Bias processor for asymmetric distortion
//! - WaveShaper for non-linear distortion
//! - DC Filter to remove unwanted DC offset
//! - Gain processor for output level control

use logic_nih_plug::prelude::*;
use logic_nih_plug_dsp::processors::bias::Bias;
use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
use logic_nih_plug_dsp::processors::gain::Gain;
use logic_nih_plug_dsp::processors::waveshaper::{transfer_functions, WaveShaper};
use std::sync::Arc;

/// Overdrive effect plugin demonstrating processor chain composition
pub struct Overdrive {
    params: Arc<OverdriveParams>,
    /// Individual processors for direct parameter access
    input_gain: Gain,
    bias: Bias,
    shaper: WaveShaper<fn(f32) -> f32>,
    dc_filter: DCFilter,
    output_gain: Gain,
    /// Temporary buffers for processing
    temp_buffers: [Vec<f32>; 4],
}

#[derive(Params)]
struct OverdriveParams {
    /// Drive amount in decibels (input gain)
    #[id = "drive"]
    pub drive: FloatParam,

    /// Bias amount for asymmetric distortion
    #[id = "bias"]
    pub bias: FloatParam,

    /// Output level in decibels
    #[id = "output"]
    pub output: FloatParam,
}

impl Default for Overdrive {
    fn default() -> Self {
        // Create individual processors
        let mut input_gain = Gain::new();
        input_gain.set_gain_db(0.0);
        input_gain.set_smoothing_time(10.0, 44100.0);

        let mut bias = Bias::new();
        bias.set_bias(0.0);

        let shaper = WaveShaper::new(transfer_functions::tanh as fn(f32) -> f32);

        let dc_filter = DCFilter::new();

        let mut output_gain = Gain::new();
        output_gain.set_gain_db(0.0);
        output_gain.set_smoothing_time(10.0, 44100.0);

        Self {
            params: Arc::new(OverdriveParams::default()),
            input_gain,
            bias,
            shaper,
            dc_filter,
            output_gain,
            temp_buffers: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        }
    }
}

impl Default for OverdriveParams {
    fn default() -> Self {
        Self {
            drive: FloatParam::new(
                "Drive",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 24.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            bias: FloatParam::new(
                "Bias",
                0.0,
                FloatRange::Linear {
                    min: -0.5,
                    max: 0.5,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            output: FloatParam::new(
                "Output",
                0.0,
                FloatRange::Linear {
                    min: -24.0,
                    max: 12.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for Overdrive {
    const NAME: &'static str = "Overdrive";
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
        let sample_rate = buffer_config.sample_rate;
        let max_buffer_size = buffer_config.max_buffer_size as usize;

        // Prepare all processors
        self.input_gain.prepare(sample_rate, max_buffer_size);
        self.bias.prepare(sample_rate, max_buffer_size);
        // WaveShaper doesn't need preparation but we call it for consistency
        self.dc_filter.prepare(sample_rate, max_buffer_size);
        self.output_gain.prepare(sample_rate, max_buffer_size);

        // Allocate temporary buffers
        for buffer in &mut self.temp_buffers {
            buffer.resize(max_buffer_size, 0.0);
        }

        // Set initial parameters
        self.update_processor_parameters(sample_rate);

        true
    }

    fn reset(&mut self) {
        self.input_gain.reset();
        self.bias.reset();
        self.dc_filter.reset();
        self.output_gain.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate;

        // Update processor parameters if they changed
        if self.params.drive.smoothed.is_smoothing()
            || self.params.bias.smoothed.is_smoothing()
            || self.params.output.smoothed.is_smoothing()
        {
            self.update_processor_parameters(sample_rate);
        }

        // Process each channel through the processor chain
        for channel_samples in buffer.iter_samples() {
            let num_channels = channel_samples.len();

            // Process each channel independently
            for (channel_idx, sample) in channel_samples.into_iter().enumerate() {
                // Process through the chain manually:
                // Input -> Gain -> Bias -> WaveShaper -> DC Filter -> Gain -> Output
                
                let mut signal = *sample;
                
                // 1. Input gain (drive)
                signal = self.input_gain.process_sample(signal);
                
                // 2. Bias
                signal = self.bias.process_sample(signal);
                
                // 3. Wave shaper
                signal = self.shaper.process_sample(signal);
                
                // 4. DC filter
                signal = self.dc_filter.process_sample(signal);
                
                // 5. Output gain
                signal = self.output_gain.process_sample(signal);
                
                *sample = signal;

                // Only process the first channel if mono
                if channel_idx == 0 && num_channels == 1 {
                    break;
                }
            }
        }

        ProcessStatus::Normal
    }
}

impl Overdrive {
    /// Updates processor parameters from the parameter values
    fn update_processor_parameters(&mut self, sample_rate: f32) {
        // Update input gain (drive)
        self.input_gain.set_gain_db(self.params.drive.value());
        self.input_gain.set_smoothing_time(10.0, sample_rate);

        // Update bias
        self.bias.set_bias(self.params.bias.value());

        // Wave shaper has no parameters to update

        // DC filter has no parameters to update

        // Update output gain
        self.output_gain.set_gain_db(self.params.output.value());
        self.output_gain.set_smoothing_time(10.0, sample_rate);
    }
}

impl ClapPlugin for Overdrive {
    const CLAP_ID: &'static str = "com.nih-plug.overdrive";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Overdrive effect demonstrating processor chain composition");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Distortion,
    ];
}

impl Vst3Plugin for Overdrive {
    const VST3_CLASS_ID: [u8; 16] = *b"OverdriveNIHPlug";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

nih_export_clap!(Overdrive);
nih_export_vst3!(Overdrive);
