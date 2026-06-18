//! MIDI-CI message bodies.
//!
//! Every concrete message type from JUCE's `juce::midi_ci::Message` namespace
//! is represented here as its own struct. They're intentionally light-weight
//! (no `Arc`, no trait objects, only a few small `Vec<u8>`s for body data) so
//! they can be cloned cheaply and stored in caches.
//!
//! The [`Message`] enum is the unified top-level value: any incoming MIDI-CI
//! message can be decoded into one of these variants. Constructing a
//! [`Message`] and handing it to the [`Device`](crate::device::Device) is
//! the typical way to send outbound messages.
//!
//! See [`crate::codec`] for the wire-format encode/decode routines.

use std::fmt;

use crate::types::{
    CapabilityFlags, ChannelInGroup, DeviceInfo, Muid, Profile, ProtocolVersion,
    RequestId, SubscriptionCommand,
};

// =============================================================================
// Message header
// =============================================================================

/// The header common to every MIDI-CI message.
///
/// Wire-format layout (10 bytes total, big-endian-ish 7-bit):
/// - `device_id`    — UMP channel byte / `ChannelInGroup`
/// - `category`     — the wire status byte (`0x70 + category_index`)
/// - `version`      — protocol version byte
/// - `source`       — the sending device's MUID (28 bits, MSB padded)
/// - `destination`  — the target MUID (28 bits, MSB padded)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Header {
    /// The UMP channel / `ChannelInGroup` byte that addressed us.
    pub device_id: ChannelInGroup,
    /// The category sub-byte (matches the wire status byte minus 0x70).
    pub category: u8,
    /// The protocol version byte (raw).
    pub version: u8,
    /// The sending device's MUID.
    pub source: Muid,
    /// The destination MUID (or `Muid::BROADCAST`).
    pub destination: Muid,
}

impl Header {
    /// Construct a header with sensible defaults for outgoing messages.
    pub const fn new(source: Muid, destination: Muid, category: u8) -> Self {
        Header {
            device_id: ChannelInGroup::WholeBlock,
            category,
            version: ProtocolVersion::IMPLEMENTATION.to_byte(),
            source,
            destination,
        }
    }

    /// Returns the protocol version, falling back to V1 for unknown bytes.
    pub fn protocol_version(&self) -> ProtocolVersion {
        ProtocolVersion::from_byte(self.version).unwrap_or(ProtocolVersion::V1)
    }
}

// =============================================================================
// Discovery / InvalidateMUID / Endpoint Inquiry family
// =============================================================================

/// Discovery message.
///
/// The first discovery message is broadcast and triggers peers to respond
/// with their own `DiscoveryReply`.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Discovery {
    /// The 4 manufacturer / family / model / revision values.
    pub device_info: DeviceInfo,
    /// Which MIDI-CI categories this device supports.
    pub capabilities: CapabilityFlags,
    /// Maximum SysEx message size in bytes.
    pub maximum_sysex_size: u32,
    /// Output path id (only meaningful for V2+).
    pub output_path_id: u8,
}

/// Reply to a `Discovery` message.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct DiscoveryReply {
    /// The 4 manufacturer / family / model / revision values.
    pub device_info: DeviceInfo,
    /// Which MIDI-CI categories this device supports.
    pub capabilities: CapabilityFlags,
    /// Maximum SysEx message size in bytes.
    pub maximum_sysex_size: u32,
    /// Output path id (only meaningful for V2+).
    pub output_path_id: u8,
    /// Function block index this reply is for (only meaningful for V2+).
    pub function_block: u8,
}

/// Notifies recipients that a previously-assigned MUID is no longer in use.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct InvalidateMuid {
    /// The MUID that is no longer in use.
    pub target: Muid,
}

/// Endpoint inquiry — ask a peer to identify itself with its product
/// instance id.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct EndpointInquiry {
    /// The status byte — 0x00 for a normal inquiry.
    pub status: u8,
}

/// Reply to an `EndpointInquiry`.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct EndpointInquiryResponse {
    /// The status byte echoed back from the inquiry.
    pub status: u8,
    /// The data payload (typically a UTF-8 encoded product instance ID).
    pub data: Vec<u8>,
}

/// Generic acknowledgement.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Ack {
    /// The category byte that the ACK refers to.
    pub original_category: u8,
    /// Status code byte.
    pub status_code: u8,
    /// Additional status data byte.
    pub status_data: u8,
    /// Vendor-defined 5-byte details payload.
    pub details: [u8; 5],
    /// Free-form message text (encoded as 7-bit ASCII per the spec).
    pub message_text: Vec<u8>,
}

