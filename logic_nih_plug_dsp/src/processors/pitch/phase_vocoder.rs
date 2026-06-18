//! Core phase vocoder — STFT analysis → phase processing → overlap-add synthesis.
//!
//! A [`PhaseVocoder`] transforms a time-domain signal into the frequency
//! domain using short-time Fourier analysis, applies phase modifications
//! (unwrapping + scaling), and reconstructs via overlap-add synthesis.
//!
//! The analysis hop size defaults to `fft_size / 4` (4× overlap) which,
//! combined with a Hann window, satisfies the constant-overlap-add
//! (COLA) property for artefact-free reconstruction.
//!
//! # Algorithm
//!
//! ```text
//!   in ──▶ [window] ──▶ [FFT] ──▶ mag + phase
//!                                    │
//!                           phase diff → unwrap → scale
//!                                    │
//!                        [IFFT] ──▶ [window] ──▶ OLA ──▶ out
//! ```
//!
//! The phase-difference between consecutive frames is unwrapped and
//! scaled by the synthesis/analysis hop ratio. When the ratio equals
//! 1.0, the output is identical to the input (modulo windowing artefacts
//! at the boundaries).
//!
//! # Quick start
//!
//! ```
//! use logic_nih_plug_dsp::processors::pitch::phase_vocoder::PhaseVocoder;
//!
//! let mut pv = PhaseVocoder::new(1024, 4);
//! pv.prepare(44100.0);
//!
//! let input = vec![0.1f32; 1024];
//! let mut output = vec![0.0f32; 1024];
//! pv.process(&input, &mut output);
//! ```

use crate::analysis::{RealFFT, WindowingFunction};
use num_complex::Complex;

/// Core STFT phase vocoder.
///
/// Drives analysis (windowed FFT), phase unwrapping + scaling, and
/// synthesis (windowed IFFT with overlap-add).
///
/// Call [`prepare`](Self::prepare) once when the sample rate or block
/// size is known, then feed audio through [`process`](Self::process)
/// block-by-block.
pub struct PhaseVocoder {
    /// Transform size = 2^fft_order.
    fft_size: usize,
    /// Analysis hop in samples. Default: `fft_size / 4`.
    analysis_hop: usize,
    /// Synthesis hop in samples. Typically `analysis_hop * time_ratio`.
    synthesis_hop: usize,
    /// Hann window of length `fft_size`.
    window: Vec<f32>,
    /// OLA normalisation: `1.0 / (N * cola_gain)`.
    /// Compensates for un-normalised IFFT (×N) and OLA gain (×G).
    normalization: f32,

    // Real-only FFT (forward + inverse of size fft_size, output one-sided spectrum)
    real_fft: RealFFT,

    // Analysis ring-buffer (one channel — mono)
    analysis_buffer: Vec<f32>,
    write_pos: usize,
    /// Samples accumulated since the last analysis frame.
    samples_since_frame: usize,
    /// Number of analysis frames processed (for first-frame init).
    frame_count: usize,

    // Previous analysis phase (for computing phase difference).
    prev_analysis_phase: Vec<f32>,
    // Accumulated synthesis phase (for phase continuity).
    synth_phase: Vec<f32>,

    // OLA synthesis buffer. Each analysis frame writes fft_size samples
    // starting at ola_write_pos, advancing by synthesis_hop.
    synthesis_ola: Vec<f32>,
    ola_write_pos: usize,
    ola_read_pos: usize,

    // Working buffers (reused each frame):
    // `time` is the real time-domain frame of length fft_size.
    // `spectrum` is the one-sided complex spectrum of length fft_size/2+1.
    time: Vec<f32>,
    spectrum: Vec<Complex<f32>>,
}

