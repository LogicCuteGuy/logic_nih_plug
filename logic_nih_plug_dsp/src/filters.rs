//! Filter implementations.
//!
//! This module provides IIR (Infinite Impulse Response) filter implementations
//! ported from JUCE's DSP module.

use crate::error::DspError;

/// An IIR (Infinite Impulse Response) filter using Transposed Direct Form II structure.
///
/// This filter processes audio using configurable coefficients and maintains
/// internal state across process calls. The implementation is based on JUCE's
/// IIR filter design.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::filters::IIRFilter;
///
/// let mut filter = IIRFilter::new();
/// // Set coefficients for a simple first-order filter
/// filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
///
/// let input = vec![1.0, 0.5, 0.25, 0.0];
/// let mut output = vec![0.0; 4];
/// filter.process(&input, &mut output);
/// ```
///
/// # Performance
///
/// Processing is optimized for different filter orders (1st, 2nd, 3rd order
/// have specialized implementations). Higher order filters use a general loop.
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance.
#[derive(Clone)]
pub struct IIRFilter {
    /// Numerator coefficients (b0, b1, b2, ...)
    b_coeffs: Vec<f32>,
    /// Denominator coefficients (a1, a2, ...) - note: a0 is normalized to 1.0
    a_coeffs: Vec<f32>,
    /// Internal state for the filter (delay line)
    pub(crate) state: Vec<f32>,
    /// Filter order (number of delay elements)
    order: usize,
}

impl IIRFilter {
    /// Creates a new IIR filter with default pass-through coefficients.
    ///
    /// The filter is initialized with coefficients that produce no filtering
    /// (output = input). Call `set_coefficients` to configure the filter.
    pub fn new() -> Self {
        Self {
            b_coeffs: vec![1.0],
            a_coeffs: vec![],
            state: vec![],
            order: 0,
        }
    }

    /// Sets the filter coefficients.
    ///
    /// # Arguments
    ///
    /// * `b_coeffs` - Numerator coefficients [b0, b1, b2, ...]
    /// * `a_coeffs` - Denominator coefficients [a0, a1, a2, ...]
    ///
    /// The coefficients are automatically normalized by a0. The filter order
    /// is determined by the length of the coefficient arrays.
    ///
    /// # Errors
    ///
    /// Returns `DspError::InvalidCoefficients` if:
    /// - Coefficient arrays are empty
    /// - Coefficient arrays have different lengths
    /// - a0 (first denominator coefficient) is zero or very close to zero
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::filters::IIRFilter;
    ///
    /// let mut filter = IIRFilter::new();
    ///
    /// // First-order lowpass filter
    /// filter.set_coefficients(&[0.5, 0.5], &[1.0, -0.5]).unwrap();
    ///
    /// // Second-order filter
    /// filter.set_coefficients(
    ///     &[0.25, 0.5, 0.25],
    ///     &[1.0, -0.5, 0.25]
    /// ).unwrap();
    /// ```
    pub fn set_coefficients(&mut self, b_coeffs: &[f32], a_coeffs: &[f32]) -> Result<(), DspError> {
        // Validate inputs
        if b_coeffs.is_empty() || a_coeffs.is_empty() {
            return Err(DspError::InvalidCoefficients);
        }

        if b_coeffs.len() != a_coeffs.len() {
            return Err(DspError::InvalidCoefficients);
        }

        // Check that a0 is not zero
        let a0 = a_coeffs[0];
        if a0.abs() < 1e-10 {
            return Err(DspError::InvalidCoefficients);
        }

        // Normalize coefficients by a0
        let a0_inv = 1.0 / a0;
        
        self.b_coeffs = b_coeffs.iter().map(|&b| b * a0_inv).collect();
        // Skip a0 since it's normalized to 1.0
        self.a_coeffs = a_coeffs[1..].iter().map(|&a| a * a0_inv).collect();
        
        self.order = self.a_coeffs.len();
        self.state = vec![0.0; self.order];

        Ok(())
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
    /// use nih_plug_dsp::filters::IIRFilter;
    ///
    /// let mut filter = IIRFilter::new();
    /// filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
    ///
    /// let input = vec![1.0; 100];
    /// let mut output = vec![0.0; 100];
    /// filter.process(&input, &mut output);
    /// ```
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len(), "Input and output buffers must have the same length");

