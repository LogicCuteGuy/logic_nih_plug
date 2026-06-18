//! Error types for MIDI-CI operations.

use thiserror::Error;

/// Errors that can occur while parsing, encoding, or dispatching MIDI-CI
/// messages.
#[derive(Debug, Error)]
pub enum MidiCiError {
    /// An incoming byte buffer was too short to contain even the minimal CI
    /// header (UMP type + status + version + 28-bit MUID + 28-bit destination
    /// MUID).
    #[error("MIDI-CI message is too short ({len} bytes; minimum is {min})")]
    TooShort {
        /// The actual length of the buffer that was rejected.
        len: usize,
        /// The minimum length that would have been accepted.
        min: usize,
    },

    /// The message was not addressed to this device (the destination MUID
    /// does not match our MUID and is not the broadcast address).
    #[error("MIDI-CI message addressed to a different MUID ({destination}); ours is {ours}")]
    MismatchedMuid {
        /// The destination MUID in the incoming message.
        destination: u32,
        /// Our device's MUID.
        ours: u32,
    },

    /// The message's source MUID collides with our own MUID. The spec requires
    /// the responder to regenerate its MUID in this case.
    #[error("MIDI-CI message source MUID {0} collides with our own")]
    CollidingMuid(u32),

    /// The version byte uses reserved bits that this implementation does not
    /// understand.
    #[error("MIDI-CI message version byte {byte:#04x} uses reserved bits")]
    ReservedVersion {
        /// The raw version byte that was rejected.
        byte: u8,
    },

    /// The status byte is not a recognised MIDI-CI category.
    #[error("unknown MIDI-CI category {0:#04x}")]
    UnknownCategory(u8),

    /// The body of the message was malformed (e.g. truncated, bad chunk
    /// numbers for a multi-chunk property exchange message).
    #[error("malformed MIDI-CI message body: {0}")]
    Malformed(&'static str),

    /// The category does not match the body type — typically an internal
    /// inconsistency in a hand-rolled message builder.
    #[error("MIDI-CI message body type does not match category {category:#04x}")]
    CategoryMismatch {
        /// The category that was on the envelope.
        category: u8,
    },

    /// A property-exchange transaction referenced an unknown request key.
    #[error("unknown property-exchange request key {0}")]
    UnknownRequestKey(u32),

    /// A property-exchange subscription referenced an unknown subscription
    /// key.
    #[error("unknown property-exchange subscription key {0}")]
    UnknownSubscriptionKey(u32),

    /// Tried to register a property-exchange listener with a name that is
    /// already in use.
    #[error("property-exchange listener {0:?} is already registered")]
    DuplicateListener(String),

    /// Tried to remove / look up a property-exchange listener by name that is
    /// not registered.
    #[error("property-exchange listener {0:?} is not registered")]
    UnknownListener(String),

    /// A property exchange transaction exceeded the per-device simultaneous
    /// request cap.
    #[error("property-exchange request limit {cap} reached for MUID {muid}")]
    TooManyRequests {
        /// The MUID we tried to send to.
        muid: u32,
        /// The limit reported by the responder (or our local default).
        cap: u8,
    },

    /// A generic "could not do this because of an internal invariant"
    /// catch-all. Used for things like "no transaction is active" or
    /// "subscription was already ended".
    #[error("MIDI-CI: {0}")]
    Other(&'static str),
}

/// Convenience alias used throughout the crate.
pub type MidiCiResult<T> = Result<T, MidiCiError>;