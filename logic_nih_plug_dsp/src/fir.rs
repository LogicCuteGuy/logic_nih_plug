//! FIR (Finite Impulse Response) filter implementations.
//!
//! This module provides FIR filter implementations with various window functions
//! for filter design. FIR filters have linear phase characteristics and are
//! always stable.

use crate::error::DspError;
use std::f32::consts::PI;

/// Window functions for FIR filter design.
///
/// Window functions are used to truncate the ideal infinite impulse response
/// and reduce spectral leakage. Different windows offer different trade-offs
/// between main lobe width and side lobe attenuation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFunction {
    /// Rectangular window (no windowing).
    /// Narrowest main lobe but highest side lobes.
    Rectangular,
    
    /// Triangular (Bartlett) window.
    /// Better side lobe attenuation than rectangular.
    Triangular,
    
    /// Hann window (raised cosine).
    /// Good general-purpose window with moderate side lobe attenuation.
    Hann,
    
    /// Hamming window.
    /// Similar to Hann but with slightly better side lobe attenuation.
    Hamming,
    
    /// Blackman window.
    /// Excellent side lobe attenuation with wider main lobe.
    Blackman,
    
    /// Blackman-Harris window.
    /// Very high side lobe attenuation.
    BlackmanHarris,
    
    /// Flat-top window.
    /// Optimized for accurate amplitude measurements.
    FlatTop,
    
    /// Kaiser window with adjustable beta parameter.
    /// Beta controls the trade-off between main lobe width and side lobe level.
    /// Typical values: 5.0 (moderate), 8.6 (good), 14.0 (excellent).
    Kaiser {
        /// Beta parameter controlling window shape
        beta: f32
    },
}

impl WindowFunction {
    /// Computes the window function value at a given position.
    ///
    /// # Arguments
    ///
    /// * `n` - Sample index (0 to length-1)
    /// * `length` - Total window length
    ///
    /// # Returns
    ///
    /// Window coefficient at position n
    pub fn compute(&self, n: usize, length: usize) -> f32 {
        let n = n as f32;
        let m = (length - 1) as f32;
        
        match self {
            WindowFunction::Rectangular => 1.0,
            
            WindowFunction::Triangular => {
                1.0 - ((2.0 * n - m) / m).abs()
            }
            
            WindowFunction::Hann => {
                0.5 * (1.0 - (2.0 * PI * n / m).cos())
            }
            
            WindowFunction::Hamming => {
                0.54 - 0.46 * (2.0 * PI * n / m).cos()
            }
            
            WindowFunction::Blackman => {
                let a0 = 0.42;
                let a1 = 0.5;
                let a2 = 0.08;
                a0 - a1 * (2.0 * PI * n / m).cos() + a2 * (4.0 * PI * n / m).cos()
            }
            
            WindowFunction::BlackmanHarris => {
                let a0 = 0.35875;
                let a1 = 0.48829;
                let a2 = 0.14128;
                let a3 = 0.01168;
                a0 - a1 * (2.0 * PI * n / m).cos() 
                   + a2 * (4.0 * PI * n / m).cos()
                   - a3 * (6.0 * PI * n / m).cos()
            }
            
            WindowFunction::FlatTop => {
                let a0 = 0.21557895;
                let a1 = 0.41663158;
                let a2 = 0.277263158;
                let a3 = 0.083578947;
                let a4 = 0.006947368;
                a0 - a1 * (2.0 * PI * n / m).cos()
                   + a2 * (4.0 * PI * n / m).cos()
                   - a3 * (6.0 * PI * n / m).cos()
                   + a4 * (8.0 * PI * n / m).cos()
            }
            
            WindowFunction::Kaiser { beta } => {
                let alpha = m / 2.0;
                let arg = (n - alpha) / alpha;
                let bessel_arg = beta * (1.0 - arg * arg).sqrt();
                bessel_i0(bessel_arg) / bessel_i0(*beta)
            }
        }
    }
}

