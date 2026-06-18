//! Pitch shifting without changing duration.
//!
//! [`PitchShift`] wraps a [`PhaseVocoder`] to shift the pitch of an
//! audio signal by a configurable ratio while keeping the output
//! duration equal to the input duration.
//!
//! A `pitch_ratio` of `1.0` is unity (no change). `2.0` is one octave
//! up, `0.5` is one octave down. The ratio can also be derived from
//! semitones: `ratio = 2.0_f32.powf(semitones / 12.0)`.
//!
//! # Algorithm
//!
//! Internally the pitch shifter runs the phase vocoder with a synthesis
//! hop that is scaled by `1 / pitch_ratio`. This *changes* both pitch
//! **and** duration. To restore the original duration, the stretched
//! signal is resampled at the original rate.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_dsp::processors::pitch::pitch_shift::{PitchShift, PitchShiftParameters};
//!
//! let mut shifter = PitchShift::new();
//! shifter.prepare(44100.0, 512);
//! shifter.set_parameters(PitchShiftParameters {
//!     pitch_ratio: 1.5,
//!     enabled: true,
//! });
//!
//! let input  = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 512];
//! shifter.process(&input, &mut output);
//! // output now contains the pitch-shifted signal
//! ```

use super::phase_vocoder::PhaseVocoder;

/// Parameters for [`PitchShift`].
#[derive(Debug, Clone)]
pub struct PitchShiftParameters {
    /// Pitch ratio. `1.0` = no change, `2.0` = +1 octave,
    /// `0.5` = −1 octave. Clamped to `[0.25, 4.0]`.
    pub pitch_ratio: f32,
    /// When `false`, the processor bypasses processing and copies
    /// input to output unchanged.
    pub enabled: bool,
}

impl Default for PitchShiftParameters {
    fn default() -> Self {
        Self {
            pitch_ratio: 1.0,
            enabled: true,
        }
    }
}

/// Real-time pitch shifter using a phase vocoder.
///
/// Shifts pitch without changing duration. The processing latency
/// equals one FFT frame (typically 1024 samples at 44.1 kHz ≈ 23 ms).
pub struct PitchShift {
    phase_vocoder: PhaseVocoder,
    /// Current pitch ratio.
    pitch_ratio: f32,
    /// Whether processing is enabled.
    enabled: bool,
    /// Sample rate (stored for resampling calculation).
    sample_rate: f32,
}

impl PitchShift {
    /// Creates a new pitch shifter with default FFT size (1024).
    ///
    /// The FFT size determines frequency resolution vs latency trade-off.
    pub fn new() -> Self {
        Self::with_fft_size(1024)
    }

    /// Creates a new pitch shifter with the given FFT size.
    ///
    /// Must be a power of two (256, 512, 1024, 2048, …).
    /// Larger → better quality but higher latency and CPU.
    pub fn with_fft_size(fft_size: usize) -> Self {
        let phase_vocoder = PhaseVocoder::new(fft_size, 4);
        Self {
            phase_vocoder,
            pitch_ratio: 1.0,
            enabled: true,
            sample_rate: 44100.0,
        }
    }

    /// Initialises for the given sample rate and block size.
    pub fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.phase_vocoder.prepare(sample_rate);
    }

    /// Applies parameter changes.
    pub fn set_parameters(&mut self, params: PitchShiftParameters) {
        self.pitch_ratio = params.pitch_ratio.clamp(0.25, 4.0);
        self.enabled = params.enabled;
    }

    /// Returns the current pitch ratio.
    pub fn pitch_ratio(&self) -> f32 {
        self.pitch_ratio
    }

    /// Returns the processing latency in samples (one FFT frame).
    pub fn latency(&self) -> usize {
        self.phase_vocoder.fft_size()
    }

    /// Returns `true` if processing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Resets the processor state.
    pub fn reset(&mut self) {
        self.phase_vocoder.reset();
    }

    /// Processes a block of mono audio, shifting the pitch.
    ///
    /// `input` and `output` must have the same length. When bypassed
    /// (disabled), `output` is a copy of `input`.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        if !self.enabled || (self.pitch_ratio - 1.0).abs() < 1e-6 {
            let n = input.len().min(output.len());
            output[..n].copy_from_slice(&input[..n]);
            return;
        }

        // For pitch ratio r, we time-stretch by 1/r (which changes both
        // pitch and duration), then resample to restore the original
        // duration while preserving the pitch change.
        //
        // Step 1: run phase vocoder with synthesis_hop = analysis_hop / r
        // Step 2: linearly interpolate resample the output
        let fft_size = self.phase_vocoder.fft_size();
        let analysis_hop = self.phase_vocoder.analysis_hop();
        let synthesis_hop = (analysis_hop as f32 / self.pitch_ratio) as usize;
        let synthesis_hop = synthesis_hop.max(1);

        self.phase_vocoder.set_synthesis_hop(synthesis_hop);

        // The stretched output is longer/shorter by factor 1/pitch_ratio.
        // We need to produce `input.len()` output samples.
        let stretched_len = ((input.len() as f32) / self.pitch_ratio * 1.1) as usize + fft_size;
        let mut stretched = vec![0.0f32; stretched_len];
        self.phase_vocoder.process(input, &mut stretched);

        // Step 2: resample stretched → output at pitch_ratio rate
        let out_len = output.len();
        for i in 0..out_len {
            let src_pos = i as f32 * self.pitch_ratio;
            let idx = src_pos as usize;
            let frac = src_pos - idx as f32;

            if idx + 1 < stretched.len() {
                output[i] = stretched[idx] * (1.0 - frac) + stretched[idx + 1] * frac;
            } else if idx < stretched.len() {
                output[i] = stretched[idx];
            } else {
                output[i] = 0.0;
            }
        }
    }
}

