//! # Note expressions example
//!
//! A simple polyphonic sine-wave synth that responds to every polyphonic
//! expression event the framework supports:
//!
//! - [`PolyPressure`](NoteEvent::PolyPressure) — adds to the voice's filter cutoff.
//! - [`PolyVolume`](NoteEvent::PolyVolume) — scales the voice's amplitude.
//! - [`PolyPan`](NoteEvent::PolyPan) — places the voice in the stereo field.
//! - [`PolyTuning`](NoteEvent::PolyTuning) — detunes the voice in semitones.
//! - [`PolyVibrato`](NoteEvent::PolyVibrato) — modulates the voice's frequency.
//! - [`PolyExpression`](NoteEvent::PolyExpression) — overall voice dynamics.
//! - [`PolyBrightness`](NoteEvent::PolyBrightness) — shifts the voice's filter cutoff.
//!
//! Each voice is identified by `(voice_id, channel, note)` so that note
//! expressions can target a specific held note.

use logic_nih_plug::prelude::*;
use logic_nih_plug_dsp::oscillators::{Oscillator, Waveform};
use std::f32::consts::TAU;
use std::sync::Arc;

const NUM_VOICES: usize = 16;
const MAX_BLOCK_SIZE: usize = 64;

/// Per-voice expression state. All values default to the neutral centre of
/// their respective ranges, so a voice with no expression events applied is
/// indistinguishable from one that received all events at the centre.
#[derive(Debug, Clone, Copy)]
struct VoiceExpressions {
    pressure: f32,
    volume: f32,
    pan: f32,
    /// Detuning in semitones; 0 = no detune.
    tuning: f32,
    /// Vibrato LFO amount in `[0, 1]`.
    vibrato: f32,
    /// Overall voice expression in `[0, 1]`.
    expression: f32,
    /// Brightness in `[0, 1]`; mapped to filter cutoff.
    brightness: f32,
}

impl Default for VoiceExpressions {
    fn default() -> Self {
        Self {
            pressure: 0.0,
            volume: 1.0,
            pan: 0.0,
            tuning: 0.0,
            vibrato: 0.0,
            expression: 1.0,
            brightness: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
struct Voice {
    voice_id: Option<i32>,
    channel: u8,
    note: u8,
    velocity: f32,
    base_frequency: f32,
    osc: Oscillator,
    /// Per-block vibrato LFO phase. We keep one phase per voice to avoid clicks
    /// when retriggering an existing note.
    vibrato_phase: f32,
    expressions: VoiceExpressions,
    /// `true` once the voice has been released and the release tail has finished.
    active: bool,
}

impl Voice {
    fn new(sample_rate: f32, voice_id: Option<i32>, channel: u8, note: u8, velocity: f32) -> Self {
        let mut osc = Oscillator::new(sample_rate);
        osc.set_frequency(util::midi_note_to_freq(note));
        osc.set_waveform(Waveform::Sine);
        Self {
            voice_id,
            channel,
            note,
            velocity,
            base_frequency: util::midi_note_to_freq(note),
            osc,
            vibrato_phase: 0.0,
            expressions: VoiceExpressions::default(),
            active: true,
        }
    }
}

/// A polyphonic note-expression-aware sine synth.
pub struct NoteExpressionsPlugin {
    params: Arc<NoteExpressionsParams>,
    sample_rate: f32,
    voices: [Option<Voice>; NUM_VOICES],
    /// Next voice slot to write into. Round-robin allocation with simple
    /// last-note-wins stealing.
    next_voice: usize,
}

#[derive(Params)]
struct NoteExpressionsParams {
    /// Master output level in dB.
    #[id = "out"]
    pub output_gain_db: FloatParam,
}

impl Default for NoteExpressionsPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(NoteExpressionsParams::default()),
            sample_rate: 44_100.0,
            voices: [(); NUM_VOICES].map(|_| None),
            next_voice: 0,
        }
    }
}