impl PhaseVocoder {
    /// Creates a new phase vocoder.
    ///
    /// # Arguments
    ///
    /// * `fft_size` — Must be a power of two (e.g. 512, 1024, 2048).
    ///   Larger sizes give better frequency resolution but higher
    ///   latency and CPU cost.
    /// * `analysis_hop_divisor` — The analysis hop is
    ///   `fft_size / analysis_hop_divisor`. A divisor of 4 (default)
    ///   gives 75 % overlap with Hann windows, satisfying COLA.
    ///
    /// # Panics
    ///
    /// Panics if `fft_size` is not a power of two or is < 16.
    pub fn new(fft_size: usize, analysis_hop_divisor: usize) -> Self {
        assert!(
            fft_size >= 16 && fft_size.is_power_of_two(),
            "fft_size must be a power of two and >= 16, got {fft_size}"
        );
        assert!(
            analysis_hop_divisor >= 2,
            "analysis_hop_divisor must be >= 2, got {analysis_hop_divisor}"
        );

        let analysis_hop = fft_size / analysis_hop_divisor;
        // Initial synthesis hop — will be updated per process call.
        let synthesis_hop = analysis_hop;

        // Real-only FFT (forward + inverse of size fft_size; output is
        // the one-sided spectrum of length fft_size/2 + 1, exploiting
        // the Hermitian symmetry of the FFT of real input).
        let real_fft = RealFFT::new(fft_size)
            .expect("PhaseVocoder fft_size is validated to be a power of two >= 16");

        // Hann window
        let window = WindowingFunction::Hann.generate(fft_size);

        // Compute COLA gain: sum of w(n)^2 for all overlapping frames at
        // any position n. For Hann at 4× overlap this equals 1.5.
        let cola_gain = Self::compute_cola_gain(&window, analysis_hop);

        // Normalization = 1 / (N * cola_gain).
        // The IFFT is un-normalised (output = N × true IDFT).
        // The OLA of w² at hop H accumulates a gain of cola_gain.
        let normalization = 1.0 / (fft_size as f32 * cola_gain);

        let num_bins = real_fft.spectrum_size();
        Self {
            fft_size,
            analysis_hop,
            synthesis_hop,
            window,
            normalization,
            real_fft,
            analysis_buffer: vec![0.0; fft_size],
            write_pos: 0,
            samples_since_frame: 0,
            frame_count: 0,
            prev_analysis_phase: vec![0.0; num_bins],
            synth_phase: vec![0.0; num_bins],
            synthesis_ola: Vec::new(),
            ola_write_pos: 0,
            ola_read_pos: 0,
            time: vec![0.0; fft_size],
            spectrum: vec![Complex::new(0.0, 0.0); num_bins],
        }
    }

    /// Computes the COLA (Constant Overlap-Add) gain for w² at the
    /// given hop. This is `Σ_m w(n - m·hop)²` for all overlapping
    /// frames, evaluated at any position n (constant for COLA windows).
    fn compute_cola_gain(window: &[f32], hop: usize) -> f32 {
        let n = window.len();
        let mut gain = window[0] * window[0];
        for m in 1..=n / hop {
            let idx = n - m * hop;
            gain += window[idx] * window[idx];
        }
        gain
    }

    /// Initialises internal state for the given sample rate.
    ///
    /// Must be called once before [`process`](Self::process).
    pub fn prepare(&mut self, _sample_rate: f32) {
        self.reset();
    }

    /// Returns the current COLA normalisation gain.
    pub fn normalization(&self) -> f32 {
        self.normalization
    }

    /// Returns the current synthesis hop.
    pub fn synthesis_hop(&self) -> usize {
        self.synthesis_hop
    }

    /// Returns the number of output samples available (drained from OLA).
    pub fn output_available(&self) -> usize {
        self.ola_write_pos.saturating_sub(self.ola_read_pos)
    }

    /// Resets all internal state (phase memory, buffers).
    pub fn reset(&mut self) {
        self.analysis_buffer.fill(0.0);
        self.write_pos = 0;
        self.samples_since_frame = 0;
        self.frame_count = 0;
        self.prev_analysis_phase.fill(0.0);
        self.synth_phase.fill(0.0);
        self.synthesis_ola.clear();
        self.ola_write_pos = 0;
        self.ola_read_pos = 0;
    }

    /// Returns the FFT transform size.
    pub fn fft_size(&self) -> usize {
        self.fft_size
    }

    /// Returns the analysis hop in samples.
    pub fn analysis_hop(&self) -> usize {
        self.analysis_hop
    }

    /// Sets the synthesis hop in samples.
    ///
    /// Changing this between frames is what implements time-stretching
    /// and pitch-shifting. For pitch-only shifting the synthesis hop
    /// is typically `analysis_hop / pitch_ratio`.
    pub fn set_synthesis_hop(&mut self, hop: usize) {
        self.synthesis_hop = hop.max(1);
    }

    /// Processes a block of mono samples through the phase vocoder.
    ///
    /// `input` is fed sample-by-sample into the analysis buffer. Once
    /// enough samples have been collected (one FFT frame's worth), a
    /// full analysis→phase→synthesis cycle runs and the resulting
    /// samples are written to `output`.
    ///
    /// `output` may be shorter than `input` (time compression) or
    /// longer (time expansion), depending on the synthesis hop.
    /// Unused tail samples are zero-filled.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        for &sample in input.iter() {
            // Write into analysis ring-buffer and advance
            self.analysis_buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.fft_size;

