//! CLI demo: send a sequence of OSC messages to a remote host.
//!
//! Usage: `cargo run -p osc_sender_demo -- [host] [port] [count]`
//!
//! Default: 127.0.0.1:9000, 3 messages.

use logic_nih_plug_osc::argument::OSCArgument;
use logic_nih_plug_osc::message::OSCMessage;
use logic_nih_plug_osc::sender::OscSender;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let host = args.get(1).map(|s| s.as_str()).unwrap_or("127.0.0.1");
    let port: u16 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9000);
    let count: usize = args
        .get(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);

    let sender = match OscSender::connect(host, port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⚠ Failed to connect OSC sender: {}", e);
            std::process::exit(3);
        }
    };

    println!("✓ Connected OSC sender to {}:{}", host, port);
    for i in 0..count {
        let msg = OSCMessage::new(
            "/synth/note",
            &[
                OSCArgument::Int32(60 + i as i32),
                OSCArgument::Float32(0.75),
                OSCArgument::Int32(1000 + i as i32),
            ],
        );
        match sender.send(&msg) {
            Ok(()) => println!("  → sent #{}: {}", i, msg.address),
            Err(e) => {
                eprintln!("⚠ Send failed: {}", e);
                std::process::exit(3);
            }
        }
    }
    println!("✓ Sent {} OSC messages", count);
}