//! Gain processor for signal level adjustment.
//!
//! This module provides a gain processor that can adjust signal levels
//! with accurate decibel conversion and parameter smoothing.

use super::Processor;

/// A gain processor that applies linear gain to audio signals.
///
/// The gain processor supports both decibel and linear gain values,
/// with parameter smoothing to avoid clicks when gain changes.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::processors::gain::Gain;
///
/// let mut gain = Gain::new();
/// gain.prepare(44100.0, 512);
/// gain.set_gain_db(6.0);
///
/// let input = vec![0.5; 512];
/// let mut output = vec![0.0; 512];
/// gain.process(&input, &mut output);
/// ```
pub struct Gain {
    /// Current target gain in linear scale
    gain_linear: f32,
    /// Smoothed gain value used for processing
    smoothed_gain: f32,
    /// Smoothing coefficient (0.0 = no smoothing, 1.0 = instant)
    smoothing_coeff: f32,
    /// Sample rate for smoothing calculations
    sample_rate: f32,
}

impl Gain {
    /// Creates a new gain processor with unity gain (0 dB).
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let gain = Gain::new();
    /// ```
    pub fn new() -> Self {
        Self {
            gain_linear: 1.0,
            smoothed_gain: 1.0,
            smoothing_coeff: 0.0,
            sample_rate: 44100.0,
        }
    }

    /// Sets the gain in decibels.
    ///
    /// Uses the standard conversion: linear = 10^(dB/20)
    ///
    /// # Arguments
    ///
    /// * `db` - Gain in decibels
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut gain = Gain::new();
    /// gain.set_gain_db(6.0);  // +6 dB gain
    /// gain.set_gain_db(-6.0); // -6 dB attenuation
    /// ```
    pub fn set_gain_db(&mut self, db: f32) {
        self.gain_linear = db_to_linear(db);
    }

    /// Sets the gain as a linear multiplier.
    ///
    /// # Arguments
    ///
    /// * `gain` - Linear gain value (1.0 = unity gain)
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut gain = Gain::new();
    /// gain.set_gain_linear(2.0);  // 2x gain
    /// gain.set_gain_linear(0.5);  // 0.5x gain (attenuation)
    /// ```
    pub fn set_gain_linear(&mut self, gain: f32) {
        self.gain_linear = gain;
    }

    /// Gets the current target gain in linear scale.
    ///
    /// # Returns
    ///
    /// The current linear gain value
    pub fn gain_linear(&self) -> f32 {
        self.gain_linear
    }

    /// Gets the current target gain in decibels.
    ///
    /// # Returns
    ///
    /// The current gain in dB
    pub fn gain_db(&self) -> f32 {
        linear_to_db(self.gain_linear)
    }

    /// Sets the smoothing time constant.
    ///
    /// This controls how quickly the gain changes when set_gain_db or
    /// set_gain_linear is called. Longer times result in smoother transitions.
    ///
    /// # Arguments
    ///
    /// * `time_ms` - Smoothing time in milliseconds
    /// * `sample_rate` - Current sample rate in Hz
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut gain = Gain::new();
    /// gain.set_smoothing_time(10.0, 44100.0); // 10ms smoothing
    /// ```
    pub fn set_smoothing_time(&mut self, time_ms: f32, sample_rate: f32) {
        self.sample_rate = sample_rate;
        if time_ms <= 0.0 {
            self.smoothing_coeff = 1.0; // Instant
        } else {
            // Calculate coefficient for exponential smoothing
            // tau = time_ms / 1000.0 (convert to seconds)
            // coeff = 1.0 - exp(-1.0 / (tau * sample_rate))
            let tau = time_ms / 1000.0;
            self.smoothing_coeff = 1.0 - (-1.0 / (tau * sample_rate)).exp();
        }
    }