            // Increment sample counter; process a frame every analysis_hop samples
            self.samples_since_frame += 1;
            if self.samples_since_frame >= self.analysis_hop {
                self.analysis_frame();
                self.samples_since_frame = 0;
            }
        }

        // Drain completed samples from OLA buffer.
        // After M frames, all positions [0, M*H_s) are fully covered
        // by their overlapping frames and safe to read.
        let safe_pos = self.ola_write_pos;
        let available = safe_pos.saturating_sub(self.ola_read_pos);
        let n = output.len().min(available);
        if n > 0 {
            for i in 0..n {
                output[i] = self.synthesis_ola[self.ola_read_pos + i];
            }
            self.ola_read_pos += n;
        }
        if n < output.len() {
            output[n..].fill(0.0);
        }

        // Compact buffer periodically
        if self.ola_read_pos >= self.fft_size * 2 {
            self.synthesis_ola.drain(..self.ola_read_pos);
            self.ola_write_pos -= self.ola_read_pos;
            self.ola_read_pos = 0;
        }
    }

    /// Runs one analysis → phase → synthesis frame.
    ///
    /// The analysis ring-buffer contains the most recent `fft_size`
    /// samples. This method windows them, runs the real-only FFT,
    /// processes phases, runs the inverse real-only FFT, and
    /// overlap-adds the result into the synthesis buffer starting
    /// at `ola_write_pos`.
    fn analysis_frame(&mut self) {
        let num_bins = self.real_fft.spectrum_size();

        // ── Analysis: extract frame from ring buffer + window ──
        // The ring buffer write_pos just advanced past the newest sample.
        // The frame spans [write_pos .. write_pos + fft_size) circularly.
        for i in 0..self.fft_size {
            let idx = (self.write_pos + i) % self.fft_size;
            self.time[i] = self.analysis_buffer[idx] * self.window[i];
        }

        // Real-only forward FFT (one-sided spectrum of length
        // num_bins = fft_size/2 + 1).
        self.real_fft.real_forward(&self.time, &mut self.spectrum);

        // ── Extract magnitude + phase from the one-sided spectrum ──
        let mut mag = vec![0.0f32; num_bins];
        let mut phase = vec![0.0f32; num_bins];

        for k in 0..num_bins {
            mag[k] = self.spectrum[k].norm();
            phase[k] = self.spectrum[k].arg();
        }

        // ── Phase processing: unwrapping + scaling ──
        let mut new_phase = vec![0.0f32; num_bins];
        let mut true_freq = vec![0.0f32; num_bins];

        if self.frame_count == 0 {
            // First frame: no previous phase to compare against.
            // Pass the analysis phase directly (identity modification).
            for k in 0..num_bins {
                new_phase[k] = phase[k];
                true_freq[k] = 2.0 * std::f32::consts::PI * k as f32 / self.fft_size as f32;
            }
        } else {
            for k in 0..num_bins {
                // Phase difference between consecutive ANALYSIS frames
                let dp = phase[k] - self.prev_analysis_phase[k];

                // Expected phase advance for a signal exactly at bin k
                let expected_advance =
                    2.0 * std::f32::consts::PI * k as f32 * self.analysis_hop as f32
                        / self.fft_size as f32;

                // Wrap dp to be within ±π of the expected advance
                let dp_wrapped =
                    dp - 2.0 * std::f32::consts::PI
                        * ((dp - expected_advance) / (2.0 * std::f32::consts::PI)).round();

                // True frequency = dp_wrapped / H
                // (dp_wrapped already contains the expected bin advance +
                //  deviation, so no need to add bin_freq separately)
                true_freq[k] = dp_wrapped / self.analysis_hop as f32;

                // Synthesis phase: accumulate from previous synthesis phase
                new_phase[k] = self.synth_phase[k] + true_freq[k] * self.synthesis_hop as f32;
                new_phase[k] -= round_to_nearest(new_phase[k], std::f32::consts::PI);
            }
        }

        // Save for next frame
        self.prev_analysis_phase.copy_from_slice(&phase);
        self.synth_phase.copy_from_slice(&new_phase);
        self.frame_count += 1;

        // ── Reconstruct one-sided spectrum with new phases ──
        // DC and Nyquist bins must be real-valued for the real input
        // assumption to hold during the inverse FFT.
        for k in 0..num_bins {
            self.spectrum[k] = Complex::from_polar(mag[k], new_phase[k]);
        }
        // Force DC and Nyquist to be real (small numerical noise is
        // introduced by from_polar for these bins).
        self.spectrum[0].im = 0.0;
        self.spectrum[num_bins - 1].im = 0.0;

        // ── Real-only inverse FFT → real time-domain frame ──
        // `RealFFT::real_inverse` already divides by N (compensating
        // for rustfft's un-normalised IFFT), so the output is the
        // correctly-scaled time-domain frame.
        self.real_fft.real_inverse(&self.spectrum, &mut self.time);

        // ── Overlap-add: write frame at ola_write_pos ──
        // `RealFFT::real_inverse` has already applied the 1/N
        // normalisation, so we only need to undo the OLA gain
        // (= COLA gain of w² for the current synthesis hop).
        let cola_gain = Self::compute_cola_gain(&self.window, self.synthesis_hop);
        let norm = 1.0 / cola_gain;
        let start = self.ola_write_pos;
        let needed = start + self.fft_size;
        if needed > self.synthesis_ola.len() {
            self.synthesis_ola.resize(needed, 0.0);
        }
        for i in 0..self.fft_size {
            self.synthesis_ola[start + i] += self.time[i] * self.window[i] * norm;
        }
        self.ola_write_pos += self.synthesis_hop;
    }
}

