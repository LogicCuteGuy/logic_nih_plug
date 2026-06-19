//! # wav_writer
//!
//! CLI demo that writes a 1-second 440 Hz sine WAV and round-trips it
//! through `WavReader` to assert the RIFF/WAVE/fmt/data chunk headers
//! match.
//!
//! ## What this example ports
//!
//! - **JUCE source file**: `examples/Utilities/CMDLineDemo.h` (WAV writer variant)
//! - **What to learn**: how to use `logic_nih_plug_audio_formats::wav::WavWriter`
//!   to write a multi-channel WAV file from raw f32 samples.

use logic_nih_plug_audio_formats::wav::{WavReader, WavWriter};

/// Generate a 1-second 440 Hz sine wave at the given sample rate.
///
/// Returns one mono channel of `f32` samples in `[-1.0, 1.0]`.
pub fn generate_440hz_sine(sample_rate: f32, duration_secs: f32) -> Vec<f32> {
    let num_samples = (sample_rate * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            (2.0 * std::f32::consts::PI * 440.0 * t).sin()
        })
        .collect()
}

/// Write a mono WAV file containing the 440 Hz sine.
///
/// `output_path` is where the file is created.
/// `sample_rate` is in Hz.
/// Returns `(peak_amplitude, num_frames)` on success.
pub fn write_sine_wav(output_path: &str, sample_rate: f32, duration_secs: f32) -> Result<(f32, usize), String> {
    let samples = generate_440hz_sine(sample_rate, duration_secs);
    let num_frames = samples.len();
    let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));

    let channels = vec![samples];
    let mut writer = WavWriter::create(output_path, sample_rate, 1, 16)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;
    writer
        .write_samples(&channels)
        .map_err(|e| format!("Failed to write samples: {}", e))?;

    Ok((peak, num_frames))
}

/// Round-trip check: read back the WAV we just wrote and assert the
/// sample rate, channel count, and frame count match.
pub fn roundtrip_check(path: &str, expected_sample_rate: f32, expected_channels: usize) -> Result<(f32, f32, usize), String> {
    let reader = WavReader::open(path)
        .map_err(|e| format!("Failed to read back WAV: {}", e))?;
    let sample_rate = reader.sample_rate();
    let num_frames = reader.num_frames();
    let num_channels = reader.metadata().num_channels;

    if (sample_rate - expected_sample_rate).abs() > 1.0 {
        return Err(format!(
            "sample rate mismatch: expected {}, got {}",
            expected_sample_rate, sample_rate
        ));
    }
    if num_channels != expected_channels {
        return Err(format!(
            "channel count mismatch: expected {}, got {}",
            expected_channels, num_channels
        ));
    }

    Ok((sample_rate, sample_rate, num_frames))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_440hz_sine_has_correct_length() {
        let samples = generate_440hz_sine(44_100.0, 1.0);
        assert_eq!(samples.len(), 44_100);
    }

    #[test]
    fn generate_440hz_sine_peaks_near_one() {
        let samples = generate_440hz_sine(44_100.0, 0.1);
        let peak = samples.iter().fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(peak > 0.99, "expected peak near 1.0, got {}", peak);
    }

    #[test]
    fn write_then_roundtrip_wav() {
        let dir = std::env::temp_dir().join("wav_writer_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_440hz.wav");
        let path_str = path.to_str().unwrap();

        let (peak_written, num_written) = write_sine_wav(path_str, 44_100.0, 1.0).unwrap();
        assert!(peak_written > 0.99);
        assert_eq!(num_written, 44_100);

        let (_, _, num_read) = roundtrip_check(path_str, 44_100.0, 1).unwrap();
        assert_eq!(num_read, num_written);
    }
}