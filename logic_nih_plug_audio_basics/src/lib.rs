//! # logic_nih_plug_audio_basics
//!
//! Audio buffer / channel-set / MIDI message primitives ported from
//! [JUCE's `juce_audio_basics` module](https://docs.juce.com/master/juce_audio_basics_README.html)
//! for the `logic_nih_plug` ecosystem.
//!
//! ## What's inside
//!
//! - [`AudioSampleBuffer`] — non-interleaved (JUCE-default) audio sample
//!   container, with helpers to interleave / deinterleave to `&[f32]`.
//! - [`AudioChannelSet`] — speaker / channel layouts (mono, stereo, 5.1,
//!   7.1, 7.1.2, 7.1.4, ambisonics, custom).
//! - [`MidiMessage`] — MIDI message parser + builder, with a sample-offset
//!   timestamp field.
//! - [`MidiRPN`] — Registered Parameter Number / Non-Registered PN helpers.
//! - [`MidiClock`] — sample / tick / ppqn math for the 24-ppqn MIDI clock.
//! - [`MTC`] — MIDI Time Code quarter-frame helpers.
//!
//! ## Feature flags
//!
//! | Flag     | Default | What it gates                                                                 |
//! |----------|---------|-------------------------------------------------------------------------------|
//! | `buffer` | ✅      | `AudioSampleBuffer`, `AudioChannelSet`                                         |
//! | `midi`   | ✅      | `MidiMessage`, `MidiRPN`, `MidiClock`, `MTC`                                   |
//! | `full`   | —       | Equivalent to the default set                                                  |
//!
//! ## Example
//!
//! ```
//! # #[cfg(feature = "buffer")] {
//! use logic_nih_plug_audio_basics::{AudioChannelSet, AudioSampleBuffer};
//!
//! // 512 frames of stereo silence, then half-volume.
//! let mut buf = AudioSampleBuffer::new(AudioChannelSet::Stereo, 512);
//! buf.clear();
//! buf.apply_gain(0.5);
//! assert_eq!(buf.num_channels(), 2);
//! # }
//! ```
//!
//! ```
//! # #[cfg(feature = "midi")] {
//! use logic_nih_plug_audio_basics::MidiMessage;
//!
//! // Build + parse a MIDI Note On.
//! let msg = MidiMessage::note_on(1, 60, 100);
//! assert!(msg.is_note_on());
//! assert_eq!(msg.note_number(), Some(60));
//!
//! let bytes = msg.to_bytes();
//! let (parsed, _consumed) = MidiMessage::parse(&bytes, 0).unwrap();
//! assert_eq!(parsed, msg);
//! # }
//! ```

#![warn(missing_docs)]

mod error;

#[cfg(feature = "buffer")]
mod audio_channel_set;

#[cfg(feature = "buffer")]
mod audio_sample_buffer;

#[cfg(feature = "midi")]
mod midi_message;

#[cfg(feature = "midi")]
mod midi_rpn;

#[cfg(feature = "midi")]
mod midi_clock;

#[cfg(feature = "midi")]
mod mtc;

pub use error::{AudioBasicsError, AudioBasicsResult};

#[cfg(feature = "buffer")]
pub use audio_channel_set::{
    AudioChannelSet, AmbisonicOrder, ChannelName, ChannelType, SpeakerPosition,
};

#[cfg(feature = "buffer")]
pub use audio_sample_buffer::AudioSampleBuffer;

#[cfg(feature = "midi")]
pub use midi_message::{MidiMessage, MidiMessageKind, QuarterFrameMessage};

#[cfg(feature = "midi")]
pub use midi_rpn::{MidiRPN, MidiRpnKind};

#[cfg(feature = "midi")]
pub use midi_clock::MidiClock;

#[cfg(feature = "midi")]
pub use mtc::{KeySignature, MtcEncoder, MtcFullFrame, MtcRate, MtcTime, TempoEvent, TimeSignature};
