//! # Sidechain compressor example
//!
//! A stereo compressor that uses an auxiliary (sidechain) input port as its
//! detection signal. This demonstrates:
//!
//! - Declaring auxiliary ports via [`AudioIOLayout::aux_input_ports`].
//! - Reading auxiliary buffers through `AuxiliaryBuffers::inputs`.
//! - Building a sidechain compressor out of [`BallisticsFilter`] (the same
//!   envelope follower JUCE's `dsp::Compressor` is built on top of) so the
//!   detection signal is independent from the audio being processed.
//!
//! [`BallisticsFilter`]: logic_nih_plug_dsp::processors::ballistics_filter::BallisticsFilter

use logic_nih_plug::prelude::*;
use logic_nih_plug_dsp::processors::ballistics_filter::BallisticsFilter;
use std::sync::Arc;

/// Stereo compressor with one mono sidechain input.
pub struct SidechainCompressor {
    params: Arc<SidechainCompressorParams>,

    /// One peak-rectifying envelope follower per main channel. We feed the
    /// *sidechain* signal into this and use the smoothed envelope to compute
    /// the per-sample gain reduction.
    envelope_l: BallisticsFilter,
    envelope_r: BallisticsFilter,
}

#[derive(Params)]
struct SidechainCompressorParams {
    /// Threshold in decibels. Anything above this level on the sidechain triggers gain reduction.
    #[id = "thresh"]
    pub threshold: FloatParam,

    /// Compression ratio. Must be >= 1.0.
    #[id = "ratio"]
    pub ratio: FloatParam,

    /// Attack time in milliseconds.
    #[id = "atk"]
    pub attack: FloatParam,

    /// Release time in milliseconds.
    #[id = "rel"]
    pub release: FloatParam,

    /// Makeup gain in decibels applied after compression.
    #[id = "makeup"]
    pub makeup: FloatParam,
}

impl Default for SidechainCompressor {
    fn default() -> Self {
        Self {
            params: Arc::new(SidechainCompressorParams::default()),
            envelope_l: BallisticsFilter::new(),
            envelope_r: BallisticsFilter::new(),
        }
    }
}

impl Default for SidechainCompressorParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                -12.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
            ratio: FloatParam::new(
                "Ratio",
                4.0,
                FloatRange::Linear {
                    min: 1.0,
                    max: 20.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            attack: FloatParam::new(
                "Attack",
                10.0,
                FloatRange::Skewed {
                    min: 0.1,
                    max: 200.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            release: FloatParam::new(
                "Release",
                100.0,
                FloatRange::Skewed {
                    min: 1.0,
                    max: 2000.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
            makeup: FloatParam::new(
                "Makeup",
                0.0,
                FloatRange::Linear {
                    min: -12.0,
                    max: 24.0,
                },
            )
            .with_smoother(SmoothingStyle::Linear(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
        }
    }
}

impl Plugin for SidechainCompressor {
    const NAME: &'static str = "Sidechain Compressor";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // Two layouts: a stereo main with a mono sidechain, and a mono main with a mono
    // sidechain. The wrapper will pick whichever one the host can satisfy.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            // A single mono sidechain input. The framework automatically names this
            // "Sidechain Input" when there's exactly one port and no name is set.
            aux_input_ports: &[new_nonzero_u32(1)],
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            aux_input_ports: &[new_nonzero_u32(1)],
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
        self.envelope_l
            .prepare_with_channels(buffer_config.sample_rate, 1);
        self.envelope_r
            .prepare_with_channels(buffer_config.sample_rate, 1);

        true
    }

    fn reset(&mut self) {
        self.envelope_l.reset();
        self.envelope_r.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Forward the smoothed parameter values into the envelope followers and the
        // per-sample gain computation.
        self.envelope_l
            .set_attack_time(self.params.attack.smoothed.next());
        self.envelope_l
            .set_release_time(self.params.release.smoothed.next());
        self.envelope_r
            .set_attack_time(self.params.attack.smoothed.next());
        self.envelope_r
            .set_release_time(self.params.release.smoothed.next());

        let threshold_db = self.params.threshold.smoothed.next();
        let ratio = self.params.ratio.smoothed.next().max(1.0);
        let makeup_db = self.params.makeup.smoothed.next();

        // Convert threshold from dB to a linear amplitude the envelope output (also
        // linear amplitude, peak-rectified) can be compared against directly.
        let threshold = util::db_to_gain(threshold_db);
        let makeup_gain = util::db_to_gain(makeup_db);
        let ratio_inverse = 1.0 / ratio;

        let num_samples = buffer.samples();
        let slices = buffer.as_slice();

        // Build a mono sidechain signal by averaging all sidechain input channels.
// This matches the typical "stereo mix bus" use case where the sidechain
// is the drum bus or vocals summed to mono. We build the mono buffer
// unconditionally and only use it if at least one sidechain channel is
// available.
        let mut sc_buf: Vec<f32> = vec![0.0_f32; num_samples];
        let has_sidechain = if let Some(buf) = aux.inputs.get_mut(0) {
            if buf.channels() >= 1 && buf.samples() >= num_samples {
                let s = buf.as_slice();
                let n = s.len() as f32;
                for channel in s.iter() {
                    for (i, sample) in channel[..num_samples].iter().enumerate() {
                        sc_buf[i] += *sample;
                    }
                }
                if n > 0.0 {
                    for m in sc_buf.iter_mut() {
                        *m /= n;
                    }
                }
                true
            } else {
                false
            }
        } else {
            false
        };
        let sc_samples: &[f32] = if has_sidechain {
            &sc_buf
        } else {
            &[]
        };

        if slices.len() >= 2 {
            let (left_channels, rest) = slices.split_at_mut(1);
            let (right_channels, _) = rest.split_at_mut(1);
            let left = &mut left_channels[0][..num_samples];
            let right = &mut right_channels[0][..num_samples];

            for i in 0..num_samples {
                // If no sidechain is connected, fall back to using the input itself as
                // the detection signal (i.e. a regular in-place compressor).
                let detection = if has_sidechain { sc_samples[i] } else { left[i] };

                let env_l = self.envelope_l.process_sample(0, detection);
                let env_r = self.envelope_r.process_sample(0, detection);

                let gain_l = if env_l < threshold {
                    1.0
                } else {
                    (env_l / threshold).powf(ratio_inverse - 1.0)
                };
                let gain_r = if env_r < threshold {
                    1.0
                } else {
                    (env_r / threshold).powf(ratio_inverse - 1.0)
                };

                left[i] *= gain_l * makeup_gain;
                right[i] *= gain_r * makeup_gain;
            }
        } else {
            // Mono main: a single envelope follower is enough.
            let mono = &mut slices[0][..num_samples];
            for i in 0..num_samples {
                let detection = if has_sidechain { sc_samples[i] } else { mono[i] };
                let env = self.envelope_l.process_sample(0, detection);
                let gain = if env < threshold {
                    1.0
                } else {
                    (env / threshold).powf(ratio_inverse - 1.0)
                };
                mono[i] *= gain * makeup_gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SidechainCompressor {
    const CLAP_ID: &'static str = "com.nih-plug.sidechain-compressor";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Stereo compressor with a mono sidechain input example");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Compressor,
        // CLAP also exposes a "Sidechain" feature on plugins that declare a
        // sidechain input port, which makes them routable in CLAP-aware hosts.
    ];
}

impl Vst3Plugin for SidechainCompressor {
    const VST3_CLASS_ID: [u8; 16] = *b"SidechainComprEx";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Dynamics,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(SidechainCompressor);
nih_export_vst3!(SidechainCompressor);