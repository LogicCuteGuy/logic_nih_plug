//! # Spectrum Analyzer Example
//!
//! This example demonstrates real-time spectrum analysis using FFT from nih_plug_dsp
//! with spectrogram display and color mapping using nih_plug_gui.

use atomic_float::AtomicF32;
use crossbeam::atomic::AtomicCell;
use nih_plug::prelude::*;
use nih_plug_dsp::analysis::FFT;
use num_complex::Complex;
use std::sync::Arc;

mod editor;

/// FFT size enum that implements the Enum trait for use with EnumParam
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum FFTSize {
    #[name = "512"]
    Size512,
    #[name = "1024"]
    Size1024,
    #[name = "2048"]
    Size2048,
    #[name = "4096"]
    Size4096,
}

impl FFTSize {
    fn to_usize(self) -> usize {
        match self {
            FFTSize::Size512 => 512,
            FFTSize::Size1024 => 1024,
            FFTSize::Size2048 => 2048,
            FFTSize::Size4096 => 4096,
        }
    }
}

/// Maximum FFT size for buffer allocation
const MAX_FFT_SIZE: usize = 4096;

/// Number of frequency bins to display (half of FFT size due to Nyquist)
const DISPLAY_BINS: usize = MAX_FFT_SIZE / 2;

/// Number of time slices to keep for spectrogram
const SPECTROGRAM_HISTORY: usize = 256;

/// Spectrum Analyzer plugin demonstrating FFT-based frequency analysis
pub struct SpectrumAnalyzerPlugin {
    params: Arc<AnalyzerParams>,

    /// Current FFT processor
    fft: Option<FFT>,

    /// Input buffer for accumulating samples
    input_buffer: Vec<f32>,
    input_write_pos: usize,

    /// Overlap-add state (75% overlap)
    hop_size: usize,
    samples_since_last_fft: usize,

    /// Window function (Hann window)
    window: Vec<f32>,

    /// FFT output buffers
    fft_output: Vec<Complex<f32>>,
    magnitude_spectrum: Vec<f32>,

    /// Magnitude spectrum data for visualization (in dB)
    /// Shared with the GUI for real-time display
    spectrum_data: Arc<[AtomicF32; DISPLAY_BINS]>,

    /// Spectrogram history (time x frequency)
    /// Each row is a magnitude spectrum at a point in time
    spectrogram_data: Arc<[[AtomicF32; DISPLAY_BINS]; SPECTROGRAM_HISTORY]>,
    spectrogram_write_pos: usize,
}

#[derive(Params)]
struct AnalyzerParams {
    /// FFT size selection
    #[id = "fft_size"]
    pub fft_size: EnumParam<FFTSize>,

    /// Display range minimum (dB)
    #[id = "min_db"]
    pub min_db: FloatParam,

    /// Display range maximum (dB)
    #[id = "max_db"]
    pub max_db: FloatParam,
}

impl Default for SpectrumAnalyzerPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(AnalyzerParams::default()),
            fft: None,
            input_buffer: vec![0.0; MAX_FFT_SIZE],
            input_write_pos: 0,
            hop_size: 0,
            samples_since_last_fft: 0,
            window: vec![0.0; MAX_FFT_SIZE],
            fft_output: vec![Complex::new(0.0, 0.0); MAX_FFT_SIZE],
            magnitude_spectrum: vec![0.0; MAX_FFT_SIZE],
            spectrum_data: Arc::new(std::array::from_fn(|_| AtomicF32::new(-100.0))),
            spectrogram_data: Arc::new(std::array::from_fn(|_| {
                std::array::from_fn(|_| AtomicF32::new(-100.0))
            })),
            spectrogram_write_pos: 0,
        }
    }
}

impl Default for AnalyzerParams {
    fn default() -> Self {
        Self {
            fft_size: EnumParam::new("FFT Size", FFTSize::Size2048),

            min_db: FloatParam::new(
                "Min dB",
                -80.0,
                FloatRange::Linear {
                    min: -120.0,
                    max: -20.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),

            max_db: FloatParam::new(
                "Max dB",
                0.0,
                FloatRange::Linear {
                    min: -20.0,
                    max: 20.0,
                },
            )
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
        }
    }
}

impl Plugin for SpectrumAnalyzerPlugin {
    const NAME: &'static str = "Spectrum Analyzer";
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
        Some(Box::new(editor::SpectrumAnalyzerEditor {
            params: Arc::clone(&self.params),
            spectrum_data: Arc::clone(&self.spectrum_data),
            spectrogram_data: Arc::clone(&self.spectrogram_data),
            scaling_factor: AtomicCell::new(None),
        }))
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Initialize FFT with current size
        self.update_fft_size();