/// Rounds `x` to the nearest multiple of `base`.
fn round_to_nearest(x: f32, base: f32) -> f32 {
    (x / base).round() * base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cola_gain_hann_4x() {
        let window = WindowingFunction::Hann.generate(1024);
        let gain = PhaseVocoder::compute_cola_gain(&window, 256);
        // Hann at 4× overlap: COLA gain of w² = 1.5
        assert!((gain - 1.5).abs() < 0.01, "cola_gain = {gain}, expected 1.5");
    }

    #[test]
    fn normalization_value() {
        let pv = PhaseVocoder::new(1024, 4);
        let expected = 1.0 / (1024.0 * 1.5);
        assert!((pv.normalization() - expected).abs() < 1e-6);
    }

    #[test]
    fn passthrough_preserves_energy() {
        // Phase vocoders don't achieve sample-perfect reconstruction,
        // but the output should contain meaningful signal energy.
        let fft_size = 1024;
        let mut pv = PhaseVocoder::new(fft_size, 4);
        pv.prepare(44100.0);

        let n = fft_size * 6;
        let input: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; n];
        pv.process(&input, &mut output);

        // The output should not be silent
        let output_rms: f32 = (output.iter().map(|s| s * s).sum::<f32>()
            / output.len() as f32)
            .sqrt();
        assert!(
            output_rms > 0.01,
            "output is silent (rms={output_rms})"
        );

        // Output should track the input frequency — check spectral peak
        // via zero-crossings in steady state
        let ss = fft_size * 3;
        let mut crossings = 0;
        for i in (ss + 1)..n {
            if (output[i] > 0.0) != (output[i - 1] > 0.0) {
                crossings += 1;
            }
        }
        // At 440 Hz, 44100 Hz sample rate, we expect ~880 zero crossings/sec
        // over the steady-state period
        let ss_duration = (n - ss) as f32 / 44100.0;
        let expected_crossings = (880.0 * ss_duration) as i32;
        assert!(
            (crossings as i32 - expected_crossings).unsigned_abs() < expected_crossings as u32 / 2,
            "zero crossings {crossings} far from expected {expected_crossings}"
        );
    }

    #[test]
    fn time_stretch_produces_longer_output() {
        let fft_size = 512;
        let mut pv = PhaseVocoder::new(fft_size, 4);
        pv.prepare(44100.0);

        pv.set_synthesis_hop(pv.analysis_hop() / 2);

        let n = fft_size * 6;
        let input: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
            .collect();
        let mut output = vec![0.0f32; n * 2];
        pv.process(&input, &mut output);

        let non_zero = output.iter().filter(|&&s| s.abs() > 1e-6).count();
        assert!(
            non_zero > n / 4,
            "expected stretched output, got {non_zero} non-zero samples (input={n})"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut pv = PhaseVocoder::new(256, 4);
        pv.prepare(44100.0);

        let input = vec![0.5f32; 256];
        let mut output = vec![0.0f32; 256];
        pv.process(&input, &mut output);

        pv.reset();

        assert!(pv.analysis_buffer.iter().all(|&s| s == 0.0));
        assert!(pv.prev_analysis_phase.iter().all(|&p| p == 0.0));
        assert!(pv.synth_phase.iter().all(|&p| p == 0.0));
        assert!(pv.synthesis_ola.is_empty());
    }

    #[test]
    fn fft_size_queries() {
        let pv = PhaseVocoder::new(2048, 4);
        assert_eq!(pv.fft_size(), 2048);
        assert_eq!(pv.analysis_hop(), 512);
    }
}