impl Default for PitchShift {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts a pitch ratio to semitones.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::processors::pitch::pitch_shift::ratio_to_semitones;
///
/// assert!((ratio_to_semitones(2.0) - 12.0).abs() < 0.01);
/// assert!((ratio_to_semitones(0.5) - (-12.0)).abs() < 0.01);
/// ```
pub fn ratio_to_semitones(ratio: f32) -> f32 {
    12.0 * ratio.log2()
}

/// Converts semitones to a pitch ratio.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::processors::pitch::pitch_shift::semitones_to_ratio;
///
/// assert!((semitones_to_ratio(12.0) - 2.0).abs() < 0.01);
/// assert!((semitones_to_ratio(-12.0) - 0.5).abs() < 0.01);
/// ```
pub fn semitones_to_ratio(semitones: f32) -> f32 {
    2.0_f32.powf(semitones / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unity_ratio_passthrough() {
        let mut shifter = PitchShift::new();
        shifter.prepare(44100.0, 512);
        shifter.set_parameters(PitchShiftParameters {
            pitch_ratio: 1.0,
            enabled: true,
        });

        let input: Vec<f32> = (0..512)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; 512];
        shifter.process(&input, &mut output);

        // Unity ratio should pass through unchanged
        for (o, i) in output.iter().zip(input.iter()) {
            assert!((o - i).abs() < 1e-6);
        }
    }

    #[test]
    fn bypass_copies_input() {
        let mut shifter = PitchShift::new();
        shifter.prepare(44100.0, 512);
        shifter.set_parameters(PitchShiftParameters {
            pitch_ratio: 2.0,
            enabled: false,
        });

        let input = vec![0.3f32; 256];
        let mut output = vec![0.0f32; 256];
        shifter.process(&input, &mut output);

        for (o, i) in output.iter().zip(input.iter()) {
            assert!((o - i).abs() < 1e-6);
        }
    }

    #[test]
    fn octave_up_preserves_energy() {
        let mut shifter = PitchShift::with_fft_size(1024);
        shifter.prepare(44100.0, 512);
        shifter.set_parameters(PitchShiftParameters {
            pitch_ratio: 2.0,
            enabled: true,
        });

        // One full second of 440 Hz sine at 44100 Hz
        let n = 44100;
        let input: Vec<f32> = (0..n)
            .map(|i| 0.5 * (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; n];
        shifter.process(&input, &mut output);

        // Output should have non-zero energy
        let rms: f32 = (output.iter().map(|s| s * s).sum::<f32>() / n as f32).sqrt();
        assert!(rms > 0.01, "output RMS {rms} too low — pitch shift failed");
    }

    #[test]
    fn ratio_clamped() {
        let mut shifter = PitchShift::new();
        shifter.prepare(44100.0, 512);
        shifter.set_parameters(PitchShiftParameters {
            pitch_ratio: 10.0, // should be clamped to 4.0
            enabled: true,
        });
        assert!((shifter.pitch_ratio() - 4.0).abs() < 1e-6);
    }

    #[test]
    fn semitone_conversions() {
        assert!((ratio_to_semitones(1.0)).abs() < 1e-6);
        assert!((ratio_to_semitones(2.0) - 12.0).abs() < 0.01);
        assert!((ratio_to_semitones(0.5) + 12.0).abs() < 0.01);

        assert!((semitones_to_ratio(0.0) - 1.0).abs() < 1e-6);
        assert!((semitones_to_ratio(12.0) - 2.0).abs() < 0.01);
        assert!((semitones_to_ratio(-12.0) - 0.5).abs() < 0.01);
    }

    #[test]
    fn latency_is_fft_size() {
        let shifter = PitchShift::with_fft_size(2048);
        assert_eq!(shifter.latency(), 2048);
    }

    #[test]
    fn reset_clears_state() {
        let mut shifter = PitchShift::new();
        shifter.prepare(44100.0, 512);

        let input = vec![0.5f32; 512];
        let mut output = vec![0.0f32; 512];
        shifter.set_parameters(PitchShiftParameters {
            pitch_ratio: 2.0,
            enabled: true,
        });
        shifter.process(&input, &mut output);

        shifter.reset();
        // Should not panic and should produce output
        shifter.process(&input, &mut output);
    }
}
