//! MIDI Time Code (MTC) helpers.
//!
//! MTC encodes SMPTE-style timecode (`HH:MM:SS:FF`) over MIDI in two
//! distinct ways:
//!
//! 1. **Quarter-frame messages** (`0xF1`): eight messages per MTC frame,
//!    each carrying one nibble of the full time. This is the
//!    in-band, always-running form.
//! 2. **Full-frame messages** (a SysEx payload of `[0x7F, 0x7F, 0x01, 0x01,
//!    hh, mm, ss, ff, 0xF7]`): a complete timecode snapshot sent on
//!    cue (e.g. transport start, locate).
//!
//! This module gives you:
//!
//! - [`MtcRate`] — the four supported MTC rates (24, 25, 29.97 drop-frame,
//!   30 fps).
//! - [`MtcTime`] — the encoded timecode (`HH:MM:SS:FF`), plus rate.
//! - [`MtcEncoder`] — emits 8 quarter-frame messages per MTC frame, plus
//!   occasional full-frame messages for cue points.
//! - [`MtcFullFrame`] — the parsed form of a full-frame SysEx message.

use crate::error::{AudioBasicsError, AudioBasicsResult};
use crate::midi_message::MidiMessage;

/// The four SMPTE / MTC rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MtcRate {
    /// 24 fps, non-drop.
    TwentyFour,
    /// 25 fps, non-drop (PAL).
    TwentyFive,
    /// 29.97 fps, drop-frame (NTSC).
    TwentyNineNineSevenDrop,
    /// 30 fps, non-drop.
    Thirty,
}

impl MtcRate {
    /// The raw fps for this rate (drop-frame has an effective rate of
    /// `29.97` even though it nominally runs at 30).
    pub fn fps(self) -> f32 {
        match self {
            MtcRate::TwentyFour => 24.0,
            MtcRate::TwentyFive => 25.0,
            MtcRate::TwentyNineNineSevenDrop => 29.97,
            MtcRate::Thirty => 30.0,
        }
    }

    /// The quarter-frame type byte that encodes this rate. The spec
    /// stores the rate in bits 4..6 of the first (frame-type = 0) quarter
    /// frame's value nibble.
    pub fn to_quarter_frame_type(self) -> u8 {
        match self {
            MtcRate::TwentyFour => 0,
            MtcRate::TwentyFive => 1,
            MtcRate::TwentyNineNineSevenDrop => 2,
            MtcRate::Thirty => 3,
        }
    }

    /// Inverse of [`to_quarter_frame_type`][Self::to_quarter_frame_type].
    /// Returns `None` for reserved / unknown rate codes.
    pub fn from_quarter_frame_type(code: u8) -> Self {
        match code & 0x03 {
            0 => MtcRate::TwentyFour,
            1 => MtcRate::TwentyFive,
            2 => MtcRate::TwentyNineNineSevenDrop,
            _ => MtcRate::Thirty,
        }
    }

    /// True if this rate uses drop-frame counting.
    pub fn is_drop_frame(self) -> bool {
        matches!(self, MtcRate::TwentyNineNineSevenDrop)
    }
}

/// A point in MTC timecode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MtcTime {
    /// The SMPTE frame rate.
    pub rate: MtcRate,
    /// Hours, in `0..24`.
    pub hours: u8,
    /// Minutes, in `0..60`.
    pub minutes: u8,
    /// Seconds, in `0..60`.
    pub seconds: u8,
    /// Frames within the current second, in `0..fps`.
    pub frames: u8,
}

