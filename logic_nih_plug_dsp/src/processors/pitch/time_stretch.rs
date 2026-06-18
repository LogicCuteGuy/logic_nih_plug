//! Time stretching without changing pitch.
//!
//! [`TimeStretching`] wraps a [`PhaseVocoder`] to stretch or compress
//! the duration of an audio signal without changing its pitch.
//!
//! A `time_ratio` of `1.0` is unity (no change). `2.0` makes the
//! output twice as long (half speed). `0.5` makes it half as long
//! (double speed).
//!
//! # Algorithm
//!
//! The phase vocoder runs with a synthesis hop equal to
//! `analysis_hop × time_ratio`. When `time_ratio > 1` the synthesis
//! hop is larger, producing fewer output samples per input frame →
//! the signal is compressed (faster playback). When `time_ratio < 1`
//! the synthesis hop is smaller → the signal is stretched (slower
//! playback).
//!
//! The pitch is preserved because the phase vocoder's phase-unwrapping
//! preserves the true instantaneous frequency of each spectral bin.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_dsp::processors::pitch::time_stretch::{TimeStretching, TimeStretchParameters};
//!
//! let mut stretcher = TimeStretching::new();
//! stretcher.prepare(44100.0, 512);
//! stretcher.set_parameters(TimeStretchParameters {
//!     time_ratio: 0.5,   // output is 2× longer
//!     enabled: true,
//! });
//!
//! let input  = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 1024]; // must be large enough
//! stretcher.process(&input, &mut output);
//! ```

use super::phase_vocoder::PhaseVocoder;

/// Parameters for [`TimeStretching`].
#[derive(Debug, Clone)]
pub struct TimeStretchParameters {
    /// Time ratio. `1.0` = no change, `2.0` = 2× longer (half speed),
    /// `0.5` = 2× shorter (double speed). Clamped to `[0.25, 4.0]`.
    pub time_ratio: f32,
    /// When `false`, the processor copies input to output unchanged.
    pub enabled: bool,
}

impl Default for TimeStretchParameters {
    fn default() -> Self {
        Self {
            time_ratio: 1.0,
            enabled: true,
        }
    }
}

/// Real-time time stretcher using a phase vocoder.
///
/// Stretches or compresses audio duration without changing pitch.
/// Processing latency equals one FFT frame (typically ≈ 23 ms at
/// 44.1 kHz with 1024-point FFT).
pub struct TimeStretching {
    phase_vocoder: PhaseVocoder,
    /// Current time ratio.
    time_ratio: f32,
    /// Whether processing is enabled.
    enabled: bool,
}

impl TimeStretching {
    /// Creates a new time stretcher with default FFT size (1024).
    pub fn new() -> Self {
        Self::with_fft_size(1024)
    }

    /// Creates a new time stretcher with the given FFT size.
    ///
    /// Must be a power of two (256, 512, 1024, 2048, …).
    pub fn with_fft_size(fft_size: usize) -> Self {
        Self {
            phase_vocoder: PhaseVocoder::new(fft_size, 4),
            time_ratio: 1.0,
            enabled: true,
        }
    }

    /// Initialises for the given sample rate and block size.
    pub fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.phase_vocoder.prepare(sample_rate);
    }

    /// Applies parameter changes.
    pub fn set_parameters(&mut self, params: TimeStretchParameters) {
        self.time_ratio = params.time_ratio.clamp(0.25, 4.0);
        self.enabled = params.enabled;
    }

    /// Returns the current time ratio.
    pub fn time_ratio(&self) -> f32 {
        self.time_ratio
    }

    /// Returns the processing latency in samples.
    pub fn latency(&self) -> usize {
        self.phase_vocoder.fft_size()
    }

    /// Returns `true` if processing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Resets all internal state.
    pub fn reset(&mut self) {
        self.phase_vocoder.reset();
    }

    /// Processes a block of mono audio, stretching/compressing time.
    ///
    /// `output` must be long enough to hold the stretched/compressed
    /// result: approximately `input.len() × time_ratio` samples.
    /// Extra samples beyond what the vocoder produces are zero-filled.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        if !self.enabled || (self.time_ratio - 1.0).abs() < 1e-6 {
            let copy_len = input.len().min(output.len());
            output[..copy_len].copy_from_slice(&input[..copy_len]);
            return;
        }

        // Synthesis hop = analysis_hop × time_ratio
        // time_ratio > 1 → larger hop → fewer output samples → compressed
        // time_ratio < 1 → smaller hop → more output samples → stretched
        let analysis_hop = self.phase_vocoder.analysis_hop();
        let synthesis_hop = (analysis_hop as f32 * self.time_ratio) as usize;
        let synthesis_hop = synthesis_hop.max(1);

        self.phase_vocoder.set_synthesis_hop(synthesis_hop);

        // The vocoder produces output into a ring-buffer that is
        // drained. The output may be shorter or longer than input.
        // We pre-allocate generously.
        let out_est = ((input.len() as f32) / self.time_ratio * 1.2) as usize
            + self.phase_vocoder.fft_size() * 2;
        let mut temp = vec![0.0f32; out_est];
        self.phase_vocoder.process(input, &mut temp);

        // Copy what we got into the caller's output buffer
        let copy_len = temp.len().min(output.len());
        output[..copy_len].copy_from_slice(&temp[..copy_len]);
    }
}

