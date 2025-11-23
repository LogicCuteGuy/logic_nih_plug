//! # nih_plug_osc
//!
//! Open Sound Control support ported from JUCE.
//!
//! This crate provides:
//!
//! - **Message Types**: Core OSC data types and message structures
//! - **Sender**: Send OSC messages over UDP/TCP
//! - **Receiver**: Receive and parse OSC messages
//! - **Bundles**: Timestamped message groups
//!
//! ## Examples
//!
//! ```
//! use nih_plug_osc::{OscMessage, OscType};
//!
//! let msg = OscMessage::new("/synth/frequency", vec![OscType::Float(440.0)]);
//! assert_eq!(msg.address, "/synth/frequency");
//! ```

#![warn(missing_docs)]

pub mod error;
pub mod message;

#[cfg(feature = "sender")]
pub mod sender;

#[cfg(feature = "sender")]
pub use sender::OscSender;

#[cfg(feature = "receiver")]
pub mod receiver;

#[cfg(feature = "receiver")]
pub use receiver::OscReceiver;

#[cfg(feature = "bundles")]
pub mod bundles;

#[cfg(feature = "bundles")]
pub use bundles::{BundleBuilder, BundleUtils};

pub use error::OscError;
pub use message::{
    OscBundle, OscColor, OscMessage, OscMidi, OscPacket, OscTime, OscType,
};
