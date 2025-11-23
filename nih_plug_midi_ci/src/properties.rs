//! MIDI-CI property exchange.
//!
//! This module provides functionality for exchanging property data between
//! MIDI-CI devices, including getting, setting, and subscribing to properties.

use crate::error::Result;
use crate::protocol::{MessageHeader, MessageType, MidiCiMessage, Muid};

/// Property exchange capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyExchangeCapabilities {
    /// Maximum number of simultaneous property requests
    pub max_simultaneous_requests: u8,
}

impl PropertyExchangeCapabilities {
    /// Creates new property exchange capabilities.
    pub fn new(max_simultaneous_requests: u8) -> Self {
        Self {
            max_simultaneous_requests,
        }
    }
}

/// Property exchange capabilities inquiry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyExchangeCapabilitiesInquiry {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
}

impl PropertyExchangeCapabilitiesInquiry {
    /// Creates a new property exchange capabilities inquiry.
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
            MessageType::PropertyExchangeCapabilitiesInquiry,
            self.source,
            self.destination,
        );
        MidiCiMessage::new(header, vec![0x01]) // Supported features byte
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
        })
    }
}

/// Property exchange capabilities reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyExchangeCapabilitiesReply {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Capabilities
    pub capabilities: PropertyExchangeCapabilities,
}

impl PropertyExchangeCapabilitiesReply {
    /// Creates a new property exchange capabilities reply.
    pub fn new(
        source: Muid,
        destination: Muid,
        capabilities: PropertyExchangeCapabilities,
    ) -> Self {
        Self {
            source,
            destination,
            capabilities,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::PropertyExchangeCapabilitiesReply,
            self.source,
            self.destination,
        );
        let payload = vec![
            0x01, // Supported features
            self.capabilities.max_simultaneous_requests,
        ];
        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        if message.payload.len() < 2 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 2,
                actual: message.payload.len(),
            });
        }

        let capabilities = PropertyExchangeCapabilities::new(message.payload[1]);

        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            capabilities,
        })
    }
}

/// Property get data request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyGetData {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Request ID
    pub request_id: u8,
    /// Property resource (JSON path or similar)
    pub resource: String,
}

impl PropertyGetData {
    /// Creates a new property get data request.
    pub fn new(source: Muid, destination: Muid, request_id: u8, resource: String) -> Self {
        Self {
            source,
            destination,
            request_id,
            resource,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::PropertyGetData,
            self.source,
            self.destination,
        );

        let mut payload = Vec::new();
        payload.push(self.request_id);

        // Encode resource as UTF-8 bytes
        let resource_bytes = self.resource.as_bytes();
        let len = resource_bytes.len() as u16;
        payload.push((len & 0x7F) as u8);
        payload.push(((len >> 7) & 0x7F) as u8);
        payload.extend_from_slice(resource_bytes);

        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        if message.payload.len() < 3 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 3,
                actual: message.payload.len(),
            });
        }

        let request_id = message.payload[0];
        let len = (message.payload[1] as usize) | ((message.payload[2] as usize) << 7);

        if message.payload.len() < 3 + len {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 3 + len,
                actual: message.payload.len(),
            });
        }

        let resource = String::from_utf8(message.payload[3..3 + len].to_vec())
            .map_err(|e| crate::error::MidiCiError::InvalidPropertyData(e.to_string()))?;

        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            request_id,
            resource,
        })
    }
}

/// Property get data reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyGetDataReply {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Request ID
    pub request_id: u8,
    /// Property data (typically JSON)
    pub data: Vec<u8>,
}

impl PropertyGetDataReply {
    /// Creates a new property get data reply.
    pub fn new(source: Muid, destination: Muid, request_id: u8, data: Vec<u8>) -> Self {
        Self {
            source,
            destination,
            request_id,
            data,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::PropertyGetDataReply,
            self.source,
            self.destination,
        );

        let mut payload = Vec::new();
        payload.push(self.request_id);

        // Encode data length
        let len = self.data.len() as u16;
        payload.push((len & 0x7F) as u8);
        payload.push(((len >> 7) & 0x7F) as u8);
        payload.extend_from_slice(&self.data);

        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        if message.payload.len() < 3 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 3,
                actual: message.payload.len(),
            });
        }

        let request_id = message.payload[0];
        let len = (message.payload[1] as usize) | ((message.payload[2] as usize) << 7);

        if message.payload.len() < 3 + len {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 3 + len,
                actual: message.payload.len(),
            });
        }

        let data = message.payload[3..3 + len].to_vec();

        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            request_id,
            data,
        })
    }
}

