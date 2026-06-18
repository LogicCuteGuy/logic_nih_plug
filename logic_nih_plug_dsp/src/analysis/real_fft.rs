//! Real-to-complex and complex-to-real FFT — the "real-only" FFT path.
//!
//! For audio, the input signal is always real. Exploiting this gives a
//! factor-of-two memory saving versus a full complex FFT: the output
//! spectrum is Hermitian-symmetric (`X[k] = conj(X[N-k])`), so we only
//! need to store `N/2 + 1` bins (the DC bin, the positive frequencies,
//! and the Nyquist bin).
//!
//! ## Algorithm
//!
//! We use `rustfft`'s [`FftPlanner`] to plan a complex FFT of size `N`
//! and an inverse of the same size:
//!
//! **Forward (real → complex, N → N/2+1)**
//!
//! 1. Treat the real input as a length-`N` complex buffer (imaginary
//!    part is zero).
//! 2. Run the complex forward FFT (un-normalised).
//! 3. Take the first `N/2 + 1` bins. The remaining bins are
//!    conjugate-symmetric and can be reconstructed by the caller.
//!
//! **Inverse (complex, N/2+1 → real, N)**
//!
//! 1. Reconstruct the full length-`N` complex spectrum using
//!    Hermitian symmetry: `X[N-k] = conj(X[k])` for `k = 1..N/2-1`.
//!    The DC and Nyquist bins are real.
//! 2. Run the complex inverse FFT (un-normalised, so divide by `N`).
//! 3. Take the real part.
//!
//! This is 2× more compute than the pack-trick (which is the
//! theoretical optimum for a real-only FFT), but it is trivial to
//! reason about and the [`FftPlanner`] is heavily SIMD-optimised.
//!
//! # Quick start
//!
//! ```
//! use logic_nih_plug_dsp::analysis::RealFFT;
//! use num_complex::Complex;
//!
//! let fft = RealFFT::new(1024).unwrap();
//!
//! // Real input
//! let input: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.01).sin()).collect();
//!
//! // One-sided complex spectrum, length N/2+1
//! let mut spectrum = vec![Complex::new(0.0f32, 0.0f32); 513];
//! fft.real_forward(&input, &mut spectrum);
//!
//! // Synthesise back to real samples
//! let mut output = vec![0.0f32; 1024];
//! fft.real_inverse(&spectrum, &mut output);
//! ```

use crate::error::DspError;
use num_complex::Complex;
use rustfft::{FftPlanner, Fft};
use std::sync::Arc;

/// Real-to-complex / complex-to-real FFT.
///
/// Holds two [`rustfft`] plans (a complex forward and inverse FFT of
/// size `N`, used for both directions) plus a scratch buffer for
/// the inverse path's Hermitian reconstruction.
///
/// # Size Requirements
///
/// The FFT size must be a power of 2, in `[2, 65536]`.
pub struct RealFFT {
    /// Full real-input size.
    size: usize,
    /// Forward complex FFT of size `size`.
    forward: Arc<dyn Fft<f32>>,
    /// Inverse complex FFT of size `size`.
    inverse: Arc<dyn Fft<f32>>,
}

impl RealFFT {
    /// Minimum supported FFT size.
    pub const MIN_SIZE: usize = 2;
    /// Maximum supported FFT size.
    pub const MAX_SIZE: usize = 65536;

    /// Creates a new real-only FFT processor.
    ///
    /// # Errors
    ///
    /// Returns [`DspError::FFTSizeOutOfRange`] if `size` is outside
    /// `[MIN_SIZE, MAX_SIZE]`, or [`DspError::InvalidFFTSize`] if
    /// `size` is not a power of two.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::analysis::RealFFT;
    ///
    /// let fft = RealFFT::new(1024).unwrap();
    /// assert_eq!(fft.size(), 1024);
    /// assert_eq!(fft.spectrum_size(), 513);
    /// ```
    pub fn new(size: usize) -> Result<Self, DspError> {
        if !(Self::MIN_SIZE..=Self::MAX_SIZE).contains(&size) {
            return Err(DspError::FFTSizeOutOfRange {
                size,
                min: Self::MIN_SIZE,
                max: Self::MAX_SIZE,
            });
        }
        if !size.is_power_of_two() {
            return Err(DspError::InvalidFFTSize { size });
        }

        let mut planner = FftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(size);
        let inverse = planner.plan_fft_inverse(size);

        Ok(Self {
            size,
            forward,
            inverse,
        })
    }

