//! Error types for OSC operations.

use thiserror::Error;

/// Errors that can occur in [`crate::sender::OscSender`] and
/// [`crate::receiver::OscReceiver`] operations.
#[derive(Debug, Error)]
pub enum OscError {
    /// A UDP socket could not be bound, connected, or read from.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A peer address could not be parsed.
    #[error("invalid OSC peer address {input:?}: {source}")]
    InvalidAddress {
        /// The unparseable string.
        input: String,
        /// The underlying parse error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// An incoming UDP datagram could not be decoded as a valid OSC packet.
    #[error("failed to decode OSC packet: {0}")]
    Decode(String),

    /// An outgoing OSC packet could not be encoded.
    #[error("failed to encode OSC packet: {0}")]
    Encode(String),

    /// An OSC address or address pattern was invalid.
    #[error("invalid OSC address {address:?}: {reason}")]
    InvalidAddressPattern {
        /// The offending address string.
        address: String,
        /// Why it was rejected.
        reason: &'static str,
    },

    /// Tried to add a listener with a name that's already registered on this
    /// receiver.
    #[error("listener {name:?} is already registered on this receiver")]
    DuplicateListener {
        /// The conflicting name.
        name: String,
    },

    /// Tried to remove or look up a listener by name that isn't registered.
    #[error("listener {name:?} is not registered on this receiver")]
    UnknownListener {
        /// The missing name.
        name: String,
    },

    /// The receiver worker thread is not running (it never started or has
    /// already exited).
    #[error("OSC receiver worker thread is not running")]
    ReceiverNotRunning,
}
