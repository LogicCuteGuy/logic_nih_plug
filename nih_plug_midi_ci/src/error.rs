//! Error types for MIDI-CI operations.

use thiserror::Error;

/// Errors that can occur during MIDI-CI operations.
#[derive(Debug, Error)]
pub enum MidiCiError {
    /// Invalid MIDI-CI message format
    #[error("Invalid MIDI-CI message format: {0}")]
    InvalidMessageFormat(String),

    /// Invalid MUID (MIDI Unique Identifier)
    #[error("Invalid MUID: {0}")]
    InvalidMuid(u32),

    /// Invalid device ID
    #[error("Invalid device ID: {0}")]
    InvalidDeviceId(u8),

    /// Invalid profile ID
    #[error("Invalid profile ID")]
    InvalidProfileId,

    /// Invalid property data
    #[error("Invalid property data: {0}")]
    InvalidPropertyData(String),

    /// Unsupported MIDI-CI version
    #[error("Unsupported MIDI-CI version: {0}")]
    UnsupportedVersion(u8),

    /// Message too short
    #[error("Message too short: expected at least {expected} bytes, got {actual}")]
    MessageTooShort { expected: usize, actual: usize },

    /// Message too long
    #[error("Message too long: maximum {max} bytes, got {actual}")]
    MessageTooLong { max: usize, actual: usize },

    /// Invalid message type
    #[error("Invalid message type: {0:#x}")]
    InvalidMessageType(u8),

    /// Protocol negotiation failed
    #[error("Protocol negotiation failed: {0}")]
    ProtocolNegotiationFailed(String),

    /// Property exchange failed
    #[error("Property exchange failed: {0}")]
    PropertyExchangeFailed(String),

    /// Device not found
    #[error("Device not found with MUID: {0}")]
    DeviceNotFound(u32),

    /// Timeout waiting for response
    #[error("Timeout waiting for response")]
    Timeout,
}

/// Result type for MIDI-CI operations.
pub type Result<T> = std::result::Result<T, MidiCiError>;
