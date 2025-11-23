//! OSC receiver implementation.
//!
//! Provides functionality to receive and parse OSC messages over UDP or TCP.
//!
//! # Examples
//!
//! ## UDP Receiver
//!
//! ```no_run
//! use nih_plug_osc::OscReceiver;
//!
//! let receiver = OscReceiver::new_udp("127.0.0.1:9000").unwrap();
//! loop {
//!     match receiver.recv() {
//!         Ok(packet) => println!("Received: {:?}", packet),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```
//!
//! ## TCP Receiver
//!
//! ```no_run
//! use nih_plug_osc::OscReceiver;
//!
//! let receiver = OscReceiver::new_tcp("127.0.0.1:9000").unwrap();
//! loop {
//!     match receiver.recv() {
//!         Ok(packet) => println!("Received: {:?}", packet),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

use crate::error::OscError;
use crate::message::{OscBundle, OscColor, OscMessage, OscMidi, OscPacket, OscTime, OscType};
use std::io::Read;
use std::net::{TcpListener, ToSocketAddrs, UdpSocket};

/// OSC receiver that can receive messages over UDP or TCP.
pub struct OscReceiver {
    transport: Transport,
}

enum Transport {
    Udp(UdpSocket),
    Tcp(TcpListener),
}

impl OscReceiver {
    /// Creates a new UDP receiver bound to the specified address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::OscReceiver;
    ///
    /// let receiver = OscReceiver::new_udp("127.0.0.1:9000").unwrap();
    /// ```
    pub fn new_udp<A: ToSocketAddrs>(addr: A) -> Result<Self, OscError> {
        let socket = UdpSocket::bind(addr)?;
        Ok(Self {
            transport: Transport::Udp(socket),
        })
    }

    /// Creates a new TCP receiver listening on the specified address.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::OscReceiver;
    ///
    /// let receiver = OscReceiver::new_tcp("127.0.0.1:9000").unwrap();
    /// ```
    pub fn new_tcp<A: ToSocketAddrs>(addr: A) -> Result<Self, OscError> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self {
            transport: Transport::Tcp(listener),
        })
    }

    /// Receives an OSC packet (message or bundle).
    ///
    /// This method blocks until a packet is received.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::OscReceiver;
    ///
    /// let receiver = OscReceiver::new_udp("127.0.0.1:9000").unwrap();
    /// let packet = receiver.recv().unwrap();
    /// ```
    pub fn recv(&self) -> Result<OscPacket, OscError> {
        match &self.transport {
            Transport::Udp(socket) => {
                let mut buffer = vec![0u8; 65536]; // Max UDP packet size
                let (size, _) = socket.recv_from(&mut buffer)?;
                buffer.truncate(size);
                decode_packet(&buffer)
            }
            Transport::Tcp(listener) => {
                let (mut stream, _) = listener.accept()?;
                let mut buffer = Vec::new();
                stream.read_to_end(&mut buffer)?;
                decode_packet(&buffer)
            }
        }
    }

    /// Attempts to receive an OSC packet without blocking.
    ///
    /// Returns `None` if no packet is available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_osc::OscReceiver;
    ///
    /// let receiver = OscReceiver::new_udp("127.0.0.1:9000").unwrap();
    /// if let Some(packet) = receiver.try_recv().unwrap() {
    ///     println!("Received: {:?}", packet);
    /// }
    /// ```
    pub fn try_recv(&self) -> Result<Option<OscPacket>, OscError> {
        match &self.transport {
            Transport::Udp(socket) => {
                socket.set_nonblocking(true)?;
                let mut buffer = vec![0u8; 65536];
                match socket.recv_from(&mut buffer) {
                    Ok((size, _)) => {
                        buffer.truncate(size);
                        Ok(Some(decode_packet(&buffer)?))
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
                    Err(e) => Err(e.into()),
                }
            }
            Transport::Tcp(_) => {
                // TCP doesn't support non-blocking in the same way
                // Would need to use async or different approach
                Err(OscError::NetworkError(
                    "Non-blocking TCP not supported".to_string(),
                ))
            }
        }
    }
}

