//! OSC argument values, time tags, and colour type.
//!
//! [`OSCArgument`] is the JUCE-compatible sum type used for the arguments of
//! an [`OSCMessage`](crate::message::OSCMessage). It supports every type
//! described by the OSC 1.0 specification (plus colour, MIDI, and arrays for
//! convenience), so the same struct can be used both for the four "classic"
//! types JUCE exposes (`Int32`/`Float32`/`String`/`Blob`) and for the
//! richer types supported by [`rosc`] under the hood.
//!
//! Conversions to and from `rosc::OscType` happen at the sender/receiver
//! boundary, so user code never has to touch rosc directly.

use std::fmt;

/// A single OSC argument value.
///
/// Mirrors the OSC 1.0 type tag set:
/// - `Int32`, `Int64`
/// - `Float32`, `Float64`
/// - `String`, `Blob` (`Vec<u8>`)
/// - `TimeTag`, `Char`
/// - `Colour`, `MidiMessage`
/// - `Bool`, `Nil`, `Inf`, `Array`
///
/// `From` impls are provided for the common scalar variants so you can write
/// `OSCMessage::new("/amp", &[0.5_f32.into()])` without spelling out the enum
/// tag every time.
#[derive(Debug, Clone, PartialEq)]
pub enum OSCArgument {
    /// 32-bit signed integer (`i`).
    Int32(i32),
    /// 64-bit signed integer (`h`).
    Int64(i64),
    /// 32-bit float (`f`).
    Float32(f32),
    /// 64-bit float (`d`).
    Float64(f64),
    /// OSC string (`s`). Length is unconstrained on our side; rosc pads on
    /// encode.
    String(String),
    /// Opaque byte blob (`b`).
    Blob(Vec<u8>),
    /// OSC time tag (`t`).
    TimeTag(OSCTimeTag),
    /// 32-bit Unicode character (`c`).
    Char(char),
    /// RGBA colour (`r`).
    Colour(OSCColour),
    /// 4-byte MIDI message (`m`).
    MidiMessage(OSCMidiMessage),
    /// Boolean (`T`/`F` — rosc encodes/decodes from `OscType::Bool`).
    Bool(bool),
    /// OSC null (`N`).
    Nil,
    /// OSC infinity / "impulse" (`I`).
    Inf,
    /// Nested array (`[ ]`).
    Array(Vec<OSCArgument>),
}

impl OSCArgument {
    /// Returns the OSC type tag character for this argument (the first byte of
    /// the type tag string).
    pub fn type_tag(&self) -> char {
        match self {
            OSCArgument::Int32(_) => 'i',
            OSCArgument::Int64(_) => 'h',
            OSCArgument::Float32(_) => 'f',
            OSCArgument::Float64(_) => 'd',
            OSCArgument::String(_) => 's',
            OSCArgument::Blob(_) => 'b',
            OSCArgument::TimeTag(_) => 't',
            OSCArgument::Char(_) => 'c',
            OSCArgument::Colour(_) => 'r',
            OSCArgument::MidiMessage(_) => 'm',
            OSCArgument::Bool(b) => {
                if *b { 'T' } else { 'F' }
            }
            OSCArgument::Nil => 'N',
            OSCArgument::Inf => 'I',
            OSCArgument::Array(_) => '[', // start of array
        }
    }

    /// Returns true if this argument is an array. Convenience for
    /// `matches!(self, OSCArgument::Array(_))`.
    pub fn is_array(&self) -> bool {
        matches!(self, OSCArgument::Array(_))
    }

    /// If this argument is an `Int32`, returns the inner value; otherwise `None`.
    pub fn as_int32(&self) -> Option<i32> {
        match self {
            OSCArgument::Int32(v) => Some(*v),
            _ => None,
        }
    }

    /// If this argument is an `Int64`, returns the inner value; otherwise `None`.
    pub fn as_int64(&self) -> Option<i64> {
        match self {
            OSCArgument::Int64(v) => Some(*v),
            _ => None,
        }
    }

    /// If this argument is a `Float32`, returns the inner value; otherwise `None`.
    pub fn as_float32(&self) -> Option<f32> {
        match self {
            OSCArgument::Float32(v) => Some(*v),
            _ => None,
        }
    }

