//! OSC message types and data structures.

/// An OSC message containing an address pattern and arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct OscMessage {
    /// The OSC address pattern (e.g., "/synth/frequency").
    pub address: String,
    /// The arguments of the message.
    pub arguments: Vec<OscType>,
}

impl OscMessage {
    /// Creates a new OSC message with the given address and arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscMessage, OscType};
    ///
    /// let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
    /// assert_eq!(msg.address, "/test");
    /// ```
    pub fn new(address: impl Into<String>, arguments: Vec<OscType>) -> Self {
        Self {
            address: address.into(),
            arguments,
        }
    }

    /// Creates a new OSC message with no arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::OscMessage;
    ///
    /// let msg = OscMessage::with_address("/test");
    /// assert_eq!(msg.address, "/test");
    /// assert!(msg.arguments.is_empty());
    /// ```
    pub fn with_address(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            arguments: Vec::new(),
        }
    }

    /// Adds an argument to the message.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscMessage, OscType};
    ///
    /// let mut msg = OscMessage::with_address("/test");
    /// msg.add_argument(OscType::Float(3.14));
    /// assert_eq!(msg.arguments.len(), 1);
    /// ```
    pub fn add_argument(&mut self, arg: OscType) {
        self.arguments.push(arg);
    }
}

/// OSC data types as defined in the OSC 1.0 specification.
#[derive(Debug, Clone, PartialEq)]
pub enum OscType {
    /// 32-bit integer.
    Int(i32),
    /// 32-bit floating point number.
    Float(f32),
    /// OSC-string (null-terminated ASCII string).
    String(String),
    /// OSC-blob (arbitrary binary data).
    Blob(Vec<u8>),
    /// 64-bit integer (OSC 1.1 extension).
    Long(i64),
    /// 64-bit floating point number (OSC 1.1 extension).
    Double(f64),
    /// OSC-timetag (NTP timestamp format).
    Time(OscTime),
    /// Character (ASCII).
    Char(char),
    /// RGBA color.
    Color(OscColor),
    /// MIDI message (4 bytes: port id, status, data1, data2).
    Midi(OscMidi),
    /// True value.
    True,
    /// False value.
    False,
    /// Nil/null value.
    Nil,
    /// Impulse/bang/infinitum (used for event triggers).
    Impulse,
    /// Array of OSC values (OSC 1.1 extension).
    Array(Vec<OscType>),
}

/// OSC time tag in NTP timestamp format.
///
/// Represents time as seconds since January 1, 1900.
/// Special value of (0, 1) means "immediately".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OscTime {
    /// Seconds since January 1, 1900.
    pub seconds: u32,
    /// Fractional part of a second (1/2^32 of a second).
    pub fractional: u32,
}

impl OscTime {
    /// Creates a new OSC time tag.
    pub fn new(seconds: u32, fractional: u32) -> Self {
        Self {
            seconds,
            fractional,
        }
    }

    /// Creates an "immediate" time tag.
    ///
    /// This is a special value (0, 1) that indicates the message
    /// should be processed immediately.
    pub fn immediate() -> Self {
        Self {
            seconds: 0,
            fractional: 1,
        }
    }

    /// Checks if this is an immediate time tag.
    pub fn is_immediate(&self) -> bool {
        self.seconds == 0 && self.fractional == 1
    }
}

impl Default for OscTime {
    fn default() -> Self {
        Self::immediate()
    }
}

/// RGBA color value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OscColor {
    /// Red component (0-255).
    pub red: u8,
    /// Green component (0-255).
    pub green: u8,
    /// Blue component (0-255).
    pub blue: u8,
    /// Alpha component (0-255).
    pub alpha: u8,
}

impl OscColor {
    /// Creates a new RGBA color.
    pub fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Creates an RGB color with full opacity.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::new(red, green, blue, 255)
    }
}

/// MIDI message in OSC format.
///
/// Contains 4 bytes: port id, status byte, data1, data2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OscMidi {
    /// MIDI port ID.
    pub port: u8,
    /// MIDI status byte.
    pub status: u8,
    /// First data byte.
    pub data1: u8,
    /// Second data byte.
    pub data2: u8,
}

impl OscMidi {
    /// Creates a new MIDI message.
    pub fn new(port: u8, status: u8, data1: u8, data2: u8) -> Self {
        Self {
            port,
            status,
            data1,
            data2,
        }
    }
}

/// An OSC packet, which can be either a message or a bundle.
#[derive(Debug, Clone, PartialEq)]
pub enum OscPacket {
    /// A single OSC message.
    Message(OscMessage),
    /// A bundle of messages with a time tag.
    Bundle(OscBundle),
}

/// An OSC bundle containing a time tag and multiple packets.
#[derive(Debug, Clone, PartialEq)]
pub struct OscBundle {
    /// Time tag for when the bundle should be processed.
    pub time_tag: OscTime,
    /// The packets contained in this bundle (can be messages or nested bundles).
    pub packets: Vec<OscPacket>,
}