/// FIR (Finite Impulse Response) filter.
///
/// FIR filters process audio using a finite set of coefficients and a delay line.
/// They have linear phase characteristics and are always stable.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::fir::{FIRFilter, WindowFunction};
///
/// // Create a filter with specific coefficients
/// let coefficients = vec![0.1, 0.2, 0.4, 0.2, 0.1];
/// let mut filter = FIRFilter::new(coefficients);
///
/// let input = vec![1.0; 100];
/// let mut output = vec![0.0; 100];
/// filter.process(&input, &mut output);
/// ```
#[derive(Clone)]
pub struct FIRFilter {
    /// Filter coefficients (impulse response)
    coefficients: Vec<f32>,
    /// Circular delay line for input samples
    delay_line: Vec<f32>,
    /// Current write position in the delay line
    write_pos: usize,
}

impl FIRFilter {
    /// Creates a new FIR filter with the given coefficients.
    ///
    /// # Arguments
    ///
    /// * `coefficients` - Filter impulse response coefficients
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::FIRFilter;
    ///
    /// let coefficients = vec![0.2, 0.6, 0.2];
    /// let filter = FIRFilter::new(coefficients);
    /// ```
    pub fn new(coefficients: Vec<f32>) -> Self {
        let length = coefficients.len();
        Self {
            coefficients,
            delay_line: vec![0.0; length],
            write_pos: 0,
        }
    }

    /// Returns the filter length (number of taps).
    pub fn length(&self) -> usize {
        self.coefficients.len()
    }

    /// Returns a reference to the filter coefficients.
    pub fn coefficients(&self) -> &[f32] {
        &self.coefficients
    }

    /// Processes a single sample through the filter.
    ///
    /// # Arguments
    ///
    /// * `input` - Input sample
    ///
    /// # Returns
    ///
    /// Filtered output sample
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::FIRFilter;
    ///
    /// let coefficients = vec![0.5, 0.5];
    /// let mut filter = FIRFilter::new(coefficients);
    ///
    /// let output = filter.process_sample(1.0);
    /// ```
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Write input to delay line
        self.delay_line[self.write_pos] = input;
        
        // Compute convolution
        let mut output = 0.0;
        let length = self.coefficients.len();
        
        for i in 0..length {
            // Read from delay line with wrap-around
            let delay_idx = (self.write_pos + length - i) % length;
            output += self.coefficients[i] * self.delay_line[delay_idx];
        }
        
        // Advance write position
        self.write_pos = (self.write_pos + 1) % length;
        
        output
    }

    /// Processes a buffer of samples through the filter.
    ///
    /// # Arguments
    ///
    /// * `input` - Input samples
    /// * `output` - Output buffer (must be same length as input)
    ///
    /// # Panics
    ///
    /// Panics if input and output buffers have different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::FIRFilter;
    ///
    /// let coefficients = vec![0.5, 0.5];
    /// let mut filter = FIRFilter::new(coefficients);
    ///
    /// let input = vec![1.0; 100];
    /// let mut output = vec![0.0; 100];
    /// filter.process(&input, &mut output);
    /// ```
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            output.len(),
            "Input and output buffers must have the same length"
        );

        for (inp, out) in input.iter().zip(output.iter_mut()) {
            *out = self.process_sample(*inp);
        }
    }

    /// Resets the filter's internal state (delay line) to zero.
    ///
    /// This clears the delay line but does not change the filter coefficients.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::FIRFilter;
    ///
    /// let coefficients = vec![0.5, 0.5];
    /// let mut filter = FIRFilter::new(coefficients);
    ///
    /// // Process some audio...
    /// let input = vec![1.0; 100];
    /// let mut output = vec![0.0; 100];
    /// filter.process(&input, &mut output);
    ///
    /// // Reset for a new stream
    /// filter.reset();
    /// ```
    pub fn reset(&mut self) {
        self.delay_line.fill(0.0);
        self.write_pos = 0;
    }
}

/// Filter design utilities for creating FIR filters.
pub struct FilterDesign;