impl MtcTime {
    /// Construct a new `MtcTime`, validating all components.
    pub fn new(rate: MtcRate, hours: u8, minutes: u8, seconds: u8, frames: u8) -> AudioBasicsResult<Self> {
        if hours >= 24 {
            return Err(AudioBasicsError::InvalidMtcTime {
                component: "hours",
                value: hours,
                range: (0, 23),
            });
        }
        if minutes >= 60 {
            return Err(AudioBasicsError::InvalidMtcTime {
                component: "minutes",
                value: minutes,
                range: (0, 59),
            });
        }
        if seconds >= 60 {
            return Err(AudioBasicsError::InvalidMtcTime {
                component: "seconds",
                value: seconds,
                range: (0, 59),
            });
        }
        let max_frames = rate.fps().ceil() as u8;
        if frames >= max_frames {
            return Err(AudioBasicsError::InvalidMtcTime {
                component: "frames",
                value: frames,
                range: (0, max_frames - 1),
            });
        }
        Ok(Self {
            rate,
            hours,
            minutes,
            seconds,
            frames,
        })
    }

    /// Construct an `MtcTime` without bounds-checking. Faster than
    /// [`new`][Self::new] but invalid values will produce nonsense when
    /// encoded.
    ///
    /// # Safety
    ///
    /// The caller must ensure all fields are within their respective ranges
    /// (see [`new`][Self::new]).
    pub const fn new_unchecked(
        rate: MtcRate,
        hours: u8,
        minutes: u8,
        seconds: u8,
        frames: u8,
    ) -> Self {
        Self {
            rate,
            hours,
            minutes,
            seconds,
            frames,
        }
    }

    /// Convert to a position measured in MTC frames (i.e. quarter-frame
    /// ticks / 8). Drop-frame time uses the standard drop-frame formula.
    pub fn to_frame_count(self) -> u64 {
        let fps = self.rate.fps();
        let nominal = self.rate.fps().round() as u64;
        let h = self.hours as u64;
        let m = self.minutes as u64;
        let s = self.seconds as u64;
        let f = self.frames as u64;

        if self.rate.is_drop_frame() {
            // Drop-frame formula: skip 2 frames at the start of every
            // minute, except every tenth minute.
            let drop_frames = 2;
            let total_minutes = 60 * h + m;
            let dropped = drop_frames * (total_minutes - total_minutes / 10);
            (h * 60 * 60 + m * 60 + s) * nominal + f - dropped
        } else {
            // Use the integer fps to keep things exact for non-drop rates.
            let _ = fps;
            (h * 3600 + m * 60 + s) * nominal + f
        }
    }

    /// Convert a position measured in MTC frames back to an `MtcTime`.
    /// The frame count is interpreted using the supplied rate's
    /// conventions.
    pub fn from_frame_count(rate: MtcRate, frames: u64) -> Self {
        let nominal = rate.fps().round() as u64;
        if rate.is_drop_frame() {
            let drop_frames = 2;
            let frames_per_10_minutes = (nominal * 60 - drop_frames) * 10 + drop_frames;
            let frames_per_minute = nominal * 60 - drop_frames;
            let d = frames / frames_per_10_minutes;
            let mut m = d * 10;
            let mut f = frames % frames_per_10_minutes;
            if f > drop_frames {
                f -= drop_frames;
                let m_bump = f / frames_per_minute;
                f %= frames_per_minute;
                if m_bump > 0 && f >= drop_frames * (m_bump - 1) {
                    f += drop_frames;
                }
                m += m_bump;
                // If we crossed a non-drop boundary and adjusted f down,
                // ensure we don't produce a frame that doesn't exist.
                if f >= nominal * 60 {
                    f -= nominal * 60;
                    m += 1;
                }
            }
            let s = f / nominal;
            f %= nominal;
            let h = m / 60;
            m %= 60;
            Self::new_unchecked(rate, h as u8, m as u8, s as u8, f as u8)
        } else {
            let h = frames / (nominal * 3600);
            let rem = frames % (nominal * 3600);
            let m = rem / (nominal * 60);
            let rem = rem % (nominal * 60);
            let s = rem / nominal;
            let f = rem % nominal;
            Self::new_unchecked(rate, h as u8, m as u8, s as u8, f as u8)
        }
    }

