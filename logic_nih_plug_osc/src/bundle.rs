//! OSC bundles — time-tagged groups of messages or nested bundles.
//!
//! [`OSCBundle`] is a wrapper around a list of [`OSCPacket`]s (which are
//! either [`OSCMessage`]s or nested [`OSCBundle`]s) plus an
//! [`OSCTimeTag`](crate::argument::OSCTimeTag) saying when the contents
//! should be dispatched.

use crate::argument::OSCTimeTag;
use crate::message::OSCMessage;

/// A single element of an [`OSCBundle`]. Either a message or a nested bundle.
#[derive(Debug, Clone, PartialEq)]
pub enum OSCPacket {
    /// A leaf message.
    Message(OSCMessage),
    /// A nested bundle.
    Bundle(OSCBundle),
}

impl OSCPacket {
    /// Returns true if this packet is a message.
    pub fn is_message(&self) -> bool {
        matches!(self, OSCPacket::Message(_))
    }

    /// Returns true if this packet is a bundle.
    pub fn is_bundle(&self) -> bool {
        matches!(self, OSCPacket::Bundle(_))
    }

    /// Returns a reference to the inner [`OSCMessage`] if this is a message,
    /// or `None` otherwise.
    pub fn as_message(&self) -> Option<&OSCMessage> {
        match self {
            OSCPacket::Message(m) => Some(m),
            OSCPacket::Bundle(_) => None,
        }
    }

    /// Returns a reference to the inner [`OSCBundle`] if this is a bundle,
    /// or `None` otherwise.
    pub fn as_bundle(&self) -> Option<&OSCBundle> {
        match self {
            OSCPacket::Bundle(b) => Some(b),
            OSCPacket::Message(_) => None,
        }
    }
}

impl From<OSCMessage> for OSCPacket {
    fn from(m: OSCMessage) -> Self {
        OSCPacket::Message(m)
    }
}

impl From<OSCBundle> for OSCPacket {
    fn from(b: OSCBundle) -> Self {
        OSCPacket::Bundle(b)
    }
}

/// A time-tagged container for zero or more [`OSCPacket`]s.
///
/// When a bundle is sent, the receiver is expected to dispatch its contents
/// at the time indicated by [`OSCBundle::time_tag`]. The conventional "as
/// soon as possible" marker is [`OSCTimeTag::immediate()`].
#[derive(Debug, Clone, PartialEq)]
pub struct OSCBundle {
    /// When the contents of this bundle should be dispatched.
    pub time_tag: OSCTimeTag,
    /// The messages and/or nested bundles contained in this bundle.
    pub packets: Vec<OSCPacket>,
}

impl OSCBundle {
    /// Builds a new, empty bundle scheduled at the given time tag.
    pub fn new(time_tag: OSCTimeTag) -> Self {
        Self { time_tag, packets: Vec::new() }
    }

    /// Builds a bundle scheduled "as soon as possible" containing the given
    /// packets.
    pub fn immediate<I, P>(packets: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<OSCPacket>,
    {
        Self {
            time_tag: OSCTimeTag::immediate(),
            packets: packets.into_iter().map(Into::into).collect(),
        }
    }

    /// Appends a packet to this bundle.
    pub fn push<P: Into<OSCPacket>>(&mut self, packet: P) {
        self.packets.push(packet.into());
    }

    /// Returns true if this bundle contains no packets.
    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    /// Returns the number of packets (messages and nested bundles) in this bundle.
    pub fn len(&self) -> usize {
        self.packets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::argument::OSCArgument;

    #[test]
    fn immediate_bundle_holds_messages() {
        let msg = OSCMessage::new("/x", &[OSCArgument::Int32(1)]);
        let bundle = OSCBundle::immediate(vec![msg.clone()]);
        assert!(bundle.time_tag == OSCTimeTag::immediate());
        assert_eq!(bundle.packets.len(), 1);
        assert_eq!(bundle.packets[0].as_message().unwrap(), &msg);
    }

    #[test]
    fn push_works() {
        let mut b = OSCBundle::new(OSCTimeTag::immediate());
        assert!(b.is_empty());
        b.push(OSCMessage::new("/x", &[OSCArgument::Float32(1.0)]));
        b.push(OSCBundle::immediate(std::iter::empty::<OSCPacket>()));
        assert_eq!(b.len(), 2);
        assert_eq!(b.packets[0].as_message().unwrap().args[0], OSCArgument::Float32(1.0));
        assert!(b.packets[1].is_bundle());
    }

    #[test]
    fn discriminators() {
        let msg = OSCPacket::Message(OSCMessage::from("/x"));
        let bundle = OSCPacket::Bundle(OSCBundle::immediate(vec![msg.clone()]));
        assert!(msg.is_message());
        assert!(bundle.is_bundle());
        assert_eq!(msg.as_message().unwrap().address, "/x");
        assert!(bundle.as_message().is_none());
    }
}
