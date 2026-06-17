//! DC filter for removing DC offset from audio signals.
//!
//! This module provides a DC filter that removes unwanted DC offset
//! using a highpass filter with a very low cutoff frequency.

use crate::filters::IIRFilter;
use super::Processor;

/// A DC filter that removes DC offset from audio signals.
///
/// The DC filter uses a first-order highpass IIR filter with a cutoff
/// frequency of 5 Hz to remove DC offset while preserving all audible
/// frequencies. The filter automatically adapts to sample rate changes.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
///
/// let mut dc_filter = DCFilter::new();
/// dc_filter.prepare(44100.0, 512);
///
/// let input = vec![0.5; 512];  // Signal with DC offset
/// let mut output = vec![0.0; 512];
/// dc_filter.process(&input, &mut output);
/// ```
pub struct DCFilter {
    /// Internal IIR filter implementing the highpass response
    filter: IIRFilter,
    /// Cutoff frequency in Hz
    cutoff_hz: f32,
    /// Current sample rate
    sample_rate: f32,
}

impl DCFilter {
    /// Default cutoff frequency for DC removal (5 Hz)
    pub const DEFAULT_CUTOFF_HZ: f32 = 5.0;

    /// Creates a new DC filter with default cutoff frequency.
    ///
    /// The filter is initialized with a 5 Hz cutoff frequency.
    /// Call `prepare()` before processing audio.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
    ///
    /// let dc_filter = DCFilter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            filter: IIRFilter::new(),
            cutoff_hz: Self::DEFAULT_CUTOFF_HZ,
            sample_rate: 44100.0,
        }
    }

    /// Creates a new DC filter with a custom cutoff frequency.
    ///
    /// # Arguments
    ///
    /// * `cutoff_hz` - Cutoff frequency in Hz (typically 5-10 Hz)
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
    ///
    /// let dc_filter = DCFilter::with_cutoff(10.0);
    /// ```
    pub fn with_cutoff(cutoff_hz: f32) -> Self {
        Self {
            filter: IIRFilter::new(),
            cutoff_hz,
            sample_rate: 44100.0,
        }
    }

    /// Sets the cutoff frequency.
    ///
    /// This will recalculate the filter coefficients based on the current
    /// sample rate. The cutoff frequency should typically be below 10 Hz
    /// to avoid affecting audible frequencies.
    ///
    /// # Arguments
    ///
    /// * `cutoff_hz` - Cutoff frequency in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
    ///
    /// let mut dc_filter = DCFilter::new();
    /// dc_filter.prepare(44100.0, 512);
    /// dc_filter.set_cutoff(10.0);
    /// ```
    pub fn set_cutoff(&mut self, cutoff_hz: f32) {
        self.cutoff_hz = cutoff_hz;
        self.update_coefficients();
    }

    /// Gets the current cutoff frequency in Hz.
    ///
    /// # Returns
    ///
    /// The current cutoff frequency
    pub fn cutoff(&self) -> f32 {
        self.cutoff_hz
    }

    /// Prepares the DC filter for audio processing.
    ///
    /// This should be called before processing audio, typically when
    /// the sample rate or buffer size changes. It calculates the filter
    /// coefficients based on the sample rate.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz
    /// * `_max_block_size` - Maximum block size (unused but kept for trait compatibility)
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
    ///
    /// let mut dc_filter = DCFilter::new();
    /// dc_filter.prepare(44100.0, 512);
    /// ```
    pub fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.update_coefficients();
    }

    /// Processes a single sample through the DC filter.
    ///
    /// # Arguments
    ///
    /// * `input` - Input sample
    ///
    /// # Returns
    ///
    /// The processed output sample with DC offset removed
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
    ///
    /// let mut dc_filter = DCFilter::new();
    /// dc_filter.prepare(44100.0, 512);
    ///
    /// let output = dc_filter.process_sample(0.5);
    /// ```
    pub fn process_sample(&mut self, input: f32) -> f32 {
        self.filter.process_sample(input)
    }

    /// Processes a buffer of samples through the DC filter.
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
    /// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
    ///
    /// let mut dc_filter = DCFilter::new();
    /// dc_filter.prepare(44100.0, 512);
    ///
    /// let input = vec![0.5; 512];
    /// let mut output = vec![0.0; 512];
    /// dc_filter.process(&input, &mut output);
    /// ```
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.filter.process(input, output);
    }

    /// Resets the filter's internal state.
    ///
    /// This clears the delay line but does not change the filter coefficients.
    /// Call this when starting to process a new audio stream.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::dc_filter::DCFilter;
    ///
    /// let mut dc_filter = DCFilter::new();
    /// dc_filter.prepare(44100.0, 512);
    /// dc_filter.reset();
    /// ```
    pub fn reset(&mut self) {
        self.filter.reset();
    }

    /// Updates the filter coefficients based on current cutoff and sample rate.
    ///
    /// This implements a first-order highpass filter using the bilinear transform.
    /// The transfer function is:
    ///
    /// H(z) = (1 - z^-1) / (1 - a1*z^-1)
    ///
    /// where a1 is calculated from the cutoff frequency and sample rate.
    fn update_coefficients(&mut self) {
        // Calculate the filter coefficient using bilinear transform
        // For a first-order highpass filter:
        // H(s) = s / (s + wc) where wc = 2*pi*fc
        // Using bilinear transform: s = 2*fs*(1-z^-1)/(1+z^-1)
        // After simplification:
        // H(z) = ((1-z^-1) / (1 + c)) / (1 - ((1-c)/(1+c))*z^-1)
        // where c = wc / (2*fs) = pi*fc/fs
        
        use std::f32::consts::PI;
        
        let c = PI * self.cutoff_hz / self.sample_rate;
        let c_plus_1 = 1.0 + c;
        let c_minus_1 = 1.0 - c;
        
        let b0 = 1.0 / c_plus_1;
        let b1 = -b0;
        let a0 = 1.0;
        let a1 = -c_minus_1 / c_plus_1;
        
        // Set the coefficients (IIRFilter expects [b0, b1, ...] and [a0, a1, ...])
        self.filter.set_coefficients(&[b0, b1], &[a0, a1])
            .expect("DC filter coefficients should always be valid");
    }
}

