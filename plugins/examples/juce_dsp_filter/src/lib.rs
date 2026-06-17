//! # JUCE DSP Filter Example
//!
//! This example demonstrates using the ported JUCE DSP modules for audio filtering.
//! It implements a simple low-pass filter plugin using the IIR filter from nih_plug_dsp.

use nih_plug::prelude::*;
use nih_plug_dsp::filters::IIRFilter;
use std::sync::Arc;

/// A simple filter plugin demonstrating the ported JUCE DSP filter module
struct JuceDspFilter {
    params: Arc<FilterParams>,
    filter_l: IIRFilter,
    filter_r: IIRFilter,
}

#[derive(Params)]
struct FilterParams {
    /// Filter cutoff frequency in Hz
    #[id = "cutoff"]
    pub cutoff: FloatParam,

    /// Filter resonance (Q factor)
    #[id = "resonance"]
    pub resonance: FloatParam,

}

impl Default for JuceDspFilter {
    fn default() -> Self {
        Self {
            params: Arc::new(FilterParams::default()),
            filter_l: IIRFilter::new(),
            filter_r: IIRFilter::new(),
        }
    }
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            cutoff: FloatParam::new(
                "Cutoff",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            // NB: `v2s_f32_hz_then_khz` already appends the unit (Hz / kHz),
            // so we deliberately don't call `.with_unit(" Hz")` here. Doing both
            // produced "20.00 kHz Hz" -> 0.0 -> "20.00 Hz Hz" and broke the
            // string-to-value roundtrip (clap-validator `param-conversions`).
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),

            resonance: FloatParam::new(
                "Resonance",
                0.707,
                FloatRange::Linear {
                    min: 0.1,
                    max: 10.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for JuceDspFilter {
    const NAME: &'static str = "JUCE DSP Filter Example";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

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
        // Set initial filter coefficients
        self.update_filter_coefficients(buffer_config.sample_rate);

        true
    }

    fn reset(&mut self) {
        self.filter_l.reset();
        self.filter_r.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate;

        // Update filter coefficients if parameters changed
        if self.params.cutoff.smoothed.is_smoothing()
            || self.params.resonance.smoothed.is_smoothing()
        {
            self.update_filter_coefficients(sample_rate);
        }

        // Process audio through filters
        for mut channel_samples in buffer.iter_samples() {
            let mut iter = channel_samples.iter_mut();

            if let Some(left) = iter.next() {
                *left = self.filter_l.process_sample(*left);
            }

            if let Some(right) = iter.next() {
                *right = self.filter_r.process_sample(*right);
            }
        }

        ProcessStatus::Normal
    }
}

impl JuceDspFilter {
    fn update_filter_coefficients(&mut self, sample_rate: f32) {
        let cutoff = self.params.cutoff.value();
        let q = self.params.resonance.value();

        // Calculate second-order low-pass filter coefficients using bilinear transform
        let omega = 2.0 * std::f32::consts::PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        // Low-pass filter coefficients
        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        // Set coefficients for both channels
        let _ = self.filter_l.set_coefficients(&[b0, b1, b2], &[a0, a1, a2]);
        let _ = self.filter_r.set_coefficients(&[b0, b1, b2], &[a0, a1, a2]);
    }
}

impl ClapPlugin for JuceDspFilter {
    const CLAP_ID: &'static str = "com.nih-plug.juce-dsp-filter-example";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Example plugin demonstrating JUCE DSP filter module");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Filter,
    ];
}

impl Vst3Plugin for JuceDspFilter {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceDspFilterPlg";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Filter];
}

nih_export_clap!(JuceDspFilter);
nih_export_vst3!(JuceDspFilter);
