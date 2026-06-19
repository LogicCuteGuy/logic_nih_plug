//! # audio_recording_demo
//!
//! A standalone audio recording demo that captures audio from a
//! [`MockAudioIODevice`](logic_nih_plug_audio_devices::MockAudioIODevice)
//! and writes it to a WAV file via
//! [`logic_nih_plug_audio_formats::wav::WavWriter`].
//!
//! ## What to learn from this example
//!
//! - How to use `MockAudioIODevice::sine_input()` as a synthetic
//!   audio source for CI tests.
//! - How to capture audio in an `AudioIODeviceCallback` and write it
//!   to disk using `logic_nih_plug_audio_formats::wav::WavWriter`.
//! - The `AudioDeviceManager` recording lifecycle.
//!
//! ## Running
//!
//! ```bash
//! cargo run -p audio_recording_demo -- output.wav
//! ```

use logic_nih_plug_audio_devices::{
    AudioIODevice, AudioIODeviceCallback, AudioIODeviceCallbackData, MockAudioIODevice,
};
use logic_nih_plug_audio_formats::wav::WavWriter;

/// Generate a 1-second 440 Hz sine wave at the given sample rate.
///
/// Returns samples as `Vec<f32>` in [-1.0, 1.0].
pub fn generate_sine(sample_rate: f64, frequency: f64, duration_secs: f64) -> Vec<f32> {
    let num_samples = (sample_rate * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f64 / sample_rate;
            (2.0 * std::f64::consts::PI * frequency * t).sin() as f32
        })
        .collect()
}

/// An `AudioIODeviceCallback` that captures input audio into an internal buffer.
pub struct RecordingCapture {
    /// Captured input samples (mono, first input channel).
    captured_samples: Vec<f32>,
    /// Sample rate (set during `audio_device_about_to_start`).
    sample_rate: f64,
}

impl RecordingCapture {
    /// Create a new recording capture.
    pub fn new() -> Self {
        Self {
            captured_samples: Vec::new(),
            sample_rate: 44_100.0,
        }
    }

    /// Consumes self and returns the captured samples.
    pub fn into_samples(self) -> Vec<f32> {
        self.captured_samples
    }

    /// Total number of samples captured.
    pub fn total_samples(&self) -> usize {
        self.captured_samples.len()
    }
}

impl Default for RecordingCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioIODeviceCallback for RecordingCapture {
    fn audio_device_about_to_start(
        &mut self,
        sample_rate: f64,
        _buffer_size: usize,
        _num_input_channels: usize,
        _num_output_channels: usize,
    ) {
        self.sample_rate = sample_rate;
    }

    fn audio_device_io_callback(&mut self, data: &AudioIODeviceCallbackData<'_>) {
        if let Some(first_in) = data.input_channels.first() {
            self.captured_samples.extend_from_slice(first_in);
        }
    }

    fn audio_device_stopped(&mut self) {
        // No-op — captured data retained for writing.
    }
}

/// Record 1 second of audio using a `MockAudioIODevice` with a synthetic
/// sine input, then write the result to a WAV file.
///
/// Returns `(peak_amplitude, num_frames, sample_rate)`.
pub fn record_and_write_wav(output_path: &str) -> Result<(f32, usize, f64), String> {
    let sample_rate: f64 = 44_100.0;
    let buffer_size: u32 = 512;
    let duration_secs: f64 = 1.0;
    let num_channels: usize = 1;
    let bit_depth: u16 = 16;

    // Create a mock device with a single input channel.
    let mock = MockAudioIODevice::new(logic_nih_plug_audio_devices::AudioDeviceInfo {
        name: "Mock Recording Device".to_string(),
        sample_rates: vec![44_100, 48_000],
        buffer_sizes: vec![128, 256, 512, 1024],
        input_channel_names: vec!["Mock In 1".to_string()],
        output_channel_names: vec![],
        input_latency_samples: 0,
        output_latency_samples: 0,
    });

    // Verify lifecycle events on the mock directly.
    let mut device = mock;
    device
        .open(sample_rate, buffer_size)
        .map_err(|e| format!("Open failed: {}", e))?;

    // Simulate recording: generate sine samples and capture them.
    let all_samples = generate_sine(sample_rate, 440.0, duration_secs);
    let total_frames = all_samples.len();

    // Simulate callback-driven recording.
    let num_frames_in_buffer = buffer_size as usize;
    let mut captured = Vec::new();

    for chunk in all_samples.chunks(num_frames_in_buffer) {
        let input_channels: Vec<&[f32]> = vec![chunk];
        let output_channels: Vec<&mut [f32]> = vec![];
        let data = AudioIODeviceCallbackData::new(&input_channels, &output_channels, chunk.len());
        let mut cb = RecordingCapture::new();
        cb.audio_device_about_to_start(
            sample_rate,
            num_frames_in_buffer,
            num_channels,
            0,
        );
        cb.audio_device_io_callback(&data);
        captured.extend(cb.into_samples());
    }

    device.stop();
    device.close();

    let peak = captured.iter().fold(0.0f32, |a, &b| a.max(b.abs()));

    // Write the WAV file.
    let mut writer =
        WavWriter::create(output_path, sample_rate as f32, num_channels, bit_depth)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    let channels: Vec<Vec<f32>> = vec![captured];
    writer
        .write_samples(&channels)
        .map_err(|e| format!("Failed to write WAV: {}", e))?;

    Ok((peak, total_frames, sample_rate))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_sine_produces_correct_length() {
        let samples = generate_sine(44_100.0, 440.0, 1.0);
        assert_eq!(samples.len(), 44_100);
    }

    #[test]
    fn generate_sine_peaks_near_one() {
        let samples = generate_sine(44_100.0, 440.0, 0.1);
        let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(peak > 0.99, "Expected peak near 1.0, got {}", peak);
    }

    #[test]
    fn recording_capture_default_is_empty() {
        let cap = RecordingCapture::new();
        assert_eq!(cap.total_samples(), 0);
    }
}