impl OscBundle {
    /// Creates a new OSC bundle with the given time tag.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::{OscBundle, OscTime};
    ///
    /// let bundle = OscBundle::new(OscTime::immediate());
    /// assert!(bundle.packets.is_empty());
    /// ```
    pub fn new(time_tag: OscTime) -> Self {
        Self {
            time_tag,
            packets: Vec::new(),
        }
    }

    /// Creates a new bundle with immediate time tag.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_osc::OscBundle;
    ///
    /// let bundle = OscBundle::immediate();
    /// assert!(bundle.time_tag.is_immediate());
    /// ```
    pub fn immediate() -> Self {
        Self::new(OscTime::immediate())
    }

    /// Adds a packet to the bundle.
    pub fn add_packet(&mut self, packet: OscPacket) {
        self.packets.push(packet);
    }

    /// Adds a message to the bundle.
    pub fn add_message(&mut self, message: OscMessage) {
        self.packets.push(OscPacket::Message(message));
    }

    /// Adds a nested bundle to the bundle.
    pub fn add_bundle(&mut self, bundle: OscBundle) {
        self.packets.push(OscPacket::Bundle(bundle));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
        assert_eq!(msg.address, "/test");
        assert_eq!(msg.arguments.len(), 1);
        assert_eq!(msg.arguments[0], OscType::Int(42));
    }

    #[test]
    fn test_message_with_address() {
        let msg = OscMessage::with_address("/test");
        assert_eq!(msg.address, "/test");
        assert!(msg.arguments.is_empty());
    }

    #[test]
    fn test_message_add_argument() {
        let mut msg = OscMessage::with_address("/test");
        msg.add_argument(OscType::Float(3.14));
        msg.add_argument(OscType::String("hello".to_string()));
        assert_eq!(msg.arguments.len(), 2);
    }

    #[test]
    fn test_time_immediate() {
        let time = OscTime::immediate();
        assert!(time.is_immediate());
        assert_eq!(time.seconds, 0);
        assert_eq!(time.fractional, 1);
    }

    #[test]
    fn test_time_default() {
        let time = OscTime::default();
        assert!(time.is_immediate());
    }

    #[test]
    fn test_color_creation() {
        let color = OscColor::new(255, 128, 64, 32);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);
        assert_eq!(color.alpha, 32);
    }

    #[test]
    fn test_color_rgb() {
        let color = OscColor::rgb(255, 128, 64);
        assert_eq!(color.red, 255);
        assert_eq!(color.green, 128);
        assert_eq!(color.blue, 64);
        assert_eq!(color.alpha, 255);
    }

    #[test]
    fn test_midi_creation() {
        let midi = OscMidi::new(0, 0x90, 60, 127);
        assert_eq!(midi.port, 0);
        assert_eq!(midi.status, 0x90);
        assert_eq!(midi.data1, 60);
        assert_eq!(midi.data2, 127);
    }

    #[test]
    fn test_bundle_creation() {
        let bundle = OscBundle::new(OscTime::immediate());
        assert!(bundle.time_tag.is_immediate());
        assert!(bundle.packets.is_empty());
    }

    #[test]
    fn test_bundle_immediate() {
        let bundle = OscBundle::immediate();
        assert!(bundle.time_tag.is_immediate());
    }

    #[test]
    fn test_bundle_add_message() {
        let mut bundle = OscBundle::immediate();
        let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
        bundle.add_message(msg);
        assert_eq!(bundle.packets.len(), 1);
        match &bundle.packets[0] {
            OscPacket::Message(m) => assert_eq!(m.address, "/test"),
            _ => panic!("Expected message"),
        }
    }

    #[test]
    fn test_bundle_add_nested_bundle() {
        let mut outer = OscBundle::immediate();
        let inner = OscBundle::immediate();
        outer.add_bundle(inner);
        assert_eq!(outer.packets.len(), 1);
        match &outer.packets[0] {
            OscPacket::Bundle(_) => {},
            _ => panic!("Expected bundle"),
        }
    }

    #[test]
    fn test_osc_type_variants() {
        let types = vec![
            OscType::Int(42),
            OscType::Float(3.14),
            OscType::String("test".to_string()),
            OscType::Blob(vec![1, 2, 3]),
            OscType::Long(1234567890),
            OscType::Double(3.14159265359),
            OscType::Time(OscTime::immediate()),
            OscType::Char('A'),
            OscType::Color(OscColor::rgb(255, 0, 0)),
            OscType::Midi(OscMidi::new(0, 0x90, 60, 127)),
            OscType::True,
            OscType::False,
            OscType::Nil,
            OscType::Impulse,
            OscType::Array(vec![OscType::Int(1), OscType::Int(2)]),
        ];
        assert_eq!(types.len(), 15);
    }
}
