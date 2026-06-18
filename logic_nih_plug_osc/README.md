# `logic_nih_plug_osc`

OSC (Open Sound Control) sender and receiver ported from JUCE for nih-plug.

This crate provides pure-Rust implementations of JUCE's `juce_osc` module:

- **`OSCArgument`** — a sum type covering every standard OSC argument type
  (`Int32`, `Float32`, `String`, `Blob`, plus `Int64`, `Float64`, `Bool`,
  `Char`, `Nil`, `Inf`, `Colour`, `MidiMessage`, `TimeTag`, and arrays).
- **`OSCMessage`** — an OSC address plus zero or more `OSCArgument`s.
- **`OSCBundle`** — a time-tagged group of `OSCMessage`s or nested
  `OSCBundle`s.
- **`OscSender`** — synchronous UDP sender, mirror of JUCE's `OSCSender`.
- **`OscReceiver`** — thread-driven UDP receiver with typed message
  listeners, mirror of JUCE's `OSCReceiver`.

All wire-format encoding and decoding is delegated to the
[`rosc`](https://docs.rs/rosc) crate.

## Feature flags

| Feature    | Default | What it adds                                            |
|------------|---------|---------------------------------------------------------|
| `sender`   | ✅      | `OscSender` (synchronous UDP OSC sender)                |
| `receiver` | ✅      | `OscReceiver` (UDP OSC receiver with listeners)         |
| `full`     | —       | Equivalent to the default set                          |

Disable what you don't need:

```toml
[dependencies]
logic_nih_plug_osc = { version = "0", default-features = false, features = ["sender"] }
```

## Examples

### Sending OSC messages

```rust
use logic_nih_plug_osc::sender::OscSender;
use logic_nih_plug_osc::message::OSCMessage;

// 127.0.0.1:9000 is a common OSC port for a host DAW.
let sender = OscSender::connect("127.0.0.1", 9000).expect("connect");
let msg = OSCMessage::new("/amp", &[0.5_f32.into(), "lead".into()]);
sender.send(&msg).expect("send");
```

### Receiving OSC messages

```rust
use logic_nih_plug_osc::receiver::OscReceiver;
use std::sync::{Arc, Mutex};

let mut receiver = OscReceiver::connect(0).expect("bind ephemeral port");
let port = receiver.local_addr().expect("addr").port();

let seen: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
let seen_for_closure = Arc::clone(&seen);
receiver
    .add_closure("record", move |event| {
        if let Some(v) = event.message.args[0].as_float32() {
            seen_for_closure.lock().unwrap().push(v);
        }
    })
    .expect("add");

// ... another process now sends OSC to 127.0.0.1:port ...

receiver.disconnect().expect("disconnect");
```

### Sending a bundle

```rust
use logic_nih_plug_osc::sender::OscSender;
use logic_nih_plug_osc::bundle::OSCBundle;
use logic_nih_plug_osc::message::OSCMessage;
use logic_nih_plug_osc::argument::OSCTimeTag;

let sender = OscSender::connect("127.0.0.1", 9000).expect("connect");
let bundle = OSCBundle::new(OSCTimeTag::immediate());
// (build your bundle with bundle.push(...))
sender.send_bundle(&bundle).expect("send bundle");
```

## Threading

- `OscSender` is a thin wrapper around a connected `std::net::UdpSocket`,
  so it can be moved between threads and called from any thread (the
  actual `send` is a single synchronous syscall that holds the kernel's
  UDP send buffer).
- `OscReceiver` owns a worker thread named `nih-plug-osc-receiver` that
  performs all blocking I/O. Listeners are invoked on that worker thread,
  so heavy work there is fine — but blocking on something that needs
  the receiver itself will deadlock.

## Performance / allocation notes

- Every `OscSender::send` (and friends) allocates a `Vec<u8>` for the
  encoded packet. If you need to ship a fixed payload over and over,
  encode it once with `OscSender::encode` and reuse the buffer with
  `OscSender::send_encoded_packet`.
- The receiver allocates one `Vec<u8>` per datagram (reused across
  reads). Decoded messages are passed by reference to listeners.

## License

ISC — same as the parent `nih-plug` project.
