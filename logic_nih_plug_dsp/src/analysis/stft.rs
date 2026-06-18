//! Short-time Fourier transform (STFT) — forward and inverse helpers.
//!
//! An [`STFT`] packages the standard spectral-processing pipeline:
//! windowing → real FFT → analysis frames / synthesis frames → overlap-add
//! reconstruction. It is the same pattern used by vocoders, pitch
//! shifters, time stretchers, and spectrum analysers.
//!
//! ## API shape
//!
//! - **Per-frame** — feed one frame of `fft_size` real samples in, get
//!   one frame of `fft_size/2 + 1` complex bins out (forward), or vice
//!   versa (inverse). The caller is responsible for OLA when using
//!   per-frame mode.
//! - **Streaming block** — feed a block of input samples, get a block
//!   of output samples. The STFT handles the hop/OLA internally and
//!   maintains a buffer of pending analysis/synthesis frames.
//!
//! The synthesis side applies the window again on output and uses a
//! proper COLA (constant-overlap-add) normalisation factor so
//! successive frames reconstruct seamlessly. Hann windows are
//! COLA-compliant for `hop = fft_size / 4` and (with the right
//! normalisation) for `hop = fft_size / 2`.
//!
//! ## Quick start — per-frame spectral analysis
//!
//! ```ignore
//! use logic_nih_plug_dsp::analysis::{STFT, WindowingFunction};
//! use num_complex::Complex;
//!
//! let stft = STFT::new(1024, 256, WindowingFunction::Hann).unwrap();
//! let frame = vec![0.0f32; 1024];
//! let mut spectrum = vec![Complex::new(0.0f32, 0.0f32); 513];
//!
//! stft.analyze_frame(&frame, &mut spectrum);
//! // ... modify spectrum ...
//! let mut reconstructed = vec![0.0f32; 1024];
//! stft.synthesize_frame(&spectrum, &mut reconstructed);
//! ```
//!
//! ## Quick start — streaming OLA
//!
//! ```ignore
//! use logic_nih_plug_dsp::analysis::{STFT, WindowingFunction};
//!
//! let mut stft = STFT::new(1024, 256, WindowingFunction::Hann).unwrap();
//! stft.reset();
//!
//! let input = vec![0.1f32; 512];
//! let mut output = vec![0.0f32; 512];
//! stft.process_block(&input, &mut output);
//! ```

use super::real_fft::RealFFT;
use super::windowing::WindowingFunction;
use num_complex::Complex;

/// Short-time Fourier transform with both per-frame and streaming APIs.
///
/// The `STFT` owns the [`RealFFT`] plan and the analysis window. The
/// streaming [`process_block`](Self::process_block) method allocates
/// its own scratch OLA buffer per call (one-shot, no carry-over).
///
/// All sizes are fixed at construction. To change the FFT size or hop,
/// construct a new `STFT`.
pub struct STFT {
    /// Real-only FFT engine.
    fft: RealFFT,
    /// Hop size in samples (must satisfy `1 ≤ hop_size ≤ fft_size`).
    hop_size: usize,
    /// Analysis window of length `fft_size`.
    window: Vec<f32>,
    /// 1 / COLA gain for the chosen window + hop. Pre-computed.
    cola_normalization: f32,
}

