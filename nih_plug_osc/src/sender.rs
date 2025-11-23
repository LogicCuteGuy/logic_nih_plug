//! OSC sender implementation.
//!
//! Provides functionality to send OSC messages over UDP or TCP.
//!
//! # Examples
//!
//! ## UDP Sender
//!
//! ```no_run
//! use nih_plug_osc::{OscSender, OscMessage, OscType};
//!
//! let sender = OscSender::new_udp("127.0.0.1:9000").unwrap();
//! let msg = OscMessage::new("/test", vec![OscType::Float(440.0)]);
//! sender.send(&msg).unwrap();
//! ```
//!
//! ## TCP Sender
//!
//! ```no_run
//! use nih_plug_osc::{OscSender, OscMessage, OscType};
//!
//! let sender = OscSender::new_tcp("127.0.0.1:9000").unwrap();
//! let msg = OscMessage::new("/synth/note", vec![
//!     OscType::Int(60),
//!     OscType::Float(0.8),
//! ]);
//! sender.send(&msg).unwrap();
//! ```

use crate::error::OscError;
use crate::message::{OscBundle, OscMessage, OscPacket, OscType};
use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs, UdpSocket};

/// OSC sender that can send messages over UDP or TCP.
pub struct OscSender {
    transport: Transport,
}

enum Transport {
    Udp(UdpSocket),
    Tcp(TcpStream),
}

impl OscSender {
    /// Creates a new UDP sender connected to the specified address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::OscSender;
    ///
    /// let sender = OscSender::new_udp("127.0.0.1:9000").unwrap();
    /// ```
    pub fn new_udp<A: ToSocketAddrs>(addr: A) -> Result<Self, OscError> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(addr)?;
        Ok(Self {
            transport: Transport::Udp(socket),
        })
    }

    /// Creates a new TCP sender connected to the specified address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::OscSender;
    ///
    /// let sender = OscSender::new_tcp("127.0.0.1:9000").unwrap();
    /// ```
    pub fn new_tcp<A: ToSocketAddrs>(addr: A) -> Result<Self, OscError> {
        let stream = TcpStream::connect(addr)?;
        Ok(Self {
            transport: Transport::Tcp(stream),
        })
    }

    /// Sends an OSC message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::{OscSender, OscMessage, OscType};
    ///
    /// let sender = OscSender::new_udp("127.0.0.1:9000").unwrap();
    /// let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
    /// sender.send(&msg).unwrap();
    /// ```
    pub fn send(&self, message: &OscMessage) -> Result<(), OscError> {
        let packet = OscPacket::Message(message.clone());
        self.send_packet(&packet)
    }

    /// Sends an OSC packet (message or bundle).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::{OscSender, OscPacket, OscMessage, OscType};
    ///
    /// let sender = OscSender::new_udp("127.0.0.1:9000").unwrap();
    /// let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
    /// let packet = OscPacket::Message(msg);
    /// sender.send_packet(&packet).unwrap();
    /// ```
    pub fn send_packet(&self, packet: &OscPacket) -> Result<(), OscError> {
        let encoded = encode_packet(packet)?;
        
        match &self.transport {
            Transport::Udp(socket) => {
                socket.send(&encoded)?;
            }
            Transport::Tcp(stream) => {
                // TcpStream::write_all works with &TcpStream
                stream.try_clone()?.write_all(&encoded)?;
            }
        }
        
        Ok(())
    }

    /// Sends an OSC bundle.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::{OscSender, OscBundle, OscMessage, OscType, OscTime};
    ///
    /// let sender = OscSender::new_udp("127.0.0.1:9000").unwrap();
    /// let mut bundle = OscBundle::new(OscTime::immediate());
    /// bundle.add_message(OscMessage::new("/test1", vec![OscType::Int(1)]));
    /// bundle.add_message(OscMessage::new("/test2", vec![OscType::Int(2)]));
    /// sender.send_bundle(&bundle).unwrap();
    /// ```
    pub fn send_bundle(&self, bundle: &OscBundle) -> Result<(), OscError> {
        let packet = OscPacket::Bundle(bundle.clone());
        self.send_packet(&packet)
    }
}

