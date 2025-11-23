//! Integration tests for OSC sender and receiver.

use nih_plug_osc::{OscBundle, OscColor, OscMessage, OscMidi, OscPacket, OscTime, OscType};

// Access internal encoding/decoding functions for testing
mod internal {
    pub use nih_plug_osc::sender::encode_packet;
    pub use nih_plug_osc::receiver::decode_packet;
}

/// Helper function to encode and decode a message.
fn roundtrip_message(msg: &OscMessage) -> OscMessage {
    // Encode the message
    let packet = OscPacket::Message(msg.clone());
    let encoded = internal::encode_packet(&packet).unwrap();
    
    // Decode the message
    let decoded_packet = internal::decode_packet(&encoded).unwrap();
    
    match decoded_packet {
        OscPacket::Message(decoded_msg) => decoded_msg,
        _ => panic!("Expected message"),
    }
}

/// Helper function to encode and decode a bundle.
fn roundtrip_bundle(bundle: &OscBundle) -> OscBundle {
    // Encode the bundle
    let packet = OscPacket::Bundle(bundle.clone());
    let encoded = internal::encode_packet(&packet).unwrap();
    
    // Decode the bundle
    let decoded_packet = internal::decode_packet(&encoded).unwrap();
    
    match decoded_packet {
        OscPacket::Bundle(decoded_bundle) => decoded_bundle,
        _ => panic!("Expected bundle"),
    }
}

#[test]
fn test_roundtrip_int() {
    let msg = OscMessage::new("/test", vec![OscType::Int(42)]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::Int(42));
}