    /// Encode this time into the 4 bytes of a quarter-frame sequence.
    ///
    /// The byte layout is the standard MTC byte for each piece:
    /// `[0] = (rate_code << 4) | frame_type`,
    /// `[1] = (frames & 0x0F) << 4 | frame_type_index`,
    /// `[2] = (seconds & 0x0F) << 4 | frame_type_index`,
    /// `[3] = (minutes & 0x0F) << 4 | frame_type_index`,
    /// `[4] = ((hours & 0x0F) << 4) | (rate_bit << 1) | frame_type_index`,
    /// (etc — see the MTC spec).
    fn to_quarter_frame_bytes(self) -> [u8; 8] {
        let rate_code = self.rate.to_quarter_frame_type();
        let frame_type_bits = match self.rate {
            MtcRate::TwentyFour => 0,
            MtcRate::TwentyFive => 0,
            MtcRate::TwentyNineNineSevenDrop => 1,
            MtcRate::Thirty => 1,
        };

        // Type 0: piece = frames (low nibble).
        let piece_0 = (rate_code << 4) | (self.frames & 0x0F);
        // Type 1: piece = frames (high nibble).
        let piece_1 = ((self.frames >> 4) & 0x01) << 4;
        // Type 2: piece = seconds (low nibble).
        let piece_2 = (self.seconds & 0x0F) << 4;
        // Type 3: piece = seconds (high nibble).
        let piece_3 = ((self.seconds >> 4) & 0x03) << 4;
        // Type 4: piece = minutes (low nibble).
        let piece_4 = (self.minutes & 0x0F) << 4;
        // Type 5: piece = minutes (high nibble).
        let piece_5 = ((self.minutes >> 4) & 0x03) << 4;
        // Type 6: piece = hours (low nibble).
        let piece_6 = (self.hours & 0x0F) << 4;
        // Type 7: piece = hours (high nibble) + rate bits.
        let piece_7 =
            ((self.hours >> 4) & 0x01) << 4 | (frame_type_bits << 1) | rate_code;

        [
            piece_0,
            piece_1 | 1,
            piece_2 | 2,
            piece_3 | 3,
            piece_4 | 4,
            piece_5 | 5,
            piece_6 | 6,
            piece_7 | 7,
        ]
    }
}

/// An MTC full-frame SysEx message, parsed and ready to use.
///
/// The wire format is `[0xF0, 0x7F, 0x7F, 0x01, 0x01, hh, mm, ss, ff, 0xF7]`
/// — the `[0x7F, 0x7F, 0x01, 0x01]` prefix is the universal non-realtime
/// "MTC cue" header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MtcFullFrame {
    /// The decoded timecode.
    pub time: MtcTime,
}

impl MtcFullFrame {
    /// The length of the full-frame message on the wire.
    pub const WIRE_LENGTH: usize = 10;

    /// Encode a full-frame message.
    pub fn to_bytes(time: MtcTime) -> [u8; Self::WIRE_LENGTH] {
        [
            0xF0,
            0x7F,
            0x7F,
            0x01,
            0x01,
            time.hours | ((time.rate.to_quarter_frame_type() & 0x03) << 5),
            time.minutes,
            time.seconds,
            time.frames,
            0xF7,
        ]
    }

    /// Decode a full-frame message.
    ///
    /// Returns `None` if `bytes` doesn't match the full-frame shape.
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::WIRE_LENGTH {
            return None;
        }
        if bytes[0] != 0xF0
            || bytes[1] != 0x7F
            || bytes[2] != 0x7F
            || bytes[3] != 0x01
            || bytes[4] != 0x01
            || bytes[9] != 0xF7
        {
            return None;
        }
        let rate_code = (bytes[5] >> 5) & 0x03;
        let rate = MtcRate::from_quarter_frame_type(rate_code);
        let hours = bytes[5] & 0x1F;
        let minutes = bytes[6] & 0x3F;
        let seconds = bytes[7] & 0x3F;
        let frames = bytes[8] & 0x1F;
        // Use new_unchecked here so a corrupt message yields a `Some` with
        // garbage instead of a Result. The caller can re-validate via
        // `MtcTime::new` if needed.
        Some(Self {
            time: MtcTime::new_unchecked(rate, hours, minutes, seconds, frames),
        })
    }
}