/// Generic non-acknowledgement.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct Nak {
    /// The category byte that the NAK refers to.
    pub original_category: u8,
    /// Status code byte.
    pub status_code: u8,
    /// Additional status data byte.
    pub status_data: u8,
    /// Vendor-defined 5-byte details payload.
    pub details: [u8; 5],
    /// Free-form message text (encoded as 7-bit ASCII per the spec).
    pub message_text: Vec<u8>,
}

// =============================================================================
// Profile Configuration
// =============================================================================

/// Asks a peer to enumerate the profiles it supports on a given address.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileInquiry;

/// Reply to a `ProfileInquiry`.
///
/// Profiles are split into "enabled" and "disabled" buckets, both
/// addressable per `ChannelAddress`.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileInquiryResponse {
    /// Profiles that are currently enabled on the addressed channel(s).
    pub enabled_profiles: Vec<Profile>,
    /// Profiles that are supported but disabled on the addressed channel(s).
    pub disabled_profiles: Vec<Profile>,
}

/// Notifies listeners that a profile has been added locally.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileAdded {
    /// The newly added profile.
    pub profile: Profile,
}

/// Notifies listeners that a profile has been removed locally.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileRemoved {
    /// The removed profile.
    pub profile: Profile,
}

/// Asks a peer for details about a profile on a given channel.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileDetails {
    /// The profile to query.
    pub profile: Profile,
    /// The detail target byte (0..=127).
    pub target: u8,
}

/// Reply to a `ProfileDetails`.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileDetailsResponse {
    /// The profile the reply is about.
    pub profile: Profile,
    /// The detail target byte (echoed).
    pub target: u8,
    /// Free-form data payload for the target.
    pub data: Vec<u8>,
}

/// Enables a profile on the addressed channels.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileOn {
    /// The profile to enable.
    pub profile: Profile,
    /// How many channels should activate this profile (only meaningful when
    /// the address is a single channel). Zero for group / block addresses.
    pub num_channels: u16,
}

/// Disables a profile on the addressed channels.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileOff {
    /// The profile to disable.
    pub profile: Profile,
}

/// Sent by a responder to confirm a profile was enabled.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileEnabledReport {
    /// The profile that was enabled.
    pub profile: Profile,
    /// The number of channels now using it.
    pub num_channels: u16,
}

/// Sent by a responder to confirm a profile was disabled.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileDisabledReport {
    /// The profile that was disabled.
    pub profile: Profile,
    /// The number of channels still using it (often zero).
    pub num_channels: u16,
}

/// Profile-specific data exchange (used by some profiles for their own
/// custom payload).
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProfileSpecificData {
    /// The profile the data is for.
    pub profile: Profile,
    /// The free-form data payload.
    pub data: Vec<u8>,
}

// =============================================================================
// Property Exchange
// =============================================================================

/// Property exchange capabilities inquiry.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertyExchangeCapabilities {
    /// Maximum number of simultaneous property-exchange requests supported.
    pub num_simultaneous_requests_supported: u8,
    /// Major version of the PE protocol implemented.
    pub major_version: u8,
    /// Minor version of the PE protocol implemented.
    pub minor_version: u8,
}

/// Reply to a `PropertyExchangeCapabilities`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertyExchangeCapabilitiesResponse {
    /// Maximum number of simultaneous property-exchange requests supported.
    pub num_simultaneous_requests_supported: u8,
    /// Major version of the PE protocol implemented.
    pub major_version: u8,
    /// Minor version of the PE protocol implemented.
    pub minor_version: u8,
}

/// A property-exchange message with a static (single-chunk) body.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct StaticSizePropertyExchange {
    /// The request id (7-bit).
    pub request_id: RequestId,
    /// The free-form header body (resource id, encoding, etc.).
    pub header: Vec<u8>,
}

/// A property-exchange message that may form part of a multi-chunk stream.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct DynamicSizePropertyExchange {
    /// The request id (7-bit).
    pub request_id: RequestId,
    /// The free-form header body.
    pub header: Vec<u8>,
    /// Total number of chunks this transaction will span.
    pub total_num_chunks: u16,
    /// Which chunk this message is (1-based).
    pub this_chunk_num: u16,
    /// This chunk's data payload.
    pub data: Vec<u8>,
}

