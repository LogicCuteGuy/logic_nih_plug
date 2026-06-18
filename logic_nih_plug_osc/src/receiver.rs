//! Thread-driven UDP OSC receiver.
//!
//! [`OscReceiver`] mirrors JUCE's
//! [`juce::OSCReceiver`](https://docs.juce.com/master/classOSCReceiver.html):
//! on [`connect`](OscReceiver::connect) it binds a UDP socket, spawns a
//! worker thread that reads datagrams and decodes them, and dispatches each
//! resulting [`OSCMessage`] to every registered [`MessageListener`].
//!
//! Bundles are walked recursively and each contained message is dispatched
//! in turn (the time tag is accepted but not scheduled — see
//! [`crate::argument::OSCTimeTag`] for what we store and forward).
//!
//! ```rust
//! # #[cfg(feature = "receiver")] {
//! use logic_nih_plug_osc::receiver::OscReceiver;
//!
//! let mut receiver = OscReceiver::connect(0).expect("bind ephemeral port");
//! let port = receiver.local_addr().expect("local addr").port();
//! receiver
//!     .add_closure("print", |event| {
//!         println!("{} from {}", event.message.address, event.sender);
//!     })
//!     .expect("add");
//!
//! // ... in a real app, another process now sends OSC to 127.0.0.1:port.
//!
//! receiver.disconnect().expect("disconnect");
//! # }
//! ```

use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::bundle::OSCPacket;
use crate::codec::decode_packet;
use crate::error::OscError;
use crate::message::OSCMessage;

/// A single OSC message along with the address of the peer that sent it.
///
/// The receiver creates one of these per inbound message and hands it to
/// every registered [`MessageListener`] (in registration order).
#[derive(Debug, Clone, Copy)]
pub struct OscMessageReceivedEvent<'a> {
    /// The decoded message.
    pub message: &'a OSCMessage,
    /// The socket address of the peer that sent this message.
    pub sender: SocketAddr,
}

