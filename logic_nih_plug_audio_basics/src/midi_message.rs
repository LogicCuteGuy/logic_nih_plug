//! MIDI message parser and builder.
//!
//! [`MidiMessage`] mirrors JUCE's `juce::MidiMessage` class. It is a value
//! type that owns a byte buffer (status byte plus the right number of data
//! bytes, or the full SysEx payload) and an optional sample-offset
//! timestamp.
//!
//! ## Building
//!
//! ```
//! use logic_nih_plug_audio_basics::MidiMessage;
//!
//! let note_on = MidiMessage::note_on(1, 60, 100);
//! assert!(note_on.is_note_on());
//! assert_eq!(note_on.get_channel(), Some(1));
//! assert_eq!(note_on.note_number(), Some(60));
//! assert_eq!(note_on.velocity(), Some(100));
//! ```
//!
//! ## Parsing
//!
//! `MidiMessage::parse` consumes a `&[u8]` and returns `Some(MidiMessage)`
//! if the bytes start with a complete, well-formed MIDI message. SysEx
//! messages need an explicit end (`0xF7`) — use `parse_with_running_status`
//! if you want to handle system exclusive streams without the end marker
//! at the end of the input.
//!
//! ## Real-time note
//!
//! [`MidiMessage::parse`] performs a single linear scan over the input —
//! it does **not** allocate, so it's safe to call from a real-time audio
//! thread. The constructors (`note_on`, `controller`, …) all allocate a
//! fresh `Vec<u8>` and are only suitable for use outside of `process()`.

use crate::mtc::MtcRate;

/// The kind of MIDI message a [`MidiMessage`] represents.
///
/// This is a strictly lossy projection of the bytes: two messages with the
/// same `MidiMessageKind` always start with the same status byte (modulo
/// channel), but `MidiMessageKind::Unknown` covers any status byte that
/// doesn't fit the recognised taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MidiMessageKind {
    /// 0x80 — Note Off.
    NoteOff,
    /// 0x90 — Note On (velocity 0 is treated as Note Off by convention).
    NoteOn,
    /// 0xA0 — Polyphonic Key Pressure (aftertouch).
    PolyPressure,
    /// 0xB0 — Control Change.
    ControlChange,
    /// 0xC0 — Program Change.
    ProgramChange,
    /// 0xD0 — Channel Pressure.
    ChannelPressure,
    /// 0xE0 — Pitch Bend.
    PitchBend,
    /// 0xF0 — System Exclusive.
    SysEx,
    /// 0xF1 — MIDI Time Code quarter-frame message.
    TimeCode,
    /// 0xF2 — Song Position Pointer.
    SongPositionPointer,
    /// 0xF3 — Song Select.
    SongSelect,
    /// 0xF6 — Tune Request.
    TuneRequest,
    /// 0xF7 — End of SysEx (only seen when parsing incomplete streams).
    EndOfSysEx,
    /// 0xF8 — MIDI Clock tick (24 per quarter note).
    Clock,
    /// 0xF9 — undefined / tick (rare, reserved).
    Tick,
    /// 0xFA — Start.
    Start,
    /// 0xFB — Continue.
    Continue,
    /// 0xFC — Stop.
    Stop,
    /// 0xFE — Active Sensing.
    ActiveSensing,
    /// 0xFF — System Reset.
    SystemReset,
    /// Any status byte that this crate doesn't recognise.
    Unknown(u8),
}

impl MidiMessageKind {
    /// The MIDI status byte for this kind (without the channel nibble for
    /// channel messages).
    ///
    /// Returns `None` for [`MidiMessageKind::Unknown`] — use the underlying
    /// byte via [`MidiMessageKind::Unknown`] for that case.
    pub fn status_byte(self) -> Option<u8> {
        Some(match self {
            MidiMessageKind::NoteOff => 0x80,
            MidiMessageKind::NoteOn => 0x90,
            MidiMessageKind::PolyPressure => 0xA0,
            MidiMessageKind::ControlChange => 0xB0,
            MidiMessageKind::ProgramChange => 0xC0,
            MidiMessageKind::ChannelPressure => 0xD0,
            MidiMessageKind::PitchBend => 0xE0,
            MidiMessageKind::SysEx => 0xF0,
            MidiMessageKind::TimeCode => 0xF1,
            MidiMessageKind::SongPositionPointer => 0xF2,
            MidiMessageKind::SongSelect => 0xF3,
            MidiMessageKind::TuneRequest => 0xF6,
            MidiMessageKind::EndOfSysEx => 0xF7,
            MidiMessageKind::Clock => 0xF8,
            MidiMessageKind::Tick => 0xF9,
            MidiMessageKind::Start => 0xFA,
            MidiMessageKind::Continue => 0xFB,
            MidiMessageKind::Stop => 0xFC,
            MidiMessageKind::ActiveSensing => 0xFE,
            MidiMessageKind::SystemReset => 0xFF,
            MidiMessageKind::Unknown(_) => return None,
        })
    }
}

