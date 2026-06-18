//! OSC messages — an address plus zero or more arguments.
//!
//! [`OSCMessage`] is the unit of OSC communication. It is also the leaf node
//! inside an [`OSCBundle`](crate::bundle::OSCBundle).
//!
//! ```rust
//! use logic_nih_plug_osc::message::OSCMessage;
//! use logic_nih_plug_osc::argument::OSCArgument;
//!
//! // Build with literal args using the From impls on OSCArgument.
//! let msg = OSCMessage::new(
//!     "/amp",
//!     &[OSCArgument::from(0.5_f32), OSCArgument::from("lead")],
//! );
//! assert_eq!(msg.address, "/amp");
//! assert_eq!(msg.args.len(), 2);
//! assert_eq!(msg.args[0], OSCArgument::Float32(0.5));
//! assert_eq!(msg.args[1], OSCArgument::String("lead".into()));
//! ```

use crate::argument::OSCArgument;

/// An OSC message: an OSC address pattern (e.g. `/mixer/1/amp`) followed by
/// zero or more [`OSCArgument`] values.
///
/// An address must start with `/` per the OSC 1.0 spec, but this crate
/// doesn't validate that for you — the receiver side uses rosc's
/// [`rosc::address::Matcher`] to match against patterns, and the encoder
/// only requires a non-empty string.
#[derive(Debug, Clone, PartialEq)]
pub struct OSCMessage {
    /// The OSC address this message is targeting. Always starts with `/` for
    /// well-formed messages.
    pub address: String,
    /// The arguments carried by this message.
    pub args: Vec<OSCArgument>,
}

impl OSCMessage {
    /// Builds a new OSC message with the given address and arguments.
    ///
    /// `args` is any `IntoIterator` whose items are convertible into
    /// [`OSCArgument`]. You can pass:
    ///
    /// - `vec![OSCArgument::Int32(1), OSCArgument::Float32(0.5)]` — owned
    /// - `&[OSCArgument::Int32(1), OSCArgument::Float32(0.5)]` — borrowed slice
    /// - `vec![OSCArgument::from(1_i32), OSCArgument::from(0.5_f32)]` — converted scalars
    ///
    /// ```rust
    /// use logic_nih_plug_osc::message::OSCMessage;
    /// use logic_nih_plug_osc::argument::OSCArgument;
    /// let msg = OSCMessage::new("/foo", &[OSCArgument::Int32(1), OSCArgument::Float32(0.25)]);
    /// # let _ = msg;
    /// ```
    pub fn new<A>(address: impl Into<String>, args: A) -> Self
    where
        A: IntoIterator,
        A::Item: Into<OSCArgument>,
    {
        Self {
            address: address.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns the OSC type tag string for the arguments, in order. Returns
    /// the empty string if there are no arguments.
    pub fn type_tag_string(&self) -> String {
        self.args.iter().map(OSCArgument::type_tag).collect()
    }

    /// Returns true if this message carries no arguments.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Returns the number of arguments carried by this message.
    pub fn len(&self) -> usize {
        self.args.len()
    }
}

impl From<&str> for OSCMessage {
    fn from(s: &str) -> Self {
        OSCMessage {
            address: s.to_owned(),
            args: Vec::new(),
        }
    }
}

impl From<String> for OSCMessage {
    fn from(s: String) -> Self {
        OSCMessage { address: s, args: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::argument::OSCArgument;

    #[test]
    fn builds_with_args() {
        let msg = OSCMessage::new(
            "/mixer/1/amp",
            &[OSCArgument::Float32(0.5), OSCArgument::String("L".into())],
        );
        assert_eq!(msg.address, "/mixer/1/amp");
        assert_eq!(msg.args.len(), 2);
        assert_eq!(msg.type_tag_string(), "fs");
    }

    #[test]
    fn builds_with_scalar_args() {
        // Type-annotated `.into()`-equivalent so the compiler can pick
        // `OSCArgument` out of the many `From<i32>` impls in scope.
        let args: Vec<OSCArgument> = vec![
            OSCArgument::from(1_i32),
            OSCArgument::from(2_i64),
            OSCArgument::from(0.25_f32),
            OSCArgument::from(true),
            OSCArgument::from("hi"),
        ];
        let msg = OSCMessage::new("/x", args);
        assert_eq!(
            msg.args,
            vec![
                OSCArgument::Int32(1),
                OSCArgument::Int64(2),
                OSCArgument::Float32(0.25),
                OSCArgument::Bool(true),
                OSCArgument::String("hi".into()),
            ]
        );
    }

    #[test]
    fn empty_message_from_str() {
        let msg: OSCMessage = "/foo".into();
        assert_eq!(msg.address, "/foo");
        assert!(msg.is_empty());
        assert_eq!(msg.type_tag_string(), "");
    }
}