/// Decodes an OSC packet from bytes.
#[doc(hidden)]
pub fn decode_packet(data: &[u8]) -> Result<OscPacket, OscError> {
    if data.is_empty() {
        return Err(OscError::InvalidMessage);
    }

    // Check if it's a bundle (starts with "#bundle")
    if data.len() >= 8 && &data[0..7] == b"#bundle" {
        decode_bundle(data)
    } else {
        decode_message(data).map(OscPacket::Message)
    }
}

/// Decodes an OSC message from bytes.
fn decode_message(data: &[u8]) -> Result<OscMessage, OscError> {
    let mut cursor = 0;

    // Decode address pattern
    let address = decode_string(data, &mut cursor)?;

    // Decode type tag string
    let type_tags = decode_string(data, &mut cursor)?;

    if !type_tags.starts_with(',') {
        return Err(OscError::InvalidMessage);
    }

    // Decode arguments based on type tags
    let mut arguments = Vec::new();
    let mut i = 1; // Skip the comma
    while i < type_tags.len() {
        let tag = type_tags.chars().nth(i).ok_or(OscError::InvalidMessage)?;
        
        match tag {
            'i' => {
                arguments.push(OscType::Int(decode_i32(data, &mut cursor)?));
            }
            'f' => {
                arguments.push(OscType::Float(decode_f32(data, &mut cursor)?));
            }
            's' => {
                arguments.push(OscType::String(decode_string(data, &mut cursor)?));
            }
            'b' => {
                arguments.push(OscType::Blob(decode_blob(data, &mut cursor)?));
            }
            'h' => {
                arguments.push(OscType::Long(decode_i64(data, &mut cursor)?));
            }
            'd' => {
                arguments.push(OscType::Double(decode_f64(data, &mut cursor)?));
            }
            't' => {
                arguments.push(OscType::Time(decode_time(data, &mut cursor)?));
            }
            'c' => {
                arguments.push(OscType::Char(decode_char(data, &mut cursor)?));
            }
            'r' => {
                arguments.push(OscType::Color(decode_color(data, &mut cursor)?));
            }
            'm' => {
                arguments.push(OscType::Midi(decode_midi(data, &mut cursor)?));
            }
            'T' => {
                arguments.push(OscType::True);
            }
            'F' => {
                arguments.push(OscType::False);
            }
            'N' => {
                arguments.push(OscType::Nil);
            }
            'I' => {
                arguments.push(OscType::Impulse);
            }
            '[' => {
                // Array start - collect elements until ']'
                let mut array_elements = Vec::new();
                i += 1;
                while i < type_tags.len() {
                    let array_tag = type_tags.chars().nth(i).ok_or(OscError::InvalidMessage)?;
                    if array_tag == ']' {
                        break;
                    }
                    
                    // Decode array element based on its type
                    match array_tag {
                        'i' => array_elements.push(OscType::Int(decode_i32(data, &mut cursor)?)),
                        'f' => array_elements.push(OscType::Float(decode_f32(data, &mut cursor)?)),
                        's' => array_elements.push(OscType::String(decode_string(data, &mut cursor)?)),
                        _ => return Err(OscError::InvalidMessage),
                    }
                    i += 1;
                }
                arguments.push(OscType::Array(array_elements));
            }
            ']' => {
                // Array end marker - should be handled in array parsing
            }
            _ => {
                return Err(OscError::InvalidMessage);
            }
        }
        
        i += 1;
    }

    Ok(OscMessage { address, arguments })
}

/// Decodes an OSC bundle from bytes.
fn decode_bundle(data: &[u8]) -> Result<OscPacket, OscError> {
    let mut cursor = 0;

    // Verify bundle identifier
    let identifier = decode_string(data, &mut cursor)?;
    if identifier != "#bundle" {
        return Err(OscError::InvalidMessage);
    }

    // Decode time tag
    let time_tag = decode_time(data, &mut cursor)?;

    // Decode packets
    let mut packets = Vec::new();
    while cursor < data.len() {
        // Read packet size
        let size = decode_i32(data, &mut cursor)? as usize;
        
        if cursor + size > data.len() {
            return Err(OscError::InvalidMessage);
        }

        // Decode packet
        let packet_data = &data[cursor..cursor + size];
        packets.push(decode_packet(packet_data)?);
        cursor += size;
    }

    Ok(OscPacket::Bundle(OscBundle {
        time_tag,
        packets,
    }))
}

