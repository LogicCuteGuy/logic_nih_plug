//! Wire-format encoders and decoders for MIDI-CI messages.
//!
//! MIDI-CI messages are carried in 64-bit Universal MIDI Packets (UMPs). The
//! structure is:
//!
//! | UMP word        | Bits           | Purpose                                            |
//! |-----------------|----------------|----------------------------------------------------|
//! | `0x7E`          | 8              | NoOp / sysex start placeholder                     |
//! | group byte      | 8              | UMP group (0..15) — also used as device_id address |
//! | `0x0D`          | 8              | UMP type 0x0D (MIDI-CI stream)                     |
//! | status byte     | 8              | CI sub-status / category                           |
//! | version byte    | 8              | Protocol version                                   |
//! | source MUID     | 4×7 = 28       | Source MUID, 28 bits, MSB-aligned 7-bit bytes     |
//! | dest MUID       | 4×7 = 28       | Destination MUID, 28 bits, MSB-aligned 7-bit bytes |
//! | body            | variable       | Per-category payload                               |
//!
//! All multi-byte fields use 7-bit-per-byte packing (the top bit of each
//! byte is always 0). This matches both JUCE's `Marshalling::Reader` and
//! `Marshalling::Writer` and is mandatory for compatibility with MIDI 1.0
//! transports that only carry 7-bit SysEx data.

use crate::error::{MidiCiError, MidiCiResult};
use crate::message::{Header, MessageBody, ParsedMessage};
use crate::types::{CapabilityFlags, ChannelInGroup, DeviceInfo, Muid, Profile, ProtocolVersion};

// =============================================================================
// Constants
// =============================================================================

/// UMP type byte for MIDI-CI stream messages (0x0D).
pub const UMP_TYPE_MIDI_CI: u8 = 0x0D;

/// The UMP NoOp byte that always starts a MIDI-CI 64-bit UMP.
pub const UMP_NOOP: u8 = 0x7E;

// =============================================================================
// Bit-packed primitive encoders / decoders
// =============================================================================

fn write_muid(out: &mut Vec<u8>, muid: Muid) {
    let v = muid.get();
    out.push((v & 0x7F) as u8);
    out.push(((v >> 7) & 0x7F) as u8);
    out.push(((v >> 14) & 0x7F) as u8);
    out.push(((v >> 21) & 0x7F) as u8);
}

fn read_muid(cursor: &mut ReadCursor<'_>) -> MidiCiResult<Muid> {
    let bytes = cursor.take(4)?;
    let v = (bytes[0] & 0x7F) as u32
        | (((bytes[1] & 0x7F) as u32) << 7)
        | (((bytes[2] & 0x7F) as u32) << 14)
        | (((bytes[3] & 0x7F) as u32) << 21);
    Ok(Muid::from_bits_truncate(v))
}

fn write_u16(out: &mut Vec<u8>, value: u16) {
    out.push((value & 0x7F) as u8);
    out.push(((value >> 7) & 0x7F) as u8);
}

fn read_u16(cursor: &mut ReadCursor<'_>) -> MidiCiResult<u16> {
    let bytes = cursor.take(2)?;
    Ok((bytes[0] & 0x7F) as u16 | (((bytes[1] & 0x7F) as u16) << 7))
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.push((value & 0x7F) as u8);
    out.push(((value >> 7) & 0x7F) as u8);
    out.push(((value >> 14) & 0x7F) as u8);
    out.push(((value >> 21) & 0x7F) as u8);
}

fn read_u32(cursor: &mut ReadCursor<'_>) -> MidiCiResult<u32> {
    let bytes = cursor.take(4)?;
    Ok((bytes[0] & 0x7F) as u32
        | (((bytes[1] & 0x7F) as u32) << 7)
        | (((bytes[2] & 0x7F) as u32) << 14)
        | (((bytes[3] & 0x7F) as u32) << 21))
}

fn write_device_info(out: &mut Vec<u8>, info: &DeviceInfo) {
    out.extend_from_slice(&info.manufacturer);
    out.extend_from_slice(&info.family);
    out.extend_from_slice(&info.model);
    out.extend_from_slice(&info.revision);
}

fn read_device_info(cursor: &mut ReadCursor<'_>) -> MidiCiResult<DeviceInfo> {
    let manufacturer = <[u8; 3]>::try_from(cursor.take(3)?).unwrap();
    let family = <[u8; 2]>::try_from(cursor.take(2)?).unwrap();
    let model = <[u8; 2]>::try_from(cursor.take(2)?).unwrap();
    let revision = <[u8; 4]>::try_from(cursor.take(4)?).unwrap();
    Ok(DeviceInfo::new(manufacturer, family, model, revision))
}

fn write_profile(out: &mut Vec<u8>, profile: &Profile) {
    out.extend_from_slice(profile.as_bytes());
}

fn read_profile(cursor: &mut ReadCursor<'_>) -> MidiCiResult<Profile> {
    let bytes = cursor.take(5)?;
    Ok(Profile::new(<[u8; 5]>::try_from(bytes).unwrap()))
}