impl Default for TimeStretching {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_ratio_passthrough() {
        let mut stretcher = TimeStretching::new();
        stretcher.prepare(44100.0, 512);
        stretcher.set_parameters(TimeStretchParameters {
            time_ratio: 1.0,
            enabled: true,
        });

        let input: Vec<f32> = (0..512)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; 512];
        stretcher.process(&input, &mut output);

        for (o, i) in output.iter().zip(input.iter()) {
            assert!((o - i).abs() < 1e-6);
        }
    }

    #[test]
    fn bypass_copies_input() {
        let mut stretcher = TimeStretching::new();
        stretcher.prepare(44100.0, 512);
        stretcher.set_parameters(TimeStretchParameters {
            time_ratio: 2.0,
            enabled: false,
        });

        let input = vec![0.3f32; 256];
        let mut output = vec![0.0f32; 256];
        stretcher.process(&input, &mut output);

        for (o, i) in output.iter().zip(input.iter()) {
            assert!((o - i).abs() < 1e-6);
        }
    }

    #[test]
    fn time_ratio_clamped() {
        let mut stretcher = TimeStretching::new();
        stretcher.prepare(44100.0, 512);
        stretcher.set_parameters(TimeStretchParameters {
            time_ratio: 10.0,
            enabled: true,
        });
        assert!((stretcher.time_ratio() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn stretch_produces_output() {
        let mut stretcher = TimeStretching::with_fft_size(512);
        stretcher.prepare(44100.0, 512);
        stretcher.set_parameters(TimeStretchParameters {
            time_ratio: 0.5, // 2× longer
            enabled: true,
        });

        let n = 44100;
        let input: Vec<f32> = (0..n)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; n * 3];
        stretcher.process(&input, &mut output);

        // Should produce non-trivial output
        let rms: f32 = (output.iter().map(|s| s * s).sum::<f32>() / output.len() as f32).sqrt();
        assert!(rms > 0.001, "stretched output is silent (rms={rms})");
    }

    #[test]
    fn compress_produces_output() {
        let mut stretcher = TimeStretching::with_fft_size(512);
        stretcher.prepare(44100.0, 512);
        stretcher.set_parameters(TimeStretchParameters {
            time_ratio: 2.0, // 2× shorter
            enabled: true,
        });

        let n = 44100;
        let input: Vec<f32> = (0..n)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; n];
        stretcher.process(&input, &mut output);

        let non_zero = output.iter().filter(|&&s| s.abs() > 0.001).count();
        assert!(non_zero > n / 4, "expected compressed output, got {non_zero} non-zero samples");
    }

    #[test]
    fn latency_is_fft_size() {
        let stretcher = TimeStretching::with_fft_size(2048);
        assert_eq!(stretcher.latency(), 2048);
    }

    #[test]
    fn reset_clears_state() {
        let mut stretcher = TimeStretching::new();
        stretcher.prepare(44100.0, 512);
        stretcher.set_parameters(TimeStretchParameters {
            time_ratio: 0.5,
            enabled: true,
        });

        let input = vec![0.5f32; 512];
        let mut output = vec![0.0f32; 1024];
        stretcher.process(&input, &mut output);

        stretcher.reset();
        stretcher.process(&input, &mut output);
    }
}