/// `GetPropertyData` — ask a responder to return a property's current value.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertyGetData {
    /// The static-size portion of the message.
    pub inner: StaticSizePropertyExchange,
}

/// Reply to a `PropertyGetData` (may be multi-chunk).
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertyGetDataResponse {
    /// The dynamic-size portion of the message.
    pub inner: DynamicSizePropertyExchange,
}

/// `SetPropertyData` — ask a responder to apply a new property value.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertySetData {
    /// The dynamic-size portion of the message.
    pub inner: DynamicSizePropertyExchange,
}

/// Reply to a `PropertySetData`.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertySetDataResponse {
    /// The static-size portion of the message.
    pub inner: StaticSizePropertyExchange,
}

/// `PropertySubscribe` — start / continue / end a subscription.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertySubscribe {
    /// The kind of update being sent.
    pub command: SubscriptionCommand,
    /// The dynamic-size portion of the message.
    pub inner: DynamicSizePropertyExchange,
}

/// Reply to a `PropertySubscribe`.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertySubscribeResponse {
    /// The dynamic-size portion of the message.
    pub inner: DynamicSizePropertyExchange,
}

/// `PropertyNotify` — server-initiated update for a subscribed property.
#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct PropertyNotify {
    /// The dynamic-size portion of the message.
    pub inner: DynamicSizePropertyExchange,
}

// =============================================================================
// Process Inquiry
// =============================================================================

/// Asks the responder whether it supports Process Inquiry features.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProcessInquiry {
    /// Bit-set of supported process inquiry features.
    pub supported_features: u8,
}

/// Reply to a `ProcessInquiry`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProcessInquiryResponse {
    /// Bit-set of supported process inquiry features.
    pub supported_features: u8,
}

/// Asks the responder for a MIDI Message Report.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProcessMidiMessageReport {
    /// Message data control byte.
    pub message_data_control: u8,
    /// Bitmap of requested MIDI message types.
    pub requested_messages: u8,
    /// Bitmap of requested channel-controller messages.
    pub channel_controller_messages: u8,
    /// Bitmap of requested note-data messages.
    pub note_data_messages: u8,
}

/// Reply to a `ProcessMidiMessageReport`.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProcessMidiMessageReportResponse {
    /// Message data control byte.
    pub message_data_control: u8,
    /// Bitmap of requested MIDI message types.
    pub requested_messages: u8,
    /// Bitmap of requested channel-controller messages.
    pub channel_controller_messages: u8,
    /// Bitmap of requested note-data messages.
    pub note_data_messages: u8,
}

/// Marks the end of a MIDI Message Report reply.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ProcessEndMidiMessageReport;

// =============================================================================
// Top-level Message enum
// =============================================================================

/// A fully-decoded MIDI-CI message (header + body).
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ParsedMessage {
    /// The shared header.
    pub header: Header,
    /// The body variant.
    pub body: MessageBody,
}

/// Every MIDI-CI body variant, kept as one enum so that
/// [`Device::process_message`](crate::device::Device::process_message) can
/// dispatch on it without trait objects.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum MessageBody {
    /// Discovery.
    Discovery(Discovery),
    /// Discovery reply.
    DiscoveryReply(DiscoveryReply),
    /// Invalidate MUID.
    InvalidateMuid(InvalidateMuid),
    /// Endpoint inquiry.
    EndpointInquiry(EndpointInquiry),
    /// Endpoint inquiry response.
    EndpointInquiryResponse(EndpointInquiryResponse),
    /// ACK.
    Ack(Ack),
    /// NAK.
    Nak(Nak),
    /// Profile inquiry.
    ProfileInquiry(ProfileInquiry),
    /// Profile inquiry response.
    ProfileInquiryResponse(ProfileInquiryResponse),
    /// Profile added.
    ProfileAdded(ProfileAdded),
    /// Profile removed.
    ProfileRemoved(ProfileRemoved),
    /// Profile details.
    ProfileDetails(ProfileDetails),
    /// Profile details response.
    ProfileDetailsResponse(ProfileDetailsResponse),
    /// Profile on.
    ProfileOn(ProfileOn),
    /// Profile off.
    ProfileOff(ProfileOff),
    /// Profile enabled report.
    ProfileEnabledReport(ProfileEnabledReport),
    /// Profile disabled report.
    ProfileDisabledReport(ProfileDisabledReport),
    /// Profile-specific data.
    ProfileSpecificData(ProfileSpecificData),
    /// Property exchange capabilities.
    PropertyExchangeCapabilities(PropertyExchangeCapabilities),
    /// Property exchange capabilities response.
    PropertyExchangeCapabilitiesResponse(PropertyExchangeCapabilitiesResponse),
    /// Property get data.
    PropertyGetData(PropertyGetData),
    /// Property get data response.
    PropertyGetDataResponse(PropertyGetDataResponse),
    /// Property set data.
    PropertySetData(PropertySetData),
    /// Property set data response.
    PropertySetDataResponse(PropertySetDataResponse),
    /// Property subscribe.
    PropertySubscribe(PropertySubscribe),
    /// Property subscribe response.
    PropertySubscribeResponse(PropertySubscribeResponse),
    /// Property notify.
    PropertyNotify(PropertyNotify),
    /// Process inquiry.
    ProcessInquiry(ProcessInquiry),
    /// Process inquiry response.
    ProcessInquiryResponse(ProcessInquiryResponse),
    /// Process MIDI message report.
    ProcessMidiMessageReport(ProcessMidiMessageReport),
    /// Process MIDI message report response.
    ProcessMidiMessageReportResponse(ProcessMidiMessageReportResponse),
    /// Process end of MIDI message report.
    ProcessEndMidiMessageReport(ProcessEndMidiMessageReport),
    /// A body that we recognised the category of but failed to decode
    /// further (malformed payload).
    Malformed(Header),
}