/// Decodes a null-terminated string with padding.
fn decode_string(data: &[u8], cursor: &mut usize) -> Result<String, OscError> {
    if *cursor >= data.len() {
        return Err(OscError::InvalidMessage);
    }

    // Find null terminator
    let start = *cursor;
    let mut end = start;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }

    if end >= data.len() {
        return Err(OscError::InvalidMessage);
    }

    // Extract string
    let string = String::from_utf8(data[start..end].to_vec())
        .map_err(|_| OscError::InvalidMessage)?;

    // Move cursor past string and padding (to 4-byte boundary)
    let string_len = end - start + 1; // Include null terminator
    let padding = (4 - (string_len % 4)) % 4;
    *cursor = end + 1 + padding;

    Ok(string)
}

/// Decodes a 32-bit integer.
fn decode_i32(data: &[u8], cursor: &mut usize) -> Result<i32, OscError> {
    if *cursor + 4 > data.len() {
        return Err(OscError::InvalidMessage);
    }

    let bytes = [data[*cursor], data[*cursor + 1], data[*cursor + 2], data[*cursor + 3]];
    *cursor += 4;
    Ok(i32::from_be_bytes(bytes))
}

/// Decodes a 32-bit float.
fn decode_f32(data: &[u8], cursor: &mut usize) -> Result<f32, OscError> {
    if *cursor + 4 > data.len() {
        return Err(OscError::InvalidMessage);
    }

    let bytes = [data[*cursor], data[*cursor + 1], data[*cursor + 2], data[*cursor + 3]];
    *cursor += 4;
    Ok(f32::from_be_bytes(bytes))
}

/// Decodes a 64-bit integer.
fn decode_i64(data: &[u8], cursor: &mut usize) -> Result<i64, OscError> {
    if *cursor + 8 > data.len() {
        return Err(OscError::InvalidMessage);
    }

    let bytes = [
        data[*cursor],
        data[*cursor + 1],
        data[*cursor + 2],
        data[*cursor + 3],
        data[*cursor + 4],
        data[*cursor + 5],
        data[*cursor + 6],
        data[*cursor + 7],
    ];
    *cursor += 8;
    Ok(i64::from_be_bytes(bytes))
}

/// Decodes a 64-bit float.
fn decode_f64(data: &[u8], cursor: &mut usize) -> Result<f64, OscError> {
    if *cursor + 8 > data.len() {
        return Err(OscError::InvalidMessage);
    }

    let bytes = [
        data[*cursor],
        data[*cursor + 1],
        data[*cursor + 2],
        data[*cursor + 3],
        data[*cursor + 4],
        data[*cursor + 5],
        data[*cursor + 6],
        data[*cursor + 7],
    ];
    *cursor += 8;
    Ok(f64::from_be_bytes(bytes))
}

/// Decodes an OSC time tag.
fn decode_time(data: &[u8], cursor: &mut usize) -> Result<OscTime, OscError> {
    let seconds = decode_i32(data, cursor)? as u32;
    let fractional = decode_i32(data, cursor)? as u32;
    Ok(OscTime::new(seconds, fractional))
}

/// Decodes a character.
fn decode_char(data: &[u8], cursor: &mut usize) -> Result<char, OscError> {
    let value = decode_i32(data, cursor)? as u32;
    char::from_u32(value).ok_or(OscError::InvalidMessage)
}

/// Decodes an RGBA color.
fn decode_color(data: &[u8], cursor: &mut usize) -> Result<OscColor, OscError> {
    if *cursor + 4 > data.len() {
        return Err(OscError::InvalidMessage);
    }

    let red = data[*cursor];
    let green = data[*cursor + 1];
    let blue = data[*cursor + 2];
    let alpha = data[*cursor + 3];
    *cursor += 4;

    Ok(OscColor::new(red, green, blue, alpha))
}

/// Decodes a MIDI message.
fn decode_midi(data: &[u8], cursor: &mut usize) -> Result<OscMidi, OscError> {
    if *cursor + 4 > data.len() {
        return Err(OscError::InvalidMessage);
    }

    let port = data[*cursor];
    let status = data[*cursor + 1];
    let data1 = data[*cursor + 2];
    let data2 = data[*cursor + 3];
    *cursor += 4;

    Ok(OscMidi::new(port, status, data1, data2))
}

