//! OSC receiver demonstration.
//!
//! This example shows how to receive OSC messages over UDP.
//!
//! To test this, you can use the sender_demo example in another terminal:
//! ```
//! cargo run --example sender_demo
//! ```
//!
//! Then run this receiver:
//! ```
//! cargo run --example receiver_demo
//! ```

use nih_plug_osc::{OscPacket, OscReceiver, OscType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting OSC receiver on 127.0.0.1:9000");
    println!("Waiting for messages...\n");

    let receiver = OscReceiver::new_udp("127.0.0.1:9000")?;

    loop {
        match receiver.recv() {
            Ok(packet) => {
                print_packet(&packet);
            }
            Err(e) => {
                eprintln!("Error receiving: {}", e);
            }
        }
    }
}

fn print_packet(packet: &OscPacket) {
    match packet {
        OscPacket::Message(msg) => {
            println!("Received message:");
            println!("  Address: {}", msg.address);
            println!("  Arguments:");
            for (i, arg) in msg.arguments.iter().enumerate() {
                println!("    [{}] {}", i, format_argument(arg));
            }
            println!();
        }
        OscPacket::Bundle(bundle) => {
            println!("Received bundle:");
            println!("  Time tag: {:?}", bundle.time_tag);
            println!("  Packets: {}", bundle.packets.len());
            for (i, packet) in bundle.packets.iter().enumerate() {
                println!("  Packet {}:", i);
                print_packet(packet);
            }
            println!();
        }
    }
}

fn format_argument(arg: &OscType) -> String {
    match arg {
        OscType::Int(i) => format!("Int({})", i),
        OscType::Float(f) => format!("Float({})", f),
        OscType::String(s) => format!("String(\"{}\")", s),
        OscType::Blob(b) => format!("Blob({} bytes)", b.len()),
        OscType::Long(l) => format!("Long({})", l),
        OscType::Double(d) => format!("Double({})", d),
        OscType::Time(t) => format!("Time({}, {})", t.seconds, t.fractional),
        OscType::Char(c) => format!("Char('{}')", c),
        OscType::Color(c) => format!("Color(r:{}, g:{}, b:{}, a:{})", c.red, c.green, c.blue, c.alpha),
        OscType::Midi(m) => format!("Midi(port:{}, status:{:#x}, data1:{}, data2:{})", m.port, m.status, m.data1, m.data2),
        OscType::True => "True".to_string(),
        OscType::False => "False".to_string(),
        OscType::Nil => "Nil".to_string(),
        OscType::Impulse => "Impulse".to_string(),
        OscType::Array(arr) => format!("Array({} elements)", arr.len()),
    }
}
