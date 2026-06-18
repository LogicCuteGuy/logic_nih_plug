//! # logic_nih_plug_osc
//!
//! OSC (Open Sound Control) sender and receiver ported from JUCE for nih-plug.
//!
//! This crate provides pure-Rust implementations of JUCE's
//! [`juce_osc`](https://docs.juce.com/master/classOSCSender.html) module:
//!
//! - **[`argument::OSCArgument`]** — a sum type covering every standard OSC
//!   argument type (`Int32`, `Float32`, `String`, `Blob`, plus `Int64`,
//!   `Float64`, `Bool`, `Char`, `Nil`, `Inf`, `Colour`, `MidiMessage`,
//!   `TimeTag`, and arrays).
//! - **[`message::OSCMessage`]** — an OSC address plus zero or more
//!   `OSCArgument`s.
//! - **[`bundle::OSCBundle`]** — a time-tagged group of `OSCMessage`s or
//!   nested `OSCBundle`s (also re-exported as the [`OSCPacket`] enum).
//! - **[`sender::OscSender`]** — synchronous UDP sender, mirror of
//!   JUCE's `OSCSender`.
//! - **[`receiver::OscReceiver`]** — thread-driven UDP receiver with
//!   typed message listeners, mirror of JUCE's `OSCReceiver`.
//!
//! All wire-format encoding and decoding is delegated to the
//! [`rosc`](https://docs.rs/rosc) crate.
//!
//! ## Feature flags
//!
//! | Feature | Default | What it adds |
//! |---|---|---|
//! | `sender` | ✅ | [`sender::OscSender`] (synchronous UDP OSC sender) |
//! | `receiver` | ✅ | [`receiver::OscReceiver`] (UDP OSC receiver with listeners) |
//! | `full` | — | Equivalent to the default set |
//!
//! Disable what you don't need:
//!
//! ```toml
//! [dependencies]
//! logic_nih_plug_osc = { version = "0", default-features = false, features = ["sender"] }
//! ```
//!
//! ## Example: send and receive in one process
//!
//! ```rust
//! # #[cfg(all(feature = "sender", feature = "receiver"))] {
//! use logic_nih_plug_osc::receiver::OscReceiver;
//! use logic_nih_plug_osc::sender::OscSender;
//! use logic_nih_plug_osc::message::OSCMessage;
//! use logic_nih_plug_osc::argument::OSCArgument;
//! use std::sync::{Arc, Mutex};
//!
//! // Bind a receiver on an ephemeral port.
//! let mut receiver = OscReceiver::connect(0).expect("bind");
//! let peer = receiver.local_addr().expect("addr");
//!
//! // Record whatever shows up at /amp.
//! let seen: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
//! let seen_for_closure = Arc::clone(&seen);
//! receiver
//!     .add_closure("record", move |event| {
//!         if let Some(v) = event.message.args[0].as_float32() {
//!             seen_for_closure.lock().unwrap().push(v);
//!         }
//!     })
//!     .expect("add");
//!
//! // Send two messages.
//! let sender = OscSender::connect_to(peer).expect("sender");
//! sender
//!     .send(&OSCMessage::new("/amp", &[OSCArgument::from(0.25_f32)]))
//!     .expect("send 1");
//! sender
//!     .send(&OSCMessage::new("/amp", &[OSCArgument::from(0.75_f32)]))
//!     .expect("send 2");
//!
//! std::thread::sleep(std::time::Duration::from_millis(250));
//! receiver.disconnect().expect("disconnect");
//!
//! assert_eq!(*seen.lock().unwrap(), vec![0.25, 0.75]);
//! # }
//! ```
//!
//! ## Threading
//!
//! - [`sender::OscSender`] is a thin wrapper around a connected
//!   [`std::net::UdpSocket`], so it can be moved between threads and
//!   called from any thread (but the actual `send` is a single
//!   synchronous syscall that holds the kernel's UDP send buffer).
//! - [`receiver::OscReceiver`] owns a worker thread named
//!   `nih-plug-osc-receiver` that performs all blocking I/O. Listeners
//!   are invoked on that worker thread, so heavy work there is fine —
//!   but blocking on something that needs the receiver itself will
//!   deadlock.
//!
//! ## Performance / allocation notes
//!
//! - Every [`sender::OscSender::send`] (and friends) allocates a
//!   `Vec<u8>` for the encoded packet. If you need to ship a fixed
//!   payload over and over, encode it once with
//!   [`sender::OscSender::encode`] and reuse the buffer with
//!   [`sender::OscSender::send_encoded_packet`].
//! - The receiver allocates one `Vec<u8>` per datagram (reused across
//!   reads). Decoded messages are passed by reference to listeners.
//!
//! ## License
//!
//! ISC — same as the parent `nih-plug` project.

#![warn(missing_docs)]

pub mod argument;
pub mod bundle;
pub mod error;
pub mod message;

#[cfg(feature = "sender")]
pub mod sender;

#[cfg(feature = "receiver")]
pub mod receiver;

#[cfg(any(feature = "sender", feature = "receiver"))]
mod codec;

pub use argument::{OSCArgument, OSCColour, OSCMidiMessage, OSCTimeTag};
pub use bundle::{OSCBundle, OSCPacket};
pub use error::OscError;
pub use message::OSCMessage;