/// Encodes MTC timecode into a sequence of `0xF1` quarter-frame MIDI
/// messages.
///
/// The encoder is **driven by quarter-frame calls**, not by wall time —
/// each call to [`encode_frame`][Self::encode_frame] advances the
/// internal state by one MTC frame and returns the 8 quarter-frame
/// messages that should be sent for that step.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "midi")] {
/// use logic_nih_plug_audio_basics::{MtcEncoder, MtcRate, MtcTime};
///
/// let mut enc = MtcEncoder::new(MtcTime::new_unchecked(MtcRate::TwentyFive, 0, 0, 0, 0));
/// enc.full_frame_lead_in = 0; // skip the lead-in for this example.
/// let msgs = enc.encode_frame();
/// assert_eq!(msgs.len(), 8);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MtcEncoder {
    current: MtcTime,
    /// Number of full-frame messages emitted at the start (one is usually
    /// enough to "lock" receivers).
    full_frames_emitted: u8,
    /// How many full-frame messages to emit at startup. Set to 0 to
    /// disable; 1 or 2 is typical.
    pub full_frame_lead_in: u8,
}

impl MtcEncoder {
    /// Construct an encoder starting at `start_time`.
    pub fn new(start_time: MtcTime) -> Self {
        Self {
            current: start_time,
            full_frames_emitted: 0,
            full_frame_lead_in: 1,
        }
    }

    /// The encoder's current timecode.
    #[inline]
    pub fn current(&self) -> MtcTime {
        self.current
    }

    /// Advance the encoder by one MTC frame and return the 8 quarter-frame
    /// messages for that step.
    ///
    /// If [`full_frame_lead_in`][Self::full_frame_lead_in] is non-zero,
    /// the first few calls prepend a full-frame message so that receivers
    /// can sync up quickly.
    pub fn encode_frame(&mut self) -> Vec<MidiMessage> {
        let mut out = Vec::with_capacity(if self.full_frames_emitted < self.full_frame_lead_in {
            9
        } else {
            8
        });

        if self.full_frames_emitted < self.full_frame_lead_in {
            let bytes = MtcFullFrame::to_bytes(self.current);
            // The parser only checks raw bytes; we don't need to go via
            // MidiMessage::sys_ex because the message is built directly.
            out.push(MidiMessage::from_bytes(bytes.to_vec(), 0));
            self.full_frames_emitted += 1;
        }

        let pieces = self.current.to_quarter_frame_bytes();
        for (i, &piece) in pieces.iter().enumerate() {
            out.push(MidiMessage::quarter_frame_msg(
                i as u8,
                (piece >> 4) & 0x0F,
            ));
        }

        self.current = MtcTime::from_frame_count(
            self.current.rate,
            self.current.to_frame_count() + 1,
        );

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_fps() {
        assert_eq!(MtcRate::TwentyFour.fps(), 24.0);
        assert_eq!(MtcRate::TwentyFive.fps(), 25.0);
        assert!((MtcRate::TwentyNineNineSevenDrop.fps() - 29.97).abs() < 1e-3);
        assert_eq!(MtcRate::Thirty.fps(), 30.0);
    }

    #[test]
    fn rate_round_trip() {
        for r in [
            MtcRate::TwentyFour,
            MtcRate::TwentyFive,
            MtcRate::TwentyNineNineSevenDrop,
            MtcRate::Thirty,
        ] {
            let code = r.to_quarter_frame_type();
            assert_eq!(MtcRate::from_quarter_frame_type(code), r);
        }
    }

    #[test]
    fn mtc_time_validation() {
        assert!(MtcTime::new(MtcRate::TwentyFive, 0, 0, 0, 0).is_ok());
        assert!(MtcTime::new(MtcRate::TwentyFive, 24, 0, 0, 0).is_err());
        assert!(MtcTime::new(MtcRate::TwentyFive, 0, 60, 0, 0).is_err());
        assert!(MtcTime::new(MtcRate::TwentyFive, 0, 0, 60, 0).is_err());
        // 30 fps allows frame 0..30.
        assert!(MtcTime::new(MtcRate::Thirty, 0, 0, 0, 30).is_err());
        assert!(MtcTime::new(MtcRate::Thirty, 0, 0, 0, 29).is_ok());
    }

    #[test]
    fn non_drop_frame_round_trip() {
        let t = MtcTime::new_unchecked(MtcRate::TwentyFive, 1, 23, 45, 12);
        let n = t.to_frame_count();
        let t2 = MtcTime::from_frame_count(t.rate, n);
        assert_eq!(t, t2);
    }

    #[test]
    fn drop_frame_round_trip_at_one_minute() {
        // 29.97 drop frame skips 2 frame numbers at the start of every
        // minute (except minute 0, 10, 20, ...). At minute 1, the
        // displayed frame count is 00:01:00;02 — let's check both
        // directions agree.
        let t = MtcTime::new_unchecked(MtcRate::TwentyNineNineSevenDrop, 0, 1, 0, 2);
        let n = t.to_frame_count();
        let t2 = MtcTime::from_frame_count(t.rate, n);
        assert_eq!(t, t2);
    }

    #[test]
    fn full_frame_encode_decode() {
        let t = MtcTime::new_unchecked(MtcRate::TwentyFour, 12, 34, 56, 7);
        let bytes = MtcFullFrame::to_bytes(t);
        assert_eq!(bytes.len(), MtcFullFrame::WIRE_LENGTH);
        assert_eq!(bytes[0], 0xF0);
        assert_eq!(bytes[9], 0xF7);
        let parsed = MtcFullFrame::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.time, t);
    }

