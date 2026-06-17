//! Wave shaping processors for non-linear audio processing.
//!
//! This module provides wave shaping functionality that applies transfer functions
//! to audio signals for distortion and saturation effects.

use super::Processor;

/// A wave shaper that applies a transfer function to audio samples.
///
/// The wave shaper processes audio by applying a custom transfer function
/// sample-by-sample. This is useful for creating distortion, saturation,
/// and other non-linear effects.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::processors::waveshaper::{WaveShaper, transfer_functions};
///
/// // Create a wave shaper with tanh saturation
/// let shaper = WaveShaper::new(transfer_functions::tanh);
///
/// // Process a sample
/// let input = 0.5;
/// let output = shaper.process_sample(input);
/// ```
pub struct WaveShaper<F>
where
    F: Fn(f32) -> f32,
{
    transfer_function: F,
}

impl<F> WaveShaper<F>
where
    F: Fn(f32) -> f32,
{
    /// Creates a new wave shaper with the given transfer function.
    ///
    /// # Arguments
    ///
    /// * `transfer_function` - A function that maps input samples to output samples
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::waveshaper::WaveShaper;
    ///
    /// let shaper = WaveShaper::new(|x| x.tanh());
    /// ```
    pub fn new(transfer_function: F) -> Self {
        Self { transfer_function }
    }

    /// Processes a single sample through the wave shaper.
    ///
    /// # Arguments
    ///
    /// * `input` - The input sample
    ///
    /// # Returns
    ///
    /// The processed output sample
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::waveshaper::WaveShaper;
    ///
    /// let shaper = WaveShaper::new(|x| x.tanh());
    /// let output = shaper.process_sample(0.5);
    /// ```
    pub fn process_sample(&self, input: f32) -> f32 {
        // Handle edge cases
        if !input.is_finite() {
            return 0.0;
        }

        let output = (self.transfer_function)(input);

        // Ensure output is finite
        if !output.is_finite() {
            0.0
        } else {
            output
        }
    }

    /// Processes a buffer of samples through the wave shaper.
    ///
    /// # Arguments
    ///
    /// * `input` - The input buffer
    /// * `output` - The output buffer (must be same length as input)
    ///
    /// # Panics
    ///
    /// Panics if input and output buffers have different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::waveshaper::WaveShaper;
    ///
    /// let shaper = WaveShaper::new(|x| x.tanh());
    /// let input = vec![0.0, 0.5, 1.0];
    /// let mut output = vec![0.0; 3];
    /// shaper.process(&input, &mut output);
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
}

impl<F> Processor for WaveShaper<F>
where
    F: Fn(f32) -> f32 + Send,
{
    fn prepare(&mut self, _sample_rate: f32, _max_block_size: usize) {
        // No preparation needed for wave shaper
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        WaveShaper::process(self, input, output);
    }

    fn reset(&mut self) {
        // No state to reset for wave shaper
    }
}

/// Predefined transfer functions for common wave shaping effects.
pub mod transfer_functions {
    /// Hyperbolic tangent saturation.
    ///
    /// Provides smooth, symmetric saturation. This is a commonly used
    /// transfer function for analog-style saturation.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::waveshaper::transfer_functions::tanh;
    ///
    /// let output = tanh(0.5);
    /// ```
    pub fn tanh(x: f32) -> f32 {
        x.tanh()
    }

    /// Fast approximation of hyperbolic tangent.
    ///
    /// Uses a rational function approximation that is faster than
    /// the standard tanh but less accurate. Good for real-time
    /// processing where performance is critical.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::waveshaper::transfer_functions::tanh_approx;
    ///
    /// let output = tanh_approx(0.5);
    /// ```
    pub fn tanh_approx(x: f32) -> f32 {
        // Clamp input to reasonable range
        let x = x.clamp(-3.0, 3.0);

        // Rational approximation: x / (1 + |x|)
        // This is a simple and fast approximation
        x / (1.0 + x.abs())
    }

    /// Hard clipping at ±1.0.
    ///
    /// Clips the signal to the range [-1.0, 1.0]. This creates
    /// harsh distortion with sharp transitions.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::waveshaper::transfer_functions::hard_clip;
    ///
    /// let output = hard_clip(1.5); // Returns 1.0
    /// ```
    pub fn hard_clip(x: f32) -> f32 {
        x.clamp(-1.0, 1.0)
    }