impl Default for DCFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for DCFilter {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        DCFilter::prepare(self, sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        DCFilter::process(self, input, output);
    }

    fn reset(&mut self) {
        DCFilter::reset(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_filter_creation() {
        let dc_filter = DCFilter::new();
        assert_eq!(dc_filter.cutoff(), DCFilter::DEFAULT_CUTOFF_HZ);
    }

    #[test]
    fn test_dc_filter_with_cutoff() {
        let dc_filter = DCFilter::with_cutoff(10.0);
        assert_eq!(dc_filter.cutoff(), 10.0);
    }

    #[test]
    fn test_set_cutoff() {
        let mut dc_filter = DCFilter::new();
        dc_filter.prepare(44100.0, 512);
        dc_filter.set_cutoff(10.0);
        assert_eq!(dc_filter.cutoff(), 10.0);
    }

    #[test]
    fn test_prepare() {
        let mut dc_filter = DCFilter::new();
        dc_filter.prepare(48000.0, 512);
        assert_eq!(dc_filter.sample_rate, 48000.0);
    }

    #[test]
    fn test_process_removes_dc() {
        let mut dc_filter = DCFilter::new();
        dc_filter.prepare(44100.0, 512);
        
        // Create a signal with DC offset
        // Need enough samples for the filter to settle (5 Hz cutoff needs ~200ms)
        let input = vec![0.5; 10000];
        let mut output = vec![0.0; 10000];
        dc_filter.process(&input, &mut output);
        
        // After processing, the DC component should be significantly reduced
        // Check the last 1000 samples after the filter has settled
        let avg_last_1000: f32 = output[9000..].iter().sum::<f32>() / 1000.0;
        assert!(avg_last_1000.abs() < 0.05, "DC offset not removed: {}", avg_last_1000);
    }

    #[test]
    fn test_process_sample() {
        let mut dc_filter = DCFilter::new();
        dc_filter.prepare(44100.0, 512);
        
        // Process a single sample
        let output = dc_filter.process_sample(1.0);
        assert!(output.is_finite());
    }

    #[test]
    fn test_reset() {
        let mut dc_filter = DCFilter::new();
        dc_filter.prepare(44100.0, 512);
        
        // Process some samples
        let input = vec![1.0; 100];
        let mut output = vec![0.0; 100];
        dc_filter.process(&input, &mut output);
        
        // Reset should clear state
        dc_filter.reset();
        
        // After reset, processing the same input should give the same initial response
        let mut output2 = vec![0.0; 100];
        dc_filter.reset();
        dc_filter.process(&input, &mut output2);
        
        // First few samples should match
        assert!((output[0] - output2[0]).abs() < 0.001);
    }

    #[test]
    fn test_sample_rate_adaptation() {
        let mut dc_filter = DCFilter::new();
        
        // Test at 44.1 kHz - need enough samples for filter to settle
        dc_filter.prepare(44100.0, 512);
        let input = vec![0.5; 10000];
        let mut output1 = vec![0.0; 10000];
        dc_filter.process(&input, &mut output1);
        
        // Reset and test at 48 kHz
        dc_filter.reset();
        dc_filter.prepare(48000.0, 512);
        let mut output2 = vec![0.0; 10000];
        dc_filter.process(&input, &mut output2);
        
        // Both should remove DC, though the exact response will differ
        // Check after filter has settled
        let avg1: f32 = output1[9000..].iter().sum::<f32>() / 1000.0;
        let avg2: f32 = output2[9000..].iter().sum::<f32>() / 1000.0;
        
        assert!(avg1.abs() < 0.05, "44.1kHz DC not removed: {}", avg1);
        assert!(avg2.abs() < 0.05, "48kHz DC not removed: {}", avg2);
    }

    #[test]
    fn test_preserves_ac_signal() {
        let mut dc_filter = DCFilter::new();
        dc_filter.prepare(44100.0, 512);
        
        // Create a 1000 Hz sine wave (well above cutoff)
        // At 1000 Hz, a 5 Hz highpass filter should have minimal attenuation
        use std::f32::consts::PI;
        let freq = 1000.0;
        let sample_rate = 44100.0;
        let input: Vec<f32> = (0..4410)  // 100ms of audio
            .map(|i| (2.0 * PI * freq * i as f32 / sample_rate).sin())
            .collect();
        
        let mut output = vec![0.0; 4410];
        dc_filter.process(&input, &mut output);
        
        // After the filter settles (skip first 1000 samples), the amplitude should be preserved
        // Check the last 1000 samples
        let start_idx = 3410;
        let input_rms: f32 = input[start_idx..].iter().map(|x| x * x).sum::<f32>() / 1000.0;
        let output_rms: f32 = output[start_idx..].iter().map(|x| x * x).sum::<f32>() / 1000.0;
        
        let input_rms = input_rms.sqrt();
        let output_rms = output_rms.sqrt();
        
        // RMS should be very similar (within 5% for a signal 200x above cutoff)
        let ratio = output_rms / input_rms;
        assert!(ratio > 0.95 && ratio < 1.05, "AC signal not preserved: ratio = {}", ratio);
    }
}
