//! Core MIDI-CI domain types: identifiers, addresses, profiles, and device
//! info.
//!
//! These mirror the equivalent JUCE `juce::midi_ci` types. They are intentionally
//! `Copy + Clone + Eq` and small enough to be embedded in vectors without
//! heap-allocating.

use std::fmt;

use crate::error::{MidiCiError, MidiCiResult};

// =============================================================================
// MUID — a 28-bit identifier unique to a MIDI-CI participant.
// =============================================================================

/// A 28-bit MUID (MIDI-CI Unique Identifier).
///
/// The MUID is regenerated if a collision is detected during discovery. The
/// range `0x0FFFFE00..=0x0FFFFFFE` is reserved; `0x0FFFFFFF` is the broadcast
/// address.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
#[repr(transparent)]
pub struct Muid(u32);

impl Muid {
    /// The maximum value of a valid MUID (`0x0FFFFFFF`). The high nibble of
    /// the 32-bit storage is always `0`.
    pub const MASK: u32 = 0x0FFFFFFF;

    /// The reserved high range — these MUIDs cannot be assigned to a device.
    pub const RESERVED_START: u32 = 0x0FFFFE00;

    /// The broadcast MUID (used as a destination when a message should be
    /// received by every device on the bus).
    pub const BROADCAST: Muid = Muid(Self::MASK);

    /// The lower bound of user-assignable MUIDs.
    pub const USER_START: u32 = 1;

    /// The upper bound of user-assignable MUIDs (just below the reserved
    /// range).
    pub const USER_END: u32 = Self::RESERVED_START - 1;

    /// Construct a MUID from a raw 32-bit value. Returns an error if any of
    /// the high four bits are set.
    pub const fn new(value: u32) -> MidiCiResult<Self> {
        if (value & !Self::MASK) != 0 {
            Err(MidiCiError::Other("MUID must fit in 28 bits"))
        } else {
            Ok(Muid(value))
        }
    }

    /// Construct a MUID without any validation. Caller is responsible for
    /// ensuring the value fits in 28 bits.
    pub const fn from_bits_truncate(value: u32) -> Self {
        Muid(value & Self::MASK)
    }

    /// Return the raw 28-bit value.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns `true` for the special broadcast MUID.
    pub const fn is_broadcast(self) -> bool {
        self.0 == Self::MASK
    }

    /// Returns `true` if the MUID is in the reserved range and therefore
    /// cannot be assigned to a device.
    pub const fn is_reserved(self) -> bool {
        self.0 >= Self::RESERVED_START && !self.is_broadcast()
    }

    /// Generate a pseudo-random MUID.
    ///
    /// The JUCE implementation uses `Random::nextInt`. We use a thread-local
    /// xorshift64* state seeded from `std::collections::hash_map::RandomState`,
    /// which keeps the crate dependency-free and avoids forcing callers to
    /// thread a `SmallRng` through their code.
    pub fn random() -> Self {
        use std::cell::Cell;
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};

        thread_local! {
            static SEED: Cell<u64> = Cell::new({
                let h1 = RandomState::new().build_hasher().finish();
                let h2 = RandomState::new().build_hasher().finish();
                h1.wrapping_add(h2.rotate_left(17))
            });
        }

        SEED.with(|s| {
            // xorshift64* — fast, good enough for MUID uniqueness.
            let mut x = s.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            s.set(x);
            let raw = (x as u32) & Self::MASK;
            let value = if raw == 0 {
                Self::USER_START
            } else if raw >= Self::RESERVED_START {
                // Fold reserved-range values back into the user space.
                raw & 0x00FF_FFFF
            } else {
                raw
            };
            Muid(value)
        })
    }
}

impl fmt::Debug for Muid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_broadcast() {
            write!(f, "Muid(broadcast)")
        } else if self.is_reserved() {
            write!(f, "Muid(reserved:0x{:07x})", self.0)
        } else {
            write!(f, "Muid(0x{:07x})", self.0)
        }
    }
}

impl fmt::Display for Muid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// =============================================================================
// ChannelInGroup — the per-channel / whole-group / whole-block address byte.
// =============================================================================