    /// Soft clipping with smooth transitions.
    ///
    /// Provides a smoother transition into clipping than hard_clip.
    /// Uses a cubic polynomial for the transition region.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::waveshaper::transfer_functions::soft_clip;
    ///
    /// let output = soft_clip(0.5);
    /// ```
    pub fn soft_clip(x: f32) -> f32 {
        if x > 1.0 {
            1.0
        } else if x < -1.0 {
            -1.0
        } else if x > 0.5 {
            // Smooth transition region using cubic
            let t = (x - 0.5) * 2.0; // Map [0.5, 1.0] to [0, 1]
            0.5 + 0.5 * (1.0 - (1.0 - t).powi(3))
        } else if x < -0.5 {
            // Smooth transition region using cubic
            let t = (x + 0.5) * 2.0; // Map [-1.0, -0.5] to [-1, 0]
            -0.5 - 0.5 * (1.0 - (1.0 + t).powi(3))
        } else {
            // Linear region
            x
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveshaper_creation() {
        let shaper = WaveShaper::new(|x| x * 2.0);
        assert_eq!(shaper.process_sample(0.5), 1.0);
    }

    #[test]
    fn test_waveshaper_handles_nan() {
        let shaper = WaveShaper::new(|x| x.tanh());
        let output = shaper.process_sample(f32::NAN);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_waveshaper_handles_infinity() {
        let shaper = WaveShaper::new(|x| x.tanh());
        let output = shaper.process_sample(f32::INFINITY);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_waveshaper_handles_neg_infinity() {
        let shaper = WaveShaper::new(|x| x.tanh());
        let output = shaper.process_sample(f32::NEG_INFINITY);
        assert_eq!(output, 0.0);
    }

    #[test]
    fn test_waveshaper_process_buffer() {
        let shaper = WaveShaper::new(|x| x * 2.0);
        let input = vec![0.0, 0.5, 1.0];
        let mut output = vec![0.0; 3];
        shaper.process(&input, &mut output);
        assert_eq!(output, vec![0.0, 1.0, 2.0]);
    }

    #[test]
    #[should_panic(expected = "Input and output buffers must have the same length")]
    fn test_waveshaper_mismatched_buffers() {
        let shaper = WaveShaper::new(|x| x);
        let input = vec![0.0, 0.5];
        let mut output = vec![0.0; 3];
        shaper.process(&input, &mut output);
    }

    #[test]
    fn test_tanh_transfer_function() {
        let output = transfer_functions::tanh(0.0);
        assert_eq!(output, 0.0);

        let output = transfer_functions::tanh(1.0);
        assert!((output - 0.7616).abs() < 0.001);
    }

    #[test]
    fn test_tanh_approx_transfer_function() {
        let output = transfer_functions::tanh_approx(0.0);
        assert_eq!(output, 0.0);

        // Should be close to tanh but not exact
        let approx = transfer_functions::tanh_approx(1.0);
        let exact = transfer_functions::tanh(1.0);
        assert!((approx - exact).abs() < 0.3); // Reasonable approximation
    }

    #[test]
    fn test_hard_clip_transfer_function() {
        assert_eq!(transfer_functions::hard_clip(0.5), 0.5);
        assert_eq!(transfer_functions::hard_clip(1.5), 1.0);
        assert_eq!(transfer_functions::hard_clip(-1.5), -1.0);
    }

    #[test]
    fn test_soft_clip_transfer_function() {
        // Linear region
        assert_eq!(transfer_functions::soft_clip(0.0), 0.0);
        assert_eq!(transfer_functions::soft_clip(0.25), 0.25);

        // Clipping region
        assert_eq!(transfer_functions::soft_clip(1.5), 1.0);
        assert_eq!(transfer_functions::soft_clip(-1.5), -1.0);

        // Transition region should be smooth
        let y1 = transfer_functions::soft_clip(0.6);
        let y2 = transfer_functions::soft_clip(0.8);
        assert!(y1 > 0.6 && y1 < 1.0);
        assert!(y2 > y1 && y2 < 1.0);
    }
}
