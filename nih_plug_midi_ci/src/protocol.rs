//! MIDI-CI protocol message types and parsing.
//!
//! This module defines the core MIDI-CI message types according to the MIDI-CI specification.
//! MIDI-CI (MIDI Capability Inquiry) is part of the MIDI 2.0 specification and provides
//! a standardized way for MIDI devices to discover each other's capabilities and exchange
//! configuration information.

use crate::error::{MidiCiError, Result};

/// MIDI-CI protocol version (currently 1.2)
pub const MIDI_CI_VERSION: u8 = 0x02;

/// Universal System Exclusive ID for MIDI-CI
pub const MIDI_CI_UNIVERSAL_SYSEX: u8 = 0x7E;

/// MIDI-CI Sub-ID #1
pub const MIDI_CI_SUB_ID_1: u8 = 0x0D;

/// MUID (MIDI Unique Identifier) type.
/// A 28-bit value used to uniquely identify MIDI-CI devices.
/// The value 0x0FFFFFFF is reserved for broadcast messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Muid(u32);

impl Muid {
    /// Broadcast MUID (all devices)
    pub const BROADCAST: Muid = Muid(0x0FFF_FFFF);

    /// Function block broadcast
    pub const FUNCTION_BLOCK: Muid = Muid(0x0FFF_FFFE);

    /// Creates a new MUID from a 28-bit value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value exceeds 28 bits (> 0x0FFFFFFF).
    pub fn new(value: u32) -> Result<Self> {
        if value > 0x0FFF_FFFF {
            Err(MidiCiError::InvalidMuid(value))
        } else {
            Ok(Muid(value))
        }
    }

    /// Returns the raw MUID value.
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Checks if this is a broadcast MUID.
    pub fn is_broadcast(&self) -> bool {
        self.0 == Self::BROADCAST.0 || self.0 == Self::FUNCTION_BLOCK.0
    }

    /// Encodes the MUID as 4 bytes (little-endian, 7-bit per byte).
    pub fn to_bytes(&self) -> [u8; 4] {
        [
            (self.0 & 0x7F) as u8,
            ((self.0 >> 7) & 0x7F) as u8,
            ((self.0 >> 14) & 0x7F) as u8,
            ((self.0 >> 21) & 0x7F) as u8,
        ]
    }

    /// Decodes a MUID from 4 bytes (little-endian, 7-bit per byte).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(MidiCiError::MessageTooShort {
                expected: 4,
                actual: bytes.len(),
            });
        }

        let value = (bytes[0] as u32)
            | ((bytes[1] as u32) << 7)
            | ((bytes[2] as u32) << 14)
            | ((bytes[3] as u32) << 21);

        Self::new(value)
    }
}

/// MIDI-CI message types (Sub-ID #2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    /// Discovery Inquiry
    DiscoveryInquiry = 0x70,
    /// Discovery Reply
    DiscoveryReply = 0x71,
    /// Endpoint Info Inquiry
    EndpointInfoInquiry = 0x72,
    /// Endpoint Info Reply
    EndpointInfoReply = 0x73,
    /// Invalidate MUID
    InvalidateMuid = 0x7E,
    /// NAK (Negative Acknowledgement)
    Nak = 0x7F,

    /// Profile Inquiry
    ProfileInquiry = 0x20,
    /// Profile Inquiry Reply
    ProfileInquiryReply = 0x21,
    /// Set Profile On
    SetProfileOn = 0x22,
    /// Set Profile Off
    SetProfileOff = 0x23,
    /// Profile Enabled Report
    ProfileEnabledReport = 0x24,
    /// Profile Disabled Report
    ProfileDisabledReport = 0x25,
    /// Profile Added
    ProfileAdded = 0x26,
    /// Profile Removed
    ProfileRemoved = 0x27,
    /// Profile Details Inquiry
    ProfileDetailsInquiry = 0x28,
    /// Profile Details Reply
    ProfileDetailsReply = 0x29,
    /// Profile Specific Data
    ProfileSpecificData = 0x2F,

    /// Property Exchange Capabilities Inquiry
    PropertyExchangeCapabilitiesInquiry = 0x30,
    /// Property Exchange Capabilities Reply
    PropertyExchangeCapabilitiesReply = 0x31,
    /// Property Get Data
    PropertyGetData = 0x34,
    /// Property Get Data Reply
    PropertyGetDataReply = 0x35,
    /// Property Set Data
    PropertySetData = 0x36,
    /// Property Set Data Reply
    PropertySetDataReply = 0x37,
    /// Property Subscribe
    PropertySubscribe = 0x38,
    /// Property Subscribe Reply
    PropertySubscribeReply = 0x39,
    /// Property Notify
    PropertyNotify = 0x3F,

    /// Process Inquiry
    ProcessInquiry = 0x40,
    /// Process Inquiry Reply
    ProcessInquiryReply = 0x41,
    /// MIDI Message Report
    MidiMessageReport = 0x42,
    /// MIDI Message Report Reply
    MidiMessageReplyReport = 0x43,
    /// End of MIDI Message Report
    EndOfMidiMessageReport = 0x44,
}

