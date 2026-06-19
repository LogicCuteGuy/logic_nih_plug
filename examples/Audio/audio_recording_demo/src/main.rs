//! Standalone audio recording demo.
//!
//! Records audio from a `MockAudioIODevice` with a synthetic sine input
//! and writes the result to a WAV file.

use audio_recording_demo::{generate_sine, RecordingCapture};
use logic_nih_plug_audio_devices::{
    AudioDeviceInfo, AudioIODevice, AudioIODeviceCallback, AudioIODeviceCallbackData,
    MockAudioIODevice,
};
use logic_nih_plug_audio_formats::wav::WavWriter;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let output_path = args.get(1).map(|s| s.as_str()).unwrap_or("recording_output.wav");

    let sample_rate: f64 = 44_100.0;
    let buffer_size: u32 = 512;
    let duration_secs: f64 = 1.0;
    let bit_depth: u16 = 16;

    // Create a mock device with one input channel.
    let mut device = MockAudioIODevice::new(AudioDeviceInfo {
        name: "Mock Recording Device".to_string(),
        sample_rates: vec![44_100, 48_000],
        buffer_sizes: vec![128, 256, 512, 1024],
        input_channel_names: vec!["Mock In 1".to_string()],
        output_channel_names: vec![],
        input_latency_samples: 0,
        output_latency_samples: 0,
    });

    println!("Opening mock recording device...");
    if let Err(e) = device.open(sample_rate, buffer_size) {
        eprintln!("Failed to open device: {}", e);
        std::process::exit(3);
    }

    // Simulate recording by generating sine and feeding through callback.
    let all_samples = generate_sine(sample_rate, 440.0, duration_secs);
    let mut captured = Vec::new();
    let num_frames_in_buffer = buffer_size as usize;
    let num_input_channels: usize = 1;

    for chunk in all_samples.chunks(num_frames_in_buffer) {
        let input_channels: Vec<&[f32]> = vec![chunk];
        let output_channels: Vec<&mut [f32]> = vec![];
        let data = AudioIODeviceCallbackData::new(&input_channels, &output_channels, chunk.len());
        let mut cb = RecordingCapture::new();
        cb.audio_device_about_to_start(sample_rate, num_frames_in_buffer, num_input_channels, 0);
        cb.audio_device_io_callback(&data);
        captured.extend(cb.into_samples());
    }

    device.stop();
    device.close();

    let peak = captured.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
    println!(
        "✓ Captured {} samples ({:.2} s) — peak: {:.4}",
        captured.len(),
        captured.len() as f64 / sample_rate,
        peak
    );

    // Write WAV.
    let mut writer = WavWriter::create(output_path, sample_rate as f32, 1, bit_depth)
        .expect("Failed to create WAV writer");
    let channels: Vec<Vec<f32>> = vec![captured];
    writer.write_samples(&channels).expect("Failed to write WAV");

    println!("✓ Wrote WAV to {}", output_path);
}
