//! Error types for `logic_nih_plug_audio_basics`.

use thiserror::Error;

/// Errors that can occur while constructing or parsing audio / MIDI data.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AudioBasicsError {
    /// A MIDI byte buffer did not contain a complete, well-formed message.
    #[error("MIDI message buffer is too short ({len} bytes) to contain a {kind} message (need {min})")]
    MidiTooShort {
        /// The kind of message we were trying to parse (e.g. `"note on"`).
        kind: &'static str,
        /// The actual length of the buffer that was rejected.
        len: usize,
        /// The minimum length that would have been accepted.
        min: usize,
    },

    /// The status byte at the start of a MIDI buffer was not recognised.
    #[error("unknown MIDI status byte {byte:#04x}")]
    UnknownStatus {
        /// The unknown status byte that was rejected.
        byte: u8,
    },

    /// The status byte implied a fixed number of data bytes that didn't match
    /// what was actually in the buffer (e.g. a System Common message that
    /// expected two data bytes but only one was provided).
    #[error("malformed MIDI {kind} message: {reason}")]
    MalformedMidi {
        /// The kind of message we were trying to parse (e.g. `"pitch bend"`).
        kind: &'static str,
        /// Why the message was rejected.
        reason: &'static str,
    },

    /// A `MidiMessage` was asked for a field that doesn't apply to its
    /// current status byte (e.g. asking for the note number on a SysEx
    /// message).
    #[error("MIDI {kind} message does not have a {field}")]
    InvalidField {
        /// The kind of message we asked the question of.
        kind: &'static str,
        /// The field that doesn't exist on that message kind.
        field: &'static str,
    },

    /// An invalid channel number was supplied (must be `0..16`).
    #[error("invalid MIDI channel {0} (must be in 0..16)")]
    InvalidChannel(u8),

    /// An invalid note number was supplied (must be `0..128`).
    #[error("invalid MIDI note number {0} (must be in 0..128)")]
    InvalidNote(u8),

    /// An invalid 7-bit CC value was supplied (must be `0..128`).
    #[error("invalid MIDI 7-bit value {0} (must be in 0..128)")]
    Invalid7BitValue(u8),

    /// An invalid 14-bit CC value was supplied (must be `0..16384`).
    #[error("invalid MIDI 14-bit value {0} (must be in 0..16384)")]
    Invalid14BitValue(u16),

    /// A constructor received a size of zero where a positive size is required.
    #[error("invalid size: {0} (must be > 0)")]
    InvalidSize(usize),

    /// A `MTC` time component was out of range.
    #[error("MTC {component} value {value} is out of range (expected {range:?})")]
    InvalidMtcTime {
        /// Which component was bad.
        component: &'static str,
        /// The bad value.
        value: u8,
        /// The inclusive range of valid values, as `(min, max)`.
        range: (u8, u8),
    },
}

/// Convenience alias used throughout the crate.
pub type AudioBasicsResult<T> = Result<T, AudioBasicsError>;
