//! Example demonstrating OSC sender functionality.
//!
//! This example shows how to send OSC messages with various data types
//! over UDP and TCP.

use nih_plug_osc::{OscBundle, OscColor, OscMessage, OscMidi, OscSender, OscTime, OscType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OSC Sender Demo");
    println!("===============\n");

    // Create a UDP sender
    println!("Creating UDP sender...");
    let udp_sender = OscSender::new_udp("127.0.0.1:9000")?;
    println!("UDP sender created successfully\n");

    // Send a simple message with an integer
    println!("Sending simple integer message...");
    let msg1 = OscMessage::new("/test/int", vec![OscType::Int(42)]);
    udp_sender.send(&msg1)?;
    println!("Sent: {:?}\n", msg1);

    // Send a message with multiple argument types
    println!("Sending message with multiple types...");
    let msg2 = OscMessage::new(
        "/synth/note",
        vec![
            OscType::Int(60),           // MIDI note number
            OscType::Float(0.8),        // Velocity
            OscType::String("on".to_string()), // State
        ],
    );
    udp_sender.send(&msg2)?;
    println!("Sent: {:?}\n", msg2);

    // Send a message with all basic types
    println!("Sending message with all basic types...");
    let msg3 = OscMessage::new(
        "/test/all_types",
        vec![
            OscType::Int(42),
            OscType::Float(3.14),
            OscType::String("hello".to_string()),
            OscType::Blob(vec![1, 2, 3, 4, 5]),
            OscType::Long(1234567890),
            OscType::Double(3.14159265359),
            OscType::Time(OscTime::immediate()),
            OscType::Char('A'),
            OscType::Color(OscColor::rgb(255, 128, 64)),
            OscType::Midi(OscMidi::new(0, 0x90, 60, 127)),
            OscType::True,
            OscType::False,
            OscType::Nil,
            OscType::Impulse,
        ],
    );
    udp_sender.send(&msg3)?;
    println!("Sent message with all OSC data types\n");

    // Send a bundle with multiple messages
    println!("Sending bundle with multiple messages...");
    let mut bundle = OscBundle::new(OscTime::immediate());
    bundle.add_message(OscMessage::new("/test1", vec![OscType::Int(1)]));
    bundle.add_message(OscMessage::new("/test2", vec![OscType::Int(2)]));
    bundle.add_message(OscMessage::new("/test3", vec![OscType::Int(3)]));
    udp_sender.send_bundle(&bundle)?;
    println!("Sent bundle with {} messages\n", bundle.packets.len());

    // Demonstrate TCP sender
    println!("Creating TCP sender...");
    match OscSender::new_tcp("127.0.0.1:9001") {
        Ok(tcp_sender) => {
            println!("TCP sender created successfully");
            let msg = OscMessage::new("/tcp/test", vec![OscType::String("TCP works!".to_string())]);
            tcp_sender.send(&msg)?;
            println!("Sent message over TCP: {:?}\n", msg);
        }
        Err(e) => {
            println!("TCP sender creation failed (this is expected if no server is running): {}\n", e);
        }
    }

    println!("Demo complete!");
    println!("\nNote: To actually receive these messages, you need an OSC receiver");
    println!("listening on the specified ports (9000 for UDP, 9001 for TCP).");

    Ok(())
}