/// The address byte used inside a MIDI-CI message body.
///
/// Values `0x0..=0xF` identify a single MIDI channel inside the addressed
/// group. `0x7E` means "all channels in this UMP group" and `0x7F` means
/// "all channels in the function block that contains this UMP group".
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
#[repr(u8)]
pub enum ChannelInGroup {
    /// Channel 0 (1-based in user-facing text; 0-based in storage).
    Channel0 = 0x0,
    /// Channel 1.
    Channel1 = 0x1,
    /// Channel 2.
    Channel2 = 0x2,
    /// Channel 3.
    Channel3 = 0x3,
    /// Channel 4.
    Channel4 = 0x4,
    /// Channel 5.
    Channel5 = 0x5,
    /// Channel 6.
    Channel6 = 0x6,
    /// Channel 7.
    Channel7 = 0x7,
    /// Channel 8.
    Channel8 = 0x8,
    /// Channel 9.
    Channel9 = 0x9,
    /// Channel 10.
    ChannelA = 0xA,
    /// Channel 11.
    ChannelB = 0xB,
    /// Channel 12.
    ChannelC = 0xC,
    /// Channel 13.
    ChannelD = 0xD,
    /// Channel 14.
    ChannelE = 0xE,
    /// Channel 15.
    ChannelF = 0xF,
    /// All channels in the UMP group.
    WholeGroup = 0x7E,
    /// All channels in the function block that contains the UMP group.
    WholeBlock = 0x7F,
}

impl ChannelInGroup {
    /// Convert a raw byte into a `ChannelInGroup`, returning `None` if the
    /// byte is not a valid value.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x0..=0xF => Some(unsafe {
                // Safety: 0..=0xF all match the discriminant values above.
                std::mem::transmute::<u8, ChannelInGroup>(byte)
            }),
            0x7E => Some(ChannelInGroup::WholeGroup),
            0x7F => Some(ChannelInGroup::WholeBlock),
            _ => None,
        }
    }

    /// Return the raw byte representation.
    pub const fn to_byte(self) -> u8 {
        self as u8
    }

    /// `true` if this address refers to every channel in a UMP group.
    pub const fn is_group(self) -> bool {
        matches!(self, ChannelInGroup::WholeGroup)
    }

    /// `true` if this address refers to every channel in a function block.
    pub const fn is_block(self) -> bool {
        matches!(self, ChannelInGroup::WholeBlock)
    }

    /// `true` if this address refers to a single channel (i.e. not group or
    /// block).
    pub const fn is_single_channel(self) -> bool {
        !self.is_group() && !self.is_block()
    }

    /// Returns the channel number (0-15) when this address refers to a single
    /// channel, otherwise `None`.
    pub const fn single_channel(self) -> Option<u8> {
        if self.is_single_channel() {
            Some(self.to_byte())
        } else {
            None
        }
    }
}

impl fmt::Display for ChannelInGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelInGroup::WholeGroup => f.write_str("Group"),
            ChannelInGroup::WholeBlock => f.write_str("FunctionBlock"),
            other => write!(f, "Channel{}", other.to_byte() + 1),
        }
    }
}

// =============================================================================
// ChannelAddress — a (group, channel-in-group) pair.
// =============================================================================

/// Identifies a channel or set of channels in a multi-group MIDI endpoint.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct ChannelAddress {
    group: u8,
    channel: ChannelInGroup,
}

impl ChannelAddress {
    /// A function-block-wide address on group 0.
    pub const FUNCTION_BLOCK: ChannelAddress = ChannelAddress {
        group: 0,
        channel: ChannelInGroup::WholeBlock,
    };

    /// A whole-group address on group 0.
    pub const GROUP: ChannelAddress = ChannelAddress {
        group: 0,
        channel: ChannelInGroup::WholeGroup,
    };

    /// Construct a channel address. `group` must be `< 16`.
    pub const fn new(group: u8, channel: ChannelInGroup) -> Option<Self> {
        if group < 16 {
            Some(ChannelAddress { group, channel })
        } else {
            None
        }
    }

