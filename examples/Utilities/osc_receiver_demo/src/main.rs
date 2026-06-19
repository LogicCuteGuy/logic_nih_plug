//! CLI demo: bind a UDP OSC receiver and print every incoming message.
//!
//! Usage: `cargo run -p osc_receiver_demo -- [port]`
//!
//! Default: port 9000.

use std::time::Duration;

use osc_receiver_demo::start_capture_receiver;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9000);

    let (_receiver, messages) = match start_capture_receiver(port, "capture") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("⚠ {}", e);
            std::process::exit(3);
        }
    };

    println!(
        "✓ OSC receiver bound on port {} (Ctrl-C to stop)",
        port
    );

    // Sleep briefly to let messages arrive, then dump them.
    std::thread::sleep(Duration::from_millis(200));
    let msgs = messages.lock().unwrap();
    if msgs.is_empty() {
        println!("(no messages received)");
    } else {
        for m in msgs.iter() {
            println!("  ← {} {:?}", m.address, m.args);
        }
        println!("✓ Received {} messages", msgs.len());
    }
}