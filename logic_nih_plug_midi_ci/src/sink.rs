//! The `MessageSink` trait.
//!
//! A MIDI-CI `Device` doesn't know how to send bytes — it only knows how to
//! produce them. Consumers implement [`MessageSink`] to plug the protocol
//! layer into whatever MIDI transport they have (USB-MIDI, RTP-MIDI, virtual
//! ports, OS-level MIDI APIs).

use crate::types::Muid;

/// Receives outbound MIDI-CI messages.
///
/// The [`Device`](crate::device::Device) calls [`send`](MessageSink::send)
/// with the target MUID and the wire bytes produced by
/// [`crate::codec::encode`].
pub trait MessageSink {
    /// Send a fully encoded MIDI-CI UMP payload to the device identified by
    /// `target_muid`.
    ///
    /// The default group byte used during encoding is `0` — pass it through
    /// to the transport unchanged.
    fn send(&mut self, target_muid: Muid, bytes: Vec<u8>);
}

impl<T: FnMut(Muid, Vec<u8>)> MessageSink for T {
    fn send(&mut self, target_muid: Muid, bytes: Vec<u8>) {
        self(target_muid, bytes)
    }
}

/// A `MessageSink` that collects every outbound message in a `Vec` for
/// later inspection.
#[derive(Default, Debug, Clone)]
pub struct CollectingSink {
    messages: Vec<(Muid, Vec<u8>)>,
}

impl CollectingSink {
    /// Create an empty sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrow the collected messages.
    pub fn messages(&self) -> &[(Muid, Vec<u8>)] {
        &self.messages
    }

    /// Take the collected messages, clearing the sink.
    pub fn into_messages(self) -> Vec<(Muid, Vec<u8>)> {
        self.messages
    }

    /// Drop every collected message.
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

impl MessageSink for CollectingSink {
    fn send(&mut self, target_muid: Muid, bytes: Vec<u8>) {
        self.messages.push((target_muid, bytes));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Muid;

    #[test]
    fn collecting_sink_stores_messages() {
        let mut sink = CollectingSink::new();
        let muid = Muid::from_bits_truncate(0x0102_0304);
        sink.send(muid, vec![0x01, 0x02, 0x03]);
        sink.send(muid, vec![0x04, 0x05]);
        assert_eq!(sink.messages().len(), 2);
        let collected = sink.into_messages();
        assert_eq!(collected[0].1, vec![0x01, 0x02, 0x03]);
        assert_eq!(collected[1].1, vec![0x04, 0x05]);
    }

    #[test]
    fn closure_works_as_sink() {
        let mut collected: Vec<u8> = Vec::new();
        let mut closure = |_muid: Muid, bytes: Vec<u8>| collected.extend_from_slice(&bytes);
        closure.send(Muid::from_bits_truncate(0x0102_0304), vec![1, 2, 3]);
        closure.send(Muid::from_bits_truncate(0x0102_0304), vec![4, 5]);
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }
}