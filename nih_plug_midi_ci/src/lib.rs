//! # nih_plug_midi_ci
//!
//! MIDI Capability Inquiry (MIDI-CI) support ported from JUCE for nih-plug.
//!
//! This crate provides pure Rust implementations of MIDI-CI protocol functionality,
//! which is part of the MIDI 2.0 specification. MIDI-CI enables devices to:
//!
//! - **Discover** other MIDI-CI capable devices on the network
//! - **Query capabilities** of connected devices
//! - **Negotiate profiles** to enable/disable specific MIDI features
//! - **Exchange properties** to get/set device configuration
//! - **Negotiate protocols** for MIDI communication
//!
//! ## Features
//!
//! - `discovery` - Device discovery functionality (enabled by default)
//! - `profiles` - Profile management (enabled by default)
//! - `properties` - Property exchange (enabled by default)
//! - `protocol` - Protocol negotiation
//! - `full` - Enable all features
//!
//! ## Examples
//!
//! ### Device Discovery
//!
//! ```rust
//! use nih_plug_midi_ci::{
//!     discovery::{DeviceCapabilities, DiscoveryInquiry, DiscoveryReply},
//!     protocol::{DeviceInfo, Muid},
//! };
//!
//! // Create a discovery inquiry
//! let my_muid = Muid::new(0x1234567).unwrap();
//! let device_info = DeviceInfo::new(
//!     vec![0x7D], // Manufacturer ID
//!     0x1234,     // Family
//!     0x5678,     // Model
//!     0x010000,   // Revision
//! );
//! let capabilities = DeviceCapabilities::all();
//!
//! let inquiry = DiscoveryInquiry::new(my_muid, device_info, capabilities);
//! let message = inquiry.to_message();
//! let sysex = message.to_sysex();
//!
//! // Send sysex over MIDI...
//! ```
//!
//! ### Profile Management
//!
//! ```rust
//! use nih_plug_midi_ci::{
//!     profiles::{ProfileInquiry, SetProfileOn},
//!     protocol::{Muid, ProfileId},
//! };
//!
//! // Query available profiles
//! let inquiry = ProfileInquiry::new(
//!     Muid::new(0x1234567).unwrap(),
//!     Muid::new(0x7654321).unwrap(),
//! );
//!
//! // Enable a profile
//! let profile_id = ProfileId::new([0x7E, 0x00, 0x01, 0x00, 0x00]);
//! let set_on = SetProfileOn::new(
//!     Muid::new(0x1234567).unwrap(),
//!     Muid::new(0x7654321).unwrap(),
//!     profile_id,
//! );
//! ```
//!
//! ### Property Exchange
//!
//! ```rust
//! use nih_plug_midi_ci::{
//!     properties::{PropertyGetData, PropertySetData},
//!     protocol::Muid,
//! };
//!
//! // Get a property
//! let get_data = PropertyGetData::new(
//!     Muid::new(0x1234567).unwrap(),
//!     Muid::new(0x7654321).unwrap(),
//!     1, // Request ID
//!     "/device/name".to_string(),
//! );
//!
//! // Set a property
//! let set_data = PropertySetData::new(
//!     Muid::new(0x1234567).unwrap(),
//!     Muid::new(0x7654321).unwrap(),
//!     2, // Request ID
//!     "/device/volume".to_string(),
//!     b"{\"volume\":75}".to_vec(),
//! );
//! ```
//!
//! ## Thread Safety
//!
//! All types in this crate are `Send` and `Sync` unless otherwise noted.
//!
//! ## Performance
//!
//! Message parsing and generation are designed to be efficient with minimal
//! allocations. Most operations complete in microseconds.

pub mod error;
pub mod protocol;

#[cfg(feature = "discovery")]
pub mod discovery;

#[cfg(feature = "profiles")]
pub mod profiles;

#[cfg(feature = "properties")]
pub mod properties;

// Re-export commonly used types
pub use error::{MidiCiError, Result};
pub use protocol::{
    DeviceInfo, MessageHeader, MessageType, MidiCiMessage, Muid, ProfileId, MIDI_CI_VERSION,
};

#[cfg(feature = "discovery")]
pub use discovery::{
    DeviceCapabilities, DiscoveryInquiry, DiscoveryReply, EndpointInfoInquiry, EndpointInfoReply,
};

#[cfg(feature = "profiles")]
pub use profiles::{ProfileInquiry, ProfileInquiryReply, SetProfileOff, SetProfileOn};

#[cfg(feature = "properties")]
pub use properties::{
    PropertyExchangeCapabilities, PropertyExchangeCapabilitiesInquiry,
    PropertyExchangeCapabilitiesReply, PropertyGetData, PropertyGetDataReply, PropertySetData,
};