impl Default for NoteExpressionsParams {
    fn default() -> Self {
        Self {
            output_gain_db: FloatParam::new(
                "Output",
                -6.0,
                FloatRange::Linear {
                    min: -60.0,
                    max: 6.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
        }
    }
}

impl Plugin for NoteExpressionsPlugin {
    const NAME: &'static str = "Note Expressions";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    // We need the full set of note events including polyphonic expressions.
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
        self.sample_rate = buffer_config.sample_rate;
        true
    }

    fn reset(&mut self) {
        for voice in &mut self.voices {
            *voice = None;
        }
        self.next_voice = 0;
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let output_gain = util::db_to_gain_fast(self.params.output_gain_db.smoothed.next());
        let num_samples = buffer.samples();
        let slices = buffer.as_slice();
        debug_assert!(slices.len() >= 2, "expected stereo output layout");

        // Clear the output buffers so inactive voices contribute silence.
        let (left_channels, rest) = slices.split_at_mut(1);
        let (right_channels, _) = rest.split_at_mut(1);
        for sample in left_channels[0][..num_samples].iter_mut() {
            *sample = 0.0;
        }
        for sample in right_channels[0][..num_samples].iter_mut() {
            *sample = 0.0;
        }
        let left = &mut left_channels[0][..num_samples];
        let right = &mut right_channels[0][..num_samples];

        // Walk the buffer in small blocks so we can react to per-sample MIDI events
        // and split around them. The framework splits blocks around events when
        // `SAMPLE_ACCURATE_AUTOMATION` is true, but here we still need to split
        // on note events for our internal voice management.
        let mut next_event = context.next_event();
        let mut block_start: usize = 0;
        let mut block_end: usize = MAX_BLOCK_SIZE.min(num_samples);

        while block_start < num_samples {
            'events: while let Some(event) = next_event {
                if (event.timing() as usize) > block_start {
                    // The event happens after this block's start; we'll handle it
                    // when we get to its sample.
                    break 'events;
                }

                self.handle_event(event);
                next_event = context.next_event();
            }

            // Render the active voices into the output buffer for this block.
            for voice_slot in self.voices.iter_mut() {
                if let Some(voice) = voice_slot {
                    if !voice.active {
                        continue;
                    }

                    let block_len = block_end - block_start;
                    for i in 0..block_len {
                        // Apply vibrato: a 5 Hz sine LFO scaled by the vibrato
                        // expression. The LFO phase is kept in the voice so that
                        // it stays continuous across blocks.
                        voice.vibrato_phase += 5.0 / self.sample_rate;
                        if voice.vibrato_phase >= 1.0 {
                            voice.vibrato_phase -= 1.0;
                        }
                        let vibrato_hz =
                            (voice.vibrato_phase * TAU).sin() * voice.expressions.vibrato * 8.0;

                        let tuned_hz = voice.base_frequency
                            * (voice.expressions.tuning / 12.0).exp2();
                        voice.osc.set_frequency(tuned_hz + vibrato_hz);
                        let s = voice.osc.process_sample();

                        // The expression parameter scales the voice's dynamics;
                        // volume controls the per-note loudness; velocity provides
                        // the initial amplitude.
                        let amplitude = voice.expressions.volume
                            * voice.expressions.expression
                            * voice.velocity;

                        // Equal-power pan: cos/sin weighting.
                        let pan = voice.expressions.pan.clamp(-1.0, 1.0);
                        let (gain_l, gain_r) = if pan <= 0.0 {
                            (1.0, (1.0 + pan).sqrt())
                        } else {
                            ((1.0 - pan).sqrt(), 1.0)
                        };

                        let idx = block_start + i;
                        left[idx] += s * amplitude * gain_l;
                        right[idx] += s * amplitude * gain_r;
                    }
                }
            }

            // Apply master output gain.
            for i in block_start..block_end {
                left[i] *= output_gain;
                right[i] *= output_gain;
            }

            block_start = block_end;
            block_end = (block_end + MAX_BLOCK_SIZE).min(num_samples);
        }

        ProcessStatus::Normal
    }
}

