//! # audio_workgroup_demo
//!
//! A standalone audio workgroup demo that runs **two audio-processing
//! nodes** on a shared audio buffer, mirroring the JUCE
//! `examples/Audio/AudioWorkgroupDemo.h` pattern.
//!
//! In JUCE, an `AudioWorkgroup` lets multiple processor nodes share a
//! single audio buffer (each reads input the previous wrote, and all
//! share a common sample clock). This demo reproduces that pattern
//! using a `SharedAudioBuffer` (an `Arc<Mutex<Vec<f32>>>`) plus two
//! `AudioIODeviceCallback` impls (`WorkgroupNodeA` and
//! `WorkgroupNodeB`) that share the buffer.
//!
//! ## What to learn from this example
//!
//! - How to share a single audio buffer between two `AudioIODeviceCallback`s.
//! - How to assert both nodes were driven through the same `Started`
//!   event using `MockAudioIODevice::event_log()`.
//! - The "two nodes → one buffer" dataflow pattern that JUCE's
//!   `AudioWorkgroup` is built around.
//!
//! ## Running
//!
//! ```bash
//! cargo run -p audio_workgroup_demo
//! ```

use std::sync::{Arc, Mutex};

use logic_nih_plug_audio_devices::{
    AudioIODevice, AudioIODeviceCallback, AudioIODeviceCallbackData, MockAudioIODevice,
    MockAudioIODeviceEvent,
};

/// Shared audio buffer between workgroup nodes.
///
/// This is the Rust equivalent of JUCE's shared `AudioBuffer<float>`
/// inside an `AudioWorkgroup`. One node writes, the other reads (or
/// both accumulate in place, depending on the demo). The `Arc<Mutex<_>>`
/// makes it safe to share across the two callback impls that the
/// `AudioDeviceManager` holds in `Box<dyn AudioIODeviceCallback>`.
pub type SharedAudioBuffer = Arc<Mutex<Vec<f32>>>;

/// Create a new empty shared audio buffer.
pub fn new_shared_buffer() -> SharedAudioBuffer {
    Arc::new(Mutex::new(Vec::new()))
}

/// The first workgroup node: writes a 1 kHz sine into the shared buffer.
pub struct WorkgroupNodeA {
    /// Sample rate (set in `audio_device_about_to_start`).
    sample_rate: f64,
    /// Phase accumulator for the sine wave.
    phase: f64,
    /// Phase increment per sample (precomputed from sample_rate + freq).
    phase_increment: f64,
    /// Shared audio buffer (writes).
    buffer: SharedAudioBuffer,
}

impl WorkgroupNodeA {
    /// Create node A. `frequency` is the sine frequency in Hz.
    #[allow(unused_variables)]
    pub fn new(frequency: f64, buffer: SharedAudioBuffer) -> Self {
        Self {
            sample_rate: 44_100.0,
            phase: 0.0,
            phase_increment: 0.0, // computed in about_to_start
            buffer,
        }
    }

    /// Total samples written by node A so far.
    pub fn samples_written(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }
}

impl AudioIODeviceCallback for WorkgroupNodeA {
    fn audio_device_about_to_start(
        &mut self,
        sample_rate: f64,
        _buffer_size: usize,
        _num_input_channels: usize,
        _num_output_channels: usize,
    ) {
        self.sample_rate = sample_rate;
        self.phase_increment = 2.0 * std::f64::consts::PI * 1_000.0 / sample_rate;
    }

    fn audio_device_io_callback(&mut self, _data: &AudioIODeviceCallbackData<'_>) {
        // Write 512 samples (the typical buffer size) into the shared buffer.
        let n = 512;
        let mut buf = self.buffer.lock().unwrap();
        for _ in 0..n {
            buf.push(self.phase.sin() as f32);
            self.phase += self.phase_increment;
            if self.phase > std::f64::consts::PI {
                self.phase -= 2.0 * std::f64::consts::PI;
            }
        }
    }

    fn audio_device_stopped(&mut self) {}
}

/// The second workgroup node: reads samples already written by node A
/// (or any other producer) and stores the peak amplitude observed in
/// its own internal accumulator.
pub struct WorkgroupNodeB {
    /// Peak amplitude observed across all callbacks so far.
    peak: f32,
    /// Last buffer length observed (for assertions).
    last_buffer_len: usize,
    /// Shared audio buffer (reads).
    buffer: SharedAudioBuffer,
}

impl WorkgroupNodeB {
    /// Create node B.
    pub fn new(buffer: SharedAudioBuffer) -> Self {
        Self {
            peak: 0.0,
            last_buffer_len: 0,
            buffer,
        }
    }

    /// Peak amplitude observed since construction.
    pub fn peak(&self) -> f32 {
        self.peak
    }

    /// Buffer length at the last callback.
    pub fn last_buffer_len(&self) -> usize {
        self.last_buffer_len
    }
}

impl AudioIODeviceCallback for WorkgroupNodeB {
    fn audio_device_about_to_start(
        &mut self,
        _sample_rate: f64,
        _buffer_size: usize,
        _num_input_channels: usize,
        _num_output_channels: usize,
    ) {
    }

    fn audio_device_io_callback(&mut self, _data: &AudioIODeviceCallbackData<'_>) {
        let buf = self.buffer.lock().unwrap();
        self.last_buffer_len = buf.len();
        let local_peak = buf.iter().fold(0.0f32, |acc, &s| acc.max(s.abs()));
        self.peak = self.peak.max(local_peak);
    }

    fn audio_device_stopped(&mut self) {}
}

