//! # JUCE Multi-Module Example
//!
//! This advanced example demonstrates using multiple ported JUCE modules together
//! in a single plugin. It combines DSP, data structures, audio formats, animation,
//! and cryptography modules to create a feature-rich synthesizer plugin.

use logic_nih_plug::prelude::*;
use logic_nih_plug_animation::easing::ease_in_out_quad;
use logic_nih_plug_crypto::sha256::sha256_hex;
use logic_nih_plug_data::{Value, ValueTree};
use logic_nih_plug_dsp::envelopes::Envelope;
use logic_nih_plug_dsp::filters::IIRFilter;
use logic_nih_plug_dsp::oscillators::{Oscillator, Waveform};
use std::sync::Arc;

/// An advanced synthesizer plugin demonstrating multiple ported JUCE modules
struct JuceMultiModule {
    params: Arc<SynthParams>,
    // DSP components
    oscillator_l: Oscillator,
    oscillator_r: Oscillator,
    filter_l: IIRFilter,
    filter_r: IIRFilter,
    envelope: Envelope,
    // State management
    preset_tree: ValueTree,
    // Animation state
    animation_progress: f32,
    // Current note
    current_note: Option<u8>,
    note_velocity: f32,
}

#[derive(Params)]
struct SynthParams {
    /// Oscillator waveform
    #[id = "waveform"]
    pub waveform: EnumParam<WaveformParam>,

    /// Filter cutoff frequency
    #[id = "cutoff"]
    pub cutoff: FloatParam,

    /// Filter resonance
    #[id = "resonance"]
    pub resonance: FloatParam,

    /// Envelope attack time
    #[id = "attack"]
    pub attack: FloatParam,

    /// Envelope decay time
    #[id = "decay"]
    pub decay: FloatParam,

    /// Envelope sustain level
    #[id = "sustain"]
    pub sustain: FloatParam,

    /// Envelope release time
    #[id = "release"]
    pub release: FloatParam,

    /// Master gain
    #[id = "gain"]
    pub gain: FloatParam,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
enum WaveformParam {
    #[name = "Sine"]
    Sine,
    #[name = "Saw"]
    Saw,
    #[name = "Square"]
    Square,
    #[name = "Triangle"]
    Triangle,
}

impl Default for JuceMultiModule {
    fn default() -> Self {
        Self {
            params: Arc::new(SynthParams::default()),
            oscillator_l: Oscillator::new(44100.0),
            oscillator_r: Oscillator::new(44100.0),
            filter_l: IIRFilter::new(),
            filter_r: IIRFilter::new(),
            envelope: Envelope::new(44100.0),
            preset_tree: ValueTree::new("SynthPreset"),
            animation_progress: 0.0,
            current_note: None,
            note_velocity: 0.0,
        }
    }
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            waveform: EnumParam::new("Waveform", WaveformParam::Saw),

            cutoff: FloatParam::new(
                "Cutoff",
                2000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" Hz")
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

