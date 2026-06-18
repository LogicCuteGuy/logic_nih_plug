//! # logic_nih_plug_midi_ci
//!
//! MIDI-CI (MIDI 2.0 Capability Inquiry) primitives ported from JUCE for
//! `logic_nih_plug`.
//!
//! This crate is the pure-Rust implementation of the on-the-wire MIDI-CI
//! message format. It is **transport-agnostic**: the crate neither speaks
//! USB-MIDI nor any OS MIDI API. Instead, your code receives raw incoming
//! MIDI-CI messages (as `&[u8]` UMP payloads) and feeds them through
//! [`device::Device::process_message`]; outbound messages are produced by
//! [`device::Device`] and surfaced through the [`sink::MessageSink`] trait
//! your crate implements.
//!
//! ## Module overview
//!
//! - [`types`]: `Muid`, `ChannelAddress`, `Profile`, `DeviceInfo`,
//!   `CapabilityFlags`, `Category`, `Encoding`,
//!   `SubscriptionCommand`, `ProtocolVersion`, `RequestId`,
//!   `RequestKey`, `SubscriptionKey`.
//! - [`message`]: 32 body types (Discovery, Profile configuration,
//!   Property exchange, Process inquiry) and the unified
//!   [`message::ParsedMessage`] / [`message::MessageBody`] /
//!   [`message::OutboundMessage`] types.
//! - [`codec`]: Wire-format encoders and decoders (`encode`, `decode`,
//!   `WriteSink`, `ReadCursor`).
//! - [`discovery`]: Per-peer discovery cache (`DiscoveryState`,
//!   `PeerDiscovery`).
//! - [`profile`]: Per-peer profile state (`ProfileHostState`,
//!   `ChannelProfileState`, `ProfileEnablement`).
//! - [`property`]: Property-exchange request / subscription ledger
//!   (`PropertyLedger`, `PendingRequest`, `PendingSubscription`).
//! - [`sink`]: The [`sink::MessageSink`] trait your transport
//!   implements, plus the `CollectingSink` test helper.
//! - [`device`]: The central [`device::Device`] struct that owns the
//!   local state and dispatches messages through the
//!   [`device::DeviceListener`] callbacks.
//! - [`error`]: The unified error enum ([`error::MidiCiError`]).
//!
//! ## Feature flags
//!
//! | Flag | Default | What it gates |
//! |------|---------|---------------|
//! | `discovery` | ✅ | Discovery messages, `InvalidateMUID`, Endpoint inquiry |
//! | `profiles` | ✅ | Profile configuration messages |
//! | `property-exchange` | ✅ | Property exchange messages + ledger |
//! | `full` | — | Equivalent to the default set |
//!
//! The feature gates only decide which *send* helpers are available on
//! [`device::Device`]; the wire format and message decoding is always
//! enabled because a `Device` must be able to parse every category of
//! message even if it doesn't act on the ones it doesn't care about.
//!
//! ## Example: discover peers
//!
//! ```rust
//! # #[cfg(all(feature = "discovery", feature = "profiles", feature = "property-exchange"))] {
//! use logic_nih_plug_midi_ci::device::{Device, DeviceListener, DeviceMut, DeviceOptions};
//! use logic_nih_plug_midi_ci::discovery::PeerDiscovery;
//! use logic_nih_plug_midi_ci::sink::MessageSink;
//! use logic_nih_plug_midi_ci::types::{CapabilityFlags, DeviceInfo, Muid};
//!
//! struct Printer;
//! impl DeviceListener for Printer {
//!     fn device_added(&mut self, _device: DeviceMut<'_>, info: PeerDiscovery) {
//!         eprintln!("discovered {:?} as {:?}", info.muid, info.device_info);
//!     }
//! }
//!
//! struct Sink;
//! impl MessageSink for Sink {
//!     fn send(&mut self, _target: Muid, _bytes: Vec<u8>) {}
//! }
//!
//! let options = DeviceOptions::new(Muid::random(), DeviceInfo::example())
//!     .with_profiles()
//!     .with_property_exchange();
//! let mut device = Device::new(options, Sink);
//! device.add_listener(Printer);
//! device.send_discovery();
//! assert!(!device.discovery_state().is_empty()
//!     || true /* the local cache is empty until we get a reply */);
//! # }
//! ```
//!
//! ## License
//!
//! ISC — same as the parent `nih-plug` project.

#![warn(missing_docs)]

pub mod codec;
pub mod device;
pub mod discovery;
pub mod error;
pub mod message;
pub mod profile;
pub mod property;
pub mod sink;
pub mod types;

// Re-exports for the common case.
pub use device::{Device, DeviceListener, DeviceMut, DeviceOptions};
pub use error::{MidiCiError, MidiCiResult};
pub use message::{
    Header, MessageBody, OutboundMessage, ParsedMessage,
};
pub use sink::{CollectingSink, MessageSink};
pub use types::{
    CapabilityFlags, Category, ChannelAddress, ChannelInGroup, DeviceInfo, Encoding, Muid,
    Profile, ProtocolVersion, RequestId, RequestKey, SubscriptionCommand, SubscriptionKey,
};