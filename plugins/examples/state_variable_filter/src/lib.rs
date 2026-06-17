//! # State Variable Filter Example
//!
//! This example demonstrates the state variable filter (TPT) from logic_nih_plug_dsp
//! with real-time frequency response visualization using logic_nih_plug_gui.

use atomic_float::AtomicF32;
use crossbeam::atomic::AtomicCell;
use logic_nih_plug::prelude::*;
use logic_nih_plug_dsp::state_variable::StateVariableFilter;
use std::sync::Arc;

mod editor;

/// Filter type enum that implements the Enum trait for use with EnumParam
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum FilterType {
    #[name = "Lowpass"]
    Lowpass,
    #[name = "Bandpass"]
    Bandpass,
    #[name = "Highpass"]
    Highpass,
}

impl From<FilterType> for logic_nih_plug_dsp::state_variable::FilterType {
    fn from(ft: FilterType) -> Self {
        match ft {
            FilterType::Lowpass => logic_nih_plug_dsp::state_variable::FilterType::Lowpass,
            FilterType::Bandpass => logic_nih_plug_dsp::state_variable::FilterType::Bandpass,
            FilterType::Highpass => logic_nih_plug_dsp::state_variable::FilterType::Highpass,
        }
    }
}

/// State Variable Filter plugin demonstrating TPT filtering with visualization
pub struct StateVariableFilterPlugin {
    params: Arc<FilterParams>,

    /// Filters for left and right channels
    filter_l: StateVariableFilter,
    filter_r: StateVariableFilter,

    /// Frequency response data for visualization (magnitude in dB at various frequencies)
    /// Shared with the GUI for real-time display
    frequency_response: Arc<[AtomicF32; 128]>,
}

#[derive(Params)]
struct FilterParams {
    /// Filter type selection
    #[id = "type"]
    pub filter_type: EnumParam<FilterType>,

    /// Filter cutoff frequency in Hz
    #[id = "cutoff"]
    pub cutoff: FloatParam,

    /// Filter resonance (Q factor)
    #[id = "resonance"]
    pub resonance: FloatParam,
}

impl Default for StateVariableFilterPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(FilterParams::default()),
            filter_l: StateVariableFilter::new(),
            filter_r: StateVariableFilter::new(),
            frequency_response: Arc::new(std::array::from_fn(|_| AtomicF32::new(0.0))),
        }
    }
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            filter_type: EnumParam::new("Type", FilterType::Lowpass),

            cutoff: FloatParam::new(
                "Cutoff",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.5),
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
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
        }
    }
}

impl Plugin for StateVariableFilterPlugin {
    const NAME: &'static str = "State Variable Filter";
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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(editor::StateVariableFilterEditor {
            params: Arc::clone(&self.params),
            frequency_response: Arc::clone(&self.frequency_response),
            scaling_factor: AtomicCell::new(None),
        }))
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Prepare filters with the sample rate
        let _ = self.filter_l.prepare(buffer_config.sample_rate);
        let _ = self.filter_r.prepare(buffer_config.sample_rate);

        // Set initial parameters
        self.update_filter_parameters(buffer_config.sample_rate);

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

        // Update filter parameters if they changed
        if self.params.cutoff.smoothed.is_smoothing()
            || self.params.resonance.smoothed.is_smoothing()
        {
            self.update_filter_parameters(sample_rate);
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

        // Update frequency response for visualization
        self.update_frequency_response(sample_rate);

        ProcessStatus::Normal
    }
}

impl StateVariableFilterPlugin {
    /// Updates filter parameters from the parameter values
    fn update_filter_parameters(&mut self, _sample_rate: f32) {
        let filter_type: logic_nih_plug_dsp::state_variable::FilterType =
            self.params.filter_type.value().into();
        let cutoff = self.params.cutoff.value();
        let resonance = self.params.resonance.value();

        self.filter_l.set_type(filter_type);
        self.filter_l.set_cutoff(cutoff);
        self.filter_l.set_resonance(resonance);

        self.filter_r.set_type(filter_type);
        self.filter_r.set_cutoff(cutoff);
        self.filter_r.set_resonance(resonance);
    }

    /// Calculates and updates the frequency response for visualization
    fn update_frequency_response(&self, sample_rate: f32) {
        let cutoff = self.params.cutoff.value();
        let resonance = self.params.resonance.value();
        let filter_type: logic_nih_plug_dsp::state_variable::FilterType =
            self.params.filter_type.value().into();

        // Calculate frequency response at 128 logarithmically-spaced points
        for i in 0..128 {
            // Logarithmic frequency spacing from 20 Hz to 20 kHz
            let t = i as f32 / 127.0;
            let freq = 20.0_f32 * (20000.0_f32 / 20.0_f32).powf(t);

            // Calculate magnitude response using the TPT filter equations
            let magnitude_db =
                calculate_svf_magnitude_db(freq, cutoff, resonance, sample_rate, filter_type);

            self.frequency_response[i].store(magnitude_db, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

/// Calculates the magnitude response of a state variable filter at a given frequency
fn calculate_svf_magnitude_db(
    _freq: f32,
    cutoff: f32,
    resonance: f32,
    sample_rate: f32,
    filter_type: logic_nih_plug_dsp::state_variable::FilterType,
) -> f32 {
    use std::f32::consts::PI;

    // TPT coefficients
    let g = (PI * cutoff / sample_rate).tan();
    let k = 2.0 - 2.0 * resonance;

    // Simplified magnitude calculation for each filter type
    let magnitude_squared = match filter_type {
        logic_nih_plug_dsp::state_variable::FilterType::Lowpass => {
            // Lowpass: H(z) ≈ g² / (1 + g(g+k) + ...)
            let denominator = 1.0 + g * g + g * k;
            let numerator = g * g;
            numerator / denominator.max(1e-10)
        }
        logic_nih_plug_dsp::state_variable::FilterType::Bandpass => {
            // Bandpass: H(z) ≈ g / (1 + g(g+k) + ...)
            let denominator = 1.0 + g * g + g * k;
            let numerator = g;
            numerator / denominator.max(1e-10)
        }
        logic_nih_plug_dsp::state_variable::FilterType::Highpass => {
            // Highpass: H(z) ≈ 1 / (1 + g(g+k) + ...)
            let denominator = 1.0 + g * g + g * k;
            1.0 / denominator.max(1e-10)
        }
    };

    // Convert to dB, with a floor to avoid log(0)
    let magnitude = magnitude_squared.sqrt().max(1e-6);
    20.0 * magnitude.log10()
}

impl ClapPlugin for StateVariableFilterPlugin {
    const CLAP_ID: &'static str = "com.nih-plug.state-variable-filter";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("State variable filter with real-time frequency response visualization");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Filter,
    ];
}

impl Vst3Plugin for StateVariableFilterPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"StateVarFilterXX";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Filter];
}

nih_export_clap!(StateVariableFilterPlugin);
nih_export_vst3!(StateVariableFilterPlugin);