fn write_size_prefixed_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    let len = bytes.len() as u16;
    out.push((len & 0x7F) as u8);
    out.push(((len >> 7) & 0x7F) as u8);
    out.extend_from_slice(bytes);
}

fn read_size_prefixed_bytes(cursor: &mut ReadCursor<'_>) -> MidiCiResult<Vec<u8>> {
    let len = read_u16(cursor)? as usize;
    Ok(cursor.take(len)?.to_vec())
}

// =============================================================================
// WriteSink / ReadCursor helpers
// =============================================================================

/// A simple `Vec<u8>` wrapper used to assemble an outbound message.
#[derive(Debug, Default)]
pub struct WriteSink {
    bytes: Vec<u8>,
}

impl WriteSink {
    /// Start a fresh sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one byte.
    pub fn push(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    /// Append multiple bytes.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        self.bytes.extend_from_slice(slice);
    }

    /// Consume the sink and return the assembled byte vector.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Borrow the assembled bytes so far.
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    /// Append a `Muid` in the 4-byte 7-bit form.
    pub fn write_muid(&mut self, muid: Muid) {
        write_muid(&mut self.bytes, muid);
    }

    /// Append a `u16` in the 2-byte 7-bit form.
    pub fn write_u16(&mut self, value: u16) {
        write_u16(&mut self.bytes, value);
    }

    /// Append a `u32` in the 4-byte 7-bit form.
    pub fn write_u32(&mut self, value: u32) {
        write_u32(&mut self.bytes, value);
    }

    /// Append a size-prefixed byte block.
    pub fn write_size_prefixed(&mut self, slice: &[u8]) {
        write_size_prefixed_bytes(&mut self.bytes, slice);
    }

    /// Append a `Profile`.
    pub fn write_profile(&mut self, profile: &Profile) {
        write_profile(&mut self.bytes, profile);
    }

    /// Append a `DeviceInfo`.
    pub fn write_device_info(&mut self, info: &DeviceInfo) {
        write_device_info(&mut self.bytes, info);
    }
}

/// A read cursor over an input byte slice.
#[derive(Debug)]
pub struct ReadCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ReadCursor<'a> {
    /// Wrap a slice for incremental reading.
    pub fn new(bytes: &'a [u8]) -> Self {
        ReadCursor { bytes, pos: 0 }
    }

    /// How many bytes remain.
    pub fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    /// Take `n` bytes off the front. Returns an error if the cursor has been
    /// exhausted.
    pub fn take(&mut self, n: usize) -> MidiCiResult<&'a [u8]> {
        if self.remaining() < n {
            return Err(MidiCiError::TooShort {
                len: self.remaining(),
                min: n,
            });
        }
        let out = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(out)
    }

    /// Take a single byte.
    pub fn take_byte(&mut self) -> MidiCiResult<u8> {
        Ok(self.take(1)?[0])
    }

    /// Read a `Muid`.
    pub fn read_muid(&mut self) -> MidiCiResult<Muid> {
        read_muid(self)
    }

    /// Read a `u16`.
    pub fn read_u16(&mut self) -> MidiCiResult<u16> {
        read_u16(self)
    }

    /// Read a `u32`.
    pub fn read_u32(&mut self) -> MidiCiResult<u32> {
        read_u32(self)
    }

    /// Read a size-prefixed byte block.
    pub fn read_size_prefixed(&mut self) -> MidiCiResult<Vec<u8>> {
        read_size_prefixed_bytes(self)
    }

    /// Read a `Profile`.
    pub fn read_profile(&mut self) -> MidiCiResult<Profile> {
        read_profile(self)
    }

    /// Read a `DeviceInfo`.
    pub fn read_device_info(&mut self) -> MidiCiResult<DeviceInfo> {
        read_device_info(self)
    }
}

// =============================================================================
// Per-category body encoders / decoders
// =============================================================================

/// Encode a [`MessageBody`] and the given header into a 64-bit UMP packet
/// (the UMP framing — `0x7E`, group, `0x0D` — is included at the start).
pub fn encode(header: &Header, body: &MessageBody, group: u8) -> Vec<u8> {
    let mut sink = WriteSink::new();
    // UMP framing.
    sink.push(UMP_NOOP);
    sink.push(group & 0x0F);
    sink.push(UMP_TYPE_MIDI_CI);
    encode_body(&mut sink, header, body);
    sink.into_bytes()
}

