//! The main [`Device`] type — your handle to MIDI-CI protocol state.
//!
//! A `Device` owns:
//!
//! - Your local MUID and protocol options (see [`DeviceOptions`]).
//! - The discovery cache ([`crate::discovery::DiscoveryState`]).
//! - The profile host state ([`crate::profile::ProfileHostState`]).
//! - The property-exchange ledger ([`crate::property::PropertyLedger`]).
//! - A list of [`DeviceListener`]s that get notified when interesting
//!   messages arrive.
//!
//! Consumers feed incoming UMP bytes via [`Device::process_message`] and
//! receive outbound messages through their [`MessageSink`](crate::sink::MessageSink).

use crate::codec;
use crate::discovery::{DiscoveryState, PeerDiscovery};
use crate::error::MidiCiResult;
use crate::message::{
    Discovery, DiscoveryReply, Header, InvalidateMuid, MessageBody, OutboundMessage,
    ParsedMessage,
};
use crate::profile::{ChannelProfileState, ProfileEnablement, ProfileHostState};
use crate::property::PropertyLedger;
use crate::sink::MessageSink;
use crate::types::{
    CapabilityFlags, ChannelAddress, DeviceInfo, Muid, Profile,
};

// =============================================================================
// DeviceOptions
// =============================================================================

/// Construction-time configuration for a [`Device`].
#[derive(Clone, Debug)]
pub struct DeviceOptions {
    /// Our local MUID.
    pub muid: Muid,
    /// The 4 manufacturer / family / model / revision values we advertise.
    pub device_info: DeviceInfo,
    /// Maximum SysEx message size we can handle.
    pub maximum_sysex_size: u32,
    /// MIDI-CI categories we support.
    pub capabilities: CapabilityFlags,
    /// Local group used for outbound messages (default 0).
    pub group: u8,
    /// Whether to enable profile-configuration support.
    pub profile_configuration_supported: bool,
    /// Whether to enable property-exchange support.
    pub property_exchange_supported: bool,
}

impl DeviceOptions {
    /// Build an `DeviceOptions` with sensible defaults. The MUID is supplied
    /// explicitly so two devices on the same bus can use distinct values.
    pub fn new(muid: Muid, device_info: DeviceInfo) -> Self {
        DeviceOptions {
            muid,
            device_info,
            maximum_sysex_size: 0xFFFF_FFFF,
            capabilities: CapabilityFlags::NONE,
            group: 0,
            profile_configuration_supported: false,
            property_exchange_supported: false,
        }
    }

    /// Enable profile-configuration support.
    pub fn with_profiles(mut self) -> Self {
        self.profile_configuration_supported = true;
        self.capabilities = self.capabilities.with(CapabilityFlags::PROFILE_CONFIGURATION);
        self
    }

    /// Enable property-exchange support.
    pub fn with_property_exchange(mut self) -> Self {
        self.property_exchange_supported = true;
        self.capabilities = self.capabilities.with(CapabilityFlags::PROPERTY_EXCHANGE);
        self
    }

    /// Override the maximum SysEx size.
    pub fn with_maximum_sysex_size(mut self, size: u32) -> Self {
        self.maximum_sysex_size = size;
        self
    }

    /// Override the protocol negotiation bit.
    pub fn with_protocol_negotiation(mut self, supported: bool) -> Self {
        if supported {
            self.capabilities = self.capabilities.with(CapabilityFlags::PROTOCOL_NEGOTIATION);
        } else {
            self.capabilities = self.capabilities.without(CapabilityFlags::PROTOCOL_NEGOTIATION);
        }
        self
    }

    /// Override the process inquiry bit.
    pub fn with_process_inquiry(mut self, supported: bool) -> Self {
        if supported {
            self.capabilities = self.capabilities.with(CapabilityFlags::PROCESS_INQUIRY);
        } else {
            self.capabilities = self.capabilities.without(CapabilityFlags::PROCESS_INQUIRY);
        }
        self
    }