impl MessageType {
    /// Converts a byte to a MessageType.
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0x70 => Ok(MessageType::DiscoveryInquiry),
            0x71 => Ok(MessageType::DiscoveryReply),
            0x72 => Ok(MessageType::EndpointInfoInquiry),
            0x73 => Ok(MessageType::EndpointInfoReply),
            0x7E => Ok(MessageType::InvalidateMuid),
            0x7F => Ok(MessageType::Nak),

            0x20 => Ok(MessageType::ProfileInquiry),
            0x21 => Ok(MessageType::ProfileInquiryReply),
            0x22 => Ok(MessageType::SetProfileOn),
            0x23 => Ok(MessageType::SetProfileOff),
            0x24 => Ok(MessageType::ProfileEnabledReport),
            0x25 => Ok(MessageType::ProfileDisabledReport),
            0x26 => Ok(MessageType::ProfileAdded),
            0x27 => Ok(MessageType::ProfileRemoved),
            0x28 => Ok(MessageType::ProfileDetailsInquiry),
            0x29 => Ok(MessageType::ProfileDetailsReply),
            0x2F => Ok(MessageType::ProfileSpecificData),

            0x30 => Ok(MessageType::PropertyExchangeCapabilitiesInquiry),
            0x31 => Ok(MessageType::PropertyExchangeCapabilitiesReply),
            0x34 => Ok(MessageType::PropertyGetData),
            0x35 => Ok(MessageType::PropertyGetDataReply),
            0x36 => Ok(MessageType::PropertySetData),
            0x37 => Ok(MessageType::PropertySetDataReply),
            0x38 => Ok(MessageType::PropertySubscribe),
            0x39 => Ok(MessageType::PropertySubscribeReply),
            0x3F => Ok(MessageType::PropertyNotify),

            0x40 => Ok(MessageType::ProcessInquiry),
            0x41 => Ok(MessageType::ProcessInquiryReply),
            0x42 => Ok(MessageType::MidiMessageReport),
            0x43 => Ok(MessageType::MidiMessageReplyReport),
            0x44 => Ok(MessageType::EndOfMidiMessageReport),

            _ => Err(MidiCiError::InvalidMessageType(value)),
        }
    }

    /// Converts the MessageType to a byte.
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// Device information for MIDI-CI discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    /// Manufacturer ID (3 bytes for extended, 1 byte for standard)
    pub manufacturer: Vec<u8>,
    /// Device family (2 bytes)
    pub family: u16,
    /// Device model (2 bytes)
    pub model: u16,
    /// Software revision (4 bytes)
    pub revision: u32,
}

impl DeviceInfo {
    /// Creates a new DeviceInfo.
    pub fn new(manufacturer: Vec<u8>, family: u16, model: u16, revision: u32) -> Self {
        Self {
            manufacturer,
            family,
            model,
            revision,
        }
    }