/// Encode just the body (header + category payload), without the UMP framing.
/// Useful when an outer layer (e.g. a UMP packet builder) has already
/// provided the framing.
pub fn encode_body(sink: &mut WriteSink, header: &Header, body: &MessageBody) {
    use crate::message::MessageBody as MB;
    let status = body.category_byte();
    sink.push(header.device_id.to_byte());
    sink.push(status);
    sink.push(header.version);
    sink.write_muid(header.source);
    sink.write_muid(header.destination);

    let version = header.protocol_version();
    match body {
        MB::Discovery(m) => {
            sink.write_device_info(&m.device_info);
            sink.push(m.capabilities.bits());
            sink.write_u32(m.maximum_sysex_size);
            if version == ProtocolVersion::V2 {
                sink.push(m.output_path_id);
            }
        }
        MB::DiscoveryReply(m) => {
            sink.write_device_info(&m.device_info);
            sink.push(m.capabilities.bits());
            sink.write_u32(m.maximum_sysex_size);
            if version == ProtocolVersion::V2 {
                sink.push(m.output_path_id);
                sink.push(m.function_block);
            }
        }
        MB::InvalidateMuid(m) => {
            sink.write_muid(m.target);
        }
        MB::EndpointInquiry(m) => {
            sink.push(m.status);
        }
        MB::EndpointInquiryResponse(m) => {
            sink.push(m.status);
            sink.write_size_prefixed(&m.data);
        }
        MB::Ack(m) => {
            sink.push(m.original_category);
            sink.push(m.status_code);
            sink.push(m.status_data);
            sink.extend_from_slice(&m.details);
            sink.write_size_prefixed(&m.message_text);
        }
        MB::Nak(m) => {
            sink.push(m.original_category);
            sink.push(m.status_code);
            sink.push(m.status_data);
            sink.extend_from_slice(&m.details);
            sink.write_size_prefixed(&m.message_text);
        }
        MB::ProfileInquiry(_) => {}
        MB::ProfileInquiryResponse(m) => {
            sink.write_size_prefixed(&profile_bytes(&m.enabled_profiles));
            sink.write_size_prefixed(&profile_bytes(&m.disabled_profiles));
        }
        MB::ProfileAdded(m) => {
            sink.write_profile(&m.profile);
        }
        MB::ProfileRemoved(m) => {
            sink.write_profile(&m.profile);
        }
        MB::ProfileDetails(m) => {
            sink.write_profile(&m.profile);
            sink.push(m.target);
        }
        MB::ProfileDetailsResponse(m) => {
            sink.write_profile(&m.profile);
            sink.push(m.target);
            sink.write_size_prefixed(&m.data);
        }
        MB::ProfileOn(m) => {
            sink.write_profile(&m.profile);
            if version == ProtocolVersion::V2 {
                sink.write_u16(m.num_channels);
            }
        }
        MB::ProfileOff(m) => {
            sink.write_profile(&m.profile);
        }
        MB::ProfileEnabledReport(m) => {
            sink.write_profile(&m.profile);
            if version == ProtocolVersion::V2 {
                sink.write_u16(m.num_channels);
            }
        }
        MB::ProfileDisabledReport(m) => {
            sink.write_profile(&m.profile);
            if version == ProtocolVersion::V2 {
                sink.write_u16(m.num_channels);
            }
        }
        MB::ProfileSpecificData(m) => {
            sink.write_profile(&m.profile);
            sink.write_size_prefixed(&m.data);
        }
        MB::PropertyExchangeCapabilities(m) => {
            sink.push(m.num_simultaneous_requests_supported);
            if version == ProtocolVersion::V2 {
                sink.push(m.major_version);
                sink.push(m.minor_version);
            }
        }
        MB::PropertyExchangeCapabilitiesResponse(m) => {
            sink.push(m.num_simultaneous_requests_supported);
            if version == ProtocolVersion::V2 {
                sink.push(m.major_version);
                sink.push(m.minor_version);
            }
        }
        MB::PropertyGetData(m) => {
            encode_static_pe(sink, &m.inner);
        }
        MB::PropertyGetDataResponse(m) => {
            encode_dynamic_pe(sink, &m.inner);
        }
        MB::PropertySetData(m) => {
            encode_dynamic_pe(sink, &m.inner);
        }
        MB::PropertySetDataResponse(m) => {
            encode_static_pe(sink, &m.inner);
        }
        MB::PropertySubscribe(m) => {
            sink.push(m.command.to_byte());
            encode_dynamic_pe(sink, &m.inner);
        }
        MB::PropertySubscribeResponse(m) => {
            encode_dynamic_pe(sink, &m.inner);
        }
        MB::PropertyNotify(m) => {
            encode_dynamic_pe(sink, &m.inner);
        }
        MB::ProcessInquiry(m) => {
            sink.push(m.supported_features);
        }
        MB::ProcessInquiryResponse(m) => {
            sink.push(m.supported_features);
        }
        MB::ProcessMidiMessageReport(m) => {
            sink.push(m.message_data_control);
            sink.push(m.requested_messages);
            sink.push(m.channel_controller_messages);
            sink.push(m.note_data_messages);
        }
        MB::ProcessMidiMessageReportResponse(m) => {
            sink.push(m.message_data_control);
            sink.push(m.requested_messages);
            sink.push(m.channel_controller_messages);
            sink.push(m.note_data_messages);
        }
        MB::ProcessEndMidiMessageReport(_) => {}
        MB::Malformed(_) => {}
    }
}

impl MessageBody {
    /// The wire-format status byte (0x70..=0x8F) for this body type.
    ///
    /// This is the byte that gets written to the wire in the encoded header
    /// and that the decoder looks at to pick the body parser.
    pub fn category_byte(&self) -> u8 {
        0x70 + self.as_outbound().category()
    }

    /// Same as [`Self::category_byte`].
    pub fn outbound_category(&self) -> u8 {
        self.category_byte()
    }

