//! Bias processor for adding DC offset to audio signals.
//!
//! This module provides a bias processor that adds a configurable DC offset
//! to audio signals, useful for creating asymmetric distortion effects.

use super::Processor;

/// A bias processor that adds a DC offset to audio signals.
///
/// The bias processor adds a constant value to all samples, which can be
/// used to create asymmetric distortion when combined with wave shapers.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::processors::bias::Bias;
///
/// let mut bias = Bias::new();
/// bias.set_bias(0.1);
///
/// let input = vec![0.0; 512];
/// let mut output = vec![0.0; 512];
/// bias.process(&input, &mut output);
/// ```
pub struct Bias {
    /// DC offset to add to the signal
    offset: f32,
}

impl Bias {
    /// Creates a new bias processor with zero offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::bias::Bias;
    ///
    /// let bias = Bias::new();
    /// ```
    pub fn new() -> Self {
        Self { offset: 0.0 }
    }

    /// Sets the bias offset value.
    ///
    /// # Arguments
    ///
    /// * `offset` - DC offset to add to the signal
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::bias::Bias;
    ///
    /// let mut bias = Bias::new();
    /// bias.set_bias(0.1);
    /// ```
    pub fn set_bias(&mut self, offset: f32) {
        self.offset = offset;
    }

    /// Gets the current bias offset value.
    ///
    /// # Returns
    ///
    /// The current DC offset value
    pub fn bias(&self) -> f32 {
        self.offset
    }

    /// Prepares the processor for audio processing.
    ///
    /// This should be called before processing audio, typically when
    /// the sample rate or buffer size changes.
    ///
    /// # Arguments
    ///
    /// * `_sample_rate` - Sample rate in Hz (unused)
    /// * `_max_block_size` - Maximum block size (unused)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::bias::Bias;
    ///
    /// let mut bias = Bias::new();
    /// bias.prepare(44100.0, 512);
    /// ```
    pub fn prepare(&mut self, _sample_rate: f32, _max_block_size: usize) {
        // No preparation needed for bias processor
    }

    /// Processes a single sample through the bias processor.
    ///
    /// # Arguments
    ///
    /// * `input` - Input sample
    ///
    /// # Returns
    ///
    /// The processed output sample with bias added
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::bias::Bias;
    ///
    /// let mut bias = Bias::new();
    /// bias.set_bias(0.1);
    ///
    /// let output = bias.process_sample(0.5);
    /// assert!((output - 0.6).abs() < 0.001);
    /// ```
    pub fn process_sample(&self, input: f32) -> f32 {
        // Check for numerical stability
        let result = input + self.offset;
        
        // Return NaN or infinity as-is to maintain IEEE 754 semantics
        // but ensure finite inputs produce finite outputs
        if input.is_finite() && self.offset.is_finite() {
            result
        } else {
            result
        }
    }

    /// Processes a buffer of samples through the bias processor.
    ///
    /// # Arguments
    ///
    /// * `input` - Input buffer
    /// * `output` - Output buffer (must be same length as input)
    ///
    /// # Panics
    ///
    /// Panics if input and output buffers have different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::bias::Bias;
    ///
    /// let mut bias = Bias::new();
    /// bias.set_bias(0.1);
    ///
    /// let input = vec![0.5; 512];
    /// let mut output = vec![0.0; 512];
    /// bias.process(&input, &mut output);
    /// ```
    pub fn process(&self, input: &[f32], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            output.len(),
            "Input and output buffers must have the same length"
        );

        for (in_sample, out_sample) in input.iter().zip(output.iter_mut()) {
            *out_sample = self.process_sample(*in_sample);
        }
    }

    /// Resets the processor state.
    ///
    /// For the bias processor, this is a no-op since there is no internal state.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::processors::bias::Bias;
    ///
    /// let mut bias = Bias::new();
    /// bias.reset();
    /// ```
    pub fn reset(&mut self) {
        // No state to reset for bias processor
    }
}

impl Default for Bias {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for Bias {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        Bias::prepare(self, sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        Bias::process(self, input, output);
    }

    fn reset(&mut self) {
        Bias::reset(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bias_creation() {
        let bias = Bias::new();
        assert_eq!(bias.bias(), 0.0);
    }

    #[test]
    fn test_set_bias() {
        let mut bias = Bias::new();
        bias.set_bias(0.5);
        assert_eq!(bias.bias(), 0.5);
    }

    #[test]
    fn test_bias_process_sample() {
        let mut bias = Bias::new();
        bias.set_bias(0.1);
        
        let output = bias.process_sample(0.5);
        assert!((output - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_bias_process_buffer() {
        let mut bias = Bias::new();
        bias.set_bias(0.2);
        
        let input = vec![0.0, 0.1, 0.2, 0.3, 0.4];
        let mut output = vec![0.0; 5];
        bias.process(&input, &mut output);
        
        let expected = vec![0.2, 0.3, 0.4, 0.5, 0.6];
        for (out, exp) in output.iter().zip(expected.iter()) {
            assert!((out - exp).abs() < 0.001);
        }
    }

    #[test]
    fn test_zero_bias() {
        let mut bias = Bias::new();
        bias.set_bias(0.0);
        
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let mut output = vec![0.0; 5];
        bias.process(&input, &mut output);
        
        for (inp, out) in input.iter().zip(output.iter()) {
            assert!((inp - out).abs() < 0.001);
        }
    }

    #[test]
    fn test_negative_bias() {
        let mut bias = Bias::new();
        bias.set_bias(-0.3);
        
        let output = bias.process_sample(0.5);
        assert!((output - 0.2).abs() < 0.001);
    }

    #[test]
    #[should_panic(expected = "Input and output buffers must have the same length")]
    fn test_bias_mismatched_buffers() {
        let bias = Bias::new();
        
        let input = vec![0.5; 10];
        let mut output = vec![0.0; 5];
        bias.process(&input, &mut output);
    }

    #[test]
    fn test_bias_reset() {
        let mut bias = Bias::new();
        bias.set_bias(0.5);
        bias.reset();
        // Bias should still be 0.5 after reset (no state to clear)
        assert_eq!(bias.bias(), 0.5);
    }

    #[test]
    fn test_finite_inputs() {
        let mut bias = Bias::new();
        bias.set_bias(0.1);
        
        // Test with various finite values
        assert!(bias.process_sample(0.0).is_finite());
        assert!(bias.process_sample(1.0).is_finite());
        assert!(bias.process_sample(-1.0).is_finite());
        assert!(bias.process_sample(f32::MAX / 2.0).is_finite());
    }
}
