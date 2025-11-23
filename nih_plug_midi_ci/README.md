# nih_plug_midi_ci

MIDI Capability Inquiry (MIDI-CI) support ported from JUCE for nih-plug.

## Overview

This crate provides pure Rust implementations of the MIDI-CI protocol, which is part of the MIDI 2.0 specification. MIDI-CI enables MIDI devices to discover each other's capabilities and exchange configuration information.

## Features

✅ **Device Discovery**: Broadcast discovery inquiries and respond to them
✅ **Capability Queries**: Query device capabilities (profiles, property exchange, process inquiry)
✅ **Profile Management**: Query, enable, and disable MIDI-CI profiles
✅ **Property Exchange**: Get and set device properties using JSON
✅ **Endpoint Information**: Query detailed endpoint information from devices

## Implemented Requirements

This implementation satisfies all MIDI-CI requirements (28.1-28.5):

- **28.1** - Capability queries using MIDI-CI protocol ✅
- **28.2** - Profile negotiation (enable/disable profiles) ✅
- **28.3** - Property exchange (get/set device properties) ✅
- **28.4** - MIDI-CI message parsing and generation ✅
- **28.5** - Device discovery (broadcast/respond to discovery messages) ✅

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
nih_plug_midi_ci = "0.0.0"
```

### Device Discovery Example

```rust
use nih_plug_midi_ci::{
    discovery::{DeviceCapabilities, DiscoveryInquiry},
    protocol::{DeviceInfo, Muid},
};

// Create your device info
let my_muid = Muid::new(0x1234567).unwrap();
let device_info = DeviceInfo::new(
    vec![0x7D],  // Manufacturer ID
    0x1234,      // Family
    0x5678,      // Model
    0x010000,    // Revision
);

// Specify device capabilities
let capabilities = DeviceCapabilities::all();

// Create and send a discovery inquiry
let inquiry = DiscoveryInquiry::new(my_muid, device_info, capabilities);
let message = inquiry.to_message();
let sysex = message.to_sysex();

// Send sysex over MIDI...
```

### Profile Management Example

```rust
use nih_plug_midi_ci::{
    profiles::{ProfileInquiry, SetProfileOn},
    protocol::{Muid, ProfileId},
};

// Query available profiles
let inquiry = ProfileInquiry::new(
    Muid::new(0x1234567).unwrap(),
    Muid::new(0x7654321).unwrap(),
);

// Enable a specific profile
let profile_id = ProfileId::new([0x7E, 0x00, 0x01, 0x00, 0x00]);
let set_on = SetProfileOn::new(
    Muid::new(0x1234567).unwrap(),
    Muid::new(0x7654321).unwrap(),
    profile_id,
);
```

### Property Exchange Example

```rust
use nih_plug_midi_ci::{
    properties::{PropertyGetData, PropertySetData},
    protocol::Muid,
};

// Get a property
let get_data = PropertyGetData::new(
    Muid::new(0x1234567).unwrap(),
    Muid::new(0x7654321).unwrap(),
    1, // Request ID
    "/device/name".to_string(),
);

// Set a property
let set_data = PropertySetData::new(
    Muid::new(0x1234567).unwrap(),
    Muid::new(0x7654321).unwrap(),
    2, // Request ID
    "/device/volume".to_string(),
    b"{\"volume\":75}".to_vec(),
);
```

## MIDI-CI Protocol

MIDI-CI uses System Exclusive (SysEx) messages with the following structure:

```
F0 7E <device ID> 0D <sub-ID #2> <version> <MUID src> <MUID dest> [data] F7
```

- `F0`: SysEx start
- `7E`: Universal SysEx ID
- `<device ID>`: Target device (0x7F for all)
- `0D`: MIDI-CI Sub-ID #1
- `<sub-ID #2>`: Message type
- `<version>`: MIDI-CI version (currently 0x02)
- `<MUID src>`: Source MUID (4 bytes, 7-bit encoding)
- `<MUID dest>`: Destination MUID (4 bytes, 7-bit encoding)
- `[data]`: Message-specific payload
- `F7`: SysEx end

## Examples

Run the comprehensive demo to see all features in action:

```bash
cargo run --example midi_ci_demo --features full
```

This demonstrates:
- Device discovery with capability queries
- Endpoint information exchange
- Profile negotiation (query, enable, disable)
- Property exchange (capabilities, get, set)
- Message round-trip verification

## Cargo Features

The crate supports the following cargo features:

- `discovery` (default): Device discovery and capability queries
- `profiles` (default): Profile management
- `properties` (default): Property exchange
- `protocol`: Protocol negotiation
- `full`: Enable all features

## License

ISC License - see LICENSE file for details.

## References

- [MIDI 2.0 Specification](https://www.midi.org/specifications)
- [MIDI-CI Specification](https://www.midi.org/specifications/midi-ci-specifications)
- [JUCE Framework](https://juce.com/)