impl MessageBody {
    /// A short human-readable name for this body type, used in error messages
    /// and listener notifications.
    pub fn type_name(&self) -> &'static str {
        match self {
            MessageBody::Discovery(_) => "Discovery",
            MessageBody::DiscoveryReply(_) => "DiscoveryReply",
            MessageBody::InvalidateMuid(_) => "InvalidateMUID",
            MessageBody::EndpointInquiry(_) => "EndpointInquiry",
            MessageBody::EndpointInquiryResponse(_) => "EndpointInquiryResponse",
            MessageBody::Ack(_) => "ACK",
            MessageBody::Nak(_) => "NAK",
            MessageBody::ProfileInquiry(_) => "ProfileInquiry",
            MessageBody::ProfileInquiryResponse(_) => "ProfileInquiryResponse",
            MessageBody::ProfileAdded(_) => "ProfileAdded",
            MessageBody::ProfileRemoved(_) => "ProfileRemoved",
            MessageBody::ProfileDetails(_) => "ProfileDetails",
            MessageBody::ProfileDetailsResponse(_) => "ProfileDetailsResponse",
            MessageBody::ProfileOn(_) => "ProfileOn",
            MessageBody::ProfileOff(_) => "ProfileOff",
            MessageBody::ProfileEnabledReport(_) => "ProfileEnabledReport",
            MessageBody::ProfileDisabledReport(_) => "ProfileDisabledReport",
            MessageBody::ProfileSpecificData(_) => "ProfileSpecificData",
            MessageBody::PropertyExchangeCapabilities(_) => "PropertyExchangeCapabilities",
            MessageBody::PropertyExchangeCapabilitiesResponse(_) => {
                "PropertyExchangeCapabilitiesResponse"
            }
            MessageBody::PropertyGetData(_) => "PropertyGetData",
            MessageBody::PropertyGetDataResponse(_) => "PropertyGetDataResponse",
            MessageBody::PropertySetData(_) => "PropertySetData",
            MessageBody::PropertySetDataResponse(_) => "PropertySetDataResponse",
            MessageBody::PropertySubscribe(_) => "PropertySubscribe",
            MessageBody::PropertySubscribeResponse(_) => "PropertySubscribeResponse",
            MessageBody::PropertyNotify(_) => "PropertyNotify",
            MessageBody::ProcessInquiry(_) => "ProcessInquiry",
            MessageBody::ProcessInquiryResponse(_) => "ProcessInquiryResponse",
            MessageBody::ProcessMidiMessageReport(_) => "ProcessMidiMessageReport",
            MessageBody::ProcessMidiMessageReportResponse(_) => "ProcessMidiMessageReportResponse",
            MessageBody::ProcessEndMidiMessageReport(_) => "ProcessEndMidiMessageReport",
            MessageBody::Malformed(_) => "Malformed",
        }
    }
}

impl ParsedMessage {
    /// Wrap a header + body in the unified message struct.
    pub fn new(header: Header, body: MessageBody) -> Self {
        ParsedMessage { header, body }
    }