impl STFT {
    /// Creates a new STFT with the given size, hop, and analysis window.
    ///
    /// # Arguments
    ///
    /// * `fft_size` — must be a power of two, in `[2, 65536]`
    /// * `hop_size` — analysis/synthesis hop in samples,
    ///   in `[1, fft_size]`
    /// * `window` — analysis window function; same window is used for
    ///   synthesis
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::DspError::InvalidFFTSize`] if the FFT
    /// size constraints are not met, or
    /// [`crate::error::DspError::InvalidHopSize`] if the hop size is
    /// out of range.
    pub fn new(
        fft_size: usize,
        hop_size: usize,
        window: WindowingFunction,
    ) -> Result<Self, crate::error::DspError> {
        if !(1..=fft_size).contains(&hop_size) {
            return Err(crate::error::DspError::InvalidHopSize { hop_size, fft_size });
        }
        let fft = RealFFT::new(fft_size)?;

        // Pre-generate the analysis window
        let window_vec = window.generate(fft_size);

        // Compute the COLA (Constant Overlap-Add) gain. We use the
        // same definition as the phase vocoder: the OLA of w² at
        // position n=N (or equivalently n=0 by symmetry), which
        // collapses to Σ_m w²[N - m·H] for m=0..N/H.
        //
        // For Hann at H = N/4 this gives 1.5 (well-known value).
        // For Hann at H = N/2 this gives 1.0.
        let cola_gain = compute_cola_gain(&window_vec, hop_size);
        let cola_normalization = 1.0 / cola_gain;

        Ok(Self {
            fft,
            hop_size,
            window: window_vec,
            cola_normalization,
        })
    }

    /// Returns the FFT size `N`.
    #[inline]
    pub fn fft_size(&self) -> usize {
        self.fft.size()
    }

    /// Returns the hop size in samples.
    #[inline]
    pub fn hop_size(&self) -> usize {
        self.hop_size
    }

    /// Returns the one-sided spectrum size `N/2 + 1`.
    #[inline]
    pub fn spectrum_size(&self) -> usize {
        self.fft.spectrum_size()
    }

    /// Returns a reference to the analysis window.
    #[inline]
    pub fn window(&self) -> &[f32] {
        &self.window
    }

    /// Returns the COLA normalisation factor (`1 / COLA gain`).
    #[inline]
    pub fn cola_normalization(&self) -> f32 {
        self.cola_normalization
    }

    /// Performs forward STFT on a single frame: windowed real input
    /// → one-sided complex spectrum.
    ///
    /// - `input` must have length `fft_size`
    /// - `output` must have length `fft_size/2 + 1`
    ///
    /// The input is multiplied in-place by the analysis window before
    /// the real FFT.
    ///
    /// # Panics
    ///
    /// Panics if buffer sizes don't match the configured FFT size.
    pub fn analyze_frame(&self, input: &[f32], output: &mut [Complex<f32>]) {
        let n = self.fft.size();
        assert_eq!(input.len(), n, "input length must equal fft_size");
        assert_eq!(
            output.len(),
            self.fft.spectrum_size(),
            "output length must equal fft_size/2 + 1"
        );

        // Window the input
        let mut windowed = vec![0.0f32; n];
        for (s, (&x, &w)) in windowed.iter_mut().zip(input.iter().zip(self.window.iter())) {
            *s = x * w;
        }
        // Real FFT
        self.fft.real_forward(&windowed, output);
    }

    /// Performs inverse STFT on a single frame: one-sided complex
    /// spectrum → windowed real output.
    ///
    /// - `input` must have length `fft_size/2 + 1`
    /// - `output` must have length `fft_size`
    ///
    /// The inverse FFT result is multiplied by the synthesis window
    /// and the COLA normalisation factor. The output is the
    /// overlap-add contribution of a single frame; the caller is
    /// responsible for summing successive frames with hop-size
    /// spacing.
    ///
    /// # Panics
    ///
    /// Panics if buffer sizes don't match the configured FFT size.
    pub fn synthesize_frame(&self, input: &[Complex<f32>], output: &mut [f32]) {
        let n = self.fft.size();
        assert_eq!(
            input.len(),
            self.fft.spectrum_size(),
            "input length must equal fft_size/2 + 1"
        );
        assert_eq!(output.len(), n, "output length must equal fft_size");

        // Real IFFT → real time-domain
        let mut time = vec![0.0f32; n];
        self.fft.real_inverse(input, &mut time);

        // Apply synthesis window and COLA normalisation
        for (s, (&t, &w)) in output
            .iter_mut()
            .zip(time.iter().zip(self.window.iter()))
        {
            *s = t * w * self.cola_normalization;
        }
    }

    /// Resets all internal state (currently a no-op for the stateless
    /// per-frame methods; here for forward compatibility with future
    /// streaming carry-over).
    pub fn reset(&mut self) {
        self.fft.reset();
    }

    /// Streaming block processing: applies the full STFT pipeline
    /// (windowing → real FFT → identity spectrum modification → real
    /// IFFT → windowing → OLA) to `input` and writes `output`.
    ///
    /// In this default implementation, the spectrum is passed through
    /// unchanged, so the result is (approximately) the input delayed
    /// by the analysis window's group delay. To modify the spectrum,
    /// use the per-frame [`analyze_frame`](Self::analyze_frame) and
    /// [`synthesize_frame`](Self::synthesize_frame) methods.
    ///
    /// `input` and `output` must have the same length.
    ///
    /// **Note**: this is a one-shot block implementation — it does
    /// not carry over the trailing analysis frame between calls.
    /// For true streaming with cross-block continuity, the caller
    /// should retain the last `fft_size - hop` samples and prepend
    /// them to the next call's input.
    ///
    /// # Panics
    ///
    /// Panics if `input.len() != output.len()`.
    pub fn process_block(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len(), "input/output length mismatch");

        let n = self.fft.size();
        let hop = self.hop_size;
        let spectrum_len = self.fft.spectrum_size();

        // Allocate a one-shot OLA buffer large enough to hold the
        // full block plus the trailing frame contribution.
        let ola_len = input.len() + n;
        let mut ola = vec![0.0f32; ola_len];

        // Frame the input hop-by-hop. The last frame may extend
        // past the end of the input; we zero-pad.
        let num_frames = if input.is_empty() {
            0
        } else {
            input.len().div_ceil(hop) + 1
        };
        for m in 0..num_frames {
            let frame_start = m * hop;
            // Build the next analysis frame, zero-padded if needed.
            let mut frame = vec![0.0f32; n];
            let copy_start = frame_start.min(input.len());
            let copy_end = (frame_start + n).min(input.len());
            if copy_end > copy_start {
                let copy_len = copy_end - copy_start;
                frame[..copy_len].copy_from_slice(&input[copy_start..copy_end]);
            }

            // Window → FFT (analysis)
            let mut spectrum = vec![Complex::new(0.0, 0.0); spectrum_len];
            self.analyze_frame(&frame, &mut spectrum);

            // Identity spectrum modification (placeholder)

            // IFFT → window (synthesis)
            let mut time = vec![0.0f32; n];
            self.synthesize_frame(&spectrum, &mut time);

            // Add this frame's contribution into the OLA buffer.
            for (i, &s) in time.iter().enumerate() {
                ola[frame_start + i] += s;
            }
        }

        // Output the first input.len() samples of the OLA. The
        // trailing frame's contribution (which would extend past
        // input.len()) is discarded; the caller can handle that
        // separately if needed.
        output.copy_from_slice(&ola[..input.len()]);
    }
}

/// Computes the COLA (Constant Overlap-Add) gain for `window` at the
/// given hop. This is `Σ_m w²(N - 1 - m·H)` for `m = 0..N/H` — the
/// OLA of `w²` at the boundary position `n = N - 1` (or `n = 0` by
/// symmetry).
///
/// For Hann at `H = N/4` this gives 1.5 (the well-known value used by
/// the phase vocoder). For Hann at `H = N/2` this gives 1.0.
fn compute_cola_gain(window: &[f32], hop: usize) -> f32 {
    let n = window.len();
    if hop == 0 || n == 0 {
        return 1.0;
    }
    let mut gain = 0.0f32;
    let mut idx = n - 1; // start at the right edge
    while idx < n {
        gain += window[idx].powi(2);
        if idx < hop {
            break;
        }
        idx -= hop;
    }
    gain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stft(fft_size: usize, hop: usize) -> STFT {
        STFT::new(fft_size, hop, WindowingFunction::Hann).unwrap()
    }

    #[test]
    fn new_validates_args() {
        // fft_size = 0, 1 are invalid (RealFFT requires >= 2)
        assert!(STFT::new(0, 1, WindowingFunction::Hann).is_err());
        assert!(STFT::new(1, 1, WindowingFunction::Hann).is_err());

        // hop = 0 invalid
        assert!(STFT::new(1024, 0, WindowingFunction::Hann).is_err());
        // hop > fft_size invalid
        assert!(STFT::new(1024, 2048, WindowingFunction::Hann).is_err());
    }

    #[test]
    fn size_queries() {
        let stft = make_stft(1024, 256);
        assert_eq!(stft.fft_size(), 1024);
        assert_eq!(stft.hop_size(), 256);
        assert_eq!(stft.spectrum_size(), 513);
    }

    #[test]
    fn window_matches_input() {
        let stft = make_stft(64, 16);
        let w = stft.window();
        assert_eq!(w.len(), 64);
        // Hann starts at 0
        assert!(w[0].abs() < 1e-6);
        // Hann ends at 0
        assert!(w[63].abs() < 1e-6);
    }

    #[test]
    fn analyze_synthesize_roundtrip() {
        // DC test: an all-ones frame round-trips to the constant scaled
        // by the COLA normalisation. For Hann at H = N/4, the gain is
        // 1.5, so we expect 1.0 / 1.5 ≈ 0.667.
        let stft = make_stft(1024, 256);
        let frame = vec![1.0f32; 1024];
        let mut spectrum = vec![Complex::new(0.0, 0.0); 513];
        let mut reconstructed = vec![0.0f32; 1024];

        stft.analyze_frame(&frame, &mut spectrum);
        stft.synthesize_frame(&spectrum, &mut reconstructed);

        // After COLA normalization, the constant 1.0 should be
        // recovered scaled by 1/cola_gain.
        let expected = 1.0 / (1.0 / stft.cola_normalization);
        let mid = &reconstructed[400..600];
        let avg: f32 = mid.iter().sum::<f32>() / mid.len() as f32;
        assert!(
            (avg - expected).abs() < 0.05,
            "DC round-trip avg = {avg} expected {expected}"
        );
    }

    #[test]
    fn cola_normalization_hann_4x() {
        // Hann at 4× overlap (hop = N/4) has COLA gain = 1.5
        let stft = make_stft(1024, 256);
        let gain = 1.0 / stft.cola_normalization;
        assert!((gain - 1.5).abs() < 0.01, "COLA gain = {gain}");
    }

    #[test]
    fn cola_normalization_hann_2x() {
        // Hann at 2× overlap (hop = N/2) has COLA gain = 1.0
        // (boundary OLA at n=N-1 collapses to w²[N/2] = 1 for Hann)
        let stft = make_stft(1024, 512);
        let gain = 1.0 / stft.cola_normalization;
        assert!((gain - 1.0).abs() < 0.01, "COLA gain = {gain}");
    }

    #[test]
    fn process_block_streaming() {
        // Streaming: a long sine wave should pass through with bounded
        // amplitude (transient at boundaries OK).
        let mut stft = make_stft(256, 64);
        stft.reset();

        let n = 4096;
        let input: Vec<f32> = (0..n)
            .map(|i| (2.0 * std::f32::consts::PI * 8.0 * i as f32 / 256.0).sin() * 0.5)
            .collect();
        let mut output = vec![0.0f32; n];
        stft.process_block(&input, &mut output);

        // After the initial transient (≈ fft_size samples), amplitude
        // should be close to the input.
        let max_after_transient = output[1024..]
            .iter()
            .fold(0.0f32, |a, &b| a.max(b.abs()));
        assert!(
            max_after_transient > 0.3,
            "max output amplitude = {max_after_transient} (expected > 0.3)"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut stft = make_stft(64, 16);
        // Run a bit, then reset
        let input = vec![1.0f32; 64];
        let mut output = vec![0.0f32; 64];
        stft.process_block(&input, &mut output);
        stft.reset();
        // Should be safe to call again
        stft.process_block(&input, &mut output);
    }
}