/// The decoded payload of an MTC quarter-frame message (`0xF1`).
///
/// Quarter-frame messages arrive 8 per MTC frame; the low nibble of the
/// type byte cycles `0..8` to indicate which of the 8 fields is being
/// transmitted, and the high nibble carries the value for that field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuarterFrameMessage {
    /// The raw type byte (low nibble) — `0..8`.
    pub message_type: u8,
    /// The raw value byte (high nibble) — `0..16`.
    pub value: u8,
}

impl QuarterFrameMessage {
    /// Decode the [`crate::mtc::MtcRate`] (frames per second) from the
    /// message type.
    pub fn rate(self) -> MtcRate {
        MtcRate::from_quarter_frame_type(self.message_type)
    }
}

/// A MIDI message: a byte buffer plus a sample-offset timestamp.
///
/// The byte buffer is the wire format: for a channel message it starts
/// with the status byte (high nibble = command, low nibble = channel);
/// for a SysEx message it starts with `0xF0` and ends with `0xF7`. The
/// timestamp is the sample offset within the current buffer the message
/// is scheduled for — set by the host or the parser, never inferred.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MidiMessage {
    /// The on-the-wire bytes for this message. Always starts with a status
    /// byte. For SysEx it includes both the `0xF0` start and `0xF7` end.
    data: Vec<u8>,
    /// The sample offset within the current buffer, or `0` if un-timestamped.
    time_stamp: i32,
}

impl MidiMessage {
    // ---- Constants ------------------------------------------------------

    /// Status byte for Note Off.
    pub const NOTE_OFF: u8 = 0x80;
    /// Status byte for Note On.
    pub const NOTE_ON: u8 = 0x90;
    /// Status byte for Polyphonic Key Pressure.
    pub const POLY_PRESSURE: u8 = 0xA0;
    /// Status byte for Control Change.
    pub const CONTROL_CHANGE: u8 = 0xB0;
    /// Status byte for Program Change.
    pub const PROGRAM_CHANGE: u8 = 0xC0;
    /// Status byte for Channel Pressure.
    pub const CHANNEL_PRESSURE: u8 = 0xD0;
    /// Status byte for Pitch Bend.
    pub const PITCH_BEND: u8 = 0xE0;
    /// Status byte for SysEx start.
    pub const SYSEX: u8 = 0xF0;
    /// Status byte for MTC quarter-frame.
    pub const TIME_CODE: u8 = 0xF1;
    /// Status byte for Song Position Pointer.
    pub const SONG_POSITION_POINTER: u8 = 0xF2;
    /// Status byte for Song Select.
    pub const SONG_SELECT: u8 = 0xF3;
    /// Status byte for Bus Select (`0xF5`).
    pub const BUS_SELECT: u8 = 0xF5;
    /// Status byte for Tune Request.
    pub const TUNE_REQUEST: u8 = 0xF6;
    /// Status byte for SysEx end (only seen inside incomplete streams).
    pub const END_OF_SYSEX: u8 = 0xF7;
    /// Status byte for MIDI Clock.
    pub const CLOCK: u8 = 0xF8;
    /// Status byte for Tick (undefined in the spec; sometimes seen).
    pub const TICK: u8 = 0xF9;
    /// Status byte for Start.
    pub const START: u8 = 0xFA;
    /// Status byte for Continue.
    pub const CONTINUE: u8 = 0xFB;
    /// Status byte for Stop.
    pub const STOP: u8 = 0xFC;
    /// Status byte for Active Sensing.
    pub const ACTIVE_SENSING: u8 = 0xFE;
    /// Status byte for System Reset.
    pub const SYSTEM_RESET: u8 = 0xFF;

    /// The controller number for the All Sound Off message.
    pub const ALL_SOUND_OFF: u8 = 120;
    /// The controller number for the Reset All Controllers message.
    pub const RESET_ALL_CONTROLLERS: u8 = 121;
    /// The controller number for the Local Control message.
    pub const LOCAL_CONTROL: u8 = 122;
    /// The controller number for the All Notes Off message.
    pub const ALL_NOTES_OFF: u8 = 123;
    /// The controller number for the Omni Mode Off message.
    pub const OMNI_MODE_OFF: u8 = 124;
    /// The controller number for the Omni Mode On message.
    pub const OMNI_MODE_ON: u8 = 125;
    /// The controller number for the Mono Mode On message.
    pub const MONO_MODE_ON: u8 = 126;
    /// The controller number for the Poly Mode On message.
    pub const POLY_MODE_ON: u8 = 127;

    // ---- Constructors ---------------------------------------------------

    /// Construct a `MidiMessage` from a raw byte buffer plus a timestamp.
    /// No validation is performed — the caller is responsible for making
    /// sure `data` is a well-formed MIDI message.
    pub fn from_bytes(data: Vec<u8>, time_stamp: i32) -> Self {
        Self {
            data,
            time_stamp,
        }
    }