    /// Convert a parsed message body into its outbound counterpart.
    pub fn as_outbound(&self) -> crate::message::OutboundMessage {
        use crate::message::OutboundMessage as OM;
        match self.clone() {
            MessageBody::Discovery(m) => OM::Discovery(m),
            MessageBody::DiscoveryReply(m) => OM::DiscoveryReply(m),
            MessageBody::InvalidateMuid(m) => OM::InvalidateMuid(m),
            MessageBody::EndpointInquiry(m) => OM::EndpointInquiry(m),
            MessageBody::EndpointInquiryResponse(m) => OM::EndpointInquiryResponse(m),
            MessageBody::Ack(m) => OM::Ack(m),
            MessageBody::Nak(m) => OM::Nak(m),
            MessageBody::ProfileInquiry(m) => OM::ProfileInquiry(m),
            MessageBody::ProfileInquiryResponse(m) => OM::ProfileInquiryResponse(m),
            MessageBody::ProfileAdded(m) => OM::ProfileAdded(m),
            MessageBody::ProfileRemoved(m) => OM::ProfileRemoved(m),
            MessageBody::ProfileDetails(m) => OM::ProfileDetails(m),
            MessageBody::ProfileDetailsResponse(m) => OM::ProfileDetailsResponse(m),
            MessageBody::ProfileOn(m) => OM::ProfileOn(m),
            MessageBody::ProfileOff(m) => OM::ProfileOff(m),
            MessageBody::ProfileEnabledReport(m) => OM::ProfileEnabledReport(m),
            MessageBody::ProfileDisabledReport(m) => OM::ProfileDisabledReport(m),
            MessageBody::ProfileSpecificData(m) => OM::ProfileSpecificData(m),
            MessageBody::PropertyExchangeCapabilities(m) => OM::PropertyExchangeCapabilities(m),
            MessageBody::PropertyExchangeCapabilitiesResponse(m) => {
                OM::PropertyExchangeCapabilitiesResponse(m)
            }
            MessageBody::PropertyGetData(m) => OM::PropertyGetData(m),
            MessageBody::PropertyGetDataResponse(m) => OM::PropertyGetDataResponse(m),
            MessageBody::PropertySetData(m) => OM::PropertySetData(m),
            MessageBody::PropertySetDataResponse(m) => OM::PropertySetDataResponse(m),
            MessageBody::PropertySubscribe(m) => OM::PropertySubscribe(m),
            MessageBody::PropertySubscribeResponse(m) => OM::PropertySubscribeResponse(m),
            MessageBody::PropertyNotify(m) => OM::PropertyNotify(m),
            MessageBody::ProcessInquiry(m) => OM::ProcessInquiry(m),
            MessageBody::ProcessInquiryResponse(m) => OM::ProcessInquiryResponse(m),
            MessageBody::ProcessMidiMessageReport(m) => OM::ProcessMidiMessageReport(m),
            MessageBody::ProcessMidiMessageReportResponse(m) => {
                OM::ProcessMidiMessageReportResponse(m)
            }
            MessageBody::ProcessEndMidiMessageReport(m) => OM::ProcessEndMidiMessageReport(m),
            MessageBody::Malformed(_) => OM::Discovery(crate::message::Discovery::default()),
        }
    }
}

fn encode_static_pe(
    sink: &mut WriteSink,
    inner: &crate::message::StaticSizePropertyExchange,
) {
    sink.push(inner.request_id.get() & 0x7F);
    sink.write_size_prefixed(&inner.header);
}

fn encode_dynamic_pe(
    sink: &mut WriteSink,
    inner: &crate::message::DynamicSizePropertyExchange,
) {
    sink.push(inner.request_id.get() & 0x7F);
    sink.write_size_prefixed(&inner.header);
    sink.write_u16(inner.total_num_chunks);
    sink.write_u16(inner.this_chunk_num);
    sink.write_size_prefixed(&inner.data);
}

fn profile_bytes(profiles: &[Profile]) -> Vec<u8> {
    let mut out = Vec::with_capacity(profiles.len() * 5);
    for p in profiles {
        out.extend_from_slice(p.as_bytes());
    }
    out
}

// =============================================================================
// Decoding
// =============================================================================

/// Decode an incoming MIDI-CI UMP payload (with framing) into a parsed
/// message.
///
/// Returns `Ok(None)` if the buffer doesn't carry a CI message at all
/// (either too short to contain the framing bytes, or the framing bytes
/// don't match). Returns `Err` if the framing is present but the body is
/// malformed.
pub fn decode(bytes: &[u8]) -> MidiCiResult<Option<ParsedMessage>> {
    if bytes.len() < 3 {
        return Ok(None);
    }
    if bytes[0] != UMP_NOOP || bytes[2] != UMP_TYPE_MIDI_CI {
        return Ok(None);
    }
    let _group = bytes[1] & 0x0F;
    let body = &bytes[3..];
    let mut cursor = ReadCursor::new(body);
    let device_id = match ChannelInGroup::from_byte(cursor.take_byte()?) {
        Some(c) => c,
        None => return Err(MidiCiError::Other("invalid device_id byte")),
    };
    let status = cursor.take_byte()?;
    let version = cursor.take_byte()?;
    let source = cursor.read_muid()?;
    let destination = cursor.read_muid()?;
    let header = Header {
        device_id,
        category: status,
        version,
        source,
        destination,
    };
    let parsed_body = decode_body(&mut cursor, &header, status)?;
    Ok(Some(ParsedMessage::new(header, parsed_body)))
}