    #[test]
    fn full_frame_rejects_garbage() {
        // Wrong prefix.
        let mut bytes = MtcFullFrame::to_bytes(MtcTime::new_unchecked(
            MtcRate::TwentyFour, 0, 0, 0, 0,
        ));
        bytes[1] = 0x00;
        assert!(MtcFullFrame::from_bytes(&bytes).is_none());

        // Wrong length.
        let bytes = [0xF0; 9];
        assert!(MtcFullFrame::from_bytes(&bytes).is_none());
    }

    #[test]
    fn encoder_emits_quarter_frames() {
        let mut enc = MtcEncoder::new(MtcTime::new_unchecked(MtcRate::TwentyFive, 0, 0, 0, 0));
        let msgs = enc.encode_frame();
        // First call should also emit a full-frame (default lead-in = 1).
        assert_eq!(msgs.len(), 9);
        // Full-frame is the first message.
        assert_eq!(msgs[0].kind(), crate::midi_message::MidiMessageKind::SysEx);
        for m in &msgs[1..] {
            assert_eq!(m.kind(), crate::midi_message::MidiMessageKind::TimeCode);
        }
        // Subsequent calls should NOT emit a full-frame.
        let msgs2 = enc.encode_frame();
        assert_eq!(msgs2.len(), 8);
        for m in &msgs2 {
            assert_eq!(m.kind(), crate::midi_message::MidiMessageKind::TimeCode);
        }
    }

    #[test]
    fn encoder_advances_time() {
        let mut enc = MtcEncoder::new(MtcTime::new_unchecked(MtcRate::TwentyFive, 0, 0, 0, 0));
        enc.full_frame_lead_in = 0; // skip the lead-in to make this test fast.
        enc.encode_frame();
        assert_eq!(enc.current().frames, 1);
        // Wrap frames -> seconds: 24 more calls takes us from
        // (0, 0, 0, 1) to (0, 0, 1, 0).
        for _ in 0..24 {
            enc.encode_frame();
        }
        assert_eq!(enc.current().seconds, 1);
        assert_eq!(enc.current().frames, 0);
    }

    #[test]
    fn encoder_without_lead_in() {
        let mut enc = MtcEncoder::new(MtcTime::new_unchecked(MtcRate::TwentyFive, 0, 0, 0, 0));
        enc.full_frame_lead_in = 0;
        let msgs = enc.encode_frame();
        assert_eq!(msgs.len(), 8);
    }
}