    /// Construct a Note On message.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16`, `note >= 128`, or `velocity >= 128`.
    pub fn note_on(channel: u8, note: u8, velocity: u8) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(note < 128, "note must be in 0..128");
        assert!(velocity < 128, "velocity must be in 0..128");
        Self::from_bytes(
            vec![Self::NOTE_ON | channel, note, velocity],
            0,
        )
    }

    /// Construct a Note Off message.
    ///
    /// # Panics
    ///
    /// Same as [`note_on`][Self::note_on].
    pub fn note_off(channel: u8, note: u8, velocity: u8) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(note < 128, "note must be in 0..128");
        assert!(velocity < 128, "velocity must be in 0..128");
        Self::from_bytes(
            vec![Self::NOTE_OFF | channel, note, velocity],
            0,
        )
    }

    /// Construct a Note Off message with velocity 0 (a common encoding
    /// for "release this note").
    pub fn note_off_zero_velocity(channel: u8, note: u8) -> Self {
        Self::note_off(channel, note, 0)
    }

    /// Construct a Polyphonic Key Pressure (aftertouch) message.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16`, `note >= 128`, or `pressure >= 128`.
    pub fn polyphonic_aftertouch(channel: u8, note: u8, pressure: u8) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(note < 128, "note must be in 0..128");
        assert!(pressure < 128, "pressure must be in 0..128");
        Self::from_bytes(
            vec![Self::POLY_PRESSURE | channel, note, pressure],
            0,
        )
    }

    /// Construct a Control Change message.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16`, `cc >= 128`, or `value >= 128`.
    pub fn controller(channel: u8, cc: u8, value: u8) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(cc < 128, "cc must be in 0..128");
        assert!(value < 128, "value must be in 0..128");
        Self::from_bytes(
            vec![Self::CONTROL_CHANGE | channel, cc, value],
            0,
        )
    }

    /// Construct a Program Change message.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16` or `program >= 128`.
    pub fn program_change(channel: u8, program: u8) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(program < 128, "program must be in 0..128");
        Self::from_bytes(
            vec![Self::PROGRAM_CHANGE | channel, program],
            0,
        )
    }

    /// Construct a Channel Pressure message.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16` or `pressure >= 128`.
    pub fn channel_aftertouch(channel: u8, pressure: u8) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(pressure < 128, "pressure must be in 0..128");
        Self::from_bytes(
            vec![Self::CHANNEL_PRESSURE | channel, pressure],
            0,
        )
    }

    /// Construct a Pitch Bend message.
    ///
    /// `value` is a 14-bit value in `0..=16383`, where `0` is full down,
    /// `8192` is centred, and `16383` is full up.
    ///
    /// # Panics
    ///
    /// Panics if `channel >= 16` or `value > 16383`.
    pub fn pitch_bend(channel: u8, value: u16) -> Self {
        assert!(channel < 16, "channel must be in 0..16");
        assert!(value <= 16383, "pitch bend value must be in 0..=16383");
        let lsb = (value & 0x7F) as u8;
        let msb = ((value >> 7) & 0x7F) as u8;
        Self::from_bytes(vec![Self::PITCH_BEND | channel, lsb, msb], 0)
    }

    /// Construct a SysEx message.
    ///
    /// `payload` must be the raw bytes between `0xF0` and `0xF7` — i.e.
    /// it must not include either byte itself. The resulting message
    /// contains `0xF0`, the payload, then `0xF7`.
    ///
    /// # Panics
    ///
    /// Panics if the payload contains any byte with the high bit set
    /// (i.e. it's not 7-bit clean MIDI).
    pub fn sys_ex(payload: &[u8]) -> Self {
        for (i, &b) in payload.iter().enumerate() {
            assert!(
                b < 128,
                "SysEx payload byte {i} = {b:#04x} has the high bit set"
            );
        }
        let mut data = Vec::with_capacity(payload.len() + 2);
        data.push(Self::SYSEX);
        data.extend_from_slice(payload);
        data.push(Self::END_OF_SYSEX);
        Self::from_bytes(data, 0)
    }

    /// Construct an MTC quarter-frame message from a type nibble and a
    /// value nibble.
    ///
    /// # Panics
    ///
    /// Panics if `message_type >= 8` or `value >= 16`.
    pub fn quarter_frame_msg(message_type: u8, value: u8) -> Self {
        assert!(message_type < 8, "MTC quarter-frame type must be 0..8");
        assert!(value < 16, "MTC quarter-frame value must be 0..16");
        let data_byte = (value << 4) | (message_type & 0x07);
        Self::from_bytes(vec![Self::TIME_CODE, data_byte], 0)
    }

    /// Construct a Song Position Pointer message.
    ///
    /// # Panics
    ///
    /// Panics if `position > 16383` (14-bit value).
    pub fn song_position_pointer_msg(position: u16) -> Self {
        assert!(position <= 16383, "SPP must be 0..=16383");
        let lsb = (position & 0x7F) as u8;
        let msb = ((position >> 7) & 0x7F) as u8;
        Self::from_bytes(vec![Self::SONG_POSITION_POINTER, lsb, msb], 0)
    }

    /// Construct a Song Select message.
    ///
    /// # Panics
    ///
    /// Panics if `song >= 128`.
    pub fn song_select_msg(song: u8) -> Self {
        assert!(song < 128, "song must be in 0..128");
        Self::from_bytes(vec![Self::SONG_SELECT, song], 0)
    }

    /// Construct a Tune Request message (a single `0xF6` byte).
    pub fn tune_request() -> Self {
        Self::from_bytes(vec![Self::TUNE_REQUEST], 0)
    }

    /// Construct a MIDI Clock (`0xF8`) message.
    pub fn clock() -> Self {
        Self::from_bytes(vec![Self::CLOCK], 0)
    }

    /// Construct a MIDI Start (`0xFA`) message.
    pub fn start() -> Self {
        Self::from_bytes(vec![Self::START], 0)
    }

    /// Construct a MIDI Continue (`0xFB`) message.
    pub fn r#continue() -> Self {
        Self::from_bytes(vec![Self::CONTINUE], 0)
    }

    /// Construct a MIDI Stop (`0xFC`) message.
    pub fn stop() -> Self {
        Self::from_bytes(vec![Self::STOP], 0)
    }

    /// Construct a MIDI Active Sensing (`0xFE`) message.
    pub fn active_sensing() -> Self {
        Self::from_bytes(vec![Self::ACTIVE_SENSING], 0)
    }

    /// Construct a System Reset (`0xFF`) message.
    pub fn system_reset() -> Self {
        Self::from_bytes(vec![Self::SYSTEM_RESET], 0)
    }

    /// Construct an "All Notes Off" message.
    pub fn all_notes_off(channel: u8) -> Self {
        Self::controller(channel, Self::ALL_NOTES_OFF, 0)
    }

    /// Construct an "All Sound Off" message.
    pub fn all_sound_off(channel: u8) -> Self {
        Self::controller(channel, Self::ALL_SOUND_OFF, 0)
    }

    /// Construct a "Reset All Controllers" message.
    pub fn reset_all_controllers(channel: u8) -> Self {
        Self::controller(channel, Self::RESET_ALL_CONTROLLERS, 0)
    }

    // ---- Parsing --------------------------------------------------------

    /// Attempt to parse a complete MIDI message from the start of `input`.
    ///
    /// Returns `Some((MidiMessage, bytes_consumed))` if `input` begins with
    /// a well-formed message, or `None` if the bytes are incomplete or the
    /// status byte is unknown. Running status (data bytes following a
    /// previously-parsed message) is **not** handled — every message
    /// starts with its status byte.
    ///
    /// SysEx messages are terminated by `0xF7`; if `input` doesn't contain
    /// the end byte, `parse` returns `None`.
    pub fn parse(input: &[u8], time_stamp: i32) -> Option<(Self, usize)> {
        let status = *input.first()?;
        let kind = classify_status(status);

        // SysEx: variable-length, terminated by 0xF7.
        if matches!(kind, MidiMessageKind::SysEx) {
            // We need at least 0xF0 to start.
            if input.is_empty() {
                return None;
            }
            let end_pos = input[1..].iter().position(|&b| b == 0xF7)?;
            // Consume everything up to and including the 0xF7.
            let consumed = 2 + end_pos;
            let data: Vec<u8> = input[..consumed].to_vec();
            return Some((Self::from_bytes(data, time_stamp), consumed));
        }

        let needed = data_length_for_status(status)?;

        // Channel messages: the kind encodes the data length, but data
        // bytes can be absent.
        if needed > 0 {
            if input.len() < needed {
                return None;
            }
            let consumed = needed;
            let data: Vec<u8> = input[..needed].to_vec();
            return Some((Self::from_bytes(data, time_stamp), consumed));
        }

        // 0-length messages (tune request, clock, etc.).
        if matches!(
            kind,
            MidiMessageKind::TuneRequest
                | MidiMessageKind::Clock
                | MidiMessageKind::Start
                | MidiMessageKind::Continue
                | MidiMessageKind::Stop
                | MidiMessageKind::ActiveSensing
                | MidiMessageKind::SystemReset
                | MidiMessageKind::EndOfSysEx
                | MidiMessageKind::Tick
                | MidiMessageKind::Unknown(_)
        ) {
            return Some((Self::from_bytes(vec![status], time_stamp), 1));
        }

        None
    }

    // ---- Accessors ------------------------------------------------------

    /// The raw wire bytes for this message (including status byte, and the
    /// start / end bytes for SysEx).
    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Consume the message and return its raw bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// The sample offset this message is scheduled for. Defaults to `0`
    /// when the message is constructed with the convenience methods.
    #[inline]
    pub fn time_stamp(&self) -> i32 {
        self.time_stamp
    }

    /// Set the sample offset for this message.
    #[inline]
    pub fn set_time_stamp(&mut self, ts: i32) {
        self.time_stamp = ts;
    }

    /// The size of the message's wire format in bytes.
    #[inline]
    pub fn length(&self) -> usize {
        self.data.len()
    }

    /// The first byte of the message (the status byte, including the
    /// channel nibble for channel messages).
    ///
    /// Returns `None` if the message is empty (which shouldn't happen for
    /// any message built by this crate).
    #[inline]
    pub fn status_byte(&self) -> Option<u8> {
        self.data.first().copied()
    }

    /// The kind of this message.
    pub fn kind(&self) -> MidiMessageKind {
        match self.status_byte() {
            Some(s) => classify_status(s),
            None => MidiMessageKind::Unknown(0),
        }
    }

    /// For channel messages, returns the channel number (`0..16`).
    /// For system messages, returns `None`.
    pub fn get_channel(&self) -> Option<u8> {
        let status = self.status_byte()?;
        if status < 0xF0 {
            Some(status & 0x0F)
        } else {
            None
        }
    }

    /// Returns `true` if this is a Note On with non-zero velocity.
    pub fn is_note_on(&self) -> bool {
        matches!(self.kind(), MidiMessageKind::NoteOn) && self.velocity() != Some(0)
    }

    /// Returns `true` if this is a Note Off, *or* a Note On with velocity
    /// 0 (the latter is the standard convention for "release").
    pub fn is_note_off(&self) -> bool {
        match self.kind() {
            MidiMessageKind::NoteOff => true,
            MidiMessageKind::NoteOn => self.velocity() == Some(0),
            _ => false,
        }
    }

    /// The note number for a Note On / Note Off / Poly Pressure message,
    /// or `None` for other message kinds.
    pub fn note_number(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::NoteOn
            | MidiMessageKind::NoteOff
            | MidiMessageKind::PolyPressure => self.data.get(1).copied(),
            _ => None,
        }
    }

    /// The velocity for a Note On / Note Off message, or `None` for other
    /// message kinds.
    pub fn velocity(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::NoteOn | MidiMessageKind::NoteOff => self.data.get(2).copied(),
            _ => None,
        }
    }

    /// The aftertouch pressure for a Poly Pressure message, or `None`
    /// for other message kinds.
    pub fn poly_pressure(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::PolyPressure => self.data.get(2).copied(),
            _ => None,
        }
    }

    /// The channel-aftertouch pressure value, or `None`.
    pub fn channel_pressure(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::ChannelPressure => self.data.get(1).copied(),
            _ => None,
        }
    }

    /// The CC number for a Control Change message, or `None`.
    pub fn controller_number(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::ControlChange => self.data.get(1).copied(),
            _ => None,
        }
    }

    /// The 7-bit CC value for a Control Change message, or `None`.
    pub fn controller_value(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::ControlChange => self.data.get(2).copied(),
            _ => None,
        }
    }

    /// The program number for a Program Change message, or `None`.
    pub fn program_number(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::ProgramChange => self.data.get(1).copied(),
            _ => None,
        }
    }

    /// The pitch bend value as a 14-bit integer (`0..=16383`), or `None`.
    pub fn pitch_bend_value(&self) -> Option<u16> {
        match self.kind() {
            MidiMessageKind::PitchBend => {
                let lsb = *self.data.get(1)? as u16;
                let msb = *self.data.get(2)? as u16;
                Some((msb << 7) | lsb)
            }
            _ => None,
        }
    }

    /// Returns `true` if this message targets `channel` (or if it's a
    /// system message — those go to all channels).
    pub fn is_for_channel(&self, channel: u8) -> bool {
        match self.get_channel() {
            Some(ch) => ch == channel,
            None => true,
        }
    }

    /// The decoded MTC quarter-frame payload, if this is a Time Code
    /// (`0xF1`) message.
    pub fn quarter_frame(&self) -> Option<QuarterFrameMessage> {
        match self.kind() {
            MidiMessageKind::TimeCode => {
                let byte = *self.data.get(1)?;
                Some(QuarterFrameMessage {
                    message_type: byte & 0x07,
                    value: (byte >> 4) & 0x0F,
                })
            }
            _ => None,
        }
    }

    /// The 14-bit Song Position Pointer value, if this is one.
    pub fn song_position_pointer(&self) -> Option<u16> {
        match self.kind() {
            MidiMessageKind::SongPositionPointer => {
                let lsb = *self.data.get(1)? as u16;
                let msb = *self.data.get(2)? as u16;
                Some((msb << 7) | lsb)
            }
            _ => None,
        }
    }

    /// The song number for a Song Select message, if applicable.
    pub fn song_select(&self) -> Option<u8> {
        match self.kind() {
            MidiMessageKind::SongSelect => self.data.get(1).copied(),
            _ => None,
        }
    }

    /// The SysEx payload (i.e. the bytes between `0xF0` and `0xF7`), if
    /// this is a SysEx message.
    pub fn sys_ex_payload(&self) -> Option<&[u8]> {
        match self.kind() {
            MidiMessageKind::SysEx => {
                // data is 0xF0 ... 0xF7; payload is [1..len-1].
                let n = self.data.len();
                if n < 2 {
                    return Some(&[]);
                }
                Some(&self.data[1..n - 1])
            }
            _ => None,
        }
    }

    // ---- Manipulation ---------------------------------------------------

    /// Add a constant offset to the note number. Saturates at the bounds
    /// of `0..128`.
    ///
    /// Has no effect if the message is not a Note On / Note Off.
    pub fn add_to_note_number(&mut self, delta: i32) {
        if !matches!(
            self.kind(),
            MidiMessageKind::NoteOn | MidiMessageKind::NoteOff
        ) {
            return;
        }
        let note = self.data[1] as i32 + delta;
        self.data[1] = note.clamp(0, 127) as u8;
    }

    /// Add a constant offset to the velocity. Saturates at the bounds of
    /// `0..128`.
    ///
    /// Has no effect if the message is not a Note On / Note Off.
    pub fn add_to_velocity(&mut self, delta: i32) {
        if !matches!(
            self.kind(),
            MidiMessageKind::NoteOn | MidiMessageKind::NoteOff
        ) {
            return;
        }
        let v = self.data[2] as i32 + delta;
        self.data[2] = v.clamp(0, 127) as u8;
    }

    /// Scale the velocity by `factor` (clamped to `0..128`).
    ///
    /// Has no effect if the message is not a Note On / Note Off.
    pub fn scale_velocity(&mut self, factor: f32) {
        if !matches!(
            self.kind(),
            MidiMessageKind::NoteOn | MidiMessageKind::NoteOff
        ) {
            return;
        }
        let v = (self.data[2] as f32 * factor).clamp(0.0, 127.0) as i32;
        self.data[2] = v as u8;
    }

    /// Set the channel for a channel message.
    ///
    /// Has no effect for system messages.
    pub fn set_channel(&mut self, channel: u8) {
        if let Some(current) = self.get_channel() {
            let _ = current;
            if channel < 16 {
                self.data[0] = (self.data[0] & 0xF0) | channel;
            }
        }
    }
}