    /// Convenience: the source MUID of this message.
    pub fn source(&self) -> Muid {
        self.header.source
    }

    /// Convenience: the destination MUID of this message.
    pub fn destination(&self) -> Muid {
        self.header.destination
    }
}

impl fmt::Display for ParsedMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:02x}] {} from {} to {}",
            self.header.category,
            self.body.type_name(),
            self.header.source,
            self.header.destination
        )
    }
}

// =============================================================================
// Outbound message envelope
// =============================================================================

/// Identifies which MIDI-CI body type an outbound message carries.
///
/// This is what [`Device::send`](crate::device::Device::send) consumes. The
/// builder pattern in the [`device`](crate::device) module helps you build
/// these without having to set every field on a struct.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum OutboundMessage {
    /// Discovery.
    Discovery(Discovery),
    /// Discovery reply.
    DiscoveryReply(DiscoveryReply),
    /// Invalidate MUID.
    InvalidateMuid(InvalidateMuid),
    /// Endpoint inquiry.
    EndpointInquiry(EndpointInquiry),
    /// Endpoint reply.
    EndpointInquiryResponse(EndpointInquiryResponse),
    /// ACK.
    Ack(Ack),
    /// NAK.
    Nak(Nak),
    /// Profile inquiry.
    ProfileInquiry(ProfileInquiry),
    /// Profile inquiry response.
    ProfileInquiryResponse(ProfileInquiryResponse),
    /// Profile added.
    ProfileAdded(ProfileAdded),
    /// Profile removed.
    ProfileRemoved(ProfileRemoved),
    /// Profile details.
    ProfileDetails(ProfileDetails),
    /// Profile details response.
    ProfileDetailsResponse(ProfileDetailsResponse),
    /// Profile on.
    ProfileOn(ProfileOn),
    /// Profile off.
    ProfileOff(ProfileOff),
    /// Profile enabled report.
    ProfileEnabledReport(ProfileEnabledReport),
    /// Profile disabled report.
    ProfileDisabledReport(ProfileDisabledReport),
    /// Profile specific data.
    ProfileSpecificData(ProfileSpecificData),
    /// Property exchange capabilities.
    PropertyExchangeCapabilities(PropertyExchangeCapabilities),
    /// Property exchange capabilities response.
    PropertyExchangeCapabilitiesResponse(PropertyExchangeCapabilitiesResponse),
    /// Property get data.
    PropertyGetData(PropertyGetData),
    /// Property get data response.
    PropertyGetDataResponse(PropertyGetDataResponse),
    /// Property set data.
    PropertySetData(PropertySetData),
    /// Property set data response.
    PropertySetDataResponse(PropertySetDataResponse),
    /// Property subscribe.
    PropertySubscribe(PropertySubscribe),
    /// Property subscribe response.
    PropertySubscribeResponse(PropertySubscribeResponse),
    /// Property notify.
    PropertyNotify(PropertyNotify),
    /// Process inquiry.
    ProcessInquiry(ProcessInquiry),
    /// Process inquiry response.
    ProcessInquiryResponse(ProcessInquiryResponse),
    /// Process MIDI message report.
    ProcessMidiMessageReport(ProcessMidiMessageReport),
    /// Process MIDI message report response.
    ProcessMidiMessageReportResponse(ProcessMidiMessageReportResponse),
    /// Process end of MIDI message report.
    ProcessEndMidiMessageReport(ProcessEndMidiMessageReport),
}