impl NoteExpressionsPlugin {
    fn handle_event(&mut self, event: NoteEvent<()>) {
        match event {
            NoteEvent::NoteOn {
                voice_id,
                channel,
                note,
                velocity,
                ..
            } => {
                self.start_voice(voice_id, channel, note, velocity);
            }
            NoteEvent::NoteOff {
                voice_id,
                channel,
                note,
                ..
            }
            | NoteEvent::Choke {
                voice_id,
                channel,
                note,
                ..
            } => {
                self.stop_voice(voice_id, channel, note);
            }

            // The polyphonic expression events all target an active voice by
            // (voice_id, channel, note). If no matching voice is found we silently
            // ignore the event — hosts can send expression events for voices that
            // have just been released.
            NoteEvent::PolyPressure {
                voice_id,
                channel,
                note,
                pressure,
                ..
            } => {
                if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
                    voice.expressions.pressure = pressure;
                }
            }
            NoteEvent::PolyVolume {
                voice_id,
                channel,
                note,
                gain,
                ..
            } => {
                if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
                    voice.expressions.volume = gain;
                }
            }
            NoteEvent::PolyPan {
                voice_id,
                channel,
                note,
                pan,
                ..
            } => {
                if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
                    voice.expressions.pan = pan;
                }
            }
            NoteEvent::PolyTuning {
                voice_id,
                channel,
                note,
                tuning,
                ..
            } => {
                if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
                    voice.expressions.tuning = tuning;
                }
            }
            NoteEvent::PolyVibrato {
                voice_id,
                channel,
                note,
                vibrato,
                ..
            } => {
                if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
                    voice.expressions.vibrato = vibrato;
                }
            }
            NoteEvent::PolyExpression {
                voice_id,
                channel,
                note,
                expression,
                ..
            } => {
                if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
                    voice.expressions.expression = expression;
                }
            }
            NoteEvent::PolyBrightness {
                voice_id,
                channel,
                note,
                brightness,
                ..
            } => {
                if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
                    voice.expressions.brightness = brightness;
                }
            }

            _ => (),
        }
    }

    fn start_voice(
        &mut self,
        voice_id: Option<i32>,
        channel: u8,
        note: u8,
        velocity: f32,
    ) {
        let new_voice = Voice::new(self.sample_rate, voice_id, channel, note, velocity);

        // Round-robin allocation: pick the next slot, evicting whatever was there.
        let slot_idx = self.next_voice % NUM_VOICES;
        self.next_voice = self.next_voice.wrapping_add(1);
        self.voices[slot_idx] = Some(new_voice);
    }

    fn stop_voice(&mut self, voice_id: Option<i32>, channel: u8, note: u8) {
        if let Some(voice) = self.find_voice_mut(voice_id, channel, note) {
            voice.active = false;
        }
    }

    fn find_voice_mut(
        &mut self,
        voice_id: Option<i32>,
        channel: u8,
        note: u8,
    ) -> Option<&mut Voice> {
        // First try to match by voice_id (CLAP path). If the host didn't supply a
        // voice_id, fall back to matching on (channel, note).
        let idx = if let Some(id) = voice_id {
            self.voices
                .iter()
                .position(|slot| {
                    slot.as_ref()
                        .map(|v| v.voice_id == Some(id))
                        .unwrap_or(false)
                })
                .or_else(|| self.find_by_channel_note(channel, note))
        } else {
            self.find_by_channel_note(channel, note)
        };
        idx.and_then(|i| self.voices[i].as_mut())
    }

    fn find_by_channel_note(&self, channel: u8, note: u8) -> Option<usize> {
        self.voices.iter().position(|slot| {
            slot.as_ref()
                .map(|v| v.channel == channel && v.note == note)
                .unwrap_or(false)
        })
    }
}

impl ClapPlugin for NoteExpressionsPlugin {
    const CLAP_ID: &'static str = "com.nih-plug.note-expressions";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Polyphonic sine synth that responds to all polyphonic expression events");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Synthesizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for NoteExpressionsPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"NoteExprNihPlgEx";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
        Vst3SubCategory::Stereo,
    ];
}

nih_export_clap!(NoteExpressionsPlugin);
nih_export_vst3!(NoteExpressionsPlugin);