/// Decode just the body of a CI message (header is supplied by the caller).
#[allow(unused_imports)]
pub fn decode_body(
    cursor: &mut ReadCursor<'_>,
    header: &Header,
    status: u8,
) -> MidiCiResult<MessageBody> {
    use crate::message::MessageBody as MB;
    use crate::message::{
        Ack, DiscoveryReply, EndpointInquiry, EndpointInquiryResponse, InvalidateMuid, Nak,
        ProcessEndMidiMessageReport, ProcessInquiry, ProcessInquiryResponse,
        ProcessMidiMessageReport, ProcessMidiMessageReportResponse, ProfileAdded,
        ProfileDetails, ProfileDetailsResponse, ProfileDisabledReport, ProfileEnabledReport,
        ProfileInquiry, ProfileInquiryResponse, ProfileOff, ProfileOn, ProfileRemoved,
        ProfileSpecificData,
        PropertyGetData, PropertyGetDataResponse,
        PropertyNotify, PropertySetData, PropertySetDataResponse, PropertySubscribe,
        PropertySubscribeResponse,
    };
    use crate::types::SubscriptionCommand;

    let version = header.protocol_version();
    if status < 0x70 {
        return Err(MidiCiError::UnknownCategory(status));
    }
    let category_index = status - 0x70;
    if category_index >= 32 {
        return Err(MidiCiError::UnknownCategory(status));
    }

    let body = match category_index {
        0x00 => MB::Discovery(decode_discovery(cursor, version)?),
        0x01 => MB::DiscoveryReply(decode_discovery_reply(cursor, version)?),
        0x02 => MB::InvalidateMuid(InvalidateMuid {
            target: cursor.read_muid()?,
        }),
        0x03 => MB::EndpointInquiry(EndpointInquiry {
            status: cursor.take_byte()?,
        }),
        0x04 => MB::EndpointInquiryResponse(EndpointInquiryResponse {
            status: cursor.take_byte()?,
            data: cursor.read_size_prefixed()?,
        }),
        0x05 => MB::Ack(decode_ack(cursor)?),
        0x06 => MB::Nak(decode_nak(cursor)?),
        0x07 => MB::ProfileInquiry(ProfileInquiry),
        0x08 => MB::ProfileInquiryResponse(decode_profile_inquiry_response(cursor)?),
        0x09 => MB::ProfileAdded(ProfileAdded {
            profile: cursor.read_profile()?,
        }),
        0x0A => MB::ProfileRemoved(ProfileRemoved {
            profile: cursor.read_profile()?,
        }),
        0x0B => MB::ProfileDetails(ProfileDetails {
            profile: cursor.read_profile()?,
            target: cursor.take_byte()?,
        }),
        0x0C => MB::ProfileDetailsResponse(decode_profile_details_response(cursor)?),
        0x0D => MB::ProfileOn(ProfileOn {
            profile: cursor.read_profile()?,
            num_channels: if version == ProtocolVersion::V2 {
                cursor.read_u16()?
            } else {
                0
            },
        }),
        0x0E => MB::ProfileOff(ProfileOff {
            profile: cursor.read_profile()?,
        }),
        0x0F => MB::ProfileEnabledReport(ProfileEnabledReport {
            profile: cursor.read_profile()?,
            num_channels: if version == ProtocolVersion::V2 {
                cursor.read_u16()?
            } else {
                0
            },
        }),
        0x10 => MB::ProfileDisabledReport(ProfileDisabledReport {
            profile: cursor.read_profile()?,
            num_channels: if version == ProtocolVersion::V2 {
                cursor.read_u16()?
            } else {
                0
            },
        }),
        0x11 => MB::ProfileSpecificData(decode_profile_specific_data(cursor)?),
        0x12 => MB::PropertyExchangeCapabilities(decode_pe_capabilities(cursor, version)?),
        0x13 => MB::PropertyExchangeCapabilitiesResponse(decode_pe_capabilities_response(
            cursor, version,
        )?),
        0x14 => MB::PropertyGetData(PropertyGetData {
            inner: decode_static_pe(cursor)?,
        }),
        0x15 => MB::PropertyGetDataResponse(PropertyGetDataResponse {
            inner: decode_dynamic_pe(cursor)?,
        }),
        0x16 => MB::PropertySetData(PropertySetData {
            inner: decode_dynamic_pe(cursor)?,
        }),
        0x17 => MB::PropertySetDataResponse(PropertySetDataResponse {
            inner: decode_static_pe(cursor)?,
        }),
        0x18 => {
            let cmd = cursor.take_byte()?;
            let command = SubscriptionCommand::from_byte(cmd)
                .ok_or(MidiCiError::Malformed("unknown subscription command"))?;
            MB::PropertySubscribe(PropertySubscribe {
                command,
                inner: decode_dynamic_pe(cursor)?,
            })
        }
        0x19 => MB::PropertySubscribeResponse(PropertySubscribeResponse {
            inner: decode_dynamic_pe(cursor)?,
        }),
        0x1A => MB::PropertyNotify(PropertyNotify {
            inner: decode_dynamic_pe(cursor)?,
        }),
        0x1B => MB::ProcessInquiry(ProcessInquiry {
            supported_features: cursor.take_byte()?,
        }),
        0x1C => MB::ProcessInquiryResponse(ProcessInquiryResponse {
            supported_features: cursor.take_byte()?,
        }),
        0x1D => MB::ProcessMidiMessageReport(ProcessMidiMessageReport {
            message_data_control: cursor.take_byte()?,
            requested_messages: cursor.take_byte()?,
            channel_controller_messages: cursor.take_byte()?,
            note_data_messages: cursor.take_byte()?,
        }),
        0x1E => MB::ProcessMidiMessageReportResponse(ProcessMidiMessageReportResponse {
            message_data_control: cursor.take_byte()?,
            requested_messages: cursor.take_byte()?,
            channel_controller_messages: cursor.take_byte()?,
            note_data_messages: cursor.take_byte()?,
        }),
        0x1F => MB::ProcessEndMidiMessageReport(ProcessEndMidiMessageReport),
        _ => unreachable!("category_index out of range"),
    };
    Ok(body)
}