impl OutboundMessage {
    /// Returns the `Category` byte for this message type.
    pub const fn category(&self) -> u8 {
        use crate::types::Category as C;
        match self {
            OutboundMessage::Discovery(_) => C::Discovery as u8,
            OutboundMessage::DiscoveryReply(_) => C::DiscoveryReply as u8,
            OutboundMessage::InvalidateMuid(_) => C::InvalidateMuid as u8,
            OutboundMessage::EndpointInquiry(_) => C::EndpointInquiry as u8,
            OutboundMessage::EndpointInquiryResponse(_) => C::EndpointInquiryResponse as u8,
            OutboundMessage::Ack(_) => C::Ack as u8,
            OutboundMessage::Nak(_) => C::Nak as u8,
            OutboundMessage::ProfileInquiry(_) => C::ProfileInquiry as u8,
            OutboundMessage::ProfileInquiryResponse(_) => C::ProfileInquiryResponse as u8,
            OutboundMessage::ProfileAdded(_) => C::ProfileAdded as u8,
            OutboundMessage::ProfileRemoved(_) => C::ProfileRemoved as u8,
            OutboundMessage::ProfileDetails(_) => C::ProfileDetails as u8,
            OutboundMessage::ProfileDetailsResponse(_) => C::ProfileDetailsResponse as u8,
            OutboundMessage::ProfileOn(_) => C::ProfileOn as u8,
            OutboundMessage::ProfileOff(_) => C::ProfileOff as u8,
            OutboundMessage::ProfileEnabledReport(_) => C::ProfileEnabledReport as u8,
            OutboundMessage::ProfileDisabledReport(_) => C::ProfileDisabledReport as u8,
            OutboundMessage::ProfileSpecificData(_) => C::ProfileSpecificData as u8,
            OutboundMessage::PropertyExchangeCapabilities(_) => {
                C::PropertyExchangeCapabilities as u8
            }
            OutboundMessage::PropertyExchangeCapabilitiesResponse(_) => {
                C::PropertyExchangeCapabilitiesResponse as u8
            }
            OutboundMessage::PropertyGetData(_) => C::PropertyGetData as u8,
            OutboundMessage::PropertyGetDataResponse(_) => C::PropertyGetDataResponse as u8,
            OutboundMessage::PropertySetData(_) => C::PropertySetData as u8,
            OutboundMessage::PropertySetDataResponse(_) => C::PropertySetDataResponse as u8,
            OutboundMessage::PropertySubscribe(_) => C::PropertySubscribe as u8,
            OutboundMessage::PropertySubscribeResponse(_) => C::PropertySubscribeResponse as u8,
            OutboundMessage::PropertyNotify(_) => C::PropertyNotify as u8,
            OutboundMessage::ProcessInquiry(_) => C::ProcessInquiry as u8,
            OutboundMessage::ProcessInquiryResponse(_) => C::ProcessInquiryResponse as u8,
            OutboundMessage::ProcessMidiMessageReport(_) => C::ProcessMidiMessageReport as u8,
            OutboundMessage::ProcessMidiMessageReportResponse(_) => {
                C::ProcessMidiMessageReportResponse as u8
            }
            OutboundMessage::ProcessEndMidiMessageReport(_) => {
                C::ProcessEndMidiMessageReport as u8
            }
        }
    }