        match self.order {
            0 => {
                // Pass-through (no filtering)
                output.copy_from_slice(input);
            }
            1 => {
                // Optimized first-order filter
                self.process_first_order(input, output);
            }
            2 => {
                // Optimized second-order filter
                self.process_second_order(input, output);
            }
            3 => {
                // Optimized third-order filter
                self.process_third_order(input, output);
            }
            _ => {
                // General case for higher orders
                self.process_general(input, output);
            }
        }

        // Snap very small values to zero to prevent denormals
        self.snap_to_zero();
    }

    /// Processes a single sample through the filter.
    ///
    /// This is useful for sample-by-sample processing. For block processing,
    /// use `process()` instead for better performance.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::filters::IIRFilter;
    ///
    /// let mut filter = IIRFilter::new();
    /// filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
    ///
    /// let output = filter.process_sample(1.0);
    /// ```
    pub fn process_sample(&mut self, input: f32) -> f32 {
        if self.order == 0 {
            return input;
        }

        let output = (self.b_coeffs[0] * input) + self.state[0];

        for j in 0..self.order - 1 {
            self.state[j] = (self.b_coeffs[j + 1] * input) 
                          - (self.a_coeffs[j] * output) 
                          + self.state[j + 1];
        }

        if self.order > 0 {
            self.state[self.order - 1] = (self.b_coeffs[self.order] * input) 
                                        - (self.a_coeffs[self.order - 1] * output);
        }

        output
    }

    /// Resets the filter's internal state to zero.
    ///
    /// This clears the delay line but does not change the filter coefficients.
    /// Call this when starting to process a new audio stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::filters::IIRFilter;
    ///
    /// let mut filter = IIRFilter::new();
    /// filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
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
        self.state.fill(0.0);
    }

    /// Resets the filter's internal state to a specific value.
    ///
    /// This is useful for initializing the filter to a non-zero state.
    pub fn reset_to(&mut self, value: f32) {
        self.state.fill(value);
    }

    /// Returns the current filter order.
    pub fn order(&self) -> usize {
        self.order
    }

    /// Returns a reference to the internal state (for testing purposes).
    #[doc(hidden)]
    pub fn state(&self) -> &[f32] {
        &self.state
    }

    // Optimized processing for first-order filters
    #[inline]
    fn process_first_order(&mut self, input: &[f32], output: &mut [f32]) {
        let b0 = self.b_coeffs[0];
        let b1 = self.b_coeffs[1];
        let a1 = self.a_coeffs[0];

        let mut lv1 = self.state[0];

        for (inp, out) in input.iter().zip(output.iter_mut()) {
            let input_val = *inp;
            let output_val = input_val * b0 + lv1;
            *out = output_val;

            lv1 = (input_val * b1) - (output_val * a1);
        }

        self.state[0] = lv1;
    }

    // Optimized processing for second-order filters
    #[inline]
    fn process_second_order(&mut self, input: &[f32], output: &mut [f32]) {
        let b0 = self.b_coeffs[0];
        let b1 = self.b_coeffs[1];
        let b2 = self.b_coeffs[2];
        let a1 = self.a_coeffs[0];
        let a2 = self.a_coeffs[1];

        let mut lv1 = self.state[0];
        let mut lv2 = self.state[1];

        for (inp, out) in input.iter().zip(output.iter_mut()) {
            let input_val = *inp;
            let output_val = (input_val * b0) + lv1;
            *out = output_val;

            lv1 = (input_val * b1) - (output_val * a1) + lv2;
            lv2 = (input_val * b2) - (output_val * a2);
        }

        self.state[0] = lv1;
        self.state[1] = lv2;
    }

    // Optimized processing for third-order filters
    #[inline]
    fn process_third_order(&mut self, input: &[f32], output: &mut [f32]) {
        let b0 = self.b_coeffs[0];
        let b1 = self.b_coeffs[1];
        let b2 = self.b_coeffs[2];
        let b3 = self.b_coeffs[3];
        let a1 = self.a_coeffs[0];
        let a2 = self.a_coeffs[1];
        let a3 = self.a_coeffs[2];

        let mut lv1 = self.state[0];
        let mut lv2 = self.state[1];
        let mut lv3 = self.state[2];

        for (inp, out) in input.iter().zip(output.iter_mut()) {
            let input_val = *inp;
            let output_val = (input_val * b0) + lv1;
            *out = output_val;

            lv1 = (input_val * b1) - (output_val * a1) + lv2;
            lv2 = (input_val * b2) - (output_val * a2) + lv3;
            lv3 = (input_val * b3) - (output_val * a3);
        }

        self.state[0] = lv1;
        self.state[1] = lv2;
        self.state[2] = lv3;
    }

    // General processing for higher-order filters
    #[inline]
    fn process_general(&mut self, input: &[f32], output: &mut [f32]) {
        for (inp, out) in input.iter().zip(output.iter_mut()) {
            let input_val = *inp;
            let output_val = (input_val * self.b_coeffs[0]) + self.state[0];
            *out = output_val;

            for j in 0..self.order - 1 {
                self.state[j] = (input_val * self.b_coeffs[j + 1]) 
                              - (output_val * self.a_coeffs[j]) 
                              + self.state[j + 1];
            }

            self.state[self.order - 1] = (input_val * self.b_coeffs[self.order]) 
                                        - (output_val * self.a_coeffs[self.order - 1]);
        }
    }

    // Snap very small values to zero to prevent denormal issues
    #[inline]
    fn snap_to_zero(&mut self) {
        const THRESHOLD: f32 = 1e-15;
        for state_val in &mut self.state {
            if state_val.abs() < THRESHOLD {
                *state_val = 0.0;
            }
        }
    }
}