/// Decodes a blob (binary data).
fn decode_blob(data: &[u8], cursor: &mut usize) -> Result<Vec<u8>, OscError> {
    // Read size
    let size = decode_i32(data, cursor)? as usize;

    if *cursor + size > data.len() {
        return Err(OscError::InvalidMessage);
    }

    // Extract blob data
    let blob = data[*cursor..*cursor + size].to_vec();

    // Move cursor past data and padding
    let padding = (4 - (size % 4)) % 4;
    *cursor += size + padding;

    Ok(blob)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_string() {
        // "test\0" with padding to 8 bytes
        let data = b"test\0\0\0\0";
        let mut cursor = 0;
        let result = decode_string(data, &mut cursor).unwrap();
        assert_eq!(result, "test");
        assert_eq!(cursor, 8);
    }

    #[test]
    fn test_decode_i32() {
        let data = 42i32.to_be_bytes();
        let mut cursor = 0;
        let result = decode_i32(&data, &mut cursor).unwrap();
        assert_eq!(result, 42);
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_decode_f32() {
        let data = 3.14f32.to_be_bytes();
        let mut cursor = 0;
        let result = decode_f32(&data, &mut cursor).unwrap();
        assert!((result - 3.14).abs() < 0.001);
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_decode_i64() {
        let data = 1234567890i64.to_be_bytes();
        let mut cursor = 0;
        let result = decode_i64(&data, &mut cursor).unwrap();
        assert_eq!(result, 1234567890);
        assert_eq!(cursor, 8);
    }

    #[test]
    fn test_decode_f64() {
        let data = 3.14159265359f64.to_be_bytes();
        let mut cursor = 0;
        let result = decode_f64(&data, &mut cursor).unwrap();
        assert!((result - 3.14159265359).abs() < 0.0000001);
        assert_eq!(cursor, 8);
    }

    #[test]
    fn test_decode_time() {
        let mut data = Vec::new();
        data.extend_from_slice(&100u32.to_be_bytes());
        data.extend_from_slice(&200u32.to_be_bytes());
        let mut cursor = 0;
        let result = decode_time(&data, &mut cursor).unwrap();
        assert_eq!(result.seconds, 100);
        assert_eq!(result.fractional, 200);
        assert_eq!(cursor, 8);
    }

    #[test]
    fn test_decode_char() {
        let data = ('A' as u32).to_be_bytes();
        let mut cursor = 0;
        let result = decode_char(&data, &mut cursor).unwrap();
        assert_eq!(result, 'A');
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_decode_color() {
        let data = [255u8, 128, 64, 32];
        let mut cursor = 0;
        let result = decode_color(&data, &mut cursor).unwrap();
        assert_eq!(result.red, 255);
        assert_eq!(result.green, 128);
        assert_eq!(result.blue, 64);
        assert_eq!(result.alpha, 32);
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_decode_midi() {
        let data = [0u8, 0x90, 60, 127];
        let mut cursor = 0;
        let result = decode_midi(&data, &mut cursor).unwrap();
        assert_eq!(result.port, 0);
        assert_eq!(result.status, 0x90);
        assert_eq!(result.data1, 60);
        assert_eq!(result.data2, 127);
        assert_eq!(cursor, 4);
    }

    #[test]
    fn test_decode_blob() {
        let mut data = Vec::new();
        // Size: 5
        data.extend_from_slice(&5i32.to_be_bytes());
        // Data: [1, 2, 3, 4, 5]
        data.extend_from_slice(&[1, 2, 3, 4, 5]);
        // Padding: 3 bytes
        data.extend_from_slice(&[0, 0, 0]);
        
        let mut cursor = 0;
        let result = decode_blob(&data, &mut cursor).unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
        assert_eq!(cursor, 12);
    }

    #[test]
    fn test_decode_simple_message() {
        // Manually construct a simple OSC message: /test with int 42
        let mut data = Vec::new();
        
        // Address: "/test\0" + padding
        data.extend_from_slice(b"/test\0\0\0");
        
        // Type tags: ",i\0" + padding
        data.extend_from_slice(b",i\0\0");
        
        // Argument: 42
        data.extend_from_slice(&42i32.to_be_bytes());
        
        let result = decode_message(&data).unwrap();
        assert_eq!(result.address, "/test");
        assert_eq!(result.arguments.len(), 1);
        assert_eq!(result.arguments[0], OscType::Int(42));
    }

    #[test]
    fn test_decode_message_multiple_args() {
        // Message: /synth with int 60 and float 0.8
        let mut data = Vec::new();
        
        // Address: "/synth\0" + padding
        data.extend_from_slice(b"/synth\0\0");
        
        // Type tags: ",if\0"
        data.extend_from_slice(b",if\0");
        
        // Arguments
        data.extend_from_slice(&60i32.to_be_bytes());
        data.extend_from_slice(&0.8f32.to_be_bytes());
        
        let result = decode_message(&data).unwrap();
        assert_eq!(result.address, "/synth");
        assert_eq!(result.arguments.len(), 2);
        assert_eq!(result.arguments[0], OscType::Int(60));
        match result.arguments[1] {
            OscType::Float(f) => assert!((f - 0.8).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_decode_message_with_string() {
        let mut data = Vec::new();
        
        // Address: "/test\0" + padding
        data.extend_from_slice(b"/test\0\0\0");
        
        // Type tags: ",s\0" + padding
        data.extend_from_slice(b",s\0\0");
        
        // String argument: "hello\0" + padding
        data.extend_from_slice(b"hello\0\0\0");
        
        let result = decode_message(&data).unwrap();
        assert_eq!(result.address, "/test");
        assert_eq!(result.arguments.len(), 1);
        assert_eq!(result.arguments[0], OscType::String("hello".to_string()));
    }

    #[test]
    fn test_decode_message_with_bool_types() {
        let mut data = Vec::new();
        
        // Address: "/test\0" + padding
        data.extend_from_slice(b"/test\0\0\0");
        
        // Type tags: ",TFNI"
        data.extend_from_slice(b",TFNI\0\0\0\0");
        
        let result = decode_message(&data).unwrap();
        assert_eq!(result.address, "/test");
        assert_eq!(result.arguments.len(), 4);
        assert_eq!(result.arguments[0], OscType::True);
        assert_eq!(result.arguments[1], OscType::False);
        assert_eq!(result.arguments[2], OscType::Nil);
        assert_eq!(result.arguments[3], OscType::Impulse);
    }

    #[test]
    fn test_decode_invalid_message() {
        // Empty data
        let result = decode_message(&[]);
        assert!(result.is_err());
        
        // Incomplete data
        let result = decode_message(b"/test");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_packet_message() {
        let mut data = Vec::new();
        data.extend_from_slice(b"/test\0\0\0");
        data.extend_from_slice(b",i\0\0");
        data.extend_from_slice(&42i32.to_be_bytes());
        
        let result = decode_packet(&data).unwrap();
        match result {
            OscPacket::Message(msg) => {
                assert_eq!(msg.address, "/test");
                assert_eq!(msg.arguments.len(), 1);
            }
            _ => panic!("Expected message"),
        }
    }

    #[test]
    fn test_decode_bundle() {
        let mut data = Vec::new();
        
        // Bundle identifier: "#bundle\0"
        data.extend_from_slice(b"#bundle\0");
        
        // Time tag: immediate (0, 1)
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&1u32.to_be_bytes());
        
        // Message 1: /test1 with int 1
        let mut msg1_data = Vec::new();
        msg1_data.extend_from_slice(b"/test1\0\0");
        msg1_data.extend_from_slice(b",i\0\0");
        msg1_data.extend_from_slice(&1i32.to_be_bytes());
        
        // Add message 1 with size prefix
        data.extend_from_slice(&(msg1_data.len() as i32).to_be_bytes());
        data.extend_from_slice(&msg1_data);
        
        let result = decode_bundle(&data).unwrap();
        match result {
            OscPacket::Bundle(bundle) => {
                assert!(bundle.time_tag.is_immediate());
                assert_eq!(bundle.packets.len(), 1);
            }
            _ => panic!("Expected bundle"),
        }
    }
}