            attack: FloatParam::new(
                "Attack",
                0.01,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            decay: FloatParam::new(
                "Decay",
                0.1,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            sustain: FloatParam::new(
                "Sustain",
                0.7,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            release: FloatParam::new(
                "Release",
                0.3,
                FloatRange::Skewed {
                    min: 0.001,
                    max: 5.0,
                    factor: FloatRange::skew_factor(-1.5),
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),

            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(-6.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for JuceMultiModule {
    const NAME: &'static str = "JUCE Multi-Module Synth";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
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

        // Initialize DSP components
        self.oscillator_l.set_sample_rate(sample_rate);
        self.oscillator_r.set_sample_rate(sample_rate);

        // Initialize envelope parameters
        self.update_envelope_parameters();

        // Initialize filter coefficients
        self.update_filter_coefficients(sample_rate);

        // Initialize preset data structure (ValueTree)
        self.initialize_preset_tree();

        // Generate preset hash for verification (using crypto module).
        // We hash a stable string built from the preset's properties — that gives
        // a deterministic identifier you can use to detect tampering without
        // needing a full XML serialization round-trip on the audio thread.
        let preset_fingerprint = format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.params.waveform.value() as i32,
            self.params.cutoff.value() as i32,
            self.params.resonance.value(),
            self.params.attack.value(),
            self.params.decay.value(),
            self.params.sustain.value(),
            self.params.release.value(),
        );
        let preset_hash = sha256_hex(preset_fingerprint.as_bytes());
        nih_log!("Preset hash: {}", preset_hash);

        true
    }

    fn reset(&mut self) {
        self.oscillator_l.reset();
        self.oscillator_r.reset();
        self.filter_l.reset();
        self.filter_r.reset();
        self.envelope.reset();
        self.current_note = None;
        self.animation_progress = 0.0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate;

        // Process MIDI events
        while let Some(event) = context.next_event() {
            match event {
                NoteEvent::NoteOn {
                    note, velocity, ..
                } => {
                    self.handle_note_on(note, velocity, sample_rate);
                }
                NoteEvent::NoteOff { note, .. } => {
                    self.handle_note_off(note);
                }
                _ => {}
            }
        }

        // Update parameters
        self.update_dsp_parameters(sample_rate);

        // Process audio
        for mut channel_samples in buffer.iter_samples() {
            // Get envelope value
            let env_value = self.envelope.get_next_sample();

            // Generate oscillator samples
            let osc_l = self.oscillator_l.process_sample();
            let osc_r = self.oscillator_r.process_sample();

            // Apply envelope
            let mut sample_l = osc_l * env_value * self.note_velocity;
            let mut sample_r = osc_r * env_value * self.note_velocity;

            // Apply filter with animation
            let _animated_cutoff = self.animate_filter_cutoff();
            sample_l = self.filter_l.process_sample(sample_l);
            sample_r = self.filter_r.process_sample(sample_r);

            // Apply master gain
            let gain = self.params.gain.smoothed.next();
            sample_l *= gain;
            sample_r *= gain;

            // Write to output
            let mut iter = channel_samples.iter_mut();
            if let Some(left) = iter.next() {
                *left = sample_l;
            }
            if let Some(right) = iter.next() {
                *right = sample_r;
            }

            // Update animation progress
            self.animation_progress += 1.0 / sample_rate;
            if self.animation_progress > 1.0 {
                self.animation_progress = 0.0;
            }
        }

        ProcessStatus::Normal
    }
}

impl JuceMultiModule {
    fn handle_note_on(&mut self, note: u8, velocity: f32, _sample_rate: f32) {
        self.current_note = Some(note);
        self.note_velocity = velocity;

        // Calculate frequency from MIDI note
        let frequency = util::midi_note_to_freq(note);

        // Set oscillator frequency
        self.oscillator_l.set_frequency(frequency);
        self.oscillator_r.set_frequency(frequency * 1.01); // Slight detune for stereo width

        // Trigger envelope
        self.envelope.note_on();

        // Reset animation
        self.animation_progress = 0.0;
    }

    fn handle_note_off(&mut self, note: u8) {
        if self.current_note == Some(note) {
            self.envelope.note_off();
        }
    }

    fn update_dsp_parameters(&mut self, sample_rate: f32) {
        // Update waveform
        let waveform = match self.params.waveform.value() {
            WaveformParam::Sine => Waveform::Sine,
            WaveformParam::Saw => Waveform::Saw,
            WaveformParam::Square => Waveform::Square,
            WaveformParam::Triangle => Waveform::Triangle,
        };
        self.oscillator_l.set_waveform(waveform);
        self.oscillator_r.set_waveform(waveform);

        // Update filter
        if self.params.cutoff.smoothed.is_smoothing()
            || self.params.resonance.smoothed.is_smoothing()
        {
            self.update_filter_coefficients(sample_rate);
        }

        // Update envelope if parameters changed
        if self.params.attack.smoothed.is_smoothing()
            || self.params.decay.smoothed.is_smoothing()
            || self.params.sustain.smoothed.is_smoothing()
            || self.params.release.smoothed.is_smoothing()
        {
            self.update_envelope_parameters();
        }
    }

    fn update_filter_coefficients(&mut self, sample_rate: f32) {
        let cutoff = self.params.cutoff.value();
        let q = self.params.resonance.value();

        // Calculate second-order low-pass filter coefficients
        let omega = 2.0 * std::f32::consts::PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        let _ = self.filter_l.set_coefficients(&[b0, b1, b2], &[a0, a1, a2]);
        let _ = self.filter_r.set_coefficients(&[b0, b1, b2], &[a0, a1, a2]);
    }

    fn update_envelope_parameters(&mut self) {
        let attack = self.params.attack.value();
        let decay = self.params.decay.value();
        let sustain = self.params.sustain.value();
        let release = self.params.release.value();

        self.envelope.set_attack(attack);
        self.envelope.set_decay(decay);
        self.envelope.set_sustain(sustain);
        self.envelope.set_release(release);
    }

    fn animate_filter_cutoff(&mut self) -> f32 {
        // Use animation module to create smooth filter modulation
        let t = self.animation_progress;
        let animated_value = ease_in_out_quad(t);

        // Modulate filter cutoff based on animation
        let base_cutoff = self.params.cutoff.value();
        let modulation_amount = 500.0;
        base_cutoff + (animated_value * modulation_amount)
    }

    fn initialize_preset_tree(&mut self) {
        // Use ValueTree to store preset data. `Value` mirrors JUCE's `var` —
        // floats are stored as `Double(f64)` so they round-trip through JSON
        // and XML without loss; `Int` is `i64`.
        self.preset_tree
            .set_property("name", Value::String("Default".to_string()));
        self.preset_tree
            .set_property("version", Value::Int(1));

        // Store parameter values
        let params_tree = ValueTree::new("Parameters");
        params_tree.set_property("cutoff", Value::Double(self.params.cutoff.value() as f64));
        params_tree.set_property(
            "resonance",
            Value::Double(self.params.resonance.value() as f64),
        );
        params_tree.set_property("attack", Value::Double(self.params.attack.value() as f64));
        params_tree.set_property("decay", Value::Double(self.params.decay.value() as f64));
        params_tree.set_property(
            "sustain",
            Value::Double(self.params.sustain.value() as f64),
        );
        params_tree.set_property(
            "release",
            Value::Double(self.params.release.value() as f64),
        );

        self.preset_tree.add_child(params_tree, 0);
    }
}

impl ClapPlugin for JuceMultiModule {
    const CLAP_ID: &'static str = "com.nih-plug.juce-multi-module";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Advanced example demonstrating multiple ported JUCE modules");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for JuceMultiModule {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceMultiModPlug";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
    ];
}

nih_export_clap!(JuceMultiModule);
nih_export_vst3!(JuceMultiModule);