/// Encodes an OSC packet into bytes.
#[doc(hidden)]
pub fn encode_packet(packet: &OscPacket) -> Result<Vec<u8>, OscError> {
    match packet {
        OscPacket::Message(msg) => encode_message(msg),
        OscPacket::Bundle(bundle) => encode_bundle(bundle),
    }
}

/// Encodes an OSC message into bytes.
fn encode_message(message: &OscMessage) -> Result<Vec<u8>, OscError> {
    let mut buffer = Vec::new();
    
    // Encode address pattern
    encode_string(&mut buffer, &message.address);
    
    // Encode type tag string
    let mut type_tags = String::from(",");
    for arg in &message.arguments {
        type_tags.push(get_type_tag(arg));
    }
    encode_string(&mut buffer, &type_tags);
    
    // Encode arguments
    for arg in &message.arguments {
        encode_argument(&mut buffer, arg)?;
    }
    
    Ok(buffer)
}

/// Encodes an OSC bundle into bytes.
fn encode_bundle(bundle: &OscBundle) -> Result<Vec<u8>, OscError> {
    let mut buffer = Vec::new();
    
    // Bundle identifier
    encode_string(&mut buffer, "#bundle");
    
    // Time tag
    buffer.extend_from_slice(&bundle.time_tag.seconds.to_be_bytes());
    buffer.extend_from_slice(&bundle.time_tag.fractional.to_be_bytes());
    
    // Encode each packet with size prefix
    for packet in &bundle.packets {
        let packet_data = encode_packet(packet)?;
        buffer.extend_from_slice(&(packet_data.len() as i32).to_be_bytes());
        buffer.extend_from_slice(&packet_data);
    }
    
    Ok(buffer)
}

/// Gets the OSC type tag character for a given argument.
fn get_type_tag(arg: &OscType) -> char {
    match arg {
        OscType::Int(_) => 'i',
        OscType::Float(_) => 'f',
        OscType::String(_) => 's',
        OscType::Blob(_) => 'b',
        OscType::Long(_) => 'h',
        OscType::Double(_) => 'd',
        OscType::Time(_) => 't',
        OscType::Char(_) => 'c',
        OscType::Color(_) => 'r',
        OscType::Midi(_) => 'm',
        OscType::True => 'T',
        OscType::False => 'F',
        OscType::Nil => 'N',
        OscType::Impulse => 'I',
        OscType::Array(_) => '[',
    }
}

/// Encodes an OSC argument into the buffer.
fn encode_argument(buffer: &mut Vec<u8>, arg: &OscType) -> Result<(), OscError> {
    match arg {
        OscType::Int(i) => {
            buffer.extend_from_slice(&i.to_be_bytes());
        }
        OscType::Float(f) => {
            buffer.extend_from_slice(&f.to_be_bytes());
        }
        OscType::String(s) => {
            encode_string(buffer, s);
        }
        OscType::Blob(b) => {
            // Size prefix
            buffer.extend_from_slice(&(b.len() as i32).to_be_bytes());
            // Data
            buffer.extend_from_slice(b);
            // Padding to 4-byte boundary
            let padding = (4 - (b.len() % 4)) % 4;
            buffer.extend_from_slice(&vec![0u8; padding]);
        }
        OscType::Long(l) => {
            buffer.extend_from_slice(&l.to_be_bytes());
        }
        OscType::Double(d) => {
            buffer.extend_from_slice(&d.to_be_bytes());
        }
        OscType::Time(t) => {
            buffer.extend_from_slice(&t.seconds.to_be_bytes());
            buffer.extend_from_slice(&t.fractional.to_be_bytes());
        }
        OscType::Char(c) => {
            // Char is encoded as 32-bit value
            buffer.extend_from_slice(&(*c as u32).to_be_bytes());
        }
        OscType::Color(c) => {
            buffer.push(c.red);
            buffer.push(c.green);
            buffer.push(c.blue);
            buffer.push(c.alpha);
        }
        OscType::Midi(m) => {
            buffer.push(m.port);
            buffer.push(m.status);
            buffer.push(m.data1);
            buffer.push(m.data2);
        }
        OscType::True | OscType::False | OscType::Nil | OscType::Impulse => {
            // These types have no data, only type tag
        }
        OscType::Array(arr) => {
            // Array start marker already in type tags
            for item in arr {
                encode_argument(buffer, item)?;
            }
            // Array end marker
            buffer.push(b']');
        }
    }
    Ok(())
}

