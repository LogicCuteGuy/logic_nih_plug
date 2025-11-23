//! MIDI-CI device discovery and capability queries.
//!
//! This module provides functionality for discovering MIDI-CI capable devices
//! on the network, managing device information, and querying device capabilities.

use crate::error::Result;
use crate::protocol::{DeviceInfo, MessageHeader, MessageType, MidiCiMessage, Muid};

/// Device capabilities flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceCapabilities {
    /// Supports MIDI-CI profiles
    pub supports_profiles: bool,
    /// Supports property exchange
    pub supports_property_exchange: bool,
    /// Supports process inquiry
    pub supports_process_inquiry: bool,
}

impl DeviceCapabilities {
    /// Creates new device capabilities.
    pub fn new(
        supports_profiles: bool,
        supports_property_exchange: bool,
        supports_process_inquiry: bool,
    ) -> Self {
        Self {
            supports_profiles,
            supports_property_exchange,
            supports_process_inquiry,
        }
    }

    /// Creates capabilities with all features enabled.
    pub fn all() -> Self {
        Self {
            supports_profiles: true,
            supports_property_exchange: true,
            supports_process_inquiry: true,
        }
    }

    /// Creates capabilities with no features enabled.
    pub fn none() -> Self {
        Self {
            supports_profiles: false,
            supports_property_exchange: false,
            supports_process_inquiry: false,
        }
    }

    /// Encodes capabilities to a byte.
    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.supports_profiles {
            byte |= 0x01;
        }
        if self.supports_property_exchange {
            byte |= 0x02;
        }
        if self.supports_process_inquiry {
            byte |= 0x04;
        }
        byte
    }

    /// Decodes capabilities from a byte.
    pub fn from_byte(byte: u8) -> Self {
        Self {
            supports_profiles: (byte & 0x01) != 0,
            supports_property_exchange: (byte & 0x02) != 0,
            supports_process_inquiry: (byte & 0x04) != 0,
        }
    }
}

/// Discovery inquiry message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryInquiry {
    /// Source MUID
    pub source: Muid,
    /// Device information
    pub device_info: DeviceInfo,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
}

impl DiscoveryInquiry {
    /// Creates a new discovery inquiry.
    pub fn new(source: Muid, device_info: DeviceInfo, capabilities: DeviceCapabilities) -> Self {
        Self {
            source,
            device_info,
            capabilities,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::DiscoveryInquiry,
            self.source,
            Muid::BROADCAST,
        );
        let mut payload = self.device_info.to_bytes();
        payload.push(self.capabilities.to_byte());
        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        if message.payload.is_empty() {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 1,
                actual: 0,
            });
        }

        let device_info_len = message.payload.len() - 1;
        let device_info = DeviceInfo::from_bytes(&message.payload[..device_info_len])?;
        let capabilities = DeviceCapabilities::from_byte(message.payload[device_info_len]);

        Ok(Self {
            source: message.header.source,
            device_info,
            capabilities,
        })
    }
}

/// Discovery reply message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryReply {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID (the device that sent the inquiry)
    pub destination: Muid,
    /// Device information
    pub device_info: DeviceInfo,
    /// Device capabilities
    pub capabilities: DeviceCapabilities,
}

impl DiscoveryReply {
    /// Creates a new discovery reply.
    pub fn new(
        source: Muid,
        destination: Muid,
        device_info: DeviceInfo,
        capabilities: DeviceCapabilities,
    ) -> Self {
        Self {
            source,
            destination,
            device_info,
            capabilities,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::DiscoveryReply,
            self.source,
            self.destination,
        );
        let mut payload = self.device_info.to_bytes();
        payload.push(self.capabilities.to_byte());
        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        if message.payload.is_empty() {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 1,
                actual: 0,
            });
        }

        let device_info_len = message.payload.len() - 1;
        let device_info = DeviceInfo::from_bytes(&message.payload[..device_info_len])?;
        let capabilities = DeviceCapabilities::from_byte(message.payload[device_info_len]);

        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            device_info,
            capabilities,
        })
    }
}

/// Endpoint information inquiry message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointInfoInquiry {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
}

impl EndpointInfoInquiry {
    /// Creates a new endpoint info inquiry.
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
            MessageType::EndpointInfoInquiry,
            self.source,
            self.destination,
        );
        MidiCiMessage::new(header, vec![0x01]) // Status byte
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
        })
    }
}