/// Trait for objects that want to receive inbound OSC messages.
///
/// Implement this for any stateful listener (struct holding filter state,
/// per-address routing table, etc.). For one-off handlers, see
/// [`OscReceiver::add_closure`].
pub trait MessageListener: Send {
    /// Invoked by the receiver worker thread for every decoded inbound
    /// message. This runs on the receiver worker, so it's OK to do
    /// non-trivial work here — but you should not block on anything that
    /// might in turn wait for the receiver (that would deadlock).
    fn handle_message(&mut self, event: OscMessageReceivedEvent<'_>);
}

/// A [`MessageListener`] backed by a closure.
pub struct FnListener<F>(pub F);

impl<F> MessageListener for FnListener<F>
where
    F: FnMut(OscMessageReceivedEvent<'_>) + Send,
{
    fn handle_message(&mut self, event: OscMessageReceivedEvent<'_>) {
        (self.0)(event)
    }
}

type ListenerMap = Arc<Mutex<HashMap<String, Box<dyn MessageListener>>>>;

/// Multi-listener UDP OSC receiver with a background worker thread.
pub struct OscReceiver {
    /// Main socket (kept alive so [`OscReceiver::local_addr`] still works,
    /// and so dropping the receiver on `disconnect` doesn't lose the
    /// already-cloned worker socket unexpectedly).
    _socket: UdpSocket,
    /// Handlers, shared with the worker thread.
    handlers: ListenerMap,
    /// Worker thread's "should still be running" flag. Set to `false` to
    /// ask the worker to exit on its next read timeout.
    running: Arc<AtomicBool>,
    /// Join handle for the worker thread. `Some` until `disconnect` is
    /// called.
    thread: Option<JoinHandle<()>>,
}

impl OscReceiver {
    /// Binds to `0.0.0.0:port`. Use port `0` to let the OS pick an
    /// ephemeral port, then call [`OscReceiver::local_addr`] to find out
    /// what you got.
    pub fn connect(port: u16) -> Result<Self, OscError> {
        let raw = format!("0.0.0.0:{port}");
        let bind: SocketAddr = raw.parse().map_err(|e| OscError::InvalidAddress {
            input: raw,
            source: Box::new(e),
        })?;
        Self::connect_to(bind)
    }

    /// Binds to a specific [`SocketAddr`] (use this if you want to listen
    /// on a specific interface, or on IPv6).
    pub fn connect_to(bind_addr: SocketAddr) -> Result<Self, OscError> {
        let socket = UdpSocket::bind(bind_addr)?;
        // Poll every 100 ms so the worker thread can notice when we ask it
        // to stop. The trade-off is up to 100 ms latency before the
        // `disconnect` call returns, in exchange for not having to juggle
        // non-blocking reads + manual `select!`-style wakeup.
        socket.set_read_timeout(Some(Duration::from_millis(100)))?;
        let handlers: ListenerMap = Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(AtomicBool::new(true));

        let worker_socket = socket.try_clone()?;
        let worker_handlers = Arc::clone(&handlers);
        let worker_running = Arc::clone(&running);
        let thread = thread::Builder::new()
            .name("nih-plug-osc-receiver".into())
            .spawn(move || worker_loop(worker_socket, worker_handlers, worker_running))?;

        Ok(Self {
            _socket: socket,
            handlers,
            running,
            thread: Some(thread),
        })
    }

    /// Returns the local socket address this receiver is bound to.
    ///
    /// If the receiver was bound to the wildcard address (`0.0.0.0` or
    /// `::`), this returns the matching loopback address (`127.0.0.1` /
    /// `::1`) with the same port — `local_addr()` of a wildcard-bound
    /// socket is technically `0.0.0.0`, but you can't `connect` to that,
    /// which makes it useless as a peer address for `OscSender::connect_to`.
    pub fn local_addr(&self) -> Result<SocketAddr, OscError> {
        let addr = self._socket.local_addr()?;
        Ok(match addr {
            SocketAddr::V4(v4) if v4.ip().is_unspecified() => {
                SocketAddr::V4(SocketAddrV4::new([127, 0, 0, 1].into(), v4.port()))
            }
            SocketAddr::V6(v6) if v6.ip().is_unspecified() => {
                SocketAddr::V6(SocketAddrV6::new([0, 0, 0, 0, 0, 0, 0, 1].into(), v6.port(), v6.flowinfo(), v6.scope_id()))
            }
            other => other,
        })
    }

    /// Returns the number of registered listeners.
    pub fn listener_count(&self) -> usize {
        self.handlers.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// Adds a stateful listener under `name`. Names must be unique on this
    /// receiver; registering a second listener with the same name returns
    /// [`OscError::DuplicateListener`].
    pub fn add_listener<L>(&mut self, name: impl Into<String>, listener: L) -> Result<(), OscError>
    where
        L: MessageListener + 'static,
    {
        let mut handlers = self.handlers.lock().expect("handler mutex poisoned");
        let name = name.into();
        if handlers.contains_key(&name) {
            return Err(OscError::DuplicateListener { name });
        }
        handlers.insert(name, Box::new(listener));
        Ok(())
    }

    /// Convenience wrapper around [`OscReceiver::add_listener`] for
    /// closures.
    pub fn add_closure<F>(&mut self, name: impl Into<String>, handler: F) -> Result<(), OscError>
    where
        F: FnMut(OscMessageReceivedEvent<'_>) + Send + 'static,
    {
        self.add_listener(name, FnListener(handler))
    }

    /// Removes the listener registered under `name`. Returns
    /// [`OscError::UnknownListener`] if there is no such listener.
    pub fn remove_listener(&mut self, name: &str) -> Result<(), OscError> {
        let mut handlers = self.handlers.lock().expect("handler mutex poisoned");
        if handlers.remove(name).is_none() {
            return Err(OscError::UnknownListener { name: name.to_string() });
        }
        Ok(())
    }

    /// Signals the worker thread to stop and waits for it to exit.
    ///
    /// Consumes `self` by value so you can't accidentally fire messages at
    /// a disconnected receiver.
    pub fn disconnect(mut self) -> Result<(), OscError> {
        self.running.store(false, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            // The worker checks the flag on every read timeout (every
            // ~100 ms). Joining can therefore take up to one timeout
            // cycle; we don't propagate join errors because the worker
            // only panics on programmer error, which we want surfaced
            // elsewhere.
            let _ = thread.join();
        }
        Ok(())
    }
}

impl Drop for OscReceiver {
    fn drop(&mut self) {
        // Best-effort shutdown if the user forgot to call `disconnect`.
        // We don't block on the worker join here because `drop` shouldn't
        // panic.
        self.running.store(false, Ordering::Release);
    }
}

fn worker_loop(socket: UdpSocket, handlers: ListenerMap, running: Arc<AtomicBool>) {
    // 64 KiB matches the typical UDP datagram size limit; OSC packets
    // above this are illegal anyway.
    let mut buffer = vec![0u8; 65_535];
    while running.load(Ordering::Acquire) {
        match socket.recv_from(&mut buffer) {
            Ok((len, sender)) => {
                let packet = match decode_packet(&buffer[..len]) {
                    Ok(p) => p,
                    Err(_) => continue, // swallow malformed packets
                };
                let mut guard = match handlers.lock() {
                    Ok(g) => g,
                    Err(_) => continue, // poisoned — skip this round
                };
                dispatch_packet(&packet, sender, &mut guard);
            }
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock | ErrorKind::TimedOut | ErrorKind::Interrupted => continue,
                _ => break, // socket closed or fatal — exit
            },
        }
    }
}

fn dispatch_packet(
    packet: &OSCPacket,
    sender: SocketAddr,
    handlers: &mut HashMap<String, Box<dyn MessageListener>>,
) {
    match packet {
        OSCPacket::Message(msg) => {
            let event = OscMessageReceivedEvent { message: msg, sender };
            for handler in handlers.values_mut() {
                handler.handle_message(event);
            }
        }
        OSCPacket::Bundle(bundle) => {
            for nested in &bundle.packets {
                dispatch_packet(nested, sender, handlers);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::argument::OSCArgument;
    use crate::bundle::OSCBundle;
    use crate::codec::encode_packet;
    use crate::message::OSCMessage;
    use std::sync::Arc;

    /// Counts messages per address using shared state.
    struct CountingListener {
        counts: Arc<std::sync::Mutex<HashMap<String, usize>>>,
    }

    impl MessageListener for CountingListener {
        fn handle_message(&mut self, event: OscMessageReceivedEvent<'_>) {
            let mut g = self.counts.lock().unwrap();
            *g.entry(event.message.address.to_owned()).or_insert(0) += 1;
        }
    }

    /// Builds a connected UDP socket that can be used to send raw OSC bytes
    /// at the receiver. We avoid `OscSender` here so these tests don't
    /// require the `sender` feature.
    fn connected_peer(addr: SocketAddr) -> UdpSocket {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("bind peer");
        socket.connect(addr).expect("connect peer");
        socket
    }

    /// Encodes `packet` and ships it down `socket`. Synchronous.
    fn send_encoded(socket: &UdpSocket, packet: &crate::bundle::OSCPacket) {
        let bytes = encode_packet(packet).expect("encode");
        socket.send(&bytes).expect("send");
    }

    #[test]
    fn receiver_dispatches_messages() {
        let mut receiver = OscReceiver::connect(0).expect("bind");
        let addr = receiver.local_addr().expect("addr");

        let counts: Arc<std::sync::Mutex<HashMap<String, usize>>> =
            Arc::new(std::sync::Mutex::new(HashMap::new()));
        receiver
            .add_listener(
                "counter",
                CountingListener { counts: Arc::clone(&counts) },
            )
            .expect("add");

        let peer = connected_peer(addr);
        send_encoded(
            &peer,
            &crate::bundle::OSCPacket::Message(OSCMessage::new(
                "/amp",
                &[OSCArgument::Int32(1)],
            )),
        );
        send_encoded(
            &peer,
            &crate::bundle::OSCPacket::Message(OSCMessage::new(
                "/amp",
                &[OSCArgument::Int32(2)],
            )),
        );
        send_encoded(
            &peer,
            &crate::bundle::OSCPacket::Message(OSCMessage::new(
                "/foo",
                &[OSCArgument::String("x".into())],
            )),
        );

        // Give the worker a moment to dispatch.
        std::thread::sleep(Duration::from_millis(250));

        let snapshot = counts.lock().unwrap().clone();
        assert_eq!(snapshot.get("/amp"), Some(&2));
        assert_eq!(snapshot.get("/foo"), Some(&1));

        receiver.disconnect().expect("disconnect");
    }

    #[test]
    fn closure_listener_works() {
        let mut receiver = OscReceiver::connect(0).expect("bind");
        let addr = receiver.local_addr().expect("addr");

        let counter = Arc::new(std::sync::Mutex::new(0u32));
        let counter_for_closure = Arc::clone(&counter);
        receiver
            .add_closure("inc", move |_event| {
                *counter_for_closure.lock().unwrap() += 1;
            })
            .expect("add");

        let peer = connected_peer(addr);
        for addr in ["/x", "/y", "/z"] {
            send_encoded(
                &peer,
                &crate::bundle::OSCPacket::Message(OSCMessage::new(
                    addr,
                    &[OSCArgument::Nil],
                )),
            );
        }

        std::thread::sleep(Duration::from_millis(250));
        assert_eq!(*counter.lock().unwrap(), 3);

        receiver.disconnect().expect("disconnect");
    }

    #[test]
    fn duplicate_listener_names_rejected() {
        let mut receiver = OscReceiver::connect(0).expect("bind");
        receiver
            .add_closure("a", |_| {})
            .expect("first add");
        let err = receiver.add_closure("a", |_| {}).unwrap_err();
        assert!(matches!(err, OscError::DuplicateListener { .. }));
        receiver.disconnect().expect("disconnect");
    }

    #[test]
    fn removing_unknown_listener_errors() {
        let mut receiver = OscReceiver::connect(0).expect("bind");
        let err = receiver.remove_listener("nope").unwrap_err();
        assert!(matches!(err, OscError::UnknownListener { .. }));
        receiver.disconnect().expect("disconnect");
    }

    #[test]
    fn bundle_messages_are_dispatched() {
        let mut receiver = OscReceiver::connect(0).expect("bind");
        let addr = receiver.local_addr().expect("addr");

        let counts: Arc<std::sync::Mutex<HashMap<String, usize>>> =
            Arc::new(std::sync::Mutex::new(HashMap::new()));
        receiver
            .add_listener(
                "counter",
                CountingListener { counts: Arc::clone(&counts) },
            )
            .expect("add");

        let peer = connected_peer(addr);
        let bundle = OSCBundle::immediate(vec![
            OSCMessage::new("/nested/a", &[OSCArgument::Int32(1)]),
            OSCMessage::new("/nested/b", &[OSCArgument::Int32(2)]),
        ]);
        send_encoded(&peer, &crate::bundle::OSCPacket::Bundle(bundle));

        std::thread::sleep(Duration::from_millis(250));
        let snapshot = counts.lock().unwrap().clone();
        assert_eq!(snapshot.get("/nested/a"), Some(&1));
        assert_eq!(snapshot.get("/nested/b"), Some(&1));

        receiver.disconnect().expect("disconnect");
    }

    #[test]
    fn drop_without_disconnect_does_not_panic() {
        let mut receiver = OscReceiver::connect(0).expect("bind");
        receiver.add_closure("noop", |_| {}).expect("add");
        // No `disconnect` call — Drop should clear the flag and let the
        // worker exit on its own.
        drop(receiver);
        // Give the worker a beat to exit.
        std::thread::sleep(Duration::from_millis(250));
    }
}