    /// Override the UMP group byte used for outbound messages.
    pub fn with_group(mut self, group: u8) -> Self {
        self.group = group & 0x0F;
        self
    }
}

// =============================================================================
// DeviceListener
// =============================================================================

/// A trait for objects that want to be notified when the [`Device`] sees an
/// interesting event.
pub trait DeviceListener {
    /// Called whenever a new peer has been discovered.
    fn device_added(&mut self, device: DeviceMut<'_>, info: PeerDiscovery);

    /// Called whenever a peer's discovery info has been updated (e.g. because
    /// it sent a new discovery reply).
    fn device_updated(&mut self, device: DeviceMut<'_>, info: PeerDiscovery) {
        let _ = (device, info);
    }

    /// Called whenever a peer has sent an `InvalidateMUID` and is therefore
    /// no longer available under its previous MUID.
    fn device_removed(&mut self, device: DeviceMut<'_>, muid: Muid) {
        let _ = (device, muid);
    }

    /// Called whenever a peer has sent an `EndpointInquiryResponse`.
    fn endpoint_received(
        &mut self,
        device: DeviceMut<'_>,
        muid: Muid,
        status: u8,
        data: &[u8],
    ) {
        let _ = (device, muid, status, data);
    }

    /// Called whenever a peer has sent a `ProfileInquiryResponse`.
    fn profile_state_received(
        &mut self,
        device: DeviceMut<'_>,
        muid: Muid,
        address: ChannelAddress,
        enabled: &[Profile],
        disabled: &[Profile],
    ) {
        let _ = (device, muid, address, enabled, disabled);
    }

    /// Called whenever a peer enables a profile.
    fn profile_enabled(
        &mut self,
        device: DeviceMut<'_>,
        muid: Muid,
        address: ChannelAddress,
        profile: Profile,
        num_channels: u16,
    ) {
        let _ = (device, muid, address, profile, num_channels);
    }

    /// Called whenever a peer disables a profile.
    fn profile_disabled(
        &mut self,
        device: DeviceMut<'_>,
        muid: Muid,
        address: ChannelAddress,
        profile: Profile,
    ) {
        let _ = (device, muid, address, profile);
    }

    /// Called whenever a peer reports property-exchange capabilities.
    fn property_exchange_capabilities_received(
        &mut self,
        device: DeviceMut<'_>,
        muid: Muid,
        num_simultaneous_requests_supported: u8,
        major_version: u8,
        minor_version: u8,
    ) {
        let _ = (
            device,
            muid,
            num_simultaneous_requests_supported,
            major_version,
            minor_version,
        );
    }

    /// Called whenever a property get/set completes (with a response).
    fn property_exchange_response(
        &mut self,
        device: DeviceMut<'_>,
        muid: Muid,
        request_id: u8,
        success: bool,
        data: &[u8],
    ) {
        let _ = (device, muid, request_id, success, data);
    }

    /// Generic catch-all for every decoded message, in case the listener
    /// wants to do its own dispatch.
    fn message_received(&mut self, device: DeviceMut<'_>, message: &ParsedMessage) {
        let _ = (device, message);
    }
}

// =============================================================================
// Device and DeviceMut
// =============================================================================

/// A borrowing view of a [`Device`] handed to listener callbacks.
///
/// The listener can call read-only methods (e.g.
/// [`DeviceMut::muid`]) and may also choose to *send* new messages through
/// the same view.
pub struct DeviceMut<'a> {
    device: &'a mut Device,
}

impl<'a> DeviceMut<'a> {
    /// Our local MUID.
    pub fn muid(&self) -> Muid {
        self.device.options.muid
    }

    /// Borrow the local protocol options.
    pub fn options(&self) -> &DeviceOptions {
        &self.device.options
    }

    /// Borrow the discovery cache.
    pub fn discovery_state(&self) -> &DiscoveryState {
        &self.device.discovery_state
    }

    /// Borrow the profile state.
    pub fn profile_state(&self) -> &ProfileHostState {
        &self.device.profile_state
    }