impl FilterDesign {
    /// Designs a FIR lowpass filter using the windowing method.
    ///
    /// # Arguments
    ///
    /// * `cutoff_hz` - Cutoff frequency in Hz
    /// * `sample_rate` - Sample rate in Hz
    /// * `length` - Filter length (number of taps, should be odd)
    /// * `window` - Window function to apply
    ///
    /// # Returns
    ///
    /// Filter coefficients, or an error if parameters are invalid
    ///
    /// # Errors
    ///
    /// Returns `DspError::InvalidFrequency` if cutoff >= Nyquist frequency
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::{FilterDesign, WindowFunction};
    ///
    /// let coeffs = FilterDesign::fir_lowpass(
    ///     1000.0,
    ///     44100.0,
    ///     51,
    ///     WindowFunction::Hamming
    /// ).unwrap();
    /// ```
    pub fn fir_lowpass(
        cutoff_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Result<Vec<f32>, DspError> {
        let nyquist = sample_rate / 2.0;
        if cutoff_hz >= nyquist {
            return Err(DspError::InvalidFrequency(cutoff_hz));
        }

        let normalized_cutoff = cutoff_hz / sample_rate;
        let mut coefficients = vec![0.0; length];
        let center = (length - 1) as f32 / 2.0;

        for i in 0..length {
            let n = i as f32 - center;
            
            // Ideal sinc function
            let h = if n.abs() < 1e-10 {
                2.0 * normalized_cutoff
            } else {
                (2.0 * PI * normalized_cutoff * n).sin() / (PI * n)
            };
            
            // Apply window
            let w = window.compute(i, length);
            coefficients[i] = h * w;
        }

        // Normalize for unity gain at DC
        let sum: f32 = coefficients.iter().sum();
        if sum.abs() > 1e-10 {
            for coeff in &mut coefficients {
                *coeff /= sum;
            }
        }

        Ok(coefficients)
    }

    /// Designs a FIR highpass filter using spectral inversion.
    ///
    /// # Arguments
    ///
    /// * `cutoff_hz` - Cutoff frequency in Hz
    /// * `sample_rate` - Sample rate in Hz
    /// * `length` - Filter length (number of taps, should be odd)
    /// * `window` - Window function to apply
    ///
    /// # Returns
    ///
    /// Filter coefficients, or an error if parameters are invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::{FilterDesign, WindowFunction};
    ///
    /// let coeffs = FilterDesign::fir_highpass(
    ///     1000.0,
    ///     44100.0,
    ///     51,
    ///     WindowFunction::Hamming
    /// ).unwrap();
    /// ```
    pub fn fir_highpass(
        cutoff_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Result<Vec<f32>, DspError> {
        // Design lowpass filter
        let mut coefficients = Self::fir_lowpass(cutoff_hz, sample_rate, length, window)?;
        
        // Spectral inversion: negate all coefficients and add 1 to center
        for coeff in &mut coefficients {
            *coeff = -*coeff;
        }
        let center = length / 2;
        coefficients[center] += 1.0;
        
        Ok(coefficients)
    }

    /// Designs a FIR bandpass filter.
    ///
    /// # Arguments
    ///
    /// * `low_hz` - Lower cutoff frequency in Hz
    /// * `high_hz` - Upper cutoff frequency in Hz
    /// * `sample_rate` - Sample rate in Hz
    /// * `length` - Filter length (number of taps, should be odd)
    /// * `window` - Window function to apply
    ///
    /// # Returns
    ///
    /// Filter coefficients, or an error if parameters are invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::{FilterDesign, WindowFunction};
    ///
    /// let coeffs = FilterDesign::fir_bandpass(
    ///     500.0,
    ///     2000.0,
    ///     44100.0,
    ///     51,
    ///     WindowFunction::Hamming
    /// ).unwrap();
    /// ```
    pub fn fir_bandpass(
        low_hz: f32,
        high_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Result<Vec<f32>, DspError> {
        if low_hz >= high_hz {
            return Err(DspError::InvalidFrequency(low_hz));
        }
        
        // Design two lowpass filters
        let lp_high = Self::fir_lowpass(high_hz, sample_rate, length, window)?;
        let lp_low = Self::fir_lowpass(low_hz, sample_rate, length, window)?;
        
        // Subtract to get bandpass
        let mut coefficients = vec![0.0; length];
        for i in 0..length {
            coefficients[i] = lp_high[i] - lp_low[i];
        }
        
        Ok(coefficients)
    }

    /// Designs a FIR bandstop (notch) filter.
    ///
    /// # Arguments
    ///
    /// * `low_hz` - Lower cutoff frequency in Hz
    /// * `high_hz` - Upper cutoff frequency in Hz
    /// * `sample_rate` - Sample rate in Hz
    /// * `length` - Filter length (number of taps, should be odd)
    /// * `window` - Window function to apply
    ///
    /// # Returns
    ///
    /// Filter coefficients, or an error if parameters are invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::fir::{FilterDesign, WindowFunction};
    ///
    /// let coeffs = FilterDesign::fir_bandstop(
    ///     500.0,
    ///     2000.0,
    ///     44100.0,
    ///     51,
    ///     WindowFunction::Hamming
    /// ).unwrap();
    /// ```
    pub fn fir_bandstop(
        low_hz: f32,
        high_hz: f32,
        sample_rate: f32,
        length: usize,
        window: WindowFunction,
    ) -> Result<Vec<f32>, DspError> {
        // Design bandpass filter
        let mut coefficients = Self::fir_bandpass(low_hz, high_hz, sample_rate, length, window)?;
        
        // Spectral inversion: negate all coefficients and add 1 to center
        for coeff in &mut coefficients {
            *coeff = -*coeff;
        }
        let center = length / 2;
        coefficients[center] += 1.0;
        
        Ok(coefficients)
    }
}

/// Computes the modified Bessel function of the first kind, order 0.
///
/// This is used for Kaiser window calculation.
fn bessel_i0(x: f32) -> f32 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let mut k = 1.0;
    
    // Series expansion
    while term > 1e-10 * sum {
        let half_x = x / 2.0;
        term *= (half_x / k) * (half_x / k);
        sum += term;
        k += 1.0;
        
        if k > 100.0 {
            break; // Prevent infinite loop
        }
    }
    
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fir_filter_creation() {
        let coeffs = vec![0.2, 0.6, 0.2];
        let filter = FIRFilter::new(coeffs.clone());
        assert_eq!(filter.length(), 3);
        assert_eq!(filter.coefficients(), &coeffs[..]);
    }

    #[test]
    fn test_fir_filter_reset() {
        let coeffs = vec![0.5, 0.5];
        let mut filter = FIRFilter::new(coeffs);
        
        // Process some samples
        filter.process_sample(1.0);
        filter.process_sample(1.0);
        
        // Reset
        filter.reset();
        
        // Delay line should be cleared
        assert!(filter.delay_line.iter().all(|&x| x == 0.0));
        assert_eq!(filter.write_pos, 0);
    }

    #[test]
    fn test_window_functions() {
        let length = 10;
        
        // Test that different windows produce different values
        let hann = WindowFunction::Hann.compute(5, length);
        let hamming = WindowFunction::Hamming.compute(5, length);
        let blackman = WindowFunction::Blackman.compute(5, length);
        
        assert_ne!(hann, hamming);
        assert_ne!(hann, blackman);
        assert_ne!(hamming, blackman);
    }

    #[test]
    fn test_lowpass_design() {
        let result = FilterDesign::fir_lowpass(
            1000.0,
            44100.0,
            51,
            WindowFunction::Hamming,
        );
        assert!(result.is_ok());
        let coeffs = result.unwrap();
        assert_eq!(coeffs.len(), 51);
    }

    #[test]
    fn test_lowpass_nyquist_validation() {
        let result = FilterDesign::fir_lowpass(
            25000.0, // Above Nyquist
            44100.0,
            51,
            WindowFunction::Hamming,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_highpass_design() {
        let result = FilterDesign::fir_highpass(
            1000.0,
            44100.0,
            51,
            WindowFunction::Hamming,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_bandpass_design() {
        let result = FilterDesign::fir_bandpass(
            500.0,
            2000.0,
            44100.0,
            51,
            WindowFunction::Hamming,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_bandstop_design() {
        let result = FilterDesign::fir_bandstop(
            500.0,
            2000.0,
            44100.0,
            51,
            WindowFunction::Hamming,
        );
        assert!(result.is_ok());
    }
}