    /// Prepares the processor for audio processing.
    ///
    /// This should be called before processing audio, typically when
    /// the sample rate or buffer size changes.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz
    /// * `_max_block_size` - Maximum block size (unused but kept for trait compatibility)
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut gain = Gain::new();
    /// gain.prepare(44100.0, 512);
    /// ```
    pub fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate;
        // Reset smoothed gain to current target
        self.smoothed_gain = self.gain_linear;
    }

    /// Processes a single sample through the gain processor.
    ///
    /// # Arguments
    ///
    /// * `input` - Input sample
    ///
    /// # Returns
    ///
    /// The processed output sample
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut gain = Gain::new();
    /// gain.prepare(44100.0, 512);
    /// gain.set_gain_db(6.0);
    ///
    /// let output = gain.process_sample(0.5);
    /// ```
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Apply exponential smoothing
        self.smoothed_gain += self.smoothing_coeff * (self.gain_linear - self.smoothed_gain);
        
        // Apply gain
        input * self.smoothed_gain
    }

    /// Processes a buffer of samples through the gain processor.
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
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut gain = Gain::new();
    /// gain.prepare(44100.0, 512);
    /// gain.set_gain_db(6.0);
    ///
    /// let input = vec![0.5; 512];
    /// let mut output = vec![0.0; 512];
    /// gain.process(&input, &mut output);
    /// ```
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
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
    /// This resets the smoothed gain to match the current target gain,
    /// effectively removing any smoothing in progress.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::gain::Gain;
    ///
    /// let mut gain = Gain::new();
    /// gain.set_gain_db(6.0);
    /// gain.reset();
    /// ```
    pub fn reset(&mut self) {
        self.smoothed_gain = self.gain_linear;
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for Gain {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        Gain::prepare(self, sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        Gain::process(self, input, output);
    }

    fn reset(&mut self) {
        Gain::reset(self);
    }
}

/// Converts decibels to linear gain.
///
/// Uses the formula: linear = 10^(dB/20)
///
/// # Arguments
///
/// * `db` - Gain in decibels
///
/// # Returns
///
/// Linear gain value
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::processors::gain::db_to_linear;
///
/// let linear = db_to_linear(6.0);
/// assert!((linear - 1.995).abs() < 0.01);
/// ```
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Converts linear gain to decibels.
///
/// Uses the formula: dB = 20 * log10(linear)
///
/// # Arguments
///
/// * `linear` - Linear gain value
///
/// # Returns
///
/// Gain in decibels
///
/// # Examples
///
/// ```
/// use logic_nih_plug_dsp::processors::gain::linear_to_db;
///
/// let db = linear_to_db(2.0);
/// assert!((db - 6.02).abs() < 0.01);
/// ```
pub fn linear_to_db(linear: f32) -> f32 {
    20.0 * linear.log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gain_creation() {
        let gain = Gain::new();
        assert_eq!(gain.gain_linear(), 1.0);
        assert!((gain.gain_db() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_set_gain_db() {
        let mut gain = Gain::new();
        gain.set_gain_db(6.0);
        assert!((gain.gain_linear() - 1.995).abs() < 0.01);
    }

    #[test]
    fn test_set_gain_linear() {
        let mut gain = Gain::new();
        gain.set_gain_linear(2.0);
        assert_eq!(gain.gain_linear(), 2.0);
        assert!((gain.gain_db() - 6.02).abs() < 0.01);
    }

    #[test]
    fn test_db_to_linear_conversion() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((db_to_linear(6.0) - 1.995).abs() < 0.01);
        assert!((db_to_linear(-6.0) - 0.501).abs() < 0.01);
        assert!((db_to_linear(20.0) - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_linear_to_db_conversion() {
        assert!((linear_to_db(1.0) - 0.0).abs() < 0.001);
        assert!((linear_to_db(2.0) - 6.02).abs() < 0.01);
        assert!((linear_to_db(0.5) - (-6.02)).abs() < 0.01);
        assert!((linear_to_db(10.0) - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_gain_process_sample() {
        let mut gain = Gain::new();
        gain.prepare(44100.0, 512);
        gain.set_gain_linear(2.0);
        gain.set_smoothing_time(0.0, 44100.0); // No smoothing for test
        
        let output = gain.process_sample(0.5);
        assert!((output - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_gain_process_buffer() {
        let mut gain = Gain::new();
        gain.prepare(44100.0, 512);
        gain.set_gain_linear(2.0);
        gain.set_smoothing_time(0.0, 44100.0); // No smoothing for test
        
        let input = vec![0.5; 10];
        let mut output = vec![0.0; 10];
        gain.process(&input, &mut output);
        
        for &sample in output.iter() {
            assert!((sample - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_gain_reset() {
        let mut gain = Gain::new();
        gain.prepare(44100.0, 512);
        gain.set_gain_linear(2.0);
        
        // Process some samples to change smoothed_gain
        let input = vec![0.5; 10];
        let mut output = vec![0.0; 10];
        gain.process(&input, &mut output);
        
        // Reset should set smoothed_gain to target
        gain.reset();
        assert_eq!(gain.smoothed_gain, gain.gain_linear);
    }

    #[test]
    #[should_panic(expected = "Input and output buffers must have the same length")]
    fn test_gain_mismatched_buffers() {
        let mut gain = Gain::new();
        gain.prepare(44100.0, 512);
        
        let input = vec![0.5; 10];
        let mut output = vec![0.0; 5];
        gain.process(&input, &mut output);
    }

    #[test]
    fn test_unity_gain() {
        let mut gain = Gain::new();
        gain.prepare(44100.0, 512);
        gain.set_gain_db(0.0);
        gain.set_smoothing_time(0.0, 44100.0);
        
        let input = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let mut output = vec![0.0; 5];
        gain.process(&input, &mut output);
        
        for (inp, out) in input.iter().zip(output.iter()) {
            assert!((inp - out).abs() < 0.001);
        }
    }
}