/// Encodes a string with null termination and padding to 4-byte boundary.
fn encode_string(buffer: &mut Vec<u8>, s: &str) {
    buffer.extend_from_slice(s.as_bytes());
    buffer.push(0); // Null terminator
    
    // Pad to 4-byte boundary
    let padding = (4 - (buffer.len() % 4)) % 4;
    buffer.extend_from_slice(&vec![0u8; padding]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{OscColor, OscMidi, OscTime};

    #[test]
    fn test_encode_string() {
        let mut buffer = Vec::new();
        encode_string(&mut buffer, "test");
        // "test\0" = 5 bytes, needs 3 bytes padding to reach 8
        assert_eq!(buffer.len(), 8);
        assert_eq!(&buffer[0..4], b"test");
        assert_eq!(buffer[4], 0); // null terminator
    }

    #[test]
    fn test_encode_int() {
        let mut buffer = Vec::new();
        encode_argument(&mut buffer, &OscType::Int(42)).unwrap();
        assert_eq!(buffer.len(), 4);
        assert_eq!(i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]), 42);
    }

    #[test]
    fn test_encode_float() {
        let mut buffer = Vec::new();
        encode_argument(&mut buffer, &OscType::Float(3.14)).unwrap();
        assert_eq!(buffer.len(), 4);
        let value = f32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        assert!((value - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_encode_string_arg() {
        let mut buffer = Vec::new();
        encode_argument(&mut buffer, &OscType::String("hello".to_string())).unwrap();
        // "hello\0" = 6 bytes, needs 2 bytes padding to reach 8
        assert_eq!(buffer.len(), 8);
        assert_eq!(&buffer[0..5], b"hello");
    }

    #[test]
    fn test_encode_blob() {
        let mut buffer = Vec::new();
        let data = vec![1, 2, 3, 4, 5];
        encode_argument(&mut buffer, &OscType::Blob(data)).unwrap();
        // 4 bytes size + 5 bytes data + 3 bytes padding = 12
        assert_eq!(buffer.len(), 12);
        assert_eq!(i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]), 5);
        assert_eq!(&buffer[4..9], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_encode_long() {
        let mut buffer = Vec::new();
        encode_argument(&mut buffer, &OscType::Long(1234567890)).unwrap();
        assert_eq!(buffer.len(), 8);
    }

    #[test]
    fn test_encode_double() {
        let mut buffer = Vec::new();
        encode_argument(&mut buffer, &OscType::Double(3.14159265359)).unwrap();
        assert_eq!(buffer.len(), 8);
    }

    #[test]
    fn test_encode_time() {
        let mut buffer = Vec::new();
        let time = OscTime::new(100, 200);
        encode_argument(&mut buffer, &OscType::Time(time)).unwrap();
        assert_eq!(buffer.len(), 8);
    }

    #[test]
    fn test_encode_char() {
        let mut buffer = Vec::new();
        encode_argument(&mut buffer, &OscType::Char('A')).unwrap();
        assert_eq!(buffer.len(), 4);
    }

    #[test]
    fn test_encode_color() {
        let mut buffer = Vec::new();
        let color = OscColor::new(255, 128, 64, 32);
        encode_argument(&mut buffer, &OscType::Color(color)).unwrap();
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer[0], 255);
        assert_eq!(buffer[1], 128);
        assert_eq!(buffer[2], 64);
        assert_eq!(buffer[3], 32);
    }

    #[test]
    fn test_encode_midi() {
        let mut buffer = Vec::new();
        let midi = OscMidi::new(0, 0x90, 60, 127);
        encode_argument(&mut buffer, &OscType::Midi(midi)).unwrap();
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer[0], 0);
        assert_eq!(buffer[1], 0x90);
        assert_eq!(buffer[2], 60);
        assert_eq!(buffer[3], 127);
    }

    #[test]
    fn test_encode_bool_types() {
        let mut buffer = Vec::new();
        encode_argument(&mut buffer, &OscType::True).unwrap();
        assert_eq!(buffer.len(), 0); // No data for True
        
        encode_argument(&mut buffer, &OscType::False).unwrap();
        assert_eq!(buffer.len(), 0); // No data for False
        
        encode_argument(&mut buffer, &OscType::Nil).unwrap();
        assert_eq!(buffer.len(), 0); // No data for Nil
        
        encode_argument(&mut buffer, &OscType::Impulse).unwrap();
        assert_eq!(buffer.len(), 0); // No data for Impulse
    }

    #[test]
    fn test_encode_simple_message() {
        let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
        let encoded = encode_message(&msg).unwrap();
        
        // Should contain address, type tags, and argument
        assert!(encoded.len() > 0);
        
        // Check address is present
        assert!(encoded.starts_with(b"/test"));
    }

    #[test]
    fn test_encode_message_multiple_args() {
        let msg = OscMessage::new(
            "/synth/note",
            vec![
                OscType::Int(60),
                OscType::Float(0.8),
                OscType::String("on".to_string()),
            ],
        );
        let encoded = encode_message(&msg).unwrap();
        assert!(encoded.len() > 0);
    }

    #[test]
    fn test_encode_bundle() {
        let mut bundle = OscBundle::new(OscTime::immediate());
        bundle.add_message(OscMessage::new("/test1", vec![OscType::Int(1)]));
        bundle.add_message(OscMessage::new("/test2", vec![OscType::Int(2)]));
        
        let encoded = encode_bundle(&bundle).unwrap();
        
        // Should start with "#bundle"
        assert!(encoded.starts_with(b"#bundle"));
        assert!(encoded.len() > 16); // At least bundle header + time tag
    }

    #[test]
    fn test_get_type_tags() {
        assert_eq!(get_type_tag(&OscType::Int(0)), 'i');
        assert_eq!(get_type_tag(&OscType::Float(0.0)), 'f');
        assert_eq!(get_type_tag(&OscType::String(String::new())), 's');
        assert_eq!(get_type_tag(&OscType::Blob(Vec::new())), 'b');
        assert_eq!(get_type_tag(&OscType::Long(0)), 'h');
        assert_eq!(get_type_tag(&OscType::Double(0.0)), 'd');
        assert_eq!(get_type_tag(&OscType::Time(OscTime::immediate())), 't');
        assert_eq!(get_type_tag(&OscType::Char('A')), 'c');
        assert_eq!(get_type_tag(&OscType::Color(OscColor::rgb(0, 0, 0))), 'r');
        assert_eq!(get_type_tag(&OscType::Midi(OscMidi::new(0, 0, 0, 0))), 'm');
        assert_eq!(get_type_tag(&OscType::True), 'T');
        assert_eq!(get_type_tag(&OscType::False), 'F');
        assert_eq!(get_type_tag(&OscType::Nil), 'N');
        assert_eq!(get_type_tag(&OscType::Impulse), 'I');
        assert_eq!(get_type_tag(&OscType::Array(Vec::new())), '[');
    }

    #[test]
    fn test_encode_all_types() {
        // Test that all OSC types can be encoded without panicking
        let types = vec![
            OscType::Int(42),
            OscType::Float(3.14),
            OscType::String("test".to_string()),
            OscType::Blob(vec![1, 2, 3]),
            OscType::Long(1234567890),
            OscType::Double(3.14159265359),
            OscType::Time(OscTime::immediate()),
            OscType::Char('A'),
            OscType::Color(OscColor::rgb(255, 0, 0)),
            OscType::Midi(OscMidi::new(0, 0x90, 60, 127)),
            OscType::True,
            OscType::False,
            OscType::Nil,
            OscType::Impulse,
        ];
        
        for osc_type in types {
            let msg = OscMessage::new("/test", vec![osc_type]);
            let result = encode_message(&msg);
            assert!(result.is_ok(), "Failed to encode message");
        }
    }
}
