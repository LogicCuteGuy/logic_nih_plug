//! # wav_reader
//!
//! CLI demo that prints a WAV file's header summary (sample rate,
//! channel count, bit depth, duration) using
//! `logic_nih_plug_audio_formats::wav::WavReader`.
//!
//! ## What this example ports
//!
//! - **JUCE source file**: `examples/Utilities/CMDLineDemo.h` (WAV reader variant)
//! - **What to learn**: how to use the framework's `WavReader` to
//!   parse a WAV file and print its metadata + peak amplitude.

use logic_nih_plug_audio_formats::wav::WavReader;

/// Summary of a WAV file's metadata + content.
#[derive(Debug, Clone)]
pub struct WavSummary {
    /// Sample rate in Hz.
    pub sample_rate: f32,
    /// Number of channels.
    pub num_channels: usize,
    /// Bits per sample, if known.
    pub bit_depth: Option<u16>,
    /// Number of frames (samples per channel).
    pub num_frames: usize,
    /// Peak amplitude across all channels and frames.
    pub peak_amplitude: f32,
}

impl WavSummary {
    /// Duration in seconds.
    pub fn duration_secs(&self) -> f32 {
        if self.sample_rate > 0.0 {
            self.num_frames as f32 / self.sample_rate
        } else {
            0.0
        }
    }
}

/// Read a WAV file and return its summary.
pub fn summarize_wav(path: &str) -> Result<WavSummary, String> {
    let mut reader = WavReader::open(path)
        .map_err(|e| format!("Failed to open WAV: {}", e))?;
    let metadata = reader.metadata();
    let sample_rate = reader.sample_rate();
    let num_frames = reader.num_frames();
    let bit_depth = reader.bit_depth();
    let num_channels = metadata.num_channels;

    let channels = reader
        .read_all()
        .map_err(|e| format!("Failed to read WAV: {}", e))?;
    let peak = channels
        .iter()
        .flat_map(|c| c.iter())
        .fold(0.0f32, |a, &b| a.max(b.abs()));

    Ok(WavSummary {
        sample_rate,
        num_channels,
        bit_depth,
        num_frames,
        peak_amplitude: peak,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_reference_1khz_sine_wav() {
        let path = std::env::current_dir()
            .unwrap()
            .ancestors()
            .find(|p| p.join("examples/audio-assets/sine_1khz_1s.wav").exists())
            .map(|p| p.join("examples/audio-assets/sine_1khz_1s.wav"))
            .expect("could not locate reference WAV");

        let summary = summarize_wav(path.to_str().unwrap()).unwrap();
        assert!(
            (summary.sample_rate - 44_100.0).abs() < 1.0,
            "expected 44.1 kHz, got {}",
            summary.sample_rate
        );
        assert!(summary.num_channels >= 1);
        assert!(
            (summary.duration_secs() - 1.0).abs() < 0.05,
            "expected ~1.0 s, got {}",
            summary.duration_secs()
        );
        assert!(
            summary.peak_amplitude > 0.45,
            "expected peak > 0.45, got {}",
            summary.peak_amplitude
        );
    }

    #[test]
    fn summarize_returns_err_for_missing_file() {
        let result = summarize_wav("/this/does/not/exist.wav");
        assert!(result.is_err());
    }
}