    /// The UMP group this address belongs to (0..16).
    pub const fn group(self) -> u8 {
        self.group
    }

    /// The channel-in-group component of this address.
    pub const fn channel(self) -> ChannelInGroup {
        self.channel
    }

    /// `true` if this address refers to every channel in the function block.
    pub const fn is_block(self) -> bool {
        self.channel.is_block()
    }

    /// `true` if this address refers to every channel in the group.
    pub const fn is_group(self) -> bool {
        self.channel.is_group()
    }

    /// `true` if this address refers to a single channel.
    pub const fn is_single_channel(self) -> bool {
        self.channel.is_single_channel()
    }

    /// Return a copy with the given group.
    pub const fn with_group(mut self, group: u8) -> Self {
        self.group = group;
        self
    }

    /// Return a copy with the given channel-in-group.
    pub const fn with_channel(mut self, channel: ChannelInGroup) -> Self {
        self.channel = channel;
        self
    }
}

impl Default for ChannelAddress {
    fn default() -> Self {
        ChannelAddress::FUNCTION_BLOCK
    }
}

// =============================================================================
// Profile — a 5-byte profile identifier (MMA / AMEI Standard Defined Profile
// IDs start with 0x7E).
// =============================================================================

/// A 5-byte MIDI-CI Profile ID.
///
/// Standard-defined profiles (defined by MMA / AMEI) start with `0x7E`;
/// vendor-defined ones do not. The fifth byte identifies the profile's level.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Ord, PartialOrd)]
pub struct Profile(pub [u8; 5]);

impl Profile {
    /// Construct a profile from a 5-byte identifier.
    pub const fn new(id: [u8; 5]) -> Self {
        Profile(id)
    }

    /// The raw identifier bytes.
    pub const fn as_bytes(&self) -> &[u8; 5] {
        &self.0
    }

    /// `true` if the first byte is `0x7E`, marking this as a Standard Defined
    /// Profile.
    pub const fn is_standard_defined(self) -> bool {
        self.0[0] == 0x7E
    }
}

impl fmt::Debug for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Profile({:02x}:{:02x}:{:02x}:{:02x}:{:02x})",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4]
        )
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

// =============================================================================
// DeviceInfo — the manufacturer / family / model / revision quartet.
// =============================================================================

/// Identifies the manufacturer, family, model, and software revision of a
/// MIDI-CI device. These four values are present in every discovery message.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct DeviceInfo {
    /// 3-byte manufacturer SysEx ID (or 0 if unknown).
    pub manufacturer: [u8; 3],
    /// 2-byte family code (LSB first).
    pub family: [u8; 2],
    /// 2-byte model number (LSB first).
    pub model: [u8; 2],
    /// 4-byte software revision (LSB first).
    pub revision: [u8; 4],
}

impl DeviceInfo {
    /// Construct a `DeviceInfo` from raw bytes.
    pub const fn new(
        manufacturer: [u8; 3],
        family: [u8; 2],
        model: [u8; 2],
        revision: [u8; 4],
    ) -> Self {
        DeviceInfo {
            manufacturer,
            family,
            model,
            revision,
        }
    }

    /// An example device info that's easy to recognise in test output.
    pub const fn example() -> Self {
        DeviceInfo::new(
            [0x7D, 0x02, 0x01],
            [0x00, 0x00],
            [0x00, 0x00],
            [0x00, 0x00, 0x00, 0x00],
        )
    }
}

// =============================================================================
// CapabilityFlags — bit-set of MIDI-CI categories a device supports.
// =============================================================================

/// Bit-set of MIDI-CI categories a device supports, declared in the Discovery
/// message.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct CapabilityFlags(pub u8);

impl CapabilityFlags {
    /// No supported categories.
    pub const NONE: CapabilityFlags = CapabilityFlags(0);

    /// Bit for protocol negotiation (0x01 in the JUCE implementation).
    pub const PROTOCOL_NEGOTIATION: CapabilityFlags = CapabilityFlags(0b0000_0001);