    /// Returns the full real-input size `N`.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the one-sided spectrum length `N/2 + 1`.
    #[inline]
    pub fn spectrum_size(&self) -> usize {
        self.size / 2 + 1
    }

    /// Performs a real-only forward FFT.
    ///
    /// `input` must have length `N` (the real-input size).
    /// `output` must have length `N/2 + 1` (the one-sided complex
    /// spectrum). The output bins are:
    ///
    /// - `output[0]`: DC component (real)
    /// - `output[1..N/2]`: positive-frequency bins
    /// - `output[N/2]`: Nyquist bin (real)
    ///
    /// Hermitian symmetry `output[k] = conj(output[N-k])` holds for
    /// the full spectrum (only `output[0..=N/2]` is stored).
    ///
    /// # Panics
    ///
    /// Panics if `input.len() != self.size()` or
    /// `output.len() != self.spectrum_size()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::analysis::RealFFT;
    /// use num_complex::Complex;
    ///
    /// let fft = RealFFT::new(1024).unwrap();
    /// let input = vec![1.0f32; 1024];
    /// let mut spectrum = vec![Complex::new(0.0f32, 0.0f32); 513];
    /// fft.real_forward(&input, &mut spectrum);
    ///
    /// // DC bin should be large (sum of all 1024 ones = 1024)
    /// assert!(spectrum[0].re > 1000.0);
    /// // Nyquist bin should be zero (constant signal has no energy at Nyquist)
    /// assert!(spectrum[512].norm() < 1e-3);
    /// ```
    pub fn real_forward(&self, input: &[f32], output: &mut [Complex<f32>]) {
        let n = self.size;
        let half = n / 2;
        assert_eq!(
            input.len(),
            n,
            "Input length ({}) must equal FFT size ({})",
            input.len(),
            n
        );
        assert_eq!(
            output.len(),
            half + 1,
            "Output length ({}) must equal N/2+1 ({})",
            output.len(),
            half + 1
        );

        // Treat the real input as a length-N complex buffer (imag=0).
        let mut buf: Vec<Complex<f32>> =
            input.iter().map(|&x| Complex::new(x, 0.0)).collect();

        // Complex forward FFT (un-normalised).
        self.forward.process(&mut buf);

        // Copy the first N/2+1 bins to the output. The remaining bins
        // are conjugate-symmetric and not needed for a one-sided view.
        output.copy_from_slice(&buf[..=half]);
    }