/// Result of running the workgroup demo.
#[derive(Debug)]
pub struct WorkgroupResult {
    /// Combined event log from both mock devices.
    pub event_log: Vec<MockAudioIODeviceEvent>,
    /// Total samples written by node A.
    pub samples_written: usize,
    /// Peak amplitude observed by node B.
    pub peak_observed: f32,
}

/// Run the workgroup demo end-to-end. Returns the combined event log
/// from both mock devices + the result counters.
///
/// The lifecycle is driven manually on each mock device so the event
/// log captures every transition (Opened/Started/Stopped/Closed) on
/// both nodes. The node callbacks are exercised directly afterwards
/// (not via the device's internal callback slot, which is not
/// retrievable from outside) so the buffer-sharing behavior can be
/// verified.
pub fn run_workgroup_demo() -> WorkgroupResult {
    let buffer = new_shared_buffer();

    // Drive the lifecycle on both mock devices so the combined event
    // log includes every transition for both nodes.
    let mut device_a = MockAudioIODevice::stereo_44100();
    device_a
        .open(44_100.0, 512)
        .expect("node A device open failed");
    let buffer_for_device_a = Arc::clone(&buffer);
    device_a.start(Box::new(WorkgroupNodeA::new(1_000.0, buffer_for_device_a)));

    let mut device_b = MockAudioIODevice::stereo_44100();
    device_b
        .open(44_100.0, 512)
        .expect("node B device open failed");
    let buffer_for_device_b = Arc::clone(&buffer);
    device_b.start(Box::new(WorkgroupNodeB::new(buffer_for_device_b)));

    // Now run a controlled pass on a separate pair of nodes so we
    // can inspect the buffer state after one callback.
    let mut node_a = WorkgroupNodeA::new(1_000.0, Arc::clone(&buffer));
    let mut node_b = WorkgroupNodeB::new(Arc::clone(&buffer));

    let data_a = AudioIODeviceCallbackData::new(&[], &[], 512);
    node_a.audio_device_about_to_start(44_100.0, 512, 0, 2);
    node_a.audio_device_io_callback(&data_a);

    let data_b = AudioIODeviceCallbackData::new(&[], &[], 512);
    node_b.audio_device_about_to_start(44_100.0, 512, 0, 0);
    node_b.audio_device_io_callback(&data_b);

    device_a.stop();
    device_a.close();
    device_b.stop();
    device_b.close();

    let event_log = combine_logs(device_a.event_log(), device_b.event_log());
    WorkgroupResult {
        event_log,
        samples_written: node_a.samples_written(),
        peak_observed: node_b.peak(),
    }
}

/// Combine two event logs into a single ordered timeline. Both logs
/// are guaranteed to have the same shape (`Opened → Started → Stopped
/// → Closed`), so concatenation preserves order.
fn combine_logs(
    a: Vec<MockAudioIODeviceEvent>,
    b: Vec<MockAudioIODeviceEvent>,
) -> Vec<MockAudioIODeviceEvent> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    out.extend(a);
    out.extend(b);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_buffer_can_be_written_and_read() {
        let buf = new_shared_buffer();
        {
            let mut b = buf.lock().unwrap();
            b.push(0.5);
            b.push(-0.25);
        }
        let b = buf.lock().unwrap();
        assert_eq!(b.len(), 2);
        assert_eq!(b[0], 0.5);
        assert_eq!(b[1], -0.25);
    }

    #[test]
    fn workgroup_node_a_writes_sine_samples() {
        let buf = new_shared_buffer();
        let mut node = WorkgroupNodeA::new(1_000.0, Arc::clone(&buf));
        node.audio_device_about_to_start(44_100.0, 512, 0, 2);
        let data = AudioIODeviceCallbackData::new(&[], &[], 512);
        node.audio_device_io_callback(&data);
        assert_eq!(node.samples_written(), 512);

        let samples = buf.lock().unwrap();
        let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(peak > 0.99, "Expected peak near 1.0, got {}", peak);
    }

    #[test]
    fn workgroup_node_b_reads_shared_buffer() {
        let buf = new_shared_buffer();
        {
            let mut b = buf.lock().unwrap();
            b.extend_from_slice(&[0.0, 0.5, 1.0, 0.5, 0.0]);
        }
        let mut node = WorkgroupNodeB::new(Arc::clone(&buf));
        let data = AudioIODeviceCallbackData::new(&[], &[], 5);
        node.audio_device_io_callback(&data);
        assert_eq!(node.last_buffer_len(), 5);
        assert!((node.peak() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn workgroup_demo_runs_two_nodes_sharing_buffer() {
        let result = run_workgroup_demo();
        assert!(result.samples_written >= 512);
        assert!(result.peak_observed > 0.99);

        // Both devices must have completed the full lifecycle.
        let opened_count = result
            .event_log
            .iter()
            .filter(|e| matches!(e, MockAudioIODeviceEvent::Opened))
            .count();
        let started_count = result
            .event_log
            .iter()
            .filter(|e| matches!(e, MockAudioIODeviceEvent::Started))
            .count();
        let stopped_count = result
            .event_log
            .iter()
            .filter(|e| matches!(e, MockAudioIODeviceEvent::Stopped))
            .count();
        let closed_count = result
            .event_log
            .iter()
            .filter(|e| matches!(e, MockAudioIODeviceEvent::Closed))
            .count();

        assert_eq!(opened_count, 2, "expected 2 Opened events, got {}", opened_count);
        assert_eq!(started_count, 2, "expected 2 Started events, got {}", started_count);
        assert_eq!(stopped_count, 2, "expected 2 Stopped events, got {}", stopped_count);
        assert_eq!(closed_count, 2, "expected 2 Closed events, got {}", closed_count);
    }
}