    /// Bit for profile configuration (0x02).
    pub const PROFILE_CONFIGURATION: CapabilityFlags = CapabilityFlags(0b0000_0010);

    /// Bit for property exchange (0x04).
    pub const PROPERTY_EXCHANGE: CapabilityFlags = CapabilityFlags(0b0000_0100);

    /// Bit for process inquiry (0x08).
    pub const PROCESS_INQUIRY: CapabilityFlags = CapabilityFlags(0b0000_1000);

    /// Test if a flag is set.
    pub const fn contains(self, other: CapabilityFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Set a flag (returns a new value, by-value semantics).
    pub const fn with(mut self, other: CapabilityFlags) -> Self {
        self.0 |= other.0;
        self
    }

    /// Clear a flag.
    pub const fn without(mut self, other: CapabilityFlags) -> Self {
        self.0 &= !other.0;
        self
    }

    /// The raw underlying byte.
    pub const fn bits(self) -> u8 {
        self.0
    }

    /// Reconstruct from the raw byte.
    pub const fn from_bits_truncate(bits: u8) -> Self {
        CapabilityFlags(bits)
    }
}

impl std::ops::BitOr for CapabilityFlags {
    type Output = CapabilityFlags;
    fn bitor(self, rhs: CapabilityFlags) -> CapabilityFlags {
        CapabilityFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for CapabilityFlags {
    fn bitor_assign(&mut self, rhs: CapabilityFlags) {
        self.0 |= rhs.0;
    }
}

// =============================================================================
// Category — the wire-format "sub-id #2" status byte.
// =============================================================================

/// MIDI-CI message category. The wire byte is `0x70 + (category as u8)`, so
/// the first 16 categories land in the 0x70-0x7F range and the rest extend
/// into 0x80+. This matches the JUCE `MessageMeta::Meta<T>::subID2` table.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Category {
    /// Discovery — the broadcast "who's there?" message.
    Discovery = 0x00,
    /// Discovery reply.
    DiscoveryReply = 0x01,
    /// InvalidateMUID — tells listeners our old MUID is gone.
    InvalidateMuid = 0x02,
    /// Endpoint inquiry.
    EndpointInquiry = 0x03,
    /// Endpoint reply.
    EndpointInquiryResponse = 0x04,
    /// ACK — generic acknowledgement.
    Ack = 0x05,
    /// NAK — generic non-acknowledgement.
    Nak = 0x06,
    /// Profile inquiry (PI).
    ProfileInquiry = 0x07,
    /// Profile inquiry reply.
    ProfileInquiryResponse = 0x08,
    /// Profile added notification.
    ProfileAdded = 0x09,
    /// Profile removed notification.
    ProfileRemoved = 0x0A,
    /// Profile details (PE for profile).
    ProfileDetails = 0x0B,
    /// Profile details reply.
    ProfileDetailsResponse = 0x0C,
    /// Profile set on / enabled.
    ProfileOn = 0x0D,
    /// Profile set off / disabled.
    ProfileOff = 0x0E,
    /// Profile enabled report.
    ProfileEnabledReport = 0x0F,
    /// Profile disabled report.
    ProfileDisabledReport = 0x10,
    /// Profile-specific data (PE stream for profiles).
    ProfileSpecificData = 0x11,
    /// Property exchange capabilities inquiry.
    PropertyExchangeCapabilities = 0x12,
    /// Property exchange capabilities reply.
    PropertyExchangeCapabilitiesResponse = 0x13,
    /// Property get data inquiry (PE stream).
    PropertyGetData = 0x14,
    /// Property get data reply.
    PropertyGetDataResponse = 0x15,
    /// Property set data inquiry.
    PropertySetData = 0x16,
    /// Property set data reply.
    PropertySetDataResponse = 0x17,
    /// Property subscribe.
    PropertySubscribe = 0x18,
    /// Property subscribe reply.
    PropertySubscribeResponse = 0x19,
    /// Property notify (sent by subscribers on local changes).
    PropertyNotify = 0x1A,
    /// Process inquiry.
    ProcessInquiry = 0x1B,
    /// Process inquiry reply.
    ProcessInquiryResponse = 0x1C,
    /// Process MIDI message report.
    ProcessMidiMessageReport = 0x1D,
    /// Process MIDI message report reply.
    ProcessMidiMessageReportResponse = 0x1E,
    /// Process end of MIDI message report.
    ProcessEndMidiMessageReport = 0x1F,
}

impl Category {
    /// Look up a category from the wire-format status byte.
    pub const fn from_status_byte(byte: u8) -> Option<Self> {
        if byte < 0x70 {
            return None;
        }
        let sub = byte.wrapping_sub(0x70);
        match sub {
            0x00 => Some(Category::Discovery),
            0x01 => Some(Category::DiscoveryReply),
            0x02 => Some(Category::InvalidateMuid),
            0x03 => Some(Category::EndpointInquiry),
            0x04 => Some(Category::EndpointInquiryResponse),
            0x05 => Some(Category::Ack),
            0x06 => Some(Category::Nak),
            0x07 => Some(Category::ProfileInquiry),
            0x08 => Some(Category::ProfileInquiryResponse),
            0x09 => Some(Category::ProfileAdded),
            0x0A => Some(Category::ProfileRemoved),
            0x0B => Some(Category::ProfileDetails),
            0x0C => Some(Category::ProfileDetailsResponse),
            0x0D => Some(Category::ProfileOn),
            0x0E => Some(Category::ProfileOff),
            0x0F => Some(Category::ProfileEnabledReport),
            0x10 => Some(Category::ProfileDisabledReport),
            0x11 => Some(Category::ProfileSpecificData),
            0x12 => Some(Category::PropertyExchangeCapabilities),
            0x13 => Some(Category::PropertyExchangeCapabilitiesResponse),
            0x14 => Some(Category::PropertyGetData),
            0x15 => Some(Category::PropertyGetDataResponse),
            0x16 => Some(Category::PropertySetData),
            0x17 => Some(Category::PropertySetDataResponse),
            0x18 => Some(Category::PropertySubscribe),
            0x19 => Some(Category::PropertySubscribeResponse),
            0x1A => Some(Category::PropertyNotify),
            0x1B => Some(Category::ProcessInquiry),
            0x1C => Some(Category::ProcessInquiryResponse),
            0x1D => Some(Category::ProcessMidiMessageReport),
            0x1E => Some(Category::ProcessMidiMessageReportResponse),
            0x1F => Some(Category::ProcessEndMidiMessageReport),
            _ => None,
        }
    }