/// Endpoint information reply message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointInfoReply {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Product instance ID
    pub product_instance_id: String,
    /// Endpoint name
    pub endpoint_name: String,
}

impl EndpointInfoReply {
    /// Creates a new endpoint info reply.
    pub fn new(
        source: Muid,
        destination: Muid,
        product_instance_id: String,
        endpoint_name: String,
    ) -> Self {
        Self {
            source,
            destination,
            product_instance_id,
            endpoint_name,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::EndpointInfoReply,
            self.source,
            self.destination,
        );

        let mut payload = Vec::new();
        payload.push(0x01); // Status byte

        // Encode product instance ID
        let pid_bytes = self.product_instance_id.as_bytes();
        let pid_len = pid_bytes.len() as u16;
        payload.push((pid_len & 0x7F) as u8);
        payload.push(((pid_len >> 7) & 0x7F) as u8);
        payload.extend_from_slice(pid_bytes);

        // Encode endpoint name
        let name_bytes = self.endpoint_name.as_bytes();
        let name_len = name_bytes.len() as u16;
        payload.push((name_len & 0x7F) as u8);
        payload.push(((name_len >> 7) & 0x7F) as u8);
        payload.extend_from_slice(name_bytes);

        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        if message.payload.len() < 5 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 5,
                actual: message.payload.len(),
            });
        }

        let mut offset = 1; // Skip status byte

        // Parse product instance ID
        let pid_len = (message.payload[offset] as usize)
            | ((message.payload[offset + 1] as usize) << 7);
        offset += 2;

        if message.payload.len() < offset + pid_len + 2 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: offset + pid_len + 2,
                actual: message.payload.len(),
            });
        }

        let product_instance_id = String::from_utf8(
            message.payload[offset..offset + pid_len].to_vec(),
        )
        .map_err(|e| crate::error::MidiCiError::InvalidMessageFormat(e.to_string()))?;
        offset += pid_len;

        // Parse endpoint name
        let name_len = (message.payload[offset] as usize)
            | ((message.payload[offset + 1] as usize) << 7);
        offset += 2;

        if message.payload.len() < offset + name_len {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: offset + name_len,
                actual: message.payload.len(),
            });
        }

        let endpoint_name = String::from_utf8(
            message.payload[offset..offset + name_len].to_vec(),
        )
        .map_err(|e| crate::error::MidiCiError::InvalidMessageFormat(e.to_string()))?;

        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            product_instance_id,
            endpoint_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_capabilities_encoding() {
        let caps = DeviceCapabilities::new(true, false, true);
        let byte = caps.to_byte();
        let decoded = DeviceCapabilities::from_byte(byte);
        assert_eq!(caps, decoded);
    }

    #[test]
    fn test_discovery_inquiry_roundtrip() {
        let device_info = DeviceInfo::new(vec![0x7D], 0x1234, 0x3678, 0x9ABCDEF);
        let capabilities = DeviceCapabilities::all();
        let inquiry = DiscoveryInquiry::new(
            Muid::new(0x1234567).unwrap(),
            device_info,
            capabilities,
        );
        let message = inquiry.to_message();
        let parsed = DiscoveryInquiry::from_message(&message).unwrap();
        assert_eq!(inquiry, parsed);
    }

    #[test]
    fn test_discovery_reply_roundtrip() {
        let device_info = DeviceInfo::new(vec![0x7D], 0x1234, 0x3678, 0x9ABCDEF);
        let capabilities = DeviceCapabilities::all();
        let reply = DiscoveryReply::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            device_info,
            capabilities,
        );
        let message = reply.to_message();
        let parsed = DiscoveryReply::from_message(&message).unwrap();
        assert_eq!(reply, parsed);
    }

    #[test]
    fn test_endpoint_info_inquiry_roundtrip() {
        let inquiry = EndpointInfoInquiry::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
        );
        let message = inquiry.to_message();
        let parsed = EndpointInfoInquiry::from_message(&message).unwrap();
        assert_eq!(inquiry, parsed);
    }

    #[test]
    fn test_endpoint_info_reply_roundtrip() {
        let reply = EndpointInfoReply::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            "PROD-12345".to_string(),
            "My MIDI Device".to_string(),
        );
        let message = reply.to_message();
        let parsed = EndpointInfoReply::from_message(&message).unwrap();
        assert_eq!(reply, parsed);
    }
}