fn decode_discovery(
    cursor: &mut ReadCursor<'_>,
    version: ProtocolVersion,
) -> MidiCiResult<crate::message::Discovery> {
    let info = cursor.read_device_info()?;
    let caps = CapabilityFlags::from_bits_truncate(cursor.take_byte()?);
    let max_sysex = cursor.read_u32()?;
    let output_path_id = if version == ProtocolVersion::V2 {
        cursor.take_byte()?
    } else {
        0
    };
    Ok(crate::message::Discovery {
        device_info: info,
        capabilities: caps,
        maximum_sysex_size: max_sysex,
        output_path_id,
    })
}

fn decode_discovery_reply(
    cursor: &mut ReadCursor<'_>,
    version: ProtocolVersion,
) -> MidiCiResult<crate::message::DiscoveryReply> {
    let info = cursor.read_device_info()?;
    let caps = CapabilityFlags::from_bits_truncate(cursor.take_byte()?);
    let max_sysex = cursor.read_u32()?;
    let (output_path_id, function_block) = if version == ProtocolVersion::V2 {
        (cursor.take_byte()?, cursor.take_byte()?)
    } else {
        (0, 0)
    };
    Ok(crate::message::DiscoveryReply {
        device_info: info,
        capabilities: caps,
        maximum_sysex_size: max_sysex,
        output_path_id,
        function_block,
    })
}

fn decode_ack(cursor: &mut ReadCursor<'_>) -> MidiCiResult<crate::message::Ack> {
    let original_category = cursor.take_byte()?;
    let status_code = cursor.take_byte()?;
    let status_data = cursor.take_byte()?;
    let details = <[u8; 5]>::try_from(cursor.take(5)?).unwrap();
    let message_text = cursor.read_size_prefixed()?;
    Ok(crate::message::Ack {
        original_category,
        status_code,
        status_data,
        details,
        message_text,
    })
}

fn decode_nak(cursor: &mut ReadCursor<'_>) -> MidiCiResult<crate::message::Nak> {
    let original_category = cursor.take_byte()?;
    let status_code = cursor.take_byte()?;
    let status_data = cursor.take_byte()?;
    let details = <[u8; 5]>::try_from(cursor.take(5)?).unwrap();
    let message_text = cursor.read_size_prefixed()?;
    Ok(crate::message::Nak {
        original_category,
        status_code,
        status_data,
        details,
        message_text,
    })
}

fn decode_profile_inquiry_response(
    cursor: &mut ReadCursor<'_>,
) -> MidiCiResult<crate::message::ProfileInquiryResponse> {
    let enabled_bytes = cursor.read_size_prefixed()?;
    let disabled_bytes = cursor.read_size_prefixed()?;
    Ok(crate::message::ProfileInquiryResponse {
        enabled_profiles: profiles_from_bytes(&enabled_bytes),
        disabled_profiles: profiles_from_bytes(&disabled_bytes),
    })
}

fn profiles_from_bytes(bytes: &[u8]) -> Vec<Profile> {
    bytes
        .chunks_exact(5)
        .map(|chunk| Profile::new(<[u8; 5]>::try_from(chunk).unwrap()))
        .collect()
}

fn decode_profile_details_response(
    cursor: &mut ReadCursor<'_>,
) -> MidiCiResult<crate::message::ProfileDetailsResponse> {
    Ok(crate::message::ProfileDetailsResponse {
        profile: cursor.read_profile()?,
        target: cursor.take_byte()?,
        data: cursor.read_size_prefixed()?,
    })
}

fn decode_profile_specific_data(
    cursor: &mut ReadCursor<'_>,
) -> MidiCiResult<crate::message::ProfileSpecificData> {
    Ok(crate::message::ProfileSpecificData {
        profile: cursor.read_profile()?,
        data: cursor.read_size_prefixed()?,
    })
}