// ---- Free helpers --------------------------------------------------------

/// Return the `MidiMessageKind` for a status byte, stripping the channel
/// nibble.
fn classify_status(status: u8) -> MidiMessageKind {
    match status & 0xF0 {
        0x80 => MidiMessageKind::NoteOff,
        0x90 => MidiMessageKind::NoteOn,
        0xA0 => MidiMessageKind::PolyPressure,
        0xB0 => MidiMessageKind::ControlChange,
        0xC0 => MidiMessageKind::ProgramChange,
        0xD0 => MidiMessageKind::ChannelPressure,
        0xE0 => MidiMessageKind::PitchBend,
        _ => match status {
            0xF0 => MidiMessageKind::SysEx,
            0xF1 => MidiMessageKind::TimeCode,
            0xF2 => MidiMessageKind::SongPositionPointer,
            0xF3 => MidiMessageKind::SongSelect,
            0xF6 => MidiMessageKind::TuneRequest,
            0xF7 => MidiMessageKind::EndOfSysEx,
            0xF8 => MidiMessageKind::Clock,
            0xF9 => MidiMessageKind::Tick,
            0xFA => MidiMessageKind::Start,
            0xFB => MidiMessageKind::Continue,
            0xFC => MidiMessageKind::Stop,
            0xFE => MidiMessageKind::ActiveSensing,
            0xFF => MidiMessageKind::SystemReset,
            other => MidiMessageKind::Unknown(other),
        },
    }
}