    /// If this argument is a `Float64`, returns the inner value; otherwise `None`.
    pub fn as_float64(&self) -> Option<f64> {
        match self {
            OSCArgument::Float64(v) => Some(*v),
            _ => None,
        }
    }

    /// If this argument is a `String`, returns a reference to the inner string.
    pub fn as_string(&self) -> Option<&str> {
        match self {
            OSCArgument::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// If this argument is a `Blob`, returns a reference to the inner bytes.
    pub fn as_blob(&self) -> Option<&[u8]> {
        match self {
            OSCArgument::Blob(b) => Some(b.as_slice()),
            _ => None,
        }
    }

    /// If this argument is a `Bool`, returns the inner value; otherwise `None`.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            OSCArgument::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// If this argument is a `TimeTag`, returns a reference to the inner
    /// [`OSCTimeTag`].
    pub fn as_time_tag(&self) -> Option<&OSCTimeTag> {
        match self {
            OSCArgument::TimeTag(t) => Some(t),
            _ => None,
        }
    }

    /// If this argument is a `Colour`, returns a reference to the inner
    /// [`OSCColour`].
    pub fn as_colour(&self) -> Option<&OSCColour> {
        match self {
            OSCArgument::Colour(c) => Some(c),
            _ => None,
        }
    }

    /// If this argument is a `MidiMessage`, returns a reference to the inner
    /// [`OSCMidiMessage`].
    pub fn as_midi_message(&self) -> Option<&OSCMidiMessage> {
        match self {
            OSCArgument::MidiMessage(m) => Some(m),
            _ => None,
        }
    }
}

impl fmt::Display for OSCArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OSCArgument::Int32(v) => write!(f, "{v}i"),
            OSCArgument::Int64(v) => write!(f, "{v}h"),
            OSCArgument::Float32(v) => write!(f, "{v}f"),
            OSCArgument::Float64(v) => write!(f, "{v}d"),
            OSCArgument::String(s) => write!(f, "{s:?}s"),
            OSCArgument::Blob(b) => write!(f, "blob({} bytes)", b.len()),
            OSCArgument::TimeTag(t) => write!(f, "timetag({t})"),
            OSCArgument::Char(c) => write!(f, "{c:?}c"),
            OSCArgument::Colour(c) => write!(f, "rgba({},{},{},{})", c.red, c.green, c.blue, c.alpha),
            OSCArgument::MidiMessage(m) => write!(
                f,
                "midi({:02x},{:02x},{:02x},{:02x})",
                m.port, m.status, m.data1, m.data2
            ),
            OSCArgument::Bool(b) => write!(f, "{}", if *b { "T" } else { "F" }),
            OSCArgument::Nil => f.write_str("Nil"),
            OSCArgument::Inf => f.write_str("Inf"),
            OSCArgument::Array(items) => {
                f.write_str("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_str("]")
            }
        }
    }
}

// --- Convenience From impls for the common scalar variants. ---

impl From<i32> for OSCArgument {
    fn from(v: i32) -> Self {
        OSCArgument::Int32(v)
    }
}

impl From<i64> for OSCArgument {
    fn from(v: i64) -> Self {
        OSCArgument::Int64(v)
    }
}

impl From<f32> for OSCArgument {
    fn from(v: f32) -> Self {
        OSCArgument::Float32(v)
    }
}

impl From<f64> for OSCArgument {
    fn from(v: f64) -> Self {
        OSCArgument::Float64(v)
    }
}

impl From<String> for OSCArgument {
    fn from(v: String) -> Self {
        OSCArgument::String(v)
    }
}

impl From<&str> for OSCArgument {
    fn from(v: &str) -> Self {
        OSCArgument::String(v.to_owned())
    }
}

impl From<Vec<u8>> for OSCArgument {
    fn from(v: Vec<u8>) -> Self {
        OSCArgument::Blob(v)
    }
}

impl From<bool> for OSCArgument {
    fn from(v: bool) -> Self {
        OSCArgument::Bool(v)
    }
}

impl From<char> for OSCArgument {
    fn from(v: char) -> Self {
        OSCArgument::Char(v)
    }
}

impl From<Vec<OSCArgument>> for OSCArgument {
    fn from(v: Vec<OSCArgument>) -> Self {
        OSCArgument::Array(v)
    }
}

impl From<OSCTimeTag> for OSCArgument {
    fn from(v: OSCTimeTag) -> Self {
        OSCArgument::TimeTag(v)
    }
}