#[test]
fn test_roundtrip_float() {
    let msg = OscMessage::new("/test", vec![OscType::Float(3.14)]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    match decoded.arguments[0] {
        OscType::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_roundtrip_string() {
    let msg = OscMessage::new("/test", vec![OscType::String("hello".to_string())]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::String("hello".to_string()));
}

#[test]
fn test_roundtrip_blob() {
    let data = vec![1, 2, 3, 4, 5];
    let msg = OscMessage::new("/test", vec![OscType::Blob(data.clone())]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::Blob(data));
}

#[test]
fn test_roundtrip_long() {
    let msg = OscMessage::new("/test", vec![OscType::Long(1234567890)]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::Long(1234567890));
}

#[test]
fn test_roundtrip_double() {
    let msg = OscMessage::new("/test", vec![OscType::Double(3.14159265359)]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    match decoded.arguments[0] {
        OscType::Double(d) => assert!((d - 3.14159265359).abs() < 0.0000001),
        _ => panic!("Expected double"),
    }
}

#[test]
fn test_roundtrip_time() {
    let time = OscTime::new(100, 200);
    let msg = OscMessage::new("/test", vec![OscType::Time(time)]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::Time(time));
}

#[test]
fn test_roundtrip_char() {
    let msg = OscMessage::new("/test", vec![OscType::Char('A')]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::Char('A'));
}

#[test]
fn test_roundtrip_color() {
    let color = OscColor::new(255, 128, 64, 32);
    let msg = OscMessage::new("/test", vec![OscType::Color(color)]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::Color(color));
}

#[test]
fn test_roundtrip_midi() {
    let midi = OscMidi::new(0, 0x90, 60, 127);
    let msg = OscMessage::new("/test", vec![OscType::Midi(midi)]);
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 1);
    assert_eq!(decoded.arguments[0], OscType::Midi(midi));
}

#[test]
fn test_roundtrip_bool_types() {
    let msg = OscMessage::new(
        "/test",
        vec![OscType::True, OscType::False, OscType::Nil, OscType::Impulse],
    );
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test");
    assert_eq!(decoded.arguments.len(), 4);
    assert_eq!(decoded.arguments[0], OscType::True);
    assert_eq!(decoded.arguments[1], OscType::False);
    assert_eq!(decoded.arguments[2], OscType::Nil);
    assert_eq!(decoded.arguments[3], OscType::Impulse);
}

#[test]
fn test_roundtrip_multiple_args() {
    let msg = OscMessage::new(
        "/synth/note",
        vec![
            OscType::Int(60),
            OscType::Float(0.8),
            OscType::String("on".to_string()),
        ],
    );
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/synth/note");
    assert_eq!(decoded.arguments.len(), 3);
    assert_eq!(decoded.arguments[0], OscType::Int(60));
    match decoded.arguments[1] {
        OscType::Float(f) => assert!((f - 0.8).abs() < 0.001),
        _ => panic!("Expected float"),
    }
    assert_eq!(decoded.arguments[2], OscType::String("on".to_string()));
}

#[test]
fn test_roundtrip_bundle() {
    let mut bundle = OscBundle::new(OscTime::immediate());
    bundle.add_message(OscMessage::new("/test1", vec![OscType::Int(1)]));
    bundle.add_message(OscMessage::new("/test2", vec![OscType::Int(2)]));
    
    let decoded = roundtrip_bundle(&bundle);
    
    assert_eq!(decoded.time_tag, OscTime::immediate());
    assert_eq!(decoded.packets.len(), 2);
    
    match &decoded.packets[0] {
        OscPacket::Message(msg) => {
            assert_eq!(msg.address, "/test1");
            assert_eq!(msg.arguments[0], OscType::Int(1));
        }
        _ => panic!("Expected message"),
    }
    
    match &decoded.packets[1] {
        OscPacket::Message(msg) => {
            assert_eq!(msg.address, "/test2");
            assert_eq!(msg.arguments[0], OscType::Int(2));
        }
        _ => panic!("Expected message"),
    }
}

#[test]
fn test_roundtrip_nested_bundle() {
    let mut inner = OscBundle::new(OscTime::new(100, 200));
    inner.add_message(OscMessage::new("/inner", vec![OscType::Int(42)]));
    
    let mut outer = OscBundle::immediate();
    outer.add_bundle(inner);
    outer.add_message(OscMessage::new("/outer", vec![OscType::String("test".to_string())]));
    
    let decoded = roundtrip_bundle(&outer);
    
    assert!(decoded.time_tag.is_immediate());
    assert_eq!(decoded.packets.len(), 2);
    
    match &decoded.packets[0] {
        OscPacket::Bundle(bundle) => {
            assert_eq!(bundle.time_tag.seconds, 100);
            assert_eq!(bundle.time_tag.fractional, 200);
            assert_eq!(bundle.packets.len(), 1);
        }
        _ => panic!("Expected bundle"),
    }
    
    match &decoded.packets[1] {
        OscPacket::Message(msg) => {
            assert_eq!(msg.address, "/outer");
        }
        _ => panic!("Expected message"),
    }
}

#[test]
fn test_roundtrip_all_types() {
    // Test a message with all supported OSC types
    let msg = OscMessage::new(
        "/test/all",
        vec![
            OscType::Int(42),
            OscType::Float(3.14),
            OscType::String("hello".to_string()),
            OscType::Blob(vec![1, 2, 3]),
            OscType::Long(1234567890),
            OscType::Double(3.14159265359),
            OscType::Time(OscTime::new(100, 200)),
            OscType::Char('A'),
            OscType::Color(OscColor::rgb(255, 0, 0)),
            OscType::Midi(OscMidi::new(0, 0x90, 60, 127)),
            OscType::True,
            OscType::False,
            OscType::Nil,
            OscType::Impulse,
        ],
    );
    
    let decoded = roundtrip_message(&msg);
    
    assert_eq!(decoded.address, "/test/all");
    assert_eq!(decoded.arguments.len(), 14);
    
    // Verify each type
    assert_eq!(decoded.arguments[0], OscType::Int(42));
    match decoded.arguments[1] {
        OscType::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float"),
    }
    assert_eq!(decoded.arguments[2], OscType::String("hello".to_string()));
    assert_eq!(decoded.arguments[3], OscType::Blob(vec![1, 2, 3]));
    assert_eq!(decoded.arguments[4], OscType::Long(1234567890));
    match decoded.arguments[5] {
        OscType::Double(d) => assert!((d - 3.14159265359).abs() < 0.0000001),
        _ => panic!("Expected double"),
    }
    assert_eq!(decoded.arguments[6], OscType::Time(OscTime::new(100, 200)));
    assert_eq!(decoded.arguments[7], OscType::Char('A'));
    assert_eq!(decoded.arguments[8], OscType::Color(OscColor::rgb(255, 0, 0)));
    assert_eq!(decoded.arguments[9], OscType::Midi(OscMidi::new(0, 0x90, 60, 127)));
    assert_eq!(decoded.arguments[10], OscType::True);
    assert_eq!(decoded.arguments[11], OscType::False);
    assert_eq!(decoded.arguments[12], OscType::Nil);
    assert_eq!(decoded.arguments[13], OscType::Impulse);
}

#[cfg(feature = "bundles")]
mod bundle_tests {
    use super::*;
    use nih_plug_osc::bundles::{BundleBuilder, BundleUtils};

    #[test]
    fn test_bundle_builder_roundtrip() {
        let bundle = BundleBuilder::new()
            .with_time_tag(OscTime::new(1000, 2000))
            .add_message(OscMessage::new("/test1", vec![OscType::Int(1)]))
            .add_message(OscMessage::new("/test2", vec![OscType::Float(2.0)]))
            .add_message(OscMessage::new("/test3", vec![OscType::String("three".to_string())]))
            .build();

        let decoded = super::roundtrip_bundle(&bundle);

        assert_eq!(decoded.time_tag.seconds, 1000);
        assert_eq!(decoded.time_tag.fractional, 2000);
        assert_eq!(decoded.packets.len(), 3);
    }

    #[test]
    fn test_deeply_nested_bundle_roundtrip() {
        // Create a 3-level nested bundle
        let mut level3 = OscBundle::immediate();
        level3.add_message(OscMessage::new("/level3", vec![OscType::Int(3)]));

        let mut level2 = OscBundle::new(OscTime::new(200, 0));
        level2.add_bundle(level3);
        level2.add_message(OscMessage::new("/level2", vec![OscType::Int(2)]));

        let mut level1 = OscBundle::new(OscTime::new(100, 0));
        level1.add_bundle(level2);
        level1.add_message(OscMessage::new("/level1", vec![OscType::Int(1)]));

        let decoded = super::roundtrip_bundle(&level1);

        assert_eq!(decoded.time_tag.seconds, 100);
        assert_eq!(BundleUtils::depth(&decoded), 3);
        assert_eq!(BundleUtils::count_messages(&decoded), 3);
    }

    #[test]
    fn test_bundle_with_all_message_types_roundtrip() {
        let mut bundle = OscBundle::immediate();
        
        // Add messages with different types
        bundle.add_message(OscMessage::new("/int", vec![OscType::Int(42)]));
        bundle.add_message(OscMessage::new("/float", vec![OscType::Float(3.14)]));
        bundle.add_message(OscMessage::new("/string", vec![OscType::String("test".to_string())]));
        bundle.add_message(OscMessage::new("/blob", vec![OscType::Blob(vec![1, 2, 3])]));
        bundle.add_message(OscMessage::new("/long", vec![OscType::Long(1234567890)]));
        bundle.add_message(OscMessage::new("/double", vec![OscType::Double(2.71828)]));
        bundle.add_message(OscMessage::new("/time", vec![OscType::Time(OscTime::new(50, 100))]));
        bundle.add_message(OscMessage::new("/char", vec![OscType::Char('X')]));
        bundle.add_message(OscMessage::new("/color", vec![OscType::Color(OscColor::rgb(128, 64, 32))]));
        bundle.add_message(OscMessage::new("/midi", vec![OscType::Midi(OscMidi::new(1, 0x80, 64, 0))]));
        bundle.add_message(OscMessage::new("/bool", vec![OscType::True, OscType::False]));
        bundle.add_message(OscMessage::new("/nil", vec![OscType::Nil, OscType::Impulse]));

        let decoded = super::roundtrip_bundle(&bundle);

        assert_eq!(BundleUtils::count_messages(&decoded), 12);
        
        let messages = BundleUtils::flatten(&decoded);
        assert_eq!(messages.len(), 12);
        assert_eq!(messages[0].address, "/int");
        assert_eq!(messages[1].address, "/float");
        assert_eq!(messages[2].address, "/string");
    }

    #[test]
    fn test_empty_bundle_roundtrip() {
        let bundle = OscBundle::new(OscTime::new(500, 1000));
        let decoded = super::roundtrip_bundle(&bundle);

        assert_eq!(decoded.time_tag.seconds, 500);
        assert_eq!(decoded.time_tag.fractional, 1000);
        assert_eq!(decoded.packets.len(), 0);
    }

    #[test]
    fn test_bundle_with_mixed_packets_roundtrip() {
        let mut inner_bundle = OscBundle::immediate();
        inner_bundle.add_message(OscMessage::new("/inner", vec![OscType::Int(99)]));

        let mut outer_bundle = OscBundle::new(OscTime::new(300, 400));
        outer_bundle.add_message(OscMessage::new("/msg1", vec![OscType::String("first".to_string())]));
        outer_bundle.add_bundle(inner_bundle);
        outer_bundle.add_message(OscMessage::new("/msg2", vec![OscType::String("second".to_string())]));

        let decoded = super::roundtrip_bundle(&outer_bundle);

        assert_eq!(decoded.packets.len(), 3);
        assert_eq!(BundleUtils::count_messages(&decoded), 3);

        // Verify order is preserved
        match &decoded.packets[0] {
            OscPacket::Message(msg) => assert_eq!(msg.address, "/msg1"),
            _ => panic!("Expected message"),
        }
        match &decoded.packets[1] {
            OscPacket::Bundle(_) => {},
            _ => panic!("Expected bundle"),
        }
        match &decoded.packets[2] {
            OscPacket::Message(msg) => assert_eq!(msg.address, "/msg2"),
            _ => panic!("Expected message"),
        }
    }

    #[test]
    fn test_bundle_flatten_preserves_order() {
        let mut bundle = OscBundle::immediate();
        bundle.add_message(OscMessage::new("/first", vec![]));
        bundle.add_message(OscMessage::new("/second", vec![]));
        bundle.add_message(OscMessage::new("/third", vec![]));

        let messages = BundleUtils::flatten(&bundle);
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].address, "/first");
        assert_eq!(messages[1].address, "/second");
        assert_eq!(messages[2].address, "/third");
    }

    #[test]
    fn test_bundle_filter_preserves_structure() {
        let mut inner = OscBundle::immediate();
        inner.add_message(OscMessage::new("/synth/note", vec![]));
        inner.add_message(OscMessage::new("/effect/reverb", vec![]));

        let mut outer = OscBundle::immediate();
        outer.add_bundle(inner);
        outer.add_message(OscMessage::new("/synth/velocity", vec![]));

        let filtered = BundleUtils::filter_by_address(&outer, "/synth/*");
        
        // Should preserve nested structure
        assert_eq!(filtered.packets.len(), 2);
        assert_eq!(BundleUtils::count_messages(&filtered), 2);
    }

    #[test]
    fn test_bundle_merge_preserves_first_time_tag() {
        let time1 = OscTime::new(100, 0);
        let time2 = OscTime::new(200, 0);
        let time3 = OscTime::new(300, 0);

        let mut bundle1 = OscBundle::new(time1);
        bundle1.add_message(OscMessage::new("/test1", vec![]));

        let mut bundle2 = OscBundle::new(time2);
        bundle2.add_message(OscMessage::new("/test2", vec![]));

        let mut bundle3 = OscBundle::new(time3);
        bundle3.add_message(OscMessage::new("/test3", vec![]));

        let merged = BundleUtils::merge(&[bundle1, bundle2, bundle3]);
        
        // Should use time tag from first bundle
        assert_eq!(merged.time_tag.seconds, 100);
        assert_eq!(merged.time_tag.fractional, 0);
        assert_eq!(BundleUtils::count_messages(&merged), 3);
    }

    #[test]
    fn test_large_bundle_roundtrip() {
        let mut bundle = OscBundle::immediate();
        
        // Add 100 messages
        for i in 0..100 {
            bundle.add_message(OscMessage::new(
                format!("/test/{}", i),
                vec![OscType::Int(i as i32)],
            ));
        }

        let decoded = super::roundtrip_bundle(&bundle);
        
        assert_eq!(BundleUtils::count_messages(&decoded), 100);
        
        let messages = BundleUtils::flatten(&decoded);
        for (i, msg) in messages.iter().enumerate() {
            assert_eq!(msg.address, format!("/test/{}", i));
            assert_eq!(msg.arguments[0], OscType::Int(i as i32));
        }
    }
}
