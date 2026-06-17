//! Fast Fourier Transform (FFT) for frequency-domain analysis.
//!
//! This module provides FFT functionality for converting between time-domain
//! and frequency-domain representations of audio signals.
//!
//! # Examples
//!
//! ```
//! use nih_plug_dsp::analysis::FFT;
//!
//! // Create an FFT processor for 1024-point transforms
//! let fft = FFT::new(1024).unwrap();
//!
//! // Prepare input signal
//! let input: Vec<f32> = vec![0.0; 1024];
//! let mut output = vec![num_complex::Complex::new(0.0, 0.0); 1024];
//!
//! // Perform forward FFT
//! fft.forward(&input, &mut output);
//!
//! // Get magnitude spectrum
//! let mut magnitudes = vec![0.0; 1024];
//! fft.forward_magnitude(&input, &mut magnitudes);
//! ```

use crate::error::DspError;
use num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::sync::Arc;

/// Fast Fourier Transform processor for frequency-domain analysis.
///
/// The FFT converts time-domain signals to frequency-domain representations
/// and vice versa. This implementation uses the `rustfft` crate for efficient
/// FFT computation.
///
/// # Size Requirements
///
/// The FFT size must be a power of 2 and within the range [2, 65536].
/// Common sizes include 512, 1024, 2048, 4096, and 8192.
pub struct FFT {
    size: usize,
    forward_fft: Arc<dyn Fft<f32>>,
    inverse_fft: Arc<dyn Fft<f32>>,
}

impl FFT {
    /// Minimum supported FFT size.
    pub const MIN_SIZE: usize = 2;

    /// Maximum supported FFT size.
    pub const MAX_SIZE: usize = 65536;

    /// Creates a new FFT processor with the specified size.
    ///
    /// # Arguments
    ///
    /// * `size` - The FFT size (must be a power of 2 between 2 and 65536)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The size is not a power of 2
    /// - The size is outside the valid range [2, 65536]
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::analysis::FFT;
    ///
    /// let fft = FFT::new(1024).unwrap();
    /// assert_eq!(fft.size(), 1024);
    /// ```
    pub fn new(size: usize) -> Result<Self, DspError> {
        // Validate size is within range
        if size < Self::MIN_SIZE || size > Self::MAX_SIZE {
            return Err(DspError::FFTSizeOutOfRange {
                size,
                min: Self::MIN_SIZE,
                max: Self::MAX_SIZE,
            });
        }

        // Validate size is power of 2
        if !size.is_power_of_two() {
            return Err(DspError::InvalidFFTSize { size });
        }

        let mut planner = FftPlanner::new();
        let forward_fft = planner.plan_fft_forward(size);
        let inverse_fft = planner.plan_fft_inverse(size);

        Ok(Self {
            size,
            forward_fft,
            inverse_fft,
        })
    }

    /// Returns the FFT size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Performs a forward FFT, converting time-domain to frequency-domain.
    ///
    /// # Arguments
    ///
    /// * `input` - Time-domain input samples (must have length equal to FFT size)
    /// * `output` - Frequency-domain output buffer (must have length equal to FFT size)
    ///
    /// # Panics
    ///
    /// Panics if input or output buffer sizes don't match the FFT size.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::analysis::FFT;
    /// use num_complex::Complex;
    ///
    /// let fft = FFT::new(1024).unwrap();
    /// let input = vec![0.0; 1024];
    /// let mut output = vec![Complex::new(0.0, 0.0); 1024];
    ///
    /// fft.forward(&input, &mut output);
    /// ```
    pub fn forward(&self, input: &[f32], output: &mut [Complex<f32>]) {
        assert_eq!(
            input.len(),
            self.size,
            "Input buffer size must match FFT size"
        );
        assert_eq!(
            output.len(),
            self.size,
            "Output buffer size must match FFT size"
        );

        // Convert real input to complex
        for (i, &sample) in input.iter().enumerate() {
            output[i] = Complex::new(sample, 0.0);
        }

        // Perform FFT
        self.forward_fft.process(output);
    }