impl From<OSCColour> for OSCArgument {
    fn from(v: OSCColour) -> Self {
        OSCArgument::Colour(v)
    }
}

impl From<OSCMidiMessage> for OSCArgument {
    fn from(v: OSCMidiMessage) -> Self {
        OSCArgument::MidiMessage(v)
    }
}

impl From<&OSCArgument> for OSCArgument {
    /// Clones the argument so that `&[OSCArgument; N]` works wherever
    /// `OSCMessage::new` wants `Into<OSCArgument>` items.
    fn from(v: &OSCArgument) -> Self {
        v.clone()
    }
}

/// An OSC time tag (`t`), represented as the standard pair of 32-bit words.
///
/// The first word is the number of seconds since 1900-01-01T00:00:00 UTC; the
/// second is the fractional part of a second (`1 << 32` units per second).
/// Use [`OSCTimeTag::now`] to grab the current wall-clock time as an OSC
/// time tag, or [`OSCTimeTag::immediate`] for a tag that fires "as soon as
/// possible" — the OSC convention for that is the immediate marker
/// `0x0000_0000_0000_0001`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct OSCTimeTag {
    /// Whole seconds since the OSC epoch (1900-01-01T00:00:00Z).
    pub seconds_since_1900: u32,
    /// Fractional seconds in OSC units (`1 << 32` ticks per second).
    pub fractional: u32,
}

impl OSCTimeTag {
    /// The OSC immediate marker. Per the spec this is the smallest non-zero
    /// value: 1 fractional tick.
    pub const IMMEDIATE_SECONDS: u32 = 0;
    /// The OSC immediate marker. Per the spec this is the smallest non-zero
    /// value: 1 fractional tick.
    pub const IMMEDIATE_FRACTIONAL: u32 = 1;

    /// The OSC "now" marker. Sending a bundle with this time tag tells the
    /// receiver to dispatch it as soon as possible (effectively skipping the
    /// scheduler).
    pub const fn immediate() -> Self {
        Self {
            seconds_since_1900: Self::IMMEDIATE_SECONDS,
            fractional: Self::IMMEDIATE_FRACTIONAL,
        }
    }

    /// Returns the current wall-clock time as an OSC time tag.
    ///
    /// Returns `None` if the system clock is somehow before 1900-01-01
    /// (which the spec covers, but no real OS will ever produce).
    pub fn now() -> Option<Self> {
        Self::from_system_time(std::time::SystemTime::now())
    }

    /// Converts a [`std::time::SystemTime`] into an OSC time tag.
    ///
    /// Returns `None` if `time` is earlier than the OSC epoch (1900-01-01T00:00:00Z).
    pub fn from_system_time(time: std::time::SystemTime) -> Option<Self> {
        match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(unix_secs) => {
                // Seconds between the OSC epoch (1900-01-01T00:00:00Z) and
                // the Unix epoch (1970-01-01T00:00:00Z):
                //   70 years * 365.2425 days * 86400 s/day ≈ 2_208_988_800 s.
                const OSC_TO_UNIX: u64 = 2_208_988_800;
                let total_secs = unix_secs.as_secs().checked_add(OSC_TO_UNIX)?;
                // We don't have sub-second precision from `as_secs`; rosc's
                // own conversion is also second-precision for SystemTime, so
                // mirror that here.
                Some(Self {
                    seconds_since_1900: u32::try_from(total_secs).ok()?,
                    fractional: 0,
                })
            }
            Err(_) => None,
        }
    }

    /// Converts this time tag back into a [`std::time::SystemTime`].
    ///
    /// Returns `None` if the time tag is before the Unix epoch (i.e.
    /// `seconds_since_1900` is less than 2_208_988_800).
    pub fn to_system_time(self) -> Option<std::time::SystemTime> {
        const OSC_TO_UNIX: u64 = 2_208_988_800;
        let unix_secs = u64::from(self.seconds_since_1900).checked_sub(OSC_TO_UNIX)?;
        Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(unix_secs))
    }
}

impl fmt::Display for OSCTimeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::immediate() {
            return f.write_str("immediate");
        }
        match self.to_system_time() {
            Some(t) => {
                // Use the seconds-since-UNIX_EPOCH representation so we don't
                // drag in a date-time formatting dependency.
                let secs = t
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                write!(f, "{secs}.{frac:08x}", frac = self.fractional)
            }
            None => write!(
                f,
                "before-1970({}.{:08x})",
                self.seconds_since_1900, self.fractional
            ),
        }
    }
}