fn decode_pe_capabilities(
    cursor: &mut ReadCursor<'_>,
    version: ProtocolVersion,
) -> MidiCiResult<crate::message::PropertyExchangeCapabilities> {
    let num_simultaneous_requests_supported = cursor.take_byte()?;
    let (major, minor) = if version == ProtocolVersion::V2 {
        (cursor.take_byte()?, cursor.take_byte()?)
    } else {
        (0, 0)
    };
    Ok(crate::message::PropertyExchangeCapabilities {
        num_simultaneous_requests_supported,
        major_version: major,
        minor_version: minor,
    })
}

fn decode_pe_capabilities_response(
    cursor: &mut ReadCursor<'_>,
    version: ProtocolVersion,
) -> MidiCiResult<crate::message::PropertyExchangeCapabilitiesResponse> {
    let num_simultaneous_requests_supported = cursor.take_byte()?;
    let (major, minor) = if version == ProtocolVersion::V2 {
        (cursor.take_byte()?, cursor.take_byte()?)
    } else {
        (0, 0)
    };
    Ok(crate::message::PropertyExchangeCapabilitiesResponse {
        num_simultaneous_requests_supported,
        major_version: major,
        minor_version: minor,
    })
}

fn decode_static_pe(
    cursor: &mut ReadCursor<'_>,
) -> MidiCiResult<crate::message::StaticSizePropertyExchange> {
    Ok(crate::message::StaticSizePropertyExchange {
        request_id: crate::types::RequestId::from_bits_truncate(cursor.take_byte()?),
        header: cursor.read_size_prefixed()?,
    })
}