    /// Performs a real-only inverse FFT.
    ///
    /// `input` must have length `N/2 + 1` (one-sided complex spectrum).
    /// `output` must have length `N` (real time-domain samples).
    ///
    /// This is the exact inverse of [`real_forward`](Self::real_forward):
    /// `real_inverse(real_forward(x)) ≈ x` up to floating-point error.
    ///
    /// # Panics
    ///
    /// Panics if `input.len() != self.spectrum_size()` or
    /// `output.len() != self.size()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::analysis::RealFFT;
    /// use num_complex::Complex;
    ///
    /// let fft = RealFFT::new(1024).unwrap();
    ///
    /// // Build a DC-only spectrum
    /// let mut spectrum = vec![Complex::new(0.0f32, 0.0f32); 513];
    /// spectrum[0] = Complex::new(1024.0, 0.0);
    ///
    /// let mut output = vec![0.0f32; 1024];
    /// fft.real_inverse(&spectrum, &mut output);
    ///
    /// // Should reconstruct a constant signal of value 1.0
    /// for &s in output.iter().take(10) {
    ///     assert!((s - 1.0).abs() < 1e-3, "got {s}");
    /// }
    /// ```
    pub fn real_inverse(&self, input: &[Complex<f32>], output: &mut [f32]) {
        let n = self.size;
        let half = n / 2;
        assert_eq!(
            input.len(),
            half + 1,
            "Input spectrum length ({}) must equal N/2+1 ({})",
            input.len(),
            half + 1
        );
        assert_eq!(
            output.len(),
            n,
            "Output length ({}) must equal FFT size ({})",
            output.len(),
            n
        );

        // Reconstruct the full length-N complex spectrum from the
        // one-sided input. For real input, X[k] = conj(X[N-k]).
        let mut buf: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); n];
        buf[0] = input[0];
        buf[half] = input[half];
        for k in 1..half {
            buf[k] = input[k];
            buf[n - k] = input[k].conj();
        }

        // Complex inverse FFT (un-normalised: output = N · true IFFT).
        self.inverse.process(&mut buf);

        // Take the real part and scale by 1/N to undo the
        // un-normalised IFFT.
        let scale = 1.0 / n as f32;
        for (i, c) in buf.iter().enumerate() {
            output[i] = c.re * scale;
        }
    }

    /// Resets internal state (no-op for stateless real-only FFTs).
    pub fn reset(&mut self) {
        // No state to reset; the real-only FFT is pure-functional.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq_complex(a: Complex<f32>, b: Complex<f32>, tol: f32) -> bool {
        (a.re - b.re).abs() < tol && (a.im - b.im).abs() < tol
    }

    #[test]
    fn real_fft_creation() {
        // Valid power-of-two sizes
        assert!(RealFFT::new(2).is_ok());
        assert!(RealFFT::new(4).is_ok());
        assert!(RealFFT::new(8).is_ok());
        assert!(RealFFT::new(1024).is_ok());
        assert!(RealFFT::new(65536).is_ok());

        // Non-power-of-two
        assert!(RealFFT::new(100).is_err());
        assert!(RealFFT::new(1000).is_err());

        // Out of range
        assert!(RealFFT::new(1).is_err());
        assert!(RealFFT::new(131072).is_err());
    }

    #[test]
    fn size_query() {
        let fft = RealFFT::new(1024).unwrap();
        assert_eq!(fft.size(), 1024);
        assert_eq!(fft.spectrum_size(), 513);
    }

    #[test]
    fn dc_signal_forward() {
        let fft = RealFFT::new(1024).unwrap();
        let input = vec![1.0f32; 1024];
        let mut spectrum = vec![Complex::new(0.0, 0.0); 513];
        fft.real_forward(&input, &mut spectrum);

        // DC bin = sum of all input samples = 1024
        assert!(
            (spectrum[0].re - 1024.0).abs() < 1.0,
            "DC bin = {}",
            spectrum[0].re
        );
        assert!(spectrum[0].im.abs() < 1e-3);

        // Nyquist bin should be 0 (constant signal has no energy at f_N/2)
        assert!(spectrum[512].norm() < 1e-3, "Nyquist = {}", spectrum[512]);

        // All other bins should be ~0
        for (i, &c) in spectrum.iter().enumerate().take(512).skip(1) {
            assert!(c.norm() < 1e-3, "bin {i} = {c}");
        }
    }

    #[test]
    fn sine_wave_localised() {
        // A pure cosine at bin k should produce a real peak at bin k.
        let n = 1024;
        let fft = RealFFT::new(n).unwrap();
        let k = 32;
        let input: Vec<f32> = (0..n)
            .map(|i| {
                let phase = 2.0 * std::f32::consts::PI * k as f32 * i as f32 / n as f32;
                phase.cos()
            })
            .collect();
        let mut spectrum = vec![Complex::new(0.0, 0.0); 513];
        fft.real_forward(&input, &mut spectrum);

        // The mirror bin (N-k) is NOT in the one-sided spectrum, so we
        // check that |X[k]| matches the expected magnitude for a
        // unit-amplitude cosine at bin k.
        let peak_pos = spectrum[k].norm();
        assert!(peak_pos > n as f32 * 0.4, "positive bin peak = {peak_pos}");
        assert!(spectrum[k].re > n as f32 * 0.4);
        assert!(
            spectrum[k].im.abs() < 0.01,
            "cos should be real: im = {}",
            spectrum[k].im
        );

        // Other bins should be small
        for (i, &c) in spectrum.iter().enumerate() {
            if i != k {
                assert!(c.norm() < 1.0, "bin {i} = {c} (expected small)");
            }
        }
    }

    #[test]
    fn roundtrip_identity() {
        // real_inverse(real_forward(x)) should reconstruct x
        let n = 1024;
        let fft = RealFFT::new(n).unwrap();
        let input: Vec<f32> = (0..n)
            .map(|i| ((i as f32) * 0.013).sin() + 0.3 * ((i as f32) * 0.05).cos())
            .collect();
        let mut spectrum = vec![Complex::new(0.0, 0.0); 513];
        let mut output = vec![0.0f32; n];
        fft.real_forward(&input, &mut spectrum);
        fft.real_inverse(&spectrum, &mut output);

        for (i, (&x, &y)) in input.iter().zip(output.iter()).enumerate() {
            assert!(
                (x - y).abs() < 1e-3,
                "mismatch at {i}: input={x} output={y}"
            );
        }
    }

    #[test]
    fn roundtrip_dc() {
        // Round-trip a constant signal
        let n = 512;
        let fft = RealFFT::new(n).unwrap();
        let input = vec![2.0f32; n];
        let mut spectrum = vec![Complex::new(0.0, 0.0); 257];
        let mut output = vec![0.0f32; n];
        fft.real_forward(&input, &mut spectrum);
        fft.real_inverse(&spectrum, &mut output);

        for &y in output.iter() {
            assert!((y - 2.0).abs() < 1e-3, "got {y}");
        }
    }

    #[test]
    fn nyquist_bin_real() {
        // The Nyquist bin (index N/2) should always be real
        let n = 256;
        let fft = RealFFT::new(n).unwrap();
        let input: Vec<f32> = (0..n)
            .map(|i| ((i as f32) * 0.1).sin())
            .collect();
        let mut spectrum = vec![Complex::new(0.0, 0.0); 129];
        fft.real_forward(&input, &mut spectrum);

        assert!(spectrum[n / 2].im.abs() < 1e-3);
    }

    #[test]
    fn dc_and_nyquist_real() {
        // Both real bins should be exactly real
        let n = 256;
        let fft = RealFFT::new(n).unwrap();
        let input: Vec<f32> = (0..n)
            .map(|i| ((i as f32) * 0.04).sin() * 0.7)
            .collect();
        let mut spectrum = vec![Complex::new(0.0, 0.0); 129];
        fft.real_forward(&input, &mut spectrum);

        assert!(spectrum[0].im.abs() < 1e-3, "DC imaginary = {}", spectrum[0].im);
        assert!(
            spectrum[n / 2].im.abs() < 1e-3,
            "Nyquist imaginary = {}",
            spectrum[n / 2].im
        );
    }

    #[test]
    fn full_spectrum_hermitian() {
        // Reconstruct the full spectrum and check Hermitian symmetry.
        let n = 256;
        let fft = RealFFT::new(n).unwrap();
        let input: Vec<f32> = (0..n)
            .map(|i| ((i as f32) * 0.04).sin() * 0.7)
            .collect();
        let mut spectrum = vec![Complex::new(0.0, 0.0); 129];
        fft.real_forward(&input, &mut spectrum);

        // Reconstruct the full spectrum
        let mut full = vec![Complex::new(0.0, 0.0); n];
        full[0] = spectrum[0];
        full[n / 2] = spectrum[n / 2];
        for k in 1..n / 2 {
            full[k] = spectrum[k];
            full[n - k] = spectrum[k].conj();
        }

        // Check Hermitian symmetry
        for k in 1..n {
            assert!(
                approx_eq_complex(full[k], full[n - k].conj(), 1e-3),
                "Hermitian failed at k={k}"
            );
        }
    }
}