        true
    }

    fn reset(&mut self) {
        // Clear input buffer
        self.input_buffer.fill(0.0);
        self.input_write_pos = 0;
        self.samples_since_last_fft = 0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Update FFT size if parameter changed
        let current_fft_size = self.params.fft_size.value().to_usize();
        if self.fft.as_ref().map_or(true, |f| f.size() != current_fft_size) {
            self.update_fft_size();
        }

        // Process audio samples (mix down to mono for analysis)
        for mut channel_samples in buffer.iter_samples() {
            let mut sample_sum = 0.0;
            let mut channel_count = 0;

            for sample in channel_samples.iter_mut() {
                sample_sum += *sample;
                channel_count += 1;
            }

            // Average all channels to mono
            let mono_sample = if channel_count > 0 {
                sample_sum / channel_count as f32
            } else {
                0.0
            };

            // Add to input buffer
            self.input_buffer[self.input_write_pos] = mono_sample;
            self.input_write_pos = (self.input_write_pos + 1) % current_fft_size;
            self.samples_since_last_fft += 1;

            // Perform FFT when we have enough samples (hop size)
            if self.samples_since_last_fft >= self.hop_size {
                self.perform_fft();
                self.samples_since_last_fft = 0;
            }
        }

        ProcessStatus::Normal
    }
}

impl SpectrumAnalyzerPlugin {
    /// Updates the FFT processor when size parameter changes
    fn update_fft_size(&mut self) {
        let fft_size = self.params.fft_size.value().to_usize();

        // Create new FFT processor
        self.fft = FFT::new(fft_size).ok();

        // Update hop size (75% overlap = 25% hop)
        self.hop_size = fft_size / 4;

        // Generate Hann window
        self.generate_hann_window(fft_size);

        // Reset buffers
        self.input_write_pos = 0;
        self.samples_since_last_fft = 0;
    }

    /// Generates a Hann window function
    fn generate_hann_window(&mut self, size: usize) {
        use std::f32::consts::PI;

        for i in 0..size {
            let t = i as f32 / (size - 1) as f32;
            self.window[i] = 0.5 * (1.0 - (2.0 * PI * t).cos());
        }
    }

    /// Performs FFT analysis on the current input buffer
    fn perform_fft(&mut self) {
        let Some(ref fft) = self.fft else {
            return;
        };

        let fft_size = fft.size();

        // Prepare windowed input
        let mut windowed_input = vec![0.0; fft_size];
        for i in 0..fft_size {
            let buffer_idx = (self.input_write_pos + i) % fft_size;
            windowed_input[i] = self.input_buffer[buffer_idx] * self.window[i];
        }

        // Perform FFT
        fft.forward_magnitude(&windowed_input, &mut self.magnitude_spectrum[..fft_size]);

        // Convert to dB and update spectrum data
        let num_bins = fft_size / 2; // Only use positive frequencies (Nyquist)
        for i in 0..num_bins.min(DISPLAY_BINS) {
            // Convert magnitude to dB with floor to avoid log(0)
            let magnitude = self.magnitude_spectrum[i].max(1e-10);
            let db = 20.0 * magnitude.log10();

            // Store in spectrum data
            self.spectrum_data[i].store(db, std::sync::atomic::Ordering::Relaxed);

            // Store in spectrogram history
            self.spectrogram_data[self.spectrogram_write_pos][i]
                .store(db, std::sync::atomic::Ordering::Relaxed);
        }

        // Advance spectrogram write position
        self.spectrogram_write_pos = (self.spectrogram_write_pos + 1) % SPECTROGRAM_HISTORY;
    }
}

impl ClapPlugin for SpectrumAnalyzerPlugin {
    const CLAP_ID: &'static str = "com.nih-plug.spectrum-analyzer";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Real-time spectrum analyzer with FFT and spectrogram display");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Analyzer,
    ];
}

impl Vst3Plugin for SpectrumAnalyzerPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"SpectrumAnalyzer";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Analyzer];
}

nih_export_clap!(SpectrumAnalyzerPlugin);
nih_export_vst3!(SpectrumAnalyzerPlugin);