    /// Returns the canonical `Category` for this message type (used by
    /// listeners and the device state).
    pub const fn category_enum(&self) -> crate::types::Category {
        use crate::types::Category as C;
        match self {
            OutboundMessage::Discovery(_) => C::Discovery,
            OutboundMessage::DiscoveryReply(_) => C::DiscoveryReply,
            OutboundMessage::InvalidateMuid(_) => C::InvalidateMuid,
            OutboundMessage::EndpointInquiry(_) => C::EndpointInquiry,
            OutboundMessage::EndpointInquiryResponse(_) => C::EndpointInquiryResponse,
            OutboundMessage::Ack(_) => C::Ack,
            OutboundMessage::Nak(_) => C::Nak,
            OutboundMessage::ProfileInquiry(_) => C::ProfileInquiry,
            OutboundMessage::ProfileInquiryResponse(_) => C::ProfileInquiryResponse,
            OutboundMessage::ProfileAdded(_) => C::ProfileAdded,
            OutboundMessage::ProfileRemoved(_) => C::ProfileRemoved,
            OutboundMessage::ProfileDetails(_) => C::ProfileDetails,
            OutboundMessage::ProfileDetailsResponse(_) => C::ProfileDetailsResponse,
            OutboundMessage::ProfileOn(_) => C::ProfileOn,
            OutboundMessage::ProfileOff(_) => C::ProfileOff,
            OutboundMessage::ProfileEnabledReport(_) => C::ProfileEnabledReport,
            OutboundMessage::ProfileDisabledReport(_) => C::ProfileDisabledReport,
            OutboundMessage::ProfileSpecificData(_) => C::ProfileSpecificData,
            OutboundMessage::PropertyExchangeCapabilities(_) => {
                C::PropertyExchangeCapabilities
            }
            OutboundMessage::PropertyExchangeCapabilitiesResponse(_) => {
                C::PropertyExchangeCapabilitiesResponse
            }
            OutboundMessage::PropertyGetData(_) => C::PropertyGetData,
            OutboundMessage::PropertyGetDataResponse(_) => C::PropertyGetDataResponse,
            OutboundMessage::PropertySetData(_) => C::PropertySetData,
            OutboundMessage::PropertySetDataResponse(_) => C::PropertySetDataResponse,
            OutboundMessage::PropertySubscribe(_) => C::PropertySubscribe,
            OutboundMessage::PropertySubscribeResponse(_) => C::PropertySubscribeResponse,
            OutboundMessage::PropertyNotify(_) => C::PropertyNotify,
            OutboundMessage::ProcessInquiry(_) => C::ProcessInquiry,
            OutboundMessage::ProcessInquiryResponse(_) => C::ProcessInquiryResponse,
            OutboundMessage::ProcessMidiMessageReport(_) => C::ProcessMidiMessageReport,
            OutboundMessage::ProcessMidiMessageReportResponse(_) => {
                C::ProcessMidiMessageReportResponse
            }
            OutboundMessage::ProcessEndMidiMessageReport(_) => C::ProcessEndMidiMessageReport,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_new_uses_implementation_version() {
        let source = Muid::from_bits_truncate(0x0102_0304);
        let dest = Muid::from_bits_truncate(0x7FFF_FFFF);
        let header = Header::new(source, dest, 0x07);
        assert_eq!(header.source, source);
        assert_eq!(header.destination, dest);
        assert_eq!(header.category, 0x07);
        assert_eq!(header.version, ProtocolVersion::IMPLEMENTATION.to_byte());
    }

    #[test]
    fn outbound_message_categories_are_in_range() {
        let messages = vec![
            OutboundMessage::Discovery(Discovery::default()),
            OutboundMessage::DiscoveryReply(DiscoveryReply::default()),
            OutboundMessage::InvalidateMuid(InvalidateMuid::default()),
            OutboundMessage::EndpointInquiry(EndpointInquiry::default()),
            OutboundMessage::EndpointInquiryResponse(EndpointInquiryResponse::default()),
            OutboundMessage::Ack(Ack::default()),
            OutboundMessage::Nak(Nak::default()),
            OutboundMessage::ProfileInquiry(ProfileInquiry),
            OutboundMessage::ProfileInquiryResponse(ProfileInquiryResponse::default()),
            OutboundMessage::ProfileAdded(ProfileAdded::default()),
            OutboundMessage::ProfileRemoved(ProfileRemoved::default()),
            OutboundMessage::ProfileDetails(ProfileDetails::default()),
            OutboundMessage::ProfileDetailsResponse(ProfileDetailsResponse::default()),
            OutboundMessage::ProfileOn(ProfileOn::default()),
            OutboundMessage::ProfileOff(ProfileOff::default()),
            OutboundMessage::ProfileEnabledReport(ProfileEnabledReport::default()),
            OutboundMessage::ProfileDisabledReport(ProfileDisabledReport::default()),
            OutboundMessage::ProfileSpecificData(ProfileSpecificData::default()),
            OutboundMessage::PropertyExchangeCapabilities(PropertyExchangeCapabilities::default()),
            OutboundMessage::PropertyExchangeCapabilitiesResponse(
                PropertyExchangeCapabilitiesResponse::default(),
            ),
            OutboundMessage::PropertyGetData(PropertyGetData::default()),
            OutboundMessage::PropertyGetDataResponse(PropertyGetDataResponse::default()),
            OutboundMessage::PropertySetData(PropertySetData::default()),
            OutboundMessage::PropertySetDataResponse(PropertySetDataResponse::default()),
            OutboundMessage::PropertySubscribe(PropertySubscribe::default()),
            OutboundMessage::PropertySubscribeResponse(PropertySubscribeResponse::default()),
            OutboundMessage::PropertyNotify(PropertyNotify::default()),
            OutboundMessage::ProcessInquiry(ProcessInquiry::default()),
            OutboundMessage::ProcessInquiryResponse(ProcessInquiryResponse::default()),
            OutboundMessage::ProcessMidiMessageReport(ProcessMidiMessageReport::default()),
            OutboundMessage::ProcessMidiMessageReportResponse(
                ProcessMidiMessageReportResponse::default(),
            ),
            OutboundMessage::ProcessEndMidiMessageReport(ProcessEndMidiMessageReport),
        ];
        assert_eq!(messages.len(), 32);
        for (i, msg) in messages.iter().enumerate() {
            assert_eq!(
                msg.category() as usize,
                i,
                "category mismatch for {:?}",
                msg
            );
            assert_eq!(
                msg.category_enum() as u8 as usize,
                i,
                "category_enum mismatch for {:?}",
                msg
            );
        }
    }

    #[test]
    fn message_body_type_names_are_unique() {
        let bodies = vec![
            MessageBody::Discovery(Discovery::default()),
            MessageBody::DiscoveryReply(DiscoveryReply::default()),
            MessageBody::InvalidateMuid(InvalidateMuid::default()),
            MessageBody::EndpointInquiry(EndpointInquiry::default()),
            MessageBody::EndpointInquiryResponse(EndpointInquiryResponse::default()),
            MessageBody::Ack(Ack::default()),
            MessageBody::Nak(Nak::default()),
            MessageBody::ProfileInquiry(ProfileInquiry),
            MessageBody::ProfileInquiryResponse(ProfileInquiryResponse::default()),
            MessageBody::ProfileAdded(ProfileAdded::default()),
            MessageBody::ProfileRemoved(ProfileRemoved::default()),
            MessageBody::ProfileDetails(ProfileDetails::default()),
            MessageBody::ProfileDetailsResponse(ProfileDetailsResponse::default()),
            MessageBody::ProfileOn(ProfileOn::default()),
            MessageBody::ProfileOff(ProfileOff::default()),
            MessageBody::ProfileEnabledReport(ProfileEnabledReport::default()),
            MessageBody::ProfileDisabledReport(ProfileDisabledReport::default()),
            MessageBody::ProfileSpecificData(ProfileSpecificData::default()),
            MessageBody::PropertyExchangeCapabilities(PropertyExchangeCapabilities::default()),
            MessageBody::PropertyExchangeCapabilitiesResponse(
                PropertyExchangeCapabilitiesResponse::default(),
            ),
            MessageBody::PropertyGetData(PropertyGetData::default()),
            MessageBody::PropertyGetDataResponse(PropertyGetDataResponse::default()),
            MessageBody::PropertySetData(PropertySetData::default()),
            MessageBody::PropertySetDataResponse(PropertySetDataResponse::default()),
            MessageBody::PropertySubscribe(PropertySubscribe::default()),
            MessageBody::PropertySubscribeResponse(PropertySubscribeResponse::default()),
            MessageBody::PropertyNotify(PropertyNotify::default()),
            MessageBody::ProcessInquiry(ProcessInquiry::default()),
            MessageBody::ProcessInquiryResponse(ProcessInquiryResponse::default()),
            MessageBody::ProcessMidiMessageReport(ProcessMidiMessageReport::default()),
            MessageBody::ProcessMidiMessageReportResponse(
                ProcessMidiMessageReportResponse::default(),
            ),
            MessageBody::ProcessEndMidiMessageReport(ProcessEndMidiMessageReport),
        ];
        let mut seen = std::collections::HashSet::new();
        for body in &bodies {
            let name = body.type_name();
            assert!(seen.insert(name), "duplicate name {:?}", name);
        }
        assert_eq!(seen.len(), 32);
    }

    #[test]
    fn discovery_structures_round_trip() {
        let reply = DiscoveryReply {
            device_info: DeviceInfo::example(),
            capabilities: CapabilityFlags::PROFILE_CONFIGURATION
                | CapabilityFlags::PROPERTY_EXCHANGE,
            maximum_sysex_size: 0x1234_5678,
            output_path_id: 0x42,
            function_block: 0x01,
        };

        let cloned = reply.clone();
        assert_eq!(cloned, reply);
    }

    #[test]
    fn profile_enablement_default_has_zero_channels() {
        let on = ProfileOn::default();
        assert_eq!(on.profile, Profile::default());
        assert_eq!(on.num_channels, 0);
        let off = ProfileOff::default();
        assert_eq!(off.profile, Profile::default());
    }

    #[test]
    fn property_exchange_subscriptions_have_a_command() {
        let sub = PropertySubscribe {
            command: SubscriptionCommand::Start,
            inner: DynamicSizePropertyExchange::default(),
        };
        assert_eq!(sub.command, SubscriptionCommand::Start);
    }

    #[test]
    fn parsed_message_display_includes_type_and_muids() {
        let msg = ParsedMessage::new(
            Header {
                device_id: ChannelInGroup::WholeBlock,
                category: 0x07,
                version: 2,
                source: Muid::from_bits_truncate(0x01),
                destination: Muid::BROADCAST,
            },
            MessageBody::ProfileInquiry(ProfileInquiry),
        );
        let rendered = format!("{}", msg);
        assert!(rendered.contains("ProfileInquiry"));
        assert!(rendered.contains("broadcast"));
    }
}

