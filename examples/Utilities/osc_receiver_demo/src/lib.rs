//! # osc_receiver_demo
//!
//! Library + binary that receives OSC messages on a UDP port and
//! collects them into a `Vec<ReceivedOscMessage>` for inspection.

use std::sync::{Arc, Mutex};

use logic_nih_plug_osc::argument::OSCArgument;
use logic_nih_plug_osc::receiver::{MessageListener, OscMessageReceivedEvent, OscReceiver};

/// A snapshot of a single received OSC message.
#[derive(Debug, Clone)]
pub struct ReceivedOscMessage {
    /// OSC address (e.g. `/synth/note`).
    pub address: String,
    /// Decoded arguments (stringified for easy printing).
    pub args: Vec<String>,
}

/// A listener that captures every received OSC message into a shared buffer.
pub struct CaptureListener {
    pub messages: Arc<Mutex<Vec<ReceivedOscMessage>>>,
}

impl MessageListener for CaptureListener {
    fn handle_message(&mut self, event: OscMessageReceivedEvent<'_>) {
        let args: Vec<String> = event
            .message
            .args
            .iter()
            .map(arg_to_string)
            .collect();
        let msg = ReceivedOscMessage {
            address: event.message.address.clone(),
            args,
        };
        self.messages.lock().unwrap().push(msg);
    }
}

fn arg_to_string(arg: &OSCArgument) -> String {
    match arg {
        OSCArgument::Int32(v) => format!("Int32({})", v),
        OSCArgument::Int64(v) => format!("Int64({})", v),
        OSCArgument::Float32(v) => format!("Float32({})", v),
        OSCArgument::Float64(v) => format!("Float64({})", v),
        OSCArgument::String(v) => format!("String({:?})", v),
        OSCArgument::Blob(v) => format!("Blob({} bytes)", v.len()),
        OSCArgument::Bool(v) => format!("Bool({})", v),
        OSCArgument::Char(v) => format!("Char({:?})", v),
        OSCArgument::TimeTag(v) => format!("TimeTag({:?})", v),
        OSCArgument::Colour(v) => format!("Colour({:?})", v),
        OSCArgument::MidiMessage(v) => format!("MidiMessage({:?})", v),
        OSCArgument::Array(items) => format!("Array({} items)", items.len()),
        OSCArgument::Nil => "Nil".to_string(),
        OSCArgument::Inf => "Inf".to_string(),
    }
}

/// Open a UDP OSC receiver on the given port, install a
/// [`CaptureListener`] under the given listener name, and return both
/// the receiver and the shared message buffer.
///
/// Pass `port = 0` to bind an ephemeral port.
pub fn start_capture_receiver(
    port: u16,
    listener_name: &str,
) -> Result<(OscReceiver, Arc<Mutex<Vec<ReceivedOscMessage>>>), String> {
    let mut receiver = OscReceiver::connect(port)
        .map_err(|e| format!("Failed to bind OSC receiver: {}", e))?;
    let messages = Arc::new(Mutex::new(Vec::new()));
    let listener = CaptureListener {
        messages: Arc::clone(&messages),
    };
    receiver
        .add_listener(listener_name, listener)
        .map_err(|e| format!("Failed to add OSC listener: {}", e))?;
    Ok((receiver, messages))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_to_string_handles_common_types() {
        assert_eq!(arg_to_string(&OSCArgument::Int32(42)), "Int32(42)");
        assert_eq!(arg_to_string(&OSCArgument::Float32(0.5)), "Float32(0.5)");
        assert_eq!(
            arg_to_string(&OSCArgument::String("hello".to_string())),
            "String(\"hello\")"
        );
        assert_eq!(arg_to_string(&OSCArgument::Bool(true)), "Bool(true)");
    }
}