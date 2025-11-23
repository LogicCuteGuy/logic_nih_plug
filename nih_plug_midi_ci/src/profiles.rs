//! MIDI-CI profile management.
//!
//! This module provides functionality for querying, enabling, and disabling
//! MIDI-CI profiles on devices.

use crate::error::Result;
use crate::protocol::{MessageHeader, MessageType, MidiCiMessage, Muid, ProfileId};

/// Profile inquiry message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileInquiry {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
}

impl ProfileInquiry {
    /// Creates a new profile inquiry.
    pub fn new(source: Muid, destination: Muid) -> Self {
        Self {
            source,
            destination,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::ProfileInquiry,
            self.source,
            self.destination,
        );
        MidiCiMessage::new(header, Vec::new())
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
        })
    }
}

/// Profile inquiry reply message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileInquiryReply {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Enabled profiles
    pub enabled_profiles: Vec<ProfileId>,
    /// Disabled profiles
    pub disabled_profiles: Vec<ProfileId>,
}

impl ProfileInquiryReply {
    /// Creates a new profile inquiry reply.
    pub fn new(
        source: Muid,
        destination: Muid,
        enabled_profiles: Vec<ProfileId>,
        disabled_profiles: Vec<ProfileId>,
    ) -> Self {
        Self {
            source,
            destination,
            enabled_profiles,
            disabled_profiles,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::ProfileInquiryReply,
            self.source,
            self.destination,
        );

        let mut payload = Vec::new();
        
        // Number of enabled profiles (2 bytes, 7-bit encoding)
        let enabled_count = self.enabled_profiles.len() as u16;
        payload.push((enabled_count & 0x7F) as u8);
        payload.push(((enabled_count >> 7) & 0x7F) as u8);

        // Enabled profile IDs
        for profile in &self.enabled_profiles {
            payload.extend_from_slice(&profile.bytes);
        }

        // Number of disabled profiles (2 bytes, 7-bit encoding)
        let disabled_count = self.disabled_profiles.len() as u16;
        payload.push((disabled_count & 0x7F) as u8);
        payload.push(((disabled_count >> 7) & 0x7F) as u8);

        // Disabled profile IDs
        for profile in &self.disabled_profiles {
            payload.extend_from_slice(&profile.bytes);
        }

        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        let payload = &message.payload;
        let mut offset = 0;

        if payload.len() < 2 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 2,
                actual: payload.len(),
            });
        }

        // Parse enabled profiles count
        let enabled_count =
            (payload[offset] as u16) | ((payload[offset + 1] as u16) << 7);
        offset += 2;

        // Parse enabled profiles
        let mut enabled_profiles = Vec::new();
        for _ in 0..enabled_count {
            if offset + 5 > payload.len() {
                return Err(crate::error::MidiCiError::MessageTooShort {
                    expected: offset + 5,
                    actual: payload.len(),
                });
            }
            let profile = ProfileId::from_slice(&payload[offset..offset + 5])?;
            enabled_profiles.push(profile);
            offset += 5;
        }

        if offset + 2 > payload.len() {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: offset + 2,
                actual: payload.len(),
            });
        }

        // Parse disabled profiles count
        let disabled_count =
            (payload[offset] as u16) | ((payload[offset + 1] as u16) << 7);
        offset += 2;

        // Parse disabled profiles
        let mut disabled_profiles = Vec::new();
        for _ in 0..disabled_count {
            if offset + 5 > payload.len() {
                return Err(crate::error::MidiCiError::MessageTooShort {
                    expected: offset + 5,
                    actual: payload.len(),
                });
            }
            let profile = ProfileId::from_slice(&payload[offset..offset + 5])?;
            disabled_profiles.push(profile);
            offset += 5;
        }

        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            enabled_profiles,
            disabled_profiles,
        })
    }
}

/// Set profile on message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetProfileOn {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Profile ID to enable
    pub profile: ProfileId,
}

impl SetProfileOn {
    /// Creates a new set profile on message.
    pub fn new(source: Muid, destination: Muid, profile: ProfileId) -> Self {
        Self {
            source,
            destination,
            profile,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::SetProfileOn,
            self.source,
            self.destination,
        );
        MidiCiMessage::new(header, self.profile.bytes.to_vec())
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        let profile = ProfileId::from_slice(&message.payload)?;
        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            profile,
        })
    }
}

/// Set profile off message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetProfileOff {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Profile ID to disable
    pub profile: ProfileId,
}

impl SetProfileOff {
    /// Creates a new set profile off message.
    pub fn new(source: Muid, destination: Muid, profile: ProfileId) -> Self {
        Self {
            source,
            destination,
            profile,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::SetProfileOff,
            self.source,
            self.destination,
        );
        MidiCiMessage::new(header, self.profile.bytes.to_vec())
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        let profile = ProfileId::from_slice(&message.payload)?;
        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            profile,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_inquiry_roundtrip() {
        let inquiry = ProfileInquiry::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
        );
        let message = inquiry.to_message();
        let parsed = ProfileInquiry::from_message(&message).unwrap();
        assert_eq!(inquiry, parsed);
    }

    #[test]
    fn test_profile_inquiry_reply_roundtrip() {
        let enabled = vec![
            ProfileId::new([0x01, 0x02, 0x03, 0x04, 0x05]),
            ProfileId::new([0x06, 0x07, 0x08, 0x09, 0x0A]),
        ];
        let disabled = vec![ProfileId::new([0x0B, 0x0C, 0x0D, 0x0E, 0x0F])];

        let reply = ProfileInquiryReply::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            enabled,
            disabled,
        );
        let message = reply.to_message();
        let parsed = ProfileInquiryReply::from_message(&message).unwrap();
        assert_eq!(reply, parsed);
    }

    #[test]
    fn test_set_profile_on_roundtrip() {
        let profile = ProfileId::new([0x01, 0x02, 0x03, 0x04, 0x05]);
        let set_on = SetProfileOn::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            profile,
        );
        let message = set_on.to_message();
        let parsed = SetProfileOn::from_message(&message).unwrap();
        assert_eq!(set_on, parsed);
    }

    #[test]
    fn test_set_profile_off_roundtrip() {
        let profile = ProfileId::new([0x01, 0x02, 0x03, 0x04, 0x05]);
        let set_off = SetProfileOff::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            profile,
        );
        let message = set_off.to_message();
        let parsed = SetProfileOff::from_message(&message).unwrap();
        assert_eq!(set_off, parsed);
    }
}