    /// Wire-format status byte for this category.
    pub const fn to_status_byte(self) -> u8 {
        0x70 + (self as u8)
    }
}

// =============================================================================
// Encoding — content encoding for property exchange bodies.
// =============================================================================

/// Content encoding used by property-exchange bodies.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum Encoding {
    /// ASCII (7-bit SysEx-compatible) — the default.
    Ascii = 0,
    /// Mcoded7 — for binary blobs (8-bit bytes encoded as 7-bit).
    Mcoded7 = 1,
    /// zlib-compressed then Mcoded7.
    ZlibAndMcoded7 = 2,
}

impl Encoding {
    /// Decode a raw encoding byte into an `Encoding` (defaulting to ASCII for
    /// unknown values, matching JUCE's lenient fallback).
    pub const fn from_byte(byte: u8) -> Self {
        match byte {
            1 => Encoding::Mcoded7,
            2 => Encoding::ZlibAndMcoded7,
            _ => Encoding::Ascii,
        }
    }

    /// Convert back to a raw byte.
    pub const fn to_byte(self) -> u8 {
        self as u8
    }
}

// =============================================================================
// PropertySubscriptionCommand — the 5 kinds of subscription-update commands.
// =============================================================================

/// Kinds of commands that may be sent as part of a property subscription
/// update.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[repr(u8)]
pub enum SubscriptionCommand {
    /// Begin a subscription.
    #[default]
    Start = 0,
    /// Send a partial update.
    Partial = 1,
    /// Send a full update.
    Full = 2,
    /// Notify a subscribed property change.
    Notify = 3,
    /// End the subscription.
    End = 4,
}

impl SubscriptionCommand {
    /// Decode from the wire byte.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(SubscriptionCommand::Start),
            1 => Some(SubscriptionCommand::Partial),
            2 => Some(SubscriptionCommand::Full),
            3 => Some(SubscriptionCommand::Notify),
            4 => Some(SubscriptionCommand::End),
            _ => None,
        }
    }

    /// Encode as a wire byte.
    pub const fn to_byte(self) -> u8 {
        self as u8
    }
}

