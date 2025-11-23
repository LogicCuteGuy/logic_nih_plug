//! Test demonstrating typed access to OSC message arguments.
//!
//! This test validates Requirement 23.4: "WHEN a developer receives an OSC message
//! THEN the system SHALL parse and provide typed access to arguments"

use nih_plug_osc::{OscMessage, OscPacket, OscType};

// Helper to encode and decode
fn encode_decode(msg: &OscMessage) -> OscMessage {
    let packet = OscPacket::Message(msg.clone());
    let encoded = nih_plug_osc::sender::encode_packet(&packet).unwrap();
    let decoded = nih_plug_osc::receiver::decode_packet(&encoded).unwrap();
    match decoded {
        OscPacket::Message(m) => m,
        _ => panic!("Expected message"),
    }
}

#[test]
fn test_typed_access_to_int() {
    let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
    let decoded = encode_decode(&msg);
    
    // Demonstrate typed access
    match &decoded.arguments[0] {
        OscType::Int(value) => {
            assert_eq!(*value, 42);
            println!("Received integer: {}", value);
        }
        _ => panic!("Expected Int type"),
    }
}

#[test]
fn test_typed_access_to_float() {
    let msg = OscMessage::new("/synth/frequency", vec![OscType::Float(440.0)]);
    let decoded = encode_decode(&msg);
    
    // Demonstrate typed access
    match &decoded.arguments[0] {
        OscType::Float(freq) => {
            assert!((freq - 440.0).abs() < 0.001);
            println!("Received frequency: {} Hz", freq);
        }
        _ => panic!("Expected Float type"),
    }
}

#[test]
fn test_typed_access_to_string() {
    let msg = OscMessage::new("/chat/message", vec![OscType::String("Hello, OSC!".to_string())]);
    let decoded = encode_decode(&msg);
    
    // Demonstrate typed access
    match &decoded.arguments[0] {
        OscType::String(text) => {
            assert_eq!(text, "Hello, OSC!");
            println!("Received message: {}", text);
        }
        _ => panic!("Expected String type"),
    }
}

#[test]
fn test_typed_access_to_multiple_args() {
    let msg = OscMessage::new(
        "/synth/note",
        vec![
            OscType::Int(60),           // MIDI note number
            OscType::Float(0.8),        // Velocity
            OscType::String("on".to_string()), // State
        ],
    );
    let decoded = encode_decode(&msg);
    
    // Demonstrate typed access to multiple arguments
    assert_eq!(decoded.arguments.len(), 3);
    
    let note = match &decoded.arguments[0] {
        OscType::Int(n) => *n,
        _ => panic!("Expected Int"),
    };
    
    let velocity = match &decoded.arguments[1] {
        OscType::Float(v) => *v,
        _ => panic!("Expected Float"),
    };
    
    let state = match &decoded.arguments[2] {
        OscType::String(s) => s.clone(),
        _ => panic!("Expected String"),
    };
    
    assert_eq!(note, 60);
    assert!((velocity - 0.8).abs() < 0.001);
    assert_eq!(state, "on");
    
    println!("Note: {}, Velocity: {}, State: {}", note, velocity, state);
}

#[test]
fn test_typed_access_to_blob() {
    let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let msg = OscMessage::new("/data", vec![OscType::Blob(data.clone())]);
    let decoded = encode_decode(&msg);
    
    // Demonstrate typed access to binary data
    match &decoded.arguments[0] {
        OscType::Blob(bytes) => {
            assert_eq!(bytes, &data);
            println!("Received {} bytes of binary data", bytes.len());
        }
        _ => panic!("Expected Blob type"),
    }
}

#[test]
fn test_typed_access_to_bool_types() {
    let msg = OscMessage::new(
        "/flags",
        vec![
            OscType::True,
            OscType::False,
            OscType::Nil,
            OscType::Impulse,
        ],
    );
    let decoded = encode_decode(&msg);
    
    // Demonstrate typed access to boolean and special types
    assert!(matches!(decoded.arguments[0], OscType::True));
    assert!(matches!(decoded.arguments[1], OscType::False));
    assert!(matches!(decoded.arguments[2], OscType::Nil));
    assert!(matches!(decoded.arguments[3], OscType::Impulse));
    
    println!("Received boolean and special types");
}

#[test]
fn test_typed_access_helper_function() {
    let msg = OscMessage::new(
        "/mixed",
        vec![
            OscType::Int(42),
            OscType::Float(3.14),
            OscType::String("test".to_string()),
        ],
    );
    let decoded = encode_decode(&msg);
    
    // Helper function to extract typed values
    fn get_int(args: &[OscType], index: usize) -> Option<i32> {
        match &args[index] {
            OscType::Int(i) => Some(*i),
            _ => None,
        }
    }
    
    fn get_float(args: &[OscType], index: usize) -> Option<f32> {
        match &args[index] {
            OscType::Float(f) => Some(*f),
            _ => None,
        }
    }
    
    fn get_string(args: &[OscType], index: usize) -> Option<String> {
        match &args[index] {
            OscType::String(s) => Some(s.clone()),
            _ => None,
        }
    }
    
    // Use helper functions for typed access
    assert_eq!(get_int(&decoded.arguments, 0), Some(42));
    assert_eq!(get_float(&decoded.arguments, 1), Some(3.14));
    assert_eq!(get_string(&decoded.arguments, 2), Some("test".to_string()));
    
    println!("Successfully extracted typed values using helper functions");
}
