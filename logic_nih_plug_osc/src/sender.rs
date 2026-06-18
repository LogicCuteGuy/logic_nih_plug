//! Synchronous UDP OSC sender.
//!
//! [`OscSender`] mirrors JUCE's [`juce::OSCSender`](https://docs.juce.com/master/classOSCSender.html)
//! class: you point it at a target host and port, then push [`OSCMessage`]s
//! or [`OSCBundle`](crate::bundle::OSCBundle)s at it.
//!
//! ```rust
//! # #[cfg(feature = "sender")] {
//! use logic_nih_plug_osc::sender::OscSender;
//! use logic_nih_plug_osc::message::OSCMessage;
//! use logic_nih_plug_osc::argument::OSCArgument;
//!
//! // 127.0.0.1:9000 is a common OSC port for a host DAW.
//! let sender = OscSender::connect("127.0.0.1", 9000).expect("connect");
//! let msg = OSCMessage::new("/amp", &[OSCArgument::Float32(0.5)]);
//! sender.send(&msg).expect("send");
//! # }
//! ```

use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket};

use crate::bundle::OSCBundle;
use crate::bundle::OSCPacket;
use crate::codec::encode_packet;
use crate::error::OscError;
use crate::message::OSCMessage;

/// A blocking UDP OSC sender bound to a single peer.
///
/// Internally this holds a connected [`std::net::UdpSocket`]. Sending is
/// synchronous and allocates per-message (the encoded wire bytes). If you
/// need real-time-safe sending on a dedicated thread, wrap your message
/// construction in a `Vec<u8>` once with
/// [`OscSender::send_encoded_packet`] to avoid repeated allocations.
#[derive(Debug)]
pub struct OscSender {
    socket: UdpSocket,
    target: SocketAddr,
}

impl OscSender {
    /// Connects to the given host:port and returns a sender.
    pub fn connect(host: &str, port: u16) -> Result<Self, OscError> {
        let raw = format!("{host}:{port}");
        let target: SocketAddr = raw.parse().map_err(|e| OscError::InvalidAddress {
            input: raw,
            source: Box::new(e),
        })?;
        Self::connect_to(target)
    }

    /// Connects to the given target address and returns a sender.
    pub fn connect_to(target: SocketAddr) -> Result<Self, OscError> {
        // Bind to an OS-chosen ephemeral port on the same address family as
        // the target. This is what `UdpSocket::connect` expects.
        let local: SocketAddr = match target {
            SocketAddr::V4(_) => SocketAddr::V4(SocketAddrV4::new([0, 0, 0, 0].into(), 0)),
            SocketAddr::V6(_) => SocketAddr::V6(SocketAddrV6::new([0; 8].into(), 0, 0, 0)),
        };
        let socket = UdpSocket::bind(local)?;
        socket.connect(target)?;
        Ok(Self { socket, target })
    }

    /// Returns the target peer address.
    pub fn target(&self) -> SocketAddr {
        self.target
    }

    /// Encodes `packet` to bytes and sends it as a single UDP datagram.
    pub fn send_packet(&self, packet: &OSCPacket) -> Result<(), OscError> {
        let bytes = encode_packet(packet)?;
        self.socket.send(&bytes)?;
        Ok(())
    }

    /// Sends an [`OSCMessage`] as a single UDP datagram.
    pub fn send(&self, msg: &OSCMessage) -> Result<(), OscError> {
        self.send_packet(&OSCPacket::Message(msg.clone()))
    }

    /// Sends an [`OSCBundle`] as a single UDP datagram.
    pub fn send_bundle(&self, bundle: &OSCBundle) -> Result<(), OscError> {
        self.send_packet(&OSCPacket::Bundle(bundle.clone()))
    }

    /// Sends a pre-encoded byte buffer as a single UDP datagram. Useful if
    /// you want to skip re-encoding the same packet on every send.
    pub fn send_encoded_packet(&self, bytes: &[u8]) -> Result<(), OscError> {
        self.socket.send(bytes)?;
        Ok(())
    }

    /// Encodes a packet and returns the bytes without sending. Equivalent
    /// to feeding the packet through [`crate::codec::encode_packet`] but
    /// kept here so callers don't need to import the private `codec`
    /// module.
    pub fn encode(packet: &OSCPacket) -> Result<Vec<u8>, OscError> {
        encode_packet(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::argument::OSCArgument;
    use crate::bundle::OSCBundle;
    use std::net::UdpSocket;

    /// Spawns a one-shot UDP "listener" socket on an ephemeral port, returns
    /// the socket and the address it's bound to.
    fn one_shot_listener() -> (UdpSocket, SocketAddr) {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("bind");
        let addr = socket.local_addr().expect("local_addr");
        (socket, addr)
    }

    #[test]
    fn send_message_is_received() {
        let (listener, addr) = one_shot_listener();
        let sender = OscSender::connect_to(addr).expect("connect");

        let msg = OSCMessage::new(
            "/amp",
            &[
                OSCArgument::Int32(1),
                OSCArgument::Float32(0.5),
                OSCArgument::String("hi".into()),
            ],
        );
        sender.send(&msg).expect("send");

        let mut buf = [0u8; 1024];
        let len = listener.recv(&mut buf).expect("recv");
        let received = crate::codec::decode_packet(&buf[..len]).expect("decode");
        let received_msg = received.as_message().expect("is message");
        assert_eq!(received_msg.address, "/amp");
        assert_eq!(received_msg.args.len(), 3);
        assert_eq!(received_msg.args[0].as_int32(), Some(1));
        assert_eq!(received_msg.args[1].as_float32(), Some(0.5));
        assert_eq!(received_msg.args[2].as_string(), Some("hi"));
    }

    #[test]
    fn send_bundle_is_received() {
        let (listener, addr) = one_shot_listener();
        let sender = OscSender::connect_to(addr).expect("connect");

        let bundle = OSCBundle::immediate(vec![
            OSCMessage::new("/a", &[OSCArgument::Int32(1)]),
            OSCMessage::new("/b", &[OSCArgument::Bool(true)]),
        ]);
        sender.send_bundle(&bundle).expect("send");

        let mut buf = [0u8; 1024];
        let len = listener.recv(&mut buf).expect("recv");
        let received = crate::codec::decode_packet(&buf[..len]).expect("decode");
        let received_bundle = received.as_bundle().expect("is bundle");
        assert_eq!(received_bundle.packets.len(), 2);
    }

    #[test]
    fn invalid_host_returns_error() {
        let err = OscSender::connect("not a host", 8000).unwrap_err();
        assert!(matches!(err, OscError::InvalidAddress { .. }));
    }

    #[test]
    fn encode_helper_matches_send() {
        let (listener, addr) = one_shot_listener();
        let sender = OscSender::connect_to(addr).expect("connect");
        let msg = OSCMessage::new("/x", &[OSCArgument::Int32(1)]);
        let bytes = OscSender::encode(&OSCPacket::Message(msg.clone())).expect("encode");
        sender.send_encoded_packet(&bytes).expect("send");
        let mut buf = [0u8; 1024];
        let len = listener.recv(&mut buf).expect("recv");
        let received = crate::codec::decode_packet(&buf[..len]).expect("decode");
        assert_eq!(received.as_message().unwrap(), &msg);
    }
}