/// Property set data request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertySetData {
    /// Source MUID
    pub source: Muid,
    /// Destination MUID
    pub destination: Muid,
    /// Request ID
    pub request_id: u8,
    /// Property resource
    pub resource: String,
    /// Property data
    pub data: Vec<u8>,
}

impl PropertySetData {
    /// Creates a new property set data request.
    pub fn new(
        source: Muid,
        destination: Muid,
        request_id: u8,
        resource: String,
        data: Vec<u8>,
    ) -> Self {
        Self {
            source,
            destination,
            request_id,
            resource,
            data,
        }
    }

    /// Converts to a MIDI-CI message.
    pub fn to_message(&self) -> MidiCiMessage {
        let header = MessageHeader::new(
            0x7F,
            MessageType::PropertySetData,
            self.source,
            self.destination,
        );

        let mut payload = Vec::new();
        payload.push(self.request_id);

        // Encode resource
        let resource_bytes = self.resource.as_bytes();
        let resource_len = resource_bytes.len() as u16;
        payload.push((resource_len & 0x7F) as u8);
        payload.push(((resource_len >> 7) & 0x7F) as u8);
        payload.extend_from_slice(resource_bytes);

        // Encode data
        let data_len = self.data.len() as u16;
        payload.push((data_len & 0x7F) as u8);
        payload.push(((data_len >> 7) & 0x7F) as u8);
        payload.extend_from_slice(&self.data);

        MidiCiMessage::new(header, payload)
    }

    /// Parses from a MIDI-CI message.
    pub fn from_message(message: &MidiCiMessage) -> Result<Self> {
        if message.payload.len() < 3 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 3,
                actual: message.payload.len(),
            });
        }

        let request_id = message.payload[0];
        let resource_len = (message.payload[1] as usize) | ((message.payload[2] as usize) << 7);

        if message.payload.len() < 3 + resource_len + 2 {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: 3 + resource_len + 2,
                actual: message.payload.len(),
            });
        }

        let resource = String::from_utf8(message.payload[3..3 + resource_len].to_vec())
            .map_err(|e| crate::error::MidiCiError::InvalidPropertyData(e.to_string()))?;

        let data_offset = 3 + resource_len;
        let data_len = (message.payload[data_offset] as usize)
            | ((message.payload[data_offset + 1] as usize) << 7);

        if message.payload.len() < data_offset + 2 + data_len {
            return Err(crate::error::MidiCiError::MessageTooShort {
                expected: data_offset + 2 + data_len,
                actual: message.payload.len(),
            });
        }

        let data = message.payload[data_offset + 2..data_offset + 2 + data_len].to_vec();

        Ok(Self {
            source: message.header.source,
            destination: message.header.destination,
            request_id,
            resource,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_inquiry_roundtrip() {
        let inquiry = PropertyExchangeCapabilitiesInquiry::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
        );
        let message = inquiry.to_message();
        let parsed = PropertyExchangeCapabilitiesInquiry::from_message(&message).unwrap();
        assert_eq!(inquiry, parsed);
    }

    #[test]
    fn test_capabilities_reply_roundtrip() {
        let capabilities = PropertyExchangeCapabilities::new(8);
        let reply = PropertyExchangeCapabilitiesReply::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            capabilities,
        );
        let message = reply.to_message();
        let parsed = PropertyExchangeCapabilitiesReply::from_message(&message).unwrap();
        assert_eq!(reply, parsed);
    }

    #[test]
    fn test_property_get_data_roundtrip() {
        let get_data = PropertyGetData::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            42,
            "/device/name".to_string(),
        );
        let message = get_data.to_message();
        let parsed = PropertyGetData::from_message(&message).unwrap();
        assert_eq!(get_data, parsed);
    }

    #[test]
    fn test_property_get_data_reply_roundtrip() {
        let reply = PropertyGetDataReply::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            42,
            b"{\"name\":\"Test Device\"}".to_vec(),
        );
        let message = reply.to_message();
        let parsed = PropertyGetDataReply::from_message(&message).unwrap();
        assert_eq!(reply, parsed);
    }

    #[test]
    fn test_property_set_data_roundtrip() {
        let set_data = PropertySetData::new(
            Muid::new(0x1234567).unwrap(),
            Muid::new(0x7654321).unwrap(),
            42,
            "/device/volume".to_string(),
            b"{\"volume\":75}".to_vec(),
        );
        let message = set_data.to_message();
        let parsed = PropertySetData::from_message(&message).unwrap();
        assert_eq!(set_data, parsed);
    }
}