// =============================================================================
// Version — the MIDI-CI protocol version a peer speaks.
// =============================================================================

/// The MIDI-CI protocol version. V1 = original, V2 = adds output path id /
/// larger reply bodies.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ProtocolVersion {
    /// Version 1.
    V1,
    /// Version 2.
    V2,
}

impl ProtocolVersion {
    /// Decode the version byte from a message header.
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(ProtocolVersion::V1),
            0x02 => Some(ProtocolVersion::V2),
            _ => None,
        }
    }

    /// Convert to the wire byte.
    pub const fn to_byte(self) -> u8 {
        match self {
            ProtocolVersion::V1 => 0x01,
            ProtocolVersion::V2 => 0x02,
        }
    }

    /// The highest version this implementation supports.
    pub const IMPLEMENTATION: ProtocolVersion = ProtocolVersion::V2;
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        ProtocolVersion::IMPLEMENTATION
    }
}

// =============================================================================
// RequestId / RequestKey / SubscriptionKey
// =============================================================================

/// A 7-bit request identifier used inside property exchange bodies.
///
/// MIDI-CI limits the field to 7 bits so it can fit alongside other headers
/// inside a 64-bit UMP payload.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct RequestId(pub u8);

impl RequestId {
    /// A "no request" sentinel (request id 0x00 typically means "n/a").
    pub const NONE: RequestId = RequestId(0);

    /// Maximum value of a valid request id (7-bit).
    pub const MAX: u8 = 0x7F;

    /// Construct a `RequestId` from a byte; returns `None` if the byte does
    /// not fit in 7 bits.
    pub const fn new(byte: u8) -> Option<Self> {
        if (byte & !Self::MAX) == 0 {
            Some(RequestId(byte))
        } else {
            None
        }
    }

    /// Construct a `RequestId` without validation.
    pub const fn from_bits_truncate(byte: u8) -> Self {
        RequestId(byte & Self::MAX)
    }

    /// The underlying byte.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::NONE
    }
}

/// An opaque key for an ongoing property-exchange request initiated by us.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct RequestKey(pub u32);