    /// Encodes the device info to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.manufacturer);
        bytes.push((self.family & 0x7F) as u8);
        bytes.push(((self.family >> 7) & 0x7F) as u8);
        bytes.push((self.model & 0x7F) as u8);
        bytes.push(((self.model >> 7) & 0x7F) as u8);
        bytes.push((self.revision & 0x7F) as u8);
        bytes.push(((self.revision >> 7) & 0x7F) as u8);
        bytes.push(((self.revision >> 14) & 0x7F) as u8);
        bytes.push(((self.revision >> 21) & 0x7F) as u8);
        bytes
    }

    /// Decodes device info from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let manufacturer = if bytes.len() > 0 && bytes[0] == 0x00 {
            // Extended manufacturer ID (3 bytes)
            if bytes.len() < 11 {
                return Err(MidiCiError::MessageTooShort {
                    expected: 11,
                    actual: bytes.len(),
                });
            }
            vec![bytes[0], bytes[1], bytes[2]]
        } else {
            // Standard manufacturer ID (1 byte)
            if bytes.len() < 9 {
                return Err(MidiCiError::MessageTooShort {
                    expected: 9,
                    actual: bytes.len(),
                });
            }
            vec![bytes[0]]
        };

        let offset = manufacturer.len();
        let family = (bytes[offset] as u16) | ((bytes[offset + 1] as u16) << 7);
        let model = (bytes[offset + 2] as u16) | ((bytes[offset + 3] as u16) << 7);
        let revision = (bytes[offset + 4] as u32)
            | ((bytes[offset + 5] as u32) << 7)
            | ((bytes[offset + 6] as u32) << 14)
            | ((bytes[offset + 7] as u32) << 21);

        Ok(Self {
            manufacturer,
            family,
            model,
            revision,
        })
    }
}

/// Profile ID (5 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProfileId {
    /// Profile ID bytes
    pub bytes: [u8; 5],
}

impl ProfileId {
    /// Creates a new ProfileId.
    pub fn new(bytes: [u8; 5]) -> Self {
        Self { bytes }
    }

    /// Creates a ProfileId from a slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() < 5 {
            return Err(MidiCiError::InvalidProfileId);
        }
        let mut bytes = [0u8; 5];
        bytes.copy_from_slice(&slice[0..5]);
        Ok(Self { bytes })
    }
}

/// MIDI-CI message header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageHeader {
    /// Device ID (0x7F for all channels)
    pub device_id: u8,
    /// Message type
    pub message_type: MessageType,
    /// MIDI-CI version
    pub version: u8,
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
}

impl MessageHeader {
    /// Creates a new message header.
    pub fn new(
        device_id: u8,
        message_type: MessageType,
        source: Muid,
        destination: Muid,
    ) -> Self {
        Self {
            device_id,
            message_type,
            version: MIDI_CI_VERSION,
            source,
            destination,
        }
    }

    /// Encodes the header to bytes (without SysEx framing).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.device_id);
        bytes.push(MIDI_CI_SUB_ID_1);
        bytes.push(self.message_type.to_u8());
        bytes.push(self.version);
        bytes.extend_from_slice(&self.source.to_bytes());
        bytes.extend_from_slice(&self.destination.to_bytes());
        bytes
    }

    /// Decodes a header from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 12 {
            return Err(MidiCiError::MessageTooShort {
                expected: 12,
                actual: bytes.len(),
            });
        }

        let device_id = bytes[0];
        
        if bytes[1] != MIDI_CI_SUB_ID_1 {
            return Err(MidiCiError::InvalidMessageFormat(
                "Invalid Sub-ID #1".to_string(),
            ));
        }

        let message_type = MessageType::from_u8(bytes[2])?;
        let version = bytes[3];

        let source = Muid::from_bytes(&bytes[4..8])?;
        let destination = Muid::from_bytes(&bytes[8..12])?;

        Ok(Self {
            device_id,
            message_type,
            version,
            source,
            destination,
        })
    }
}