/// A 4-byte MIDI message that can be sent as an OSC `m` argument.
///
/// This is the structure described by the OSC 1.0 spec for tunnelling MIDI
/// over OSC: a `port_id` byte plus the canonical 3-byte MIDI message.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct OSCMidiMessage {
    /// The MIDI port / cable number (0..16).
    pub port: u8,
    /// MIDI status byte (`0x80`..=`0xFF` for data bytes; `0x80`–`0xEF` for
    /// channel voice messages, `0xF0`–`0xF7` for system messages).
    pub status: u8,
    /// First data byte (`0..=127`).
    pub data1: u8,
    /// Second data byte (`0..=127`).
    pub data2: u8,
}

impl OSCMidiMessage {
    /// Builds a MIDI message from raw bytes. The first byte is treated as
    /// `port`/`status` depending on context; here we keep the same byte
    /// positions as rosc: `port`, `status`, `data1`, `data2`.
    pub fn new(port: u8, status: u8, data1: u8, data2: u8) -> Self {
        Self { port, status, data1, data2 }
    }
}

/// An RGBA colour that can be sent as an OSC `r` argument.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct OSCColour {
    /// Red component (`0..=255`).
    pub red: u8,
    /// Green component (`0..=255`).
    pub green: u8,
    /// Blue component (`0..=255`).
    pub blue: u8,
    /// Alpha component (`0..=255`).
    pub alpha: u8,
}

impl OSCColour {
    /// Builds an RGBA colour from its components.
    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self { red, green, blue, alpha }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_tags_match_osc_spec() {
        assert_eq!(OSCArgument::Int32(0).type_tag(), 'i');
        assert_eq!(OSCArgument::Int64(0).type_tag(), 'h');
        assert_eq!(OSCArgument::Float32(0.0).type_tag(), 'f');
        assert_eq!(OSCArgument::Float64(0.0).type_tag(), 'd');
        assert_eq!(OSCArgument::String(String::new()).type_tag(), 's');
        assert_eq!(OSCArgument::Blob(Vec::new()).type_tag(), 'b');
        assert_eq!(OSCArgument::TimeTag(OSCTimeTag::default()).type_tag(), 't');
        assert_eq!(OSCArgument::Char('a').type_tag(), 'c');
        assert_eq!(OSCArgument::Colour(OSCColour::default()).type_tag(), 'r');
        assert_eq!(OSCArgument::MidiMessage(OSCMidiMessage::default()).type_tag(), 'm');
        assert_eq!(OSCArgument::Bool(true).type_tag(), 'T');
        assert_eq!(OSCArgument::Bool(false).type_tag(), 'F');
        assert_eq!(OSCArgument::Nil.type_tag(), 'N');
        assert_eq!(OSCArgument::Inf.type_tag(), 'I');
        assert_eq!(OSCArgument::Array(Vec::new()).type_tag(), '[');
    }

    #[test]
    fn accessors_unwrap_correctly() {
        let a = OSCArgument::Float32(1.5);
        assert_eq!(a.as_float32(), Some(1.5));
        assert_eq!(a.as_float64(), None);

        let s = OSCArgument::String("hi".into());
        assert_eq!(s.as_string(), Some("hi"));
        assert!(!s.is_array());
    }

    #[test]
    fn from_impls_for_scalars() {
        assert_eq!(OSCArgument::from(42_i32), OSCArgument::Int32(42));
        assert_eq!(OSCArgument::from(true), OSCArgument::Bool(true));
        assert_eq!(
            OSCArgument::from("hi"),
            OSCArgument::String("hi".to_owned())
        );
    }

    #[test]
    fn immediate_time_tag() {
        let t = OSCTimeTag::immediate();
        assert_eq!(t.seconds_since_1900, 0);
        assert_eq!(t.fractional, 1);
    }

    #[test]
    fn time_tag_round_trips_for_post_1970_times() {
        let now = OSCTimeTag::now().expect("now");
        let back = now.to_system_time().expect("back");
        let now_again = OSCTimeTag::from_system_time(back).expect("again");
        assert_eq!(now.seconds_since_1900, now_again.seconds_since_1900);
    }
}
