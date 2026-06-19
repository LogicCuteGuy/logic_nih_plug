//! Standalone audio workgroup demo.
//!
//! Runs two audio-processing nodes sharing a single audio buffer,
//! demonstrating the JUCE `AudioWorkgroup` pattern. Each node is backed
//! by its own `MockAudioIODevice`; the lifecycle is driven manually so
//! both nodes' event logs can be asserted against.

use audio_workgroup_demo::run_workgroup_demo;

fn main() {
    let result = run_workgroup_demo();

    println!(
        "✓ Workgroup demo: node A wrote {} samples, node B observed peak {:.4}",
        result.samples_written, result.peak_observed,
    );

    let opened = result
        .event_log
        .iter()
        .filter(|e| matches!(e, logic_nih_plug_audio_devices::MockAudioIODeviceEvent::Opened))
        .count();
    let started = result
        .event_log
        .iter()
        .filter(|e| matches!(e, logic_nih_plug_audio_devices::MockAudioIODeviceEvent::Started))
        .count();
    let stopped = result
        .event_log
        .iter()
        .filter(|e| matches!(e, logic_nih_plug_audio_devices::MockAudioIODeviceEvent::Stopped))
        .count();
    let closed = result
        .event_log
        .iter()
        .filter(|e| matches!(e, logic_nih_plug_audio_devices::MockAudioIODeviceEvent::Closed))
        .count();

    println!(
        "✓ Combined event log: Opened×{} Started×{} Stopped×{} Closed×{}",
        opened, started, stopped, closed
    );

    if opened == 2 && started == 2 && stopped == 2 && closed == 2 {
        println!("✓ Both workgroup nodes completed the full lifecycle");
    } else {
        eprintln!("⚠ Workgroup lifecycle incomplete");
        std::process::exit(3);
    }
}