/// Return the number of bytes consumed by a channel-message status byte,
/// including the status byte itself.
///
/// Returns `None` for `0xF0` (SysEx) since SysEx is variable-length and
/// is handled separately in [`MidiMessage::parse`].
fn data_length_for_status(status: u8) -> Option<usize> {
    if status < 0xF0 {
        Some(match status & 0xF0 {
            0x80 | 0x90 | 0xA0 | 0xB0 | 0xE0 => 3,
            0xC0 | 0xD0 => 2,
            _ => return None,
        })
    } else {
        Some(match status {
            0xF0 => return None, // SysEx — handled in `parse`.
            0xF1 => 2,           // MTC quarter-frame.
            0xF2 => 3,           // Song Position Pointer.
            0xF3 => 2,           // Song Select.
            _ => 1,              // Realtime / tune request / unknown / F7.
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_build_and_inspect() {
        let m = MidiMessage::note_on(1, 60, 100);
        assert_eq!(m.to_bytes(), &[0x91, 60, 100]);
        assert!(m.is_note_on());
        assert!(!m.is_note_off());
        assert_eq!(m.get_channel(), Some(1));
        assert_eq!(m.note_number(), Some(60));
        assert_eq!(m.velocity(), Some(100));
        assert_eq!(m.kind(), MidiMessageKind::NoteOn);
    }

    #[test]
    fn note_on_zero_velocity_is_note_off() {
        let m = MidiMessage::note_on(0, 60, 0);
        assert!(!m.is_note_on());
        assert!(m.is_note_off());
    }

    #[test]
    fn note_off_build() {
        let m = MidiMessage::note_off(2, 64, 50);
        assert_eq!(m.to_bytes(), &[0x82, 64, 50]);
        assert!(m.is_note_off());
    }

    #[test]
    fn note_off_zero_velocity() {
        let m = MidiMessage::note_off_zero_velocity(2, 64);
        assert_eq!(m.to_bytes(), &[0x82, 64, 0]);
    }

    #[test]
    fn poly_aftertouch_build() {
        let m = MidiMessage::polyphonic_aftertouch(3, 70, 40);
        assert_eq!(m.to_bytes(), &[0xA3, 70, 40]);
        assert_eq!(m.poly_pressure(), Some(40));
    }

    #[test]
    fn controller_build() {
        let m = MidiMessage::controller(5, 7, 64);
        assert_eq!(m.to_bytes(), &[0xB5, 7, 64]);
        assert_eq!(m.controller_number(), Some(7));
        assert_eq!(m.controller_value(), Some(64));
    }

    #[test]
    fn program_change_build() {
        let m = MidiMessage::program_change(4, 5);
        assert_eq!(m.to_bytes(), &[0xC4, 5]);
        assert_eq!(m.program_number(), Some(5));
    }

    #[test]
    fn channel_aftertouch_build() {
        let m = MidiMessage::channel_aftertouch(2, 99);
        assert_eq!(m.to_bytes(), &[0xD2, 99]);
        assert_eq!(m.channel_pressure(), Some(99));
    }

    #[test]
    fn pitch_bend_round_trip() {
        let m = MidiMessage::pitch_bend(0, 8192);
        assert_eq!(m.to_bytes(), &[0xE0, 0x00, 0x40]);
        assert_eq!(m.pitch_bend_value(), Some(8192));

        let m = MidiMessage::pitch_bend(15, 16383);
        assert_eq!(m.to_bytes(), &[0xEF, 0x7F, 0x7F]);
        assert_eq!(m.pitch_bend_value(), Some(16383));

        let m = MidiMessage::pitch_bend(0, 0);
        assert_eq!(m.to_bytes(), &[0xE0, 0x00, 0x00]);
        assert_eq!(m.pitch_bend_value(), Some(0));
    }

    #[test]
    fn sys_ex_build_and_payload() {
        let payload = [0x7E, 0x7F, 0x06, 0x01];
        let m = MidiMessage::sys_ex(&payload);
        assert_eq!(m.to_bytes(), &[0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7]);
        assert_eq!(m.sys_ex_payload(), Some(&payload[..]));
    }

    #[test]
    fn sys_ex_empty_payload() {
        let m = MidiMessage::sys_ex(&[]);
        assert_eq!(m.to_bytes(), &[0xF0, 0xF7]);
        assert_eq!(m.sys_ex_payload(), Some(&[][..]));
    }

    #[test]
    fn quarter_frame_build_and_decode() {
        let m = MidiMessage::quarter_frame_msg(5, 12);
        assert_eq!(m.to_bytes(), &[0xF1, (12 << 4) | 5]);
        let qf = m.quarter_frame().unwrap();
        assert_eq!(qf.message_type, 5);
        assert_eq!(qf.value, 12);
        assert_eq!(qf.rate(), MtcRate::TwentyFive);
    }

    #[test]
    fn song_position_pointer_roundtrip() {
        let m = MidiMessage::song_position_pointer_msg(1024);
        // 1024 = 0x0400: lsb=0x00, msb=0x08.
        assert_eq!(m.to_bytes(), &[0xF2, 0x00, 0x08]);
        assert_eq!(m.song_position_pointer(), Some(1024));
    }

    #[test]
    fn song_select_build() {
        let m = MidiMessage::song_select_msg(7);
        assert_eq!(m.to_bytes(), &[0xF3, 7]);
        assert_eq!(m.song_select(), Some(7));
    }

    #[test]
    fn realtime_messages() {
        assert_eq!(MidiMessage::tune_request().to_bytes(), &[0xF6]);
        assert_eq!(MidiMessage::clock().to_bytes(), &[0xF8]);
        assert_eq!(MidiMessage::start().to_bytes(), &[0xFA]);
        assert_eq!(MidiMessage::r#continue().to_bytes(), &[0xFB]);
        assert_eq!(MidiMessage::stop().to_bytes(), &[0xFC]);
        assert_eq!(MidiMessage::active_sensing().to_bytes(), &[0xFE]);
        assert_eq!(MidiMessage::system_reset().to_bytes(), &[0xFF]);
    }

    #[test]
    fn all_notes_off_uses_correct_cc() {
        let m = MidiMessage::all_notes_off(2);
        assert_eq!(m.to_bytes(), &[0xB2, 123, 0]);
    }

    #[test]
    fn parse_note_on() {
        let bytes = [0x91, 60, 100];
        let (m, consumed) = MidiMessage::parse(&bytes, 17).unwrap();
        assert_eq!(consumed, 3);
        assert_eq!(m.to_bytes(), &bytes);
        assert!(m.is_note_on());
        assert_eq!(m.time_stamp(), 17);
    }

    #[test]
    fn parse_incomplete_returns_none() {
        assert!(MidiMessage::parse(&[0x91, 60], 0).is_none());
    }

    #[test]
    fn parse_realtime_returns_one_byte() {
        let (m, n) = MidiMessage::parse(&[0xF8], 0).unwrap();
        assert_eq!(n, 1);
        assert_eq!(m.kind(), MidiMessageKind::Clock);
    }

    #[test]
    fn parse_sysex_includes_end_byte() {
        let bytes = [0xF0, 0x7E, 0x7F, 0x06, 0x01, 0xF7];
        let (m, n) = MidiMessage::parse(&bytes, 0).unwrap();
        assert_eq!(n, 6);
        assert_eq!(m.kind(), MidiMessageKind::SysEx);
        assert_eq!(m.sys_ex_payload(), Some(&[0x7E, 0x7F, 0x06, 0x01][..]));
    }

    #[test]
    fn parse_sysex_without_end_returns_none() {
        let bytes = [0xF0, 0x7E, 0x7F];
        assert!(MidiMessage::parse(&bytes, 0).is_none());
    }

    #[test]
    fn parse_skips_extra_bytes() {
        let bytes = [0x91, 60, 100, 0xC2, 5];
        let (m, n) = MidiMessage::parse(&bytes, 0).unwrap();
        assert_eq!(n, 3);
        assert!(m.is_note_on());
        // Calling again on the tail yields the program change.
        let (m2, n2) = MidiMessage::parse(&bytes[n..], 0).unwrap();
        assert_eq!(n2, 2);
        assert_eq!(m2.kind(), MidiMessageKind::ProgramChange);
    }

    #[test]
    fn parse_empty_input() {
        assert!(MidiMessage::parse(&[], 0).is_none());
    }

    #[test]
    fn is_for_channel() {
        let m = MidiMessage::note_on(3, 60, 100);
        assert!(m.is_for_channel(3));
        assert!(!m.is_for_channel(4));
        let clock = MidiMessage::clock();
        assert!(clock.is_for_channel(0));
        assert!(clock.is_for_channel(15));
    }

    #[test]
    fn add_to_note_number_saturates() {
        let mut m = MidiMessage::note_on(0, 60, 100);
        m.add_to_note_number(20);
        assert_eq!(m.note_number(), Some(80));
        m.add_to_note_number(1000);
        assert_eq!(m.note_number(), Some(127));
        m.add_to_note_number(-200);
        assert_eq!(m.note_number(), Some(0));
    }

    #[test]
    fn scale_velocity_clamps() {
        let mut m = MidiMessage::note_on(0, 60, 100);
        m.scale_velocity(2.0);
        assert_eq!(m.velocity(), Some(127));
        m.scale_velocity(0.0);
        assert_eq!(m.velocity(), Some(0));
        m.scale_velocity(0.5);
        assert_eq!(m.velocity(), Some(0));
    }

    #[test]
    fn set_channel_works_on_channel_messages() {
        let mut m = MidiMessage::note_on(3, 60, 100);
        m.set_channel(7);
        assert_eq!(m.get_channel(), Some(7));
        assert_eq!(m.to_bytes()[0] & 0x0F, 7);
    }

    #[test]
    fn set_channel_no_op_for_system_messages() {
        let mut m = MidiMessage::clock();
        m.set_channel(7);
        assert_eq!(m.to_bytes(), &[0xF8]);
    }

    #[test]
    fn timestamp_round_trip() {
        let mut m = MidiMessage::note_on(0, 60, 100);
        assert_eq!(m.time_stamp(), 0);
        m.set_time_stamp(42);
        assert_eq!(m.time_stamp(), 42);
    }

    #[test]
    fn from_bytes_passthrough() {
        let m = MidiMessage::from_bytes(vec![0xB0, 1, 2], 5);
        assert_eq!(m.to_bytes(), &[0xB0, 1, 2]);
        assert_eq!(m.controller_number(), Some(1));
        assert_eq!(m.controller_value(), Some(2));
        assert_eq!(m.time_stamp(), 5);
    }

    #[test]
    fn kind_status_byte_round_trip() {
        for kind in [
            MidiMessageKind::NoteOff,
            MidiMessageKind::NoteOn,
            MidiMessageKind::PolyPressure,
            MidiMessageKind::ControlChange,
            MidiMessageKind::ProgramChange,
            MidiMessageKind::ChannelPressure,
            MidiMessageKind::PitchBend,
            MidiMessageKind::SysEx,
            MidiMessageKind::TimeCode,
            MidiMessageKind::Clock,
            MidiMessageKind::Start,
            MidiMessageKind::Stop,
        ] {
            let status = kind.status_byte().unwrap();
            assert_eq!(classify_status(status), kind);
        }
    }

    #[test]
    fn parse_then_round_trip_matches_builder() {
        for builder in [
            MidiMessage::note_on(1, 60, 100),
            MidiMessage::note_off(2, 64, 0),
            MidiMessage::controller(3, 7, 64),
            MidiMessage::program_change(4, 5),
            MidiMessage::channel_aftertouch(5, 100),
            MidiMessage::pitch_bend(6, 8192),
            MidiMessage::sys_ex(&[1, 2, 3, 4]),
            MidiMessage::quarter_frame_msg(2, 5),
            MidiMessage::song_position_pointer_msg(100),
            MidiMessage::song_select_msg(7),
            MidiMessage::clock(),
            MidiMessage::start(),
            MidiMessage::r#continue(),
            MidiMessage::stop(),
            MidiMessage::tune_request(),
            MidiMessage::active_sensing(),
            MidiMessage::system_reset(),
        ] {
            let bytes = builder.to_bytes().to_vec();
            let (parsed, consumed) = MidiMessage::parse(&bytes, 0).unwrap();
            assert_eq!(consumed, bytes.len(), "{builder:?}");
            assert_eq!(parsed.to_bytes(), bytes.as_slice(), "{builder:?}");
        }
    }

    #[test]
    #[should_panic(expected = "channel must be in 0..16")]
    fn note_on_invalid_channel_panics() {
        let _ = MidiMessage::note_on(16, 60, 100);
    }

    #[test]
    #[should_panic(expected = "note must be in 0..128")]
    fn note_on_invalid_note_panics() {
        let _ = MidiMessage::note_on(0, 128, 100);
    }
}