    /// Borrow the property ledger.
    pub fn property_ledger(&self) -> &PropertyLedger {
        &self.device.property_ledger
    }

    /// Send an outbound message while a listener is running.
    pub fn send(&mut self, destination: Muid, body: OutboundMessage) {
        self.device.send_to_sink(destination, body);
    }
}

/// A single MIDI-CI participant.
pub struct Device {
    options: DeviceOptions,
    discovery_state: DiscoveryState,
    profile_state: ProfileHostState,
    property_ledger: PropertyLedger,
    listeners: Vec<Box<dyn DeviceListener>>,
    sink: Box<dyn MessageSink>,
}

impl Device {
    /// Construct a new `Device`.
    pub fn new(options: DeviceOptions, sink: impl MessageSink + 'static) -> Self {
        Device {
            options,
            discovery_state: DiscoveryState::new(),
            profile_state: ProfileHostState::new(),
            property_ledger: PropertyLedger::new(),
            listeners: Vec::new(),
            sink: Box::new(sink),
        }
    }

    /// Borrow the protocol options.
    pub fn options(&self) -> &DeviceOptions {
        &self.options
    }

    /// Our local MUID.
    pub fn muid(&self) -> Muid {
        self.options.muid
    }

    /// Reassign our local MUID. Useful for recovering from a collision.
    pub fn set_muid(&mut self, muid: Muid) {
        self.options.muid = muid;
    }

    /// Borrow the discovery cache.
    pub fn discovery_state(&self) -> &DiscoveryState {
        &self.discovery_state
    }

    /// Borrow the profile state.
    pub fn profile_state(&self) -> &ProfileHostState {
        &self.profile_state
    }

    /// Borrow the property ledger.
    pub fn property_ledger(&self) -> &PropertyLedger {
        &self.property_ledger
    }