fn decode_dynamic_pe(
    cursor: &mut ReadCursor<'_>,
) -> MidiCiResult<crate::message::DynamicSizePropertyExchange> {
    Ok(crate::message::DynamicSizePropertyExchange {
        request_id: crate::types::RequestId::from_bits_truncate(cursor.take_byte()?),
        header: cursor.read_size_prefixed()?,
        total_num_chunks: cursor.read_u16()?,
        this_chunk_num: cursor.read_u16()?,
        data: cursor.read_size_prefixed()?,
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{
        Discovery, ProfileInquiry, ProfileInquiryResponse, PropertyGetData,
    };
    use crate::types::ChannelInGroup;

    fn header_for(category: u8, src: Muid, dst: Muid) -> Header {
        Header {
            device_id: ChannelInGroup::WholeBlock,
            category,
            version: ProtocolVersion::IMPLEMENTATION.to_byte(),
            source: src,
            destination: dst,
        }
    }

    #[test]
    fn muid_packing_round_trip() {
        let mut sink = WriteSink::new();
        sink.write_muid(Muid::from_bits_truncate(0x0102_0304));
        let bytes = sink.into_bytes();
        // 28-bit values are stored in 4×7-bit bytes, LSB-first.
        // 0x01020304 → bytes [4, 6, 8, 8].
        assert_eq!(bytes, vec![0x04, 0x06, 0x08, 0x08]);
        let mut cursor = ReadCursor::new(&bytes);
        assert_eq!(cursor.read_muid().unwrap().get(), 0x0102_0304);
    }

    #[test]
    fn u16_packing_round_trip() {
        // 14-bit values fit in 2×7-bit bytes; the high 2 bits are dropped.
        let mut sink = WriteSink::new();
        sink.write_u16(0x1234);
        let bytes = sink.into_bytes();
        assert_eq!(bytes, vec![0x34, 0x24]);
        let mut cursor = ReadCursor::new(&bytes);
        assert_eq!(cursor.read_u16().unwrap(), 0x1234);
    }

    #[test]
    fn u32_packing_round_trip() {
        // 28-bit values fit in 4×7-bit bytes; the high 4 bits are dropped.
        let mut sink = WriteSink::new();
        sink.write_u32(0x0345_6789);
        let bytes = sink.into_bytes();
        assert_eq!(bytes, vec![0x09, 0x4F, 0x15, 0x1A]);
        let mut cursor = ReadCursor::new(&bytes);
        assert_eq!(cursor.read_u32().unwrap(), 0x0345_6789);
    }

    #[test]
    fn profile_round_trip() {
        let mut sink = WriteSink::new();
        let p = Profile::new([0x7E, 0x01, 0x02, 0x03, 0x04]);
        sink.write_profile(&p);
        let bytes = sink.into_bytes();
        let mut cursor = ReadCursor::new(&bytes);
        assert_eq!(cursor.read_profile().unwrap(), p);
    }

    #[test]
    fn discovery_round_trips_v2() {
        let original = Discovery {
            device_info: DeviceInfo::example(),
            capabilities: CapabilityFlags::PROFILE_CONFIGURATION
                | CapabilityFlags::PROPERTY_EXCHANGE,
            maximum_sysex_size: 0x0012_3456,
            output_path_id: 0x05,
        };
        let body = MessageBody::Discovery(original.clone());
        let header = header_for(
            body.category_byte(),
            Muid::from_bits_truncate(0x0102_0304),
            Muid::BROADCAST,
        );
        let encoded = encode(&header, &body, 0);
        let decoded = decode(&encoded).unwrap().unwrap();
        assert_eq!(decoded.header.source, header.source);
        assert_eq!(decoded.header.destination, header.destination);
        assert_eq!(decoded.header.version, header.version);
        if let MessageBody::Discovery(dec) = decoded.body {
            assert_eq!(dec.capabilities, original.capabilities);
            assert_eq!(dec.maximum_sysex_size, original.maximum_sysex_size);
            assert_eq!(dec.output_path_id, original.output_path_id);
            assert_eq!(dec.device_info, original.device_info);
        } else {
            panic!("expected Discovery body");
        }
    }

    #[test]
    fn invalid_framing_returns_none() {
        let bytes = [0u8; 11];
        assert!(decode(&bytes).unwrap().is_none());
        let bytes = [0x00, 0x00, 0x00, 0x70, 0x02, 0, 0, 0, 0, 0, 0];
        assert!(decode(&bytes).unwrap().is_none());
    }

    #[test]
    fn truncated_body_returns_too_short() {
        // Enough framing but the body is too short to contain the header
        // (device_id + status + version + 2 × 4-byte MUID = 11 bytes).
        let bytes = [
            UMP_NOOP,
            0x00,
            UMP_TYPE_MIDI_CI,
            0x7F, // device_id = WholeBlock
            0x70, // status (Discovery)
            0x02, // version
            0x00,
            0x00,
        ];
        let err = decode(&bytes).unwrap_err();
        assert!(matches!(err, MidiCiError::TooShort { .. }));
    }

    #[test]
    fn profile_inquiry_round_trip() {
        let body = MessageBody::ProfileInquiry(ProfileInquiry);
        let header = header_for(body.category_byte(), Muid::random(), Muid::BROADCAST);
        let encoded = encode(&header, &body, 0);
        let decoded = decode(&encoded).unwrap().unwrap();
        assert!(matches!(decoded.body, MessageBody::ProfileInquiry(_)));
    }

    #[test]
    fn profile_inquiry_response_round_trip() {
        let body = MessageBody::ProfileInquiryResponse(ProfileInquiryResponse {
            enabled_profiles: vec![
                Profile::new([0x7E, 0x01, 0x02, 0x03, 0x04]),
                Profile::new([0x7E, 0x05, 0x06, 0x07, 0x08]),
            ],
            disabled_profiles: vec![Profile::new([0x7E, 0x09, 0x0A, 0x0B, 0x0C])],
        });
        let header = header_for(body.category_byte(), Muid::random(), Muid::BROADCAST);
        let encoded = encode(&header, &body, 0);
        let decoded = decode(&encoded).unwrap().unwrap();
        if let MessageBody::ProfileInquiryResponse(r) = decoded.body {
            assert_eq!(r.enabled_profiles.len(), 2);
            assert_eq!(r.disabled_profiles.len(), 1);
        } else {
            panic!("expected ProfileInquiryResponse");
        }
    }

    #[test]
    fn property_get_data_round_trip() {
        let body = MessageBody::PropertyGetData(PropertyGetData {
            inner: crate::message::StaticSizePropertyExchange {
                request_id: crate::types::RequestId::new(0x42).unwrap(),
                header: b"ResourceList".to_vec(),
            },
        });
        let header = header_for(body.category_byte(), Muid::random(), Muid::BROADCAST);
        let encoded = encode(&header, &body, 0);
        let decoded = decode(&encoded).unwrap().unwrap();
        if let MessageBody::PropertyGetData(d) = decoded.body {
            assert_eq!(d.inner.request_id.get(), 0x42);
            assert_eq!(d.inner.header, b"ResourceList");
        } else {
            panic!("expected PropertyGetData");
        }
    }

    #[test]
    fn encode_then_decode_loop_for_sample_categories() {
        let samples: Vec<(MessageBody, &str)> = vec![
            (
                MessageBody::Discovery(Discovery::default()),
                "Discovery",
            ),
            (
                MessageBody::ProfileInquiry(ProfileInquiry),
                "ProfileInquiry",
            ),
            (
                MessageBody::ProfileInquiryResponse(ProfileInquiryResponse::default()),
                "ProfileInquiryResponse",
            ),
            (
                MessageBody::ProfileAdded(crate::message::ProfileAdded {
                    profile: Profile::new([0x7E, 0x01, 0x02, 0x03, 0x04]),
                }),
                "ProfileAdded",
            ),
            (
                MessageBody::PropertyGetData(PropertyGetData {
                    inner: crate::message::StaticSizePropertyExchange {
                        request_id: crate::types::RequestId::new(1).unwrap(),
                        header: b"abc".to_vec(),
                    },
                }),
                "PropertyGetData",
            ),
        ];
        for (body, label) in samples {
            let header = header_for(body.category_byte(), Muid::random(), Muid::BROADCAST);
            let encoded = encode(&header, &body, 0);
            let decoded = decode(&encoded)
                .unwrap()
                .unwrap_or_else(|| panic!("decoded None for {}", label));
            assert_eq!(decoded.body.type_name(), label);
        }
    }
}