/// A complete MIDI-CI message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MidiCiMessage {
    /// Message header
    pub header: MessageHeader,
    /// Message payload
    pub payload: Vec<u8>,
}

impl MidiCiMessage {
    /// Creates a new MIDI-CI message.
    pub fn new(header: MessageHeader, payload: Vec<u8>) -> Self {
        Self { header, payload }
    }

    /// Encodes the message as a System Exclusive message.
    pub fn to_sysex(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0xF0); // SysEx start
        bytes.push(MIDI_CI_UNIVERSAL_SYSEX);
        bytes.extend_from_slice(&self.header.to_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes.push(0xF7); // SysEx end
        bytes
    }

    /// Decodes a MIDI-CI message from a System Exclusive message.
    pub fn from_sysex(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 14 {
            return Err(MidiCiError::MessageTooShort {
                expected: 14,
                actual: bytes.len(),
            });
        }

        if bytes[0] != 0xF0 {
            return Err(MidiCiError::InvalidMessageFormat(
                "Missing SysEx start byte".to_string(),
            ));
        }

        if bytes[1] != MIDI_CI_UNIVERSAL_SYSEX {
            return Err(MidiCiError::InvalidMessageFormat(
                "Not a Universal SysEx message".to_string(),
            ));
        }

        if bytes[bytes.len() - 1] != 0xF7 {
            return Err(MidiCiError::InvalidMessageFormat(
                "Missing SysEx end byte".to_string(),
            ));
        }

        let header = MessageHeader::from_bytes(&bytes[2..14])?;
        let payload = bytes[14..bytes.len() - 1].to_vec();

        Ok(Self { header, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muid_creation() {
        let muid = Muid::new(0x1234567).unwrap();
        assert_eq!(muid.value(), 0x1234567);

        let result = Muid::new(0x1000_0000);
        assert!(result.is_err());
    }

    #[test]
    fn test_muid_broadcast() {
        assert!(Muid::BROADCAST.is_broadcast());
        assert!(Muid::FUNCTION_BLOCK.is_broadcast());
        assert!(!Muid::new(0x123).unwrap().is_broadcast());
    }

    #[test]
    fn test_muid_encoding() {
        let muid = Muid::new(0x0FEDCBA9).unwrap();
        let bytes = muid.to_bytes();
        let decoded = Muid::from_bytes(&bytes).unwrap();
        assert_eq!(muid, decoded);
    }

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(
            MessageType::from_u8(0x70).unwrap(),
            MessageType::DiscoveryInquiry
        );
        assert_eq!(MessageType::DiscoveryInquiry.to_u8(), 0x70);
        assert!(MessageType::from_u8(0xFF).is_err());
    }

    #[test]
    fn test_device_info_encoding() {
        // Use values that fit in 14 bits (max 0x3FFF = 16383)
        let info = DeviceInfo::new(vec![0x00, 0x01, 0x02], 0x1234, 0x3678, 0x9ABCDEF);
        let bytes = info.to_bytes();
        let decoded = DeviceInfo::from_bytes(&bytes).unwrap();
        assert_eq!(info, decoded);
    }

    #[test]
    fn test_message_header_encoding() {
        let header = MessageHeader::new(
            0x7F,
            MessageType::DiscoveryInquiry,
            Muid::new(0x1234567).unwrap(),
            Muid::BROADCAST,
        );
        let bytes = header.to_bytes();
        let decoded = MessageHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header, decoded);
    }

    #[test]
    fn test_sysex_encoding() {
        let header = MessageHeader::new(
            0x7F,
            MessageType::DiscoveryInquiry,
            Muid::new(0x1234567).unwrap(),
            Muid::BROADCAST,
        );
        let message = MidiCiMessage::new(header, vec![0x01, 0x02, 0x03]);
        let sysex = message.to_sysex();
        let decoded = MidiCiMessage::from_sysex(&sysex).unwrap();
        assert_eq!(message, decoded);
    }
}