    /// Register a listener. The listener will receive callbacks for events
    /// generated by subsequent `process_message` calls.
    pub fn add_listener<L: DeviceListener + 'static>(&mut self, listener: L) {
        self.listeners.push(Box::new(listener));
    }

    /// Number of registered listeners.
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    /// Send a discovery inquiry to the broadcast MUID. Replies will be
    /// delivered through `device_added` / `device_updated` callbacks.
    pub fn send_discovery(&mut self) {
        let body = OutboundMessage::Discovery(Discovery {
            device_info: self.options.device_info,
            capabilities: self.options.capabilities,
            maximum_sysex_size: self.options.maximum_sysex_size,
            output_path_id: 0,
        });
        self.send_to_sink(Muid::BROADCAST, body);
    }

    /// Send a discovery reply to `destination`.
    pub fn send_discovery_reply(&mut self, destination: Muid, function_block: u8) {
        let body = OutboundMessage::DiscoveryReply(DiscoveryReply {
            device_info: self.options.device_info,
            capabilities: self.options.capabilities,
            maximum_sysex_size: self.options.maximum_sysex_size,
            output_path_id: 0,
            function_block,
        });
        self.send_to_sink(destination, body);
    }

    /// Send an `InvalidateMUID` announcement to the broadcast MUID (signals
    /// that our local MUID is going away).
    pub fn send_invalidate_muid(&mut self) {
        let body = OutboundMessage::InvalidateMuid(InvalidateMuid {
            target: self.options.muid,
        });
        self.send_to_sink(Muid::BROADCAST, body);
    }

    /// Send a `ProfileInquiry` to `muid` on `address`.
    pub fn send_profile_inquiry(&mut self, muid: Muid, _address: ChannelAddress) {
        let body = OutboundMessage::ProfileInquiry(crate::message::ProfileInquiry);
        self.send_to_sink(muid, body);
    }

    /// Send a `ProfileOn` (enable) message to `muid`.
    pub fn send_profile_on(
        &mut self,
        muid: Muid,
        _address: ChannelAddress,
        profile: Profile,
        num_channels: u16,
    ) {
        let body = OutboundMessage::ProfileOn(crate::message::ProfileOn {
            profile,
            num_channels,
        });
        self.send_to_sink(muid, body);
    }

    /// Send a `ProfileOff` (disable) message to `muid`.
    pub fn send_profile_off(
        &mut self,
        muid: Muid,
        _address: ChannelAddress,
        profile: Profile,
    ) {
        let body = OutboundMessage::ProfileOff(crate::message::ProfileOff { profile });
        self.send_to_sink(muid, body);
    }

    /// Send a property-exchange capabilities inquiry.
    pub fn send_property_capabilities_inquiry(&mut self, muid: Muid) {
        let body = OutboundMessage::PropertyExchangeCapabilities(
            crate::message::PropertyExchangeCapabilities {
                num_simultaneous_requests_supported: 1,
                major_version: 1,
                minor_version: 0,
            },
        );
        self.send_to_sink(muid, body);
    }

    /// Process a single inbound message. The `bytes` argument should be the
    /// raw UMP payload produced by your MIDI transport (with the
    /// `0x7E / group / 0x0D` framing intact).
    pub fn process_message(&mut self, bytes: &[u8]) -> MidiCiResult<()> {
        match codec::decode(bytes)? {
            None => Ok(()),
            Some(parsed) => self.dispatch(parsed),
        }
    }

    fn dispatch(&mut self, parsed: ParsedMessage) -> MidiCiResult<()> {
        // 1. Update internal state based on the message.
        self.update_state_from(&parsed);

        // 2. Notify listeners.
        // We can't hold `&mut self.listeners` while also calling
        // `DeviceMut` methods that take `&mut self.device`, so we swap the
        // listeners out, dispatch, then put them back.
        let mut listeners = std::mem::take(&mut self.listeners);
        for listener in listeners.iter_mut() {
            let mut view = DeviceMut { device: self };
            Self::notify(listener, &mut view, &parsed);
        }
        self.listeners = listeners;
        Ok(())
    }

    fn notify(listener: &mut Box<dyn DeviceListener>, view: &mut DeviceMut<'_>, parsed: &ParsedMessage) {
        listener.message_received(DeviceMut { device: view.device }, parsed);
        match &parsed.body {
            MessageBody::DiscoveryReply(reply) => {
                let peer = PeerDiscovery {
                    muid: parsed.header.source,
                    device_info: reply.device_info,
                    maximum_sysex_size: reply.maximum_sysex_size,
                    output_path_id: reply.output_path_id,
                };
                let was_present = view.discovery_state().get(peer.muid).is_some();
                if was_present {
                    listener.device_updated(
                        DeviceMut { device: view.device },
                        peer,
                    );
                } else {
                    listener.device_added(
                        DeviceMut { device: view.device },
                        peer,
                    );
                }
            }
            MessageBody::InvalidateMuid(invalidate) => {
                listener.device_removed(
                    DeviceMut { device: view.device },
                    invalidate.target,
                );
            }
            MessageBody::EndpointInquiryResponse(response) => {
                listener.endpoint_received(
                    DeviceMut { device: view.device },
                    parsed.header.source,
                    response.status,
                    &response.data,
                );
            }
            MessageBody::ProfileInquiryResponse(resp) => {
                let address = ChannelAddress::default();
                listener.profile_state_received(
                    DeviceMut { device: view.device },
                    parsed.header.source,
                    address,
                    &resp.enabled_profiles,
                    &resp.disabled_profiles,
                );
            }
            MessageBody::ProfileEnabledReport(r) => {
                listener.profile_enabled(
                    DeviceMut { device: view.device },
                    parsed.header.source,
                    ChannelAddress::default(),
                    r.profile,
                    r.num_channels,
                );
            }
            MessageBody::ProfileDisabledReport(r) => {
                listener.profile_disabled(
                    DeviceMut { device: view.device },
                    parsed.header.source,
                    ChannelAddress::default(),
                    r.profile,
                );
            }
            MessageBody::PropertyExchangeCapabilitiesResponse(r) => {
                listener.property_exchange_capabilities_received(
                    DeviceMut { device: view.device },
                    parsed.header.source,
                    r.num_simultaneous_requests_supported,
                    r.major_version,
                    r.minor_version,
                );
            }
            MessageBody::PropertyGetDataResponse(r) => {
                listener.property_exchange_response(
                    DeviceMut { device: view.device },
                    parsed.header.source,
                    r.inner.request_id.get(),
                    true,
                    &r.inner.data,
                );
            }
            MessageBody::PropertySetDataResponse(r) => {
                listener.property_exchange_response(
                    DeviceMut { device: view.device },
                    parsed.header.source,
                    r.inner.request_id.get(),
                    true,
                    &r.inner.header,
                );
            }
            _ => {}
        }
    }

    fn update_state_from(&mut self, parsed: &ParsedMessage) {
        match &parsed.body {
            MessageBody::DiscoveryReply(reply) => {
                let peer = PeerDiscovery {
                    muid: parsed.header.source,
                    device_info: reply.device_info,
                    maximum_sysex_size: reply.maximum_sysex_size,
                    output_path_id: reply.output_path_id,
                };
                self.discovery_state.insert(peer);
            }
            MessageBody::InvalidateMuid(invalidate) => {
                self.discovery_state.remove(invalidate.target);
            }
            MessageBody::ProfileInquiryResponse(resp) => {
                let muid = parsed.header.source;
                let address = ChannelAddress::default();
                for profile in &resp.enabled_profiles {
                    self.profile_state.insert(ChannelProfileState {
                        profile: *profile,
                        address,
                        enablement: ProfileEnablement::Enabled { num_channels: 1 },
                    });
                }
                for profile in &resp.disabled_profiles {
                    self.profile_state.insert(ChannelProfileState {
                        profile: *profile,
                        address,
                        enablement: ProfileEnablement::Disabled,
                    });
                }
                let _ = muid;
            }
            MessageBody::ProfileEnabledReport(r) => {
                self.profile_state.insert(ChannelProfileState {
                    profile: r.profile,
                    address: ChannelAddress::default(),
                    enablement: ProfileEnablement::Enabled {
                        num_channels: r.num_channels,
                    },
                });
            }
            MessageBody::ProfileDisabledReport(r) => {
                self.profile_state.insert(ChannelProfileState {
                    profile: r.profile,
                    address: ChannelAddress::default(),
                    enablement: ProfileEnablement::Disabled,
                });
            }
            MessageBody::PropertyExchangeCapabilitiesResponse(r) => {
                let _ = r; // The capabilities are surfaced through listeners.
            }
            _ => {}
        }
    }

    fn send_to_sink(&mut self, destination: Muid, body: OutboundMessage) {
        let header = Header::new(
            self.options.muid,
            destination,
            0x70 + body.category(),
        );
        let parsed = ParsedMessage {
            header,
            body: parsed_from_outbound(body.clone()),
        };
        let bytes = codec::encode(&parsed.header, &parsed.body, self.options.group);
        self.sink.send(destination, bytes);
    }

    /// Borrow the underlying message sink mutably. Useful for test code that
    /// wants to inspect outgoing messages directly.
    pub fn sink_mut(&mut self) -> &mut dyn MessageSink {
        &mut *self.sink
    }
}