    /// Performs an inverse FFT, converting frequency-domain to time-domain.
    ///
    /// The output is normalized by dividing by the FFT size to maintain
    /// proper amplitude scaling.
    ///
    /// # Arguments
    ///
    /// * `input` - Frequency-domain input (must have length equal to FFT size)
    /// * `output` - Time-domain output buffer (must have length equal to FFT size)
    ///
    /// # Panics
    ///
    /// Panics if input or output buffer sizes don't match the FFT size.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::analysis::FFT;
    /// use num_complex::Complex;
    ///
    /// let fft = FFT::new(1024).unwrap();
    /// let mut freq_data = vec![Complex::new(0.0, 0.0); 1024];
    /// let mut output = vec![0.0; 1024];
    ///
    /// fft.inverse(&freq_data, &mut output);
    /// ```
    pub fn inverse(&self, input: &[Complex<f32>], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            self.size,
            "Input buffer size must match FFT size"
        );
        assert_eq!(
            output.len(),
            self.size,
            "Output buffer size must match FFT size"
        );

        // Create a working buffer for the inverse FFT
        let mut buffer: Vec<Complex<f32>> = input.to_vec();

        // Perform inverse FFT
        self.inverse_fft.process(&mut buffer);

        // Extract real part and normalize
        let scale = 1.0 / self.size as f32;
        for (i, &complex_sample) in buffer.iter().enumerate() {
            output[i] = complex_sample.re * scale;
        }
    }

    /// Performs a forward FFT and returns only the magnitude spectrum.
    ///
    /// This is useful for spectrum analysis where phase information is not needed.
    /// The output contains the magnitude (absolute value) of each frequency bin.
    ///
    /// # Arguments
    ///
    /// * `input` - Time-domain input samples (must have length equal to FFT size)
    /// * `output` - Magnitude spectrum output (must have length equal to FFT size)
    ///
    /// # Panics
    ///
    /// Panics if input or output buffer sizes don't match the FFT size.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::analysis::FFT;
    ///
    /// let fft = FFT::new(1024).unwrap();
    /// let input = vec![0.0; 1024];
    /// let mut magnitudes = vec![0.0; 1024];
    ///
    /// fft.forward_magnitude(&input, &mut magnitudes);
    /// ```
    pub fn forward_magnitude(&self, input: &[f32], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            self.size,
            "Input buffer size must match FFT size"
        );
        assert_eq!(
            output.len(),
            self.size,
            "Output buffer size must match FFT size"
        );

        // Create a working buffer for the FFT
        let mut buffer: Vec<Complex<f32>> = input.iter().map(|&x| Complex::new(x, 0.0)).collect();

        // Perform FFT
        self.forward_fft.process(&mut buffer);

        // Calculate magnitudes
        for (i, &complex_sample) in buffer.iter().enumerate() {
            output[i] = complex_sample.norm();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_creation() {
        // Valid power-of-2 sizes should succeed
        assert!(FFT::new(2).is_ok());
        assert!(FFT::new(1024).is_ok());
        assert!(FFT::new(65536).is_ok());

        // Non-power-of-2 sizes should fail
        assert!(FFT::new(100).is_err());
        assert!(FFT::new(1000).is_err());

        // Out of range sizes should fail
        assert!(FFT::new(1).is_err());
        assert!(FFT::new(131072).is_err());
    }

    #[test]
    fn test_fft_size() {
        let fft = FFT::new(1024).unwrap();
        assert_eq!(fft.size(), 1024);
    }

    #[test]
    fn test_dc_signal() {
        let fft = FFT::new(1024).unwrap();
        let input = vec![1.0; 1024];
        let mut output = vec![Complex::new(0.0, 0.0); 1024];

        fft.forward(&input, &mut output);

        // DC component should be at bin 0
        assert!(output[0].norm() > 1000.0);
        // Other bins should be near zero
        for i in 1..10 {
            assert!(output[i].norm() < 0.01);
        }
    }

    #[test]
    fn test_magnitude_spectrum_non_negative() {
        let fft = FFT::new(1024).unwrap();
        let input = vec![1.0; 1024];
        let mut magnitudes = vec![0.0; 1024];

        fft.forward_magnitude(&input, &mut magnitudes);

        // All magnitudes should be non-negative
        for &mag in &magnitudes {
            assert!(mag >= 0.0);
        }
    }
}