/// An opaque key for an ongoing property subscription initiated by us.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct SubscriptionKey(pub u32);

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muid_validates_range() {
        assert!(Muid::new(0).is_ok());
        assert!(Muid::new(Muid::MASK).is_ok());
        assert!(Muid::new(0x10000000).is_err());
    }

    #[test]
    fn muid_broadcast_and_reserved() {
        assert!(Muid::BROADCAST.is_broadcast());
        assert!(!Muid::BROADCAST.is_reserved());
        assert!(Muid::new(Muid::RESERVED_START).unwrap().is_reserved());
    }

    #[test]
    fn muid_random_is_unique_across_two_draws() {
        let a = Muid::random();
        let b = Muid::random();
        assert!(a != b, "two random MUIDs should not collide");
        assert!(!a.is_reserved() && !a.is_broadcast());
        assert!(!b.is_reserved() && !b.is_broadcast());
    }

    #[test]
    fn channel_in_group_round_trip() {
        for byte in 0u8..=15 {
            let c = ChannelInGroup::from_byte(byte).unwrap();
            assert_eq!(c.to_byte(), byte);
            assert!(c.is_single_channel());
            assert_eq!(c.single_channel(), Some(byte));
        }
        assert!(ChannelInGroup::from_byte(0x10).is_none());
        assert_eq!(
            ChannelInGroup::from_byte(0x7E),
            Some(ChannelInGroup::WholeGroup)
        );
        assert_eq!(
            ChannelInGroup::from_byte(0x7F),
            Some(ChannelInGroup::WholeBlock)
        );
        assert!(ChannelInGroup::WholeGroup.is_group());
        assert!(ChannelInGroup::WholeBlock.is_block());
        assert!(!ChannelInGroup::WholeGroup.is_block());
        assert!(!ChannelInGroup::Channel5.is_group());
    }

    #[test]
    fn channel_address_validates_group_range() {
        assert!(ChannelAddress::new(0, ChannelInGroup::WholeGroup).is_some());
        assert!(ChannelAddress::new(15, ChannelInGroup::ChannelF).is_some());
        assert!(ChannelAddress::new(16, ChannelInGroup::Channel0).is_none());
    }

    #[test]
    fn profile_is_standard_defined() {
        let std_profile = Profile::new([0x7E, 0x01, 0x02, 0x03, 0x04]);
        let vend_profile = Profile::new([0x7D, 0x01, 0x02, 0x03, 0x04]);
        assert!(std_profile.is_standard_defined());
        assert!(!vend_profile.is_standard_defined());
    }

    #[test]
    fn capability_flags_combine() {
        let combined =
            CapabilityFlags::PROFILE_CONFIGURATION | CapabilityFlags::PROPERTY_EXCHANGE;
        assert!(combined.contains(CapabilityFlags::PROFILE_CONFIGURATION));
        assert!(combined.contains(CapabilityFlags::PROPERTY_EXCHANGE));
        assert!(!combined.contains(CapabilityFlags::PROTOCOL_NEGOTIATION));
    }

    #[test]
    fn category_status_byte_round_trip() {
        let cases = [
            (Category::Discovery, 0x70),
            (Category::DiscoveryReply, 0x71),
            (Category::InvalidateMuid, 0x72),
            (Category::ProfileInquiry, 0x77),
            (Category::PropertyExchangeCapabilities, 0x82),
            (Category::Ack, 0x75),
            (Category::Nak, 0x76),
            (Category::ProcessEndMidiMessageReport, 0x8F),
        ];
        for (cat, byte) in cases {
            assert_eq!(cat.to_status_byte(), byte, "to_byte for {:?}", cat);
            assert_eq!(
                Category::from_status_byte(byte),
                Some(cat),
                "from_status_byte for {:#x}",
                byte
            );
        }
        assert_eq!(Category::from_status_byte(0x6F), None);
        assert_eq!(Category::from_status_byte(0x90), None);
    }

    #[test]
    fn encoding_default_is_ascii() {
        assert_eq!(Encoding::from_byte(0), Encoding::Ascii);
        assert_eq!(Encoding::from_byte(99), Encoding::Ascii);
        assert_eq!(Encoding::from_byte(1), Encoding::Mcoded7);
        assert_eq!(Encoding::from_byte(2), Encoding::ZlibAndMcoded7);
    }

    #[test]
    fn subscription_command_round_trip() {
        for byte in 0u8..=4 {
            let cmd = SubscriptionCommand::from_byte(byte).unwrap();
            assert_eq!(cmd.to_byte(), byte);
        }
        assert_eq!(SubscriptionCommand::from_byte(5), None);
    }

    #[test]
    fn protocol_version_round_trip() {
        assert_eq!(ProtocolVersion::V1.to_byte(), 1);
        assert_eq!(ProtocolVersion::V2.to_byte(), 2);
        assert_eq!(ProtocolVersion::from_byte(1), Some(ProtocolVersion::V1));
        assert_eq!(ProtocolVersion::from_byte(2), Some(ProtocolVersion::V2));
        assert_eq!(ProtocolVersion::from_byte(3), None);
    }

    #[test]
    fn request_id_validates_seven_bits() {
        assert_eq!(RequestId::new(0).unwrap().get(), 0);
        assert_eq!(RequestId::new(0x7F).unwrap().get(), 0x7F);
        assert!(RequestId::new(0x80).is_none());
    }
}