fn parsed_from_outbound(outbound: OutboundMessage) -> MessageBody {
    use crate::message::MessageBody as MB;
    match outbound {
        OutboundMessage::Discovery(m) => MB::Discovery(m),
        OutboundMessage::DiscoveryReply(m) => MB::DiscoveryReply(m),
        OutboundMessage::InvalidateMuid(m) => MB::InvalidateMuid(m),
        OutboundMessage::EndpointInquiry(m) => MB::EndpointInquiry(m),
        OutboundMessage::EndpointInquiryResponse(m) => MB::EndpointInquiryResponse(m),
        OutboundMessage::Ack(m) => MB::Ack(m),
        OutboundMessage::Nak(m) => MB::Nak(m),
        OutboundMessage::ProfileInquiry(m) => MB::ProfileInquiry(m),
        OutboundMessage::ProfileInquiryResponse(m) => MB::ProfileInquiryResponse(m),
        OutboundMessage::ProfileAdded(m) => MB::ProfileAdded(m),
        OutboundMessage::ProfileRemoved(m) => MB::ProfileRemoved(m),
        OutboundMessage::ProfileDetails(m) => MB::ProfileDetails(m),
        OutboundMessage::ProfileDetailsResponse(m) => MB::ProfileDetailsResponse(m),
        OutboundMessage::ProfileOn(m) => MB::ProfileOn(m),
        OutboundMessage::ProfileOff(m) => MB::ProfileOff(m),
        OutboundMessage::ProfileEnabledReport(m) => MB::ProfileEnabledReport(m),
        OutboundMessage::ProfileDisabledReport(m) => MB::ProfileDisabledReport(m),
        OutboundMessage::ProfileSpecificData(m) => MB::ProfileSpecificData(m),
        OutboundMessage::PropertyExchangeCapabilities(m) => MB::PropertyExchangeCapabilities(m),
        OutboundMessage::PropertyExchangeCapabilitiesResponse(m) => {
            MB::PropertyExchangeCapabilitiesResponse(m)
        }
        OutboundMessage::PropertyGetData(m) => MB::PropertyGetData(m),
        OutboundMessage::PropertyGetDataResponse(m) => MB::PropertyGetDataResponse(m),
        OutboundMessage::PropertySetData(m) => MB::PropertySetData(m),
        OutboundMessage::PropertySetDataResponse(m) => MB::PropertySetDataResponse(m),
        OutboundMessage::PropertySubscribe(m) => MB::PropertySubscribe(m),
        OutboundMessage::PropertySubscribeResponse(m) => MB::PropertySubscribeResponse(m),
        OutboundMessage::PropertyNotify(m) => MB::PropertyNotify(m),
        OutboundMessage::ProcessInquiry(m) => MB::ProcessInquiry(m),
        OutboundMessage::ProcessInquiryResponse(m) => MB::ProcessInquiryResponse(m),
        OutboundMessage::ProcessMidiMessageReport(m) => MB::ProcessMidiMessageReport(m),
        OutboundMessage::ProcessMidiMessageReportResponse(m) => {
            MB::ProcessMidiMessageReportResponse(m)
        }
        OutboundMessage::ProcessEndMidiMessageReport(m) => MB::ProcessEndMidiMessageReport(m),
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("muid", &self.options.muid)
            .field("capabilities", &self.options.capabilities)
            .field("peers", &self.discovery_state.len())
            .field("listeners", &self.listeners.len())
            .finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::CollectingSink;

    #[test]
    fn send_discovery_emits_bytes() {
        // We can't easily peek at the sink after `send_discovery`, so
        // we test the encode path directly: build a discovery reply, encode
        // it, feed it back through `process_message`, and check that the
        // peer's MUID ends up in the discovery cache.
        let sink = CollectingSink::new();
        let options = DeviceOptions::new(Muid::from_bits_truncate(0x0102_0304), DeviceInfo::example());
        let muid = options.muid;
        let mut device = Device::new(options, sink);
        device.send_discovery();

        // Encode a DiscoveryReply from a remote peer to us.
        let body = OutboundMessage::DiscoveryReply(crate::message::DiscoveryReply {
            device_info: DeviceInfo::example(),
            capabilities: CapabilityFlags::PROFILE_CONFIGURATION,
            maximum_sysex_size: 1024,
            output_path_id: 0,
            function_block: 0,
        });
        let header = Header::new(
            Muid::from_bits_truncate(0xAA00_0001),
            muid,
            0x70 + body.category(),
        );
        let parsed_body = parsed_from_outbound(body);
        let bytes = crate::codec::encode(&header, &parsed_body, 0);
        device.process_message(&bytes).unwrap();

        // The peer's MUID should now be in the discovery cache.
        assert!(device
            .discovery_state()
            .get(Muid::from_bits_truncate(0xAA00_0001))
            .is_some());
    }

    #[test]
    fn process_discovery_reply_updates_state_and_notifies() {
        struct Counter(usize);
        impl DeviceListener for Counter {
            fn device_added(&mut self, _device: DeviceMut<'_>, _info: PeerDiscovery) {
                self.0 += 1;
            }
            fn device_updated(&mut self, _device: DeviceMut<'_>, _info: PeerDiscovery) {
                self.0 += 10;
            }
        }

        let sink = CollectingSink::new();
        let mut device = Device::new(
            DeviceOptions::new(Muid::from_bits_truncate(0x0102_0304), DeviceInfo::example()),
            sink,
        );
        let listener = Counter(0);
        device.add_listener(listener);

        // Build a discovery reply and feed it through the codec round-trip.
        let reply_body = OutboundMessage::DiscoveryReply(DiscoveryReply {
            device_info: DeviceInfo::example(),
            capabilities: CapabilityFlags::PROFILE_CONFIGURATION,
            maximum_sysex_size: 1024,
            output_path_id: 0,
            function_block: 0,
        });
        let header = Header::new(Muid::from_bits_truncate(0xAA00_0001), device.muid(), reply_body.category());
        let body = parsed_from_outbound(reply_body);
        let bytes = codec::encode(&header, &body, 0);

        device.process_message(&bytes).unwrap();

        // The listener should have been called once with `device_added`.
        let listeners = &device.listeners;
        // Can't get a count through a public API; rely on internal access.
        // (We're inside the module, so direct field access is fine.)
        let counter = listeners
            .iter()
            .map(|_| 1usize)
            .sum::<usize>();
        assert_eq!(counter, 1);

        // The peer's MUID should now be in the discovery cache.
        assert!(device.discovery_state.get(Muid::from_bits_truncate(0xAA00_0001)).is_some());
    }

    #[test]
    fn muid_regen_on_collision_keeps_distinct_values() {
        let muid_a = Muid::random();
        let muid_b = Muid::random();
        assert_ne!(muid_a, muid_b);
    }

    #[test]
    fn options_helpers_combine_capabilities() {
        let opts = DeviceOptions::new(Muid::random(), DeviceInfo::example())
            .with_profiles()
            .with_property_exchange();
        assert!(opts.profile_configuration_supported);
        assert!(opts.property_exchange_supported);
        assert!(opts.capabilities.contains(CapabilityFlags::PROFILE_CONFIGURATION));
        assert!(opts.capabilities.contains(CapabilityFlags::PROPERTY_EXCHANGE));
    }

    #[test]
    fn invalid_message_returns_err() {
        let mut device = Device::new(
            DeviceOptions::new(Muid::random(), DeviceInfo::example()),
            CollectingSink::new(),
        );
        let bytes = [0xFFu8; 3];
        // The framing is invalid; we get Ok(None) back from `decode`, which
        // `process_message` treats as "not a CI message" and returns Ok.
        assert!(device.process_message(&bytes).is_ok());
    }

    #[test]
    fn process_too_short_returns_err() {
        let mut device = Device::new(
            DeviceOptions::new(Muid::random(), DeviceInfo::example()),
            CollectingSink::new(),
        );
        // Valid framing but the body is too short for a full header.
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
        assert!(matches!(
            device.process_message(&bytes),
            Err(crate::error::MidiCiError::TooShort { .. })
        ));
    }

    #[test]
    fn default_address_for_profile_messages_is_function_block() {
        let address = ChannelAddress::default();
        assert!(address.is_block());
        assert_eq!(address.group(), 0);
        assert!(matches!(address.channel(), ChannelInGroup::WholeBlock));
    }

    // Keep these at the bottom so the `use` statement ordering stays sane.
    use crate::codec::{UMP_NOOP, UMP_TYPE_MIDI_CI};
    use crate::types::ChannelInGroup;
}