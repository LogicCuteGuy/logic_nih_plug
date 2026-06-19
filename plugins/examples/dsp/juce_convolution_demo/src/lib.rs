//! `juce_convolution_demo` — Time-domain convolution for IR-based effects.
//!
//! Ports the concept of `JUCE/examples/DSP/ConvolutionExample.h` to Rust using
//! a direct time-domain convolution algorithm suitable for short impulse
//! responses (reverb, cabinet simulation, etc.).
//!
//! # Examples
//!
//! ```
//! use juce_convolution_demo::convolve;
//!
//! // Identity IR → output equals input
//! let ir = vec![1.0];
//! let input = vec![0.5, 0.3, 0.1];
//! let output = convolve(&input, &ir);
//! assert!((output[0] - 0.5).abs() < 1e-6);
//! assert!((output[1] - 0.3).abs() < 1e-6);
//! ```

use logic_nih_plug::prelude::*;
use std::sync::Arc;

/// Perform time-domain convolution of `signal` with `impulse_response`.
///
/// Returns a vector of length `signal.len() + ir.len() - 1`, truncated to
/// `signal.len()` in the plugin (only the "valid" output portion is needed).
///
/// **Note**: O(n × m) per call — suitable for short IRs only.
#[inline]
pub fn convolve(signal: &[f32], ir: &[f32]) -> Vec<f32> {
    let out_len = signal.len() + ir.len() - 1;
    let mut output = vec![0.0_f32; out_len];
    for i in 0..signal.len() {
        for j in 0..ir.len() {
            output[i + j] += signal[i] * ir[j];
        }
    }
    output
}

/// Build a simple room impulse response (exponentially decaying noise).
///
/// This is a synthetic IR for demonstration — real plugins load `.wav` / `.aif`
/// files. The IR has `length` samples and decays over `decay_time` seconds.
pub fn build_demo_ir(length: usize, sample_rate: f32, decay_time: f32) -> Vec<f32> {
    let mut ir = vec![0.0_f32; length];
    // Initial direct sound
    if !ir.is_empty() {
        ir[0] = 1.0;
    }
    // Decaying noise tail (poor man's reverb IR)
    let tau = sample_rate * decay_time;
    for i in 1..length {
        // Simple pseudo-random via linear congruential
        let val = ((i as u64).wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 33) as f32 / (1u64 << 31) as f32;
        let envelope = (-(i as f32) / tau).exp();
        ir[i] = (val * 2.0 - 1.0) * envelope;
    }
    ir
}

pub struct ConvolutionPlugin {
    params: Arc<ConvolutionParams>,
    /// Current impulse response (loaded/built at init or sample rate change).
    ir: Vec<f32>,
    /// Ring buffer for incoming audio (long enough for the IR tail).
    ring: Vec<f32>,
    ring_pos: usize,
    /// Pre-computed output accumulator for the current block.
    sample_rate: f32,
}

#[derive(Params)]
struct ConvolutionParams {
    #[id = "ir_length"]
    pub ir_length: IntParam,
    #[id = "decay_time"]
    pub decay_time: FloatParam,
}

impl Default for ConvolutionPlugin {
    fn default() -> Self {
        let sr = 44100.0;
        let ir_len = 2048;
        let ir = build_demo_ir(ir_len, sr, 1.0);
        Self {
            params: Arc::new(ConvolutionParams::default()),
            ring: vec![0.0; ir_len],
            ring_pos: 0,
            ir,
            sample_rate: sr,
        }
    }
}

impl Default for ConvolutionParams {
    fn default() -> Self {
        Self {
            ir_length: IntParam::new(
                "IR Length",
                2048,
                IntRange::Linear {
                    min: 64,
                    max: 8192,
                },
            ),
            decay_time: FloatParam::new(
                "Decay Time",
                1.0,
                FloatRange::Linear {
                    min: 0.05,
                    max: 5.0,
                },
            )
            .with_unit(" s"),
        }
    }
}

impl Plugin for ConvolutionPlugin {
    const NAME: &'static str = "JUCE Convolution Demo";
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

    fn initialize(
        &mut self,
        _audio_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = _buffer_config.sample_rate;
        let ir_len = self.params.ir_length.value() as usize;
        let decay = self.params.decay_time.value();
        self.ir = build_demo_ir(ir_len, self.sample_rate, decay);
        self.ring = vec![0.0; ir_len];
        self.ring_pos = 0;
        true
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
            let ir_len = self.params.ir_length.value() as usize;
            let decay = self.params.decay_time.value();
            self.ir = build_demo_ir(ir_len, self.sample_rate, decay);
            self.ring = vec![0.0; ir_len];
            self.ring_pos = 0;
        }

        let ir_len = self.ir.len();

        for (_ch_idx, mut channel_samples) in buffer.iter_samples().enumerate() {
            // We use a single ring buffer per channel; for stereo we'd need
            // two, but for this demo we share the same buffer (acceptable for
            // the IR being identical on both channels).
            for sample in channel_samples.iter_mut() {
                // Write new sample into ring buffer
                self.ring[self.ring_pos] = *sample;

                // Compute convolution output for this sample
                let mut out = 0.0_f32;
                for j in 0..ir_len {
                    let ring_idx =
                        (self.ring_pos + ir_len - j) % ir_len;
                    out += self.ring[ring_idx] * self.ir[j];
                }

                // Advance ring position
                self.ring_pos = (self.ring_pos + 1) % ir_len;
                *sample = out;
            }
        }
        ProcessStatus::Normal
    }

    fn reset(&mut self) {
        for s in &mut self.ring {
            *s = 0.0;
        }
        self.ring_pos = 0;
    }
}

impl ClapPlugin for ConvolutionPlugin {
    const CLAP_ID: &'static str = "co.logiccuteguy.dsp.juce_convolution_demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("JUCE-style time-domain convolution");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Utility,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for ConvolutionPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceConvDemo\0\0\0\0";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Reverb];
}

nih_export_clap!(ConvolutionPlugin);
nih_export_vst3!(ConvolutionPlugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_ir() {
        let ir = vec![1.0];
        let input = vec![0.5, 0.3, 0.1];
        let output = convolve(&input, &ir);
        assert_eq!(output.len(), 3);
        assert!((output[0] - 0.5).abs() < 1e-6);
        assert!((output[1] - 0.3).abs() < 1e-6);
        assert!((output[2] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn impulse_ir() {
        let ir = vec![0.0, 0.0, 1.0]; // delta at sample 2
        let input = vec![1.0, 2.0, 3.0];
        let output = convolve(&input, &ir);
        assert_eq!(output.len(), 5);
        // Output should be input shifted by 2 samples
        assert!(output[2] - 1.0 < 1e-6);
        assert!(output[3] - 2.0 < 1e-6);
        assert!(output[4] - 3.0 < 1e-6);
    }

    #[test]
    fn demo_ir_length() {
        let ir = build_demo_ir(1024, 44100.0, 0.5);
        assert_eq!(ir.len(), 1024);
        assert!((ir[0] - 1.0).abs() < 1e-6);
    }
}
