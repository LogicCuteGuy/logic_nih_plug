//! Comprehensive MIDI-CI protocol demonstration.
//!
//! This example demonstrates all major MIDI-CI features:
//! - Device discovery
//! - Capability queries
//! - Profile negotiation
//! - Property exchange
//!
//! Run with: cargo run --example midi_ci_demo --features full

use nih_plug_midi_ci::{
    discovery::{
        DeviceCapabilities, DiscoveryInquiry, DiscoveryReply, EndpointInfoInquiry,
        EndpointInfoReply,
    },
    profiles::{ProfileInquiry, ProfileInquiryReply, SetProfileOff, SetProfileOn},
    properties::{
        PropertyExchangeCapabilities, PropertyExchangeCapabilitiesInquiry,
        PropertyExchangeCapabilitiesReply, PropertyGetData, PropertyGetDataReply, PropertySetData,
    },
    protocol::{DeviceInfo, MidiCiMessage, Muid, ProfileId},
};

fn main() {
    println!("=== MIDI-CI Protocol Demonstration ===\n");

    // Create device identities
    let device_a_muid = Muid::new(0x1234567).unwrap();
    let device_b_muid = Muid::new(0x7654321).unwrap();

    // Device A information
    let device_a_info = DeviceInfo::new(
        vec![0x7D],  // Manufacturer ID (Educational/Development)
        0x1234,      // Family
        0x5678,      // Model
        0x010203,    // Revision 1.2.3
    );

    // Device B information
    let device_b_info = DeviceInfo::new(
        vec![0x00, 0x01, 0x02], // Extended manufacturer ID
        0x2345,                  // Family
        0x6789,                  // Model
        0x020304,                // Revision 2.3.4
    );

    // ===== 1. DEVICE DISCOVERY =====
    println!("1. Device Discovery");
    println!("-------------------");

    // Device A broadcasts discovery inquiry
    let capabilities_a = DeviceCapabilities::all();
    let discovery_inquiry = DiscoveryInquiry::new(
        device_a_muid,
        device_a_info.clone(),
        capabilities_a,
    );
    let inquiry_message = discovery_inquiry.to_message();
    let inquiry_sysex = inquiry_message.to_sysex();
    println!("Device A broadcasts discovery inquiry:");
    println!("  MUID: 0x{:07X}", device_a_muid.value());
    println!("  Capabilities: Profiles={}, Properties={}, Process={}",
        capabilities_a.supports_profiles,
        capabilities_a.supports_property_exchange,
        capabilities_a.supports_process_inquiry
    );
    println!("  SysEx length: {} bytes", inquiry_sysex.len());

    // Device B receives and responds
    let parsed_inquiry = DiscoveryInquiry::from_message(&inquiry_message).unwrap();
    println!("\nDevice B receives inquiry from MUID 0x{:07X}", parsed_inquiry.source.value());

    let capabilities_b = DeviceCapabilities::new(true, true, false);
    let discovery_reply = DiscoveryReply::new(
        device_b_muid,
        device_a_muid,
        device_b_info.clone(),
        capabilities_b,
    );
    let reply_message = discovery_reply.to_message();
    println!("Device B sends discovery reply:");
    println!("  MUID: 0x{:07X}", device_b_muid.value());
    println!("  Capabilities: Profiles={}, Properties={}, Process={}",
        capabilities_b.supports_profiles,
        capabilities_b.supports_property_exchange,
        capabilities_b.supports_process_inquiry
    );

    // ===== 2. ENDPOINT INFO QUERY =====
    println!("\n2. Endpoint Information Query");
    println!("------------------------------");

    let endpoint_inquiry = EndpointInfoInquiry::new(device_a_muid, device_b_muid);
    let endpoint_inquiry_msg = endpoint_inquiry.to_message();
    println!("Device A queries endpoint info from Device B");

    let endpoint_reply = EndpointInfoReply::new(
        device_b_muid,
        device_a_muid,
        "PROD-B-12345".to_string(),
        "MIDI Synthesizer B".to_string(),
    );
    let endpoint_reply_msg = endpoint_reply.to_message();
    let parsed_endpoint = EndpointInfoReply::from_message(&endpoint_reply_msg).unwrap();
    println!("Device B replies:");
    println!("  Product Instance: {}", parsed_endpoint.product_instance_id);
    println!("  Endpoint Name: {}", parsed_endpoint.endpoint_name);

    // ===== 3. PROFILE NEGOTIATION =====
    println!("\n3. Profile Negotiation");
    println!("----------------------");

    // Query available profiles
    let profile_inquiry = ProfileInquiry::new(device_a_muid, device_b_muid);
    let profile_inquiry_msg = profile_inquiry.to_message();
    println!("Device A queries profiles from Device B");

    // Device B responds with available profiles
    let profile_1 = ProfileId::new([0x7E, 0x00, 0x01, 0x00, 0x00]); // General MIDI
    let profile_2 = ProfileId::new([0x7E, 0x00, 0x02, 0x00, 0x00]); // MIDI Show Control
    let profile_3 = ProfileId::new([0x7E, 0x00, 0x03, 0x00, 0x00]); // MIDI Machine Control

    let profile_reply = ProfileInquiryReply::new(
        device_b_muid,
        device_a_muid,
        vec![profile_1], // Enabled
        vec![profile_2, profile_3], // Disabled
    );
    let profile_reply_msg = profile_reply.to_message();
    let parsed_profiles = ProfileInquiryReply::from_message(&profile_reply_msg).unwrap();
    println!("Device B reports:");
    println!("  Enabled profiles: {}", parsed_profiles.enabled_profiles.len());
    println!("  Disabled profiles: {}", parsed_profiles.disabled_profiles.len());

    // Enable a profile
    let set_profile_on = SetProfileOn::new(device_a_muid, device_b_muid, profile_2);
    let set_on_msg = set_profile_on.to_message();
    println!("\nDevice A requests to enable profile [7E 00 02 00 00]");

    // Disable a profile
    let set_profile_off = SetProfileOff::new(device_a_muid, device_b_muid, profile_1);
    let set_off_msg = set_profile_off.to_message();
    println!("Device A requests to disable profile [7E 00 01 00 00]");

    // ===== 4. PROPERTY EXCHANGE =====
    println!("\n4. Property Exchange");
    println!("--------------------");

    // Query property exchange capabilities
    let prop_cap_inquiry = PropertyExchangeCapabilitiesInquiry::new(device_a_muid, device_b_muid);
    let prop_cap_inquiry_msg = prop_cap_inquiry.to_message();
    println!("Device A queries property exchange capabilities");

    let prop_capabilities = PropertyExchangeCapabilities::new(8);
    let prop_cap_reply = PropertyExchangeCapabilitiesReply::new(
        device_b_muid,
        device_a_muid,
        prop_capabilities,
    );
    let prop_cap_reply_msg = prop_cap_reply.to_message();
    let parsed_prop_cap = PropertyExchangeCapabilitiesReply::from_message(&prop_cap_reply_msg).unwrap();
    println!("Device B supports {} simultaneous property requests",
        parsed_prop_cap.capabilities.max_simultaneous_requests
    );

    // Get a property
    println!("\nDevice A requests property '/device/name'");
    let prop_get = PropertyGetData::new(
        device_a_muid,
        device_b_muid,
        1, // Request ID
        "/device/name".to_string(),
    );
    let prop_get_msg = prop_get.to_message();
    let parsed_get = PropertyGetData::from_message(&prop_get_msg).unwrap();
    println!("  Request ID: {}", parsed_get.request_id);
    println!("  Resource: {}", parsed_get.resource);

    // Reply with property data
    let prop_get_reply = PropertyGetDataReply::new(
        device_b_muid,
        device_a_muid,
        1,
        b"{\"name\":\"MIDI Synthesizer B\",\"version\":\"2.3.4\"}".to_vec(),
    );
    let prop_get_reply_msg = prop_get_reply.to_message();
    let parsed_reply = PropertyGetDataReply::from_message(&prop_get_reply_msg).unwrap();
    println!("Device B replies with data:");
    println!("  {}", String::from_utf8_lossy(&parsed_reply.data));

    // Set a property
    println!("\nDevice A sets property '/device/volume'");
    let prop_set = PropertySetData::new(
        device_a_muid,
        device_b_muid,
        2, // Request ID
        "/device/volume".to_string(),
        b"{\"volume\":75,\"muted\":false}".to_vec(),
    );
    let prop_set_msg = prop_set.to_message();
    let parsed_set = PropertySetData::from_message(&prop_set_msg).unwrap();
    println!("  Request ID: {}", parsed_set.request_id);
    println!("  Resource: {}", parsed_set.resource);
    println!("  Data: {}", String::from_utf8_lossy(&parsed_set.data));

    // ===== 5. MESSAGE ROUND-TRIP VERIFICATION =====
    println!("\n5. Message Round-Trip Verification");
    println!("-----------------------------------");

    // Verify all messages can be encoded to SysEx and decoded back
    let messages = vec![
        ("Discovery Inquiry", inquiry_message),
        ("Discovery Reply", reply_message),
        ("Endpoint Info Inquiry", endpoint_inquiry_msg),
        ("Endpoint Info Reply", endpoint_reply_msg),
        ("Profile Inquiry", profile_inquiry_msg),
        ("Profile Reply", profile_reply_msg),
        ("Set Profile On", set_on_msg),
        ("Set Profile Off", set_off_msg),
        ("Property Capabilities Inquiry", prop_cap_inquiry_msg),
        ("Property Capabilities Reply", prop_cap_reply_msg),
        ("Property Get", prop_get_msg),
        ("Property Get Reply", prop_get_reply_msg),
        ("Property Set", prop_set_msg),
    ];

    for (name, msg) in messages {
        let sysex = msg.to_sysex();
        let decoded = MidiCiMessage::from_sysex(&sysex).unwrap();
        assert_eq!(msg, decoded);
        println!("✓ {} round-trip successful ({} bytes)", name, sysex.len());
    }

    println!("\n=== All MIDI-CI Protocol Features Demonstrated Successfully ===");
}