impl Default for IIRFilter {
    fn default() -> Self {
        Self::new()
    }
}

// IIRFilter is Send because it contains only owned data
unsafe impl Send for IIRFilter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_filter() {
        let filter = IIRFilter::new();
        assert_eq!(filter.order(), 0);
    }

    #[test]
    fn test_set_coefficients() {
        let mut filter = IIRFilter::new();
        let result = filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]);
        assert!(result.is_ok());
        assert_eq!(filter.order(), 1);
    }

    #[test]
    fn test_invalid_coefficients_empty() {
        let mut filter = IIRFilter::new();
        let result = filter.set_coefficients(&[], &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_coefficients_mismatched_length() {
        let mut filter = IIRFilter::new();
        let result = filter.set_coefficients(&[1.0, 0.5], &[1.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_coefficients_zero_a0() {
        let mut filter = IIRFilter::new();
        let result = filter.set_coefficients(&[1.0, 0.5], &[0.0, -0.5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_passthrough() {
        let mut filter = IIRFilter::new();
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let mut output = vec![0.0; 4];
        filter.process(&input, &mut output);
        assert_eq!(input, output);
    }

    #[test]
    fn test_reset() {
        let mut filter = IIRFilter::new();
        filter.set_coefficients(&[1.0, 0.5], &[1.0, -0.5]).unwrap();
        
        // Process some audio to populate state
        let input = vec![1.0; 10];
        let mut output = vec![0.0; 10];
        filter.process(&input, &mut output);
        
        // Reset should clear state
        filter.reset();
        
        // State should be all zeros
        assert!(filter.state().iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_process_sample() {
        let mut filter = IIRFilter::new();
        filter.set_coefficients(&[1.0, 0.0], &[1.0, 0.0]).unwrap();
        
        let output = filter.process_sample(1.0);
        assert_eq!(output, 1.0);
    }
}
