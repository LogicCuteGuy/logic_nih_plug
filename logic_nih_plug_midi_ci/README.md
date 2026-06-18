# logic_nih_plug_midi_ci

MIDI-CI (MIDI 2.0 Capability Inquiry) primitives ported from
[JUCE's `juce_midi_ci` module](https://docs.juce.com/master/namespacejuce_1_1midi__ci.html)
for the `logic_nih_plug` ecosystem.

This crate is a pure-Rust implementation of the on-the-wire MIDI-CI message
format. It is **transport-agnostic**: the crate neither speaks USB-MIDI nor any
OS MIDI API. Instead, your code receives raw incoming MIDI-CI messages (as
`&[u8]` SysEx blobs or 64-bit UMPs) and feeds them through
[`device::Device::process_message`]; outbound messages are produced by
[`device::Device`] and surfaced through the [`sink::MessageSink`] trait your
crate implements. This matches how JUCE's `Device` separates protocol parsing
from the byte-level transport.

## What's included

| Module                                | Purpose                                                                       |
|---------------------------------------|-------------------------------------------------------------------------------|
| [`types`](src/types.rs)               | Core domain types — `Muid`, `ChannelAddress`, `Profile`, `DeviceInfo`, …      |
| [`message`](src/message.rs)           | Every MIDI-CI message body type (Discovery, Profile*, PropertyExchange*, …)   |
| [`codec`](src/codec.rs)               | Wire-format encoders / decoders (SysEx + 64-bit UMP form)                     |
| [`discovery`](src/discovery.rs)       | Discovery + endpoint discovery (gated behind the `discovery` feature)         |
| [`profile`](src/profile.rs)           | Profile configuration state + helpers (gated behind the `profiles` feature)   |
| [`property`](src/property.rs)         | Property exchange request / subscription accounting (gated behind `property-exchange`) |
| [`sink`](src/sink.rs)                 | The `MessageSink` trait your transport implements                            |
| [`device`](src/device.rs)             | `Device` — the central object that processes incoming messages & dispatches  |
| [`error`](src/error.rs)               | The unified error enum (`MidiCiError`)                                        |

## Feature flags

| Flag                | Default | What it adds                                                                 |
|---------------------|---------|------------------------------------------------------------------------------|
| `discovery`         | ✅      | Discovery message handling, `Muid` regeneration on collision                  |
| `profiles`          | ✅      | Profile configuration support (PE/PI/PE stream for profiles)                 |
| `property-exchange` | ✅      | Property exchange support (Get/Set/Subscribe/Notify + chunked reassembly)    |
| `full`              | —       | Equivalent to the default set                                                |

```toml
[dependencies]
# Receive only profile configuration messages:
logic_nih_plug_midi_ci = { version = "0", default-features = false, features = ["profiles"] }
```

## Quick start

```rust
use logic_nih_plug_midi_ci::device::{Device, DeviceListener, DeviceOptions};
use logic_nih_plug_midi_ci::message::Discovery;
use logic_nih_plug_midi_ci::sink::MessageSink;
use logic_nih_plug_midi_ci::types::{CapabilityFlags, DeviceInfo, Muid};

struct StderrSink;
impl MessageSink for StderrSink {
    fn send(&mut self, _muid: Muid, bytes: Vec<u8>) {
        eprintln!("would transmit {} bytes", bytes.len());
    }
}

struct Listener;
impl DeviceListener for Listener {
    fn device_added(&mut self, _device: Device, info: DeviceInfo) {
        eprintln!("discovered: {:?}", info);
    }
}

let mut device = Device::new(
    DeviceOptions::new(Muid::random(), DeviceInfo::example())
        .with_capabilities(CapabilityFlags::default())
        .with_discovery()
        .with_profiles()
        .with_property_exchange(),
    StderrSink,
);
device.add_listener(Listener);
device.send_discovery();
```

## License

ISC — same as the parent `nih-plug` project.