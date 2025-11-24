//! State Variable Filter (TPT) implementation.
//!
//! This module provides a Topology-Preserving Transform (TPT) state variable filter
//! that can produce lowpass, bandpass, and highpass outputs simultaneously.
//! The TPT method ensures stability at all parameter settings.

use crate::error::DspError;
use std::f32::consts::PI;

/// Filter type for state variable filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterType {
    /// Lowpass filter output
    Lowpass,
    /// Bandpass filter output
    Bandpass,
    /// Highpass filter output
    Highpass,
}

/// A state variable filter using the Topology-Preserving Transform (TPT) method.
///
/// This filter can produce lowpass, bandpass, and highpass outputs simultaneously
/// and maintains stability at all parameter settings. The implementation follows
/// the TPT method which uses a trapezoidal integrator structure.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::state_variable::{StateVariableFilter, FilterType};
///
/// let mut filter = StateVariableFilter::new();
/// filter.prepare(44100.0);
/// filter.set_type(FilterType::Lowpass);
/// filter.set_cutoff(1000.0);
/// filter.set_resonance(0.7);
///
/// let input = vec![1.0, 0.5, 0.25, 0.0];
/// let mut output = vec![0.0; 4];
/// filter.process(&input, &mut output);
/// ```
#[derive(Clone)]
pub struct StateVariableFilter {
    /// Current filter type
    filter_type: FilterType,
    /// Cutoff frequency in Hz
    cutoff_hz: f32,
    /// Resonance (Q factor, typically 0.0 to 1.0)
    resonance: f32,
    /// Sample rate in Hz
    sample_rate: f32,
    
    // TPT coefficients
    g: f32,  // tan(π * cutoff / sample_rate)
    k: f32,  // 2 - 2 * resonance
    a1: f32, // 1 / (1 + g * (g + k))
    a2: f32, // g * a1
    a3: f32, // g * a2
    
    // State variables
    s1: f32,
    s2: f32,
}

impl StateVariableFilter {
    /// Creates a new state variable filter with default parameters.
    ///
    /// The filter is initialized with:
    /// - Filter type: Lowpass
    /// - Cutoff: 1000 Hz
    /// - Resonance: 0.7
    /// - Sample rate: 44100 Hz
    ///
    /// Call `prepare()` with the actual sample rate before processing.
    pub fn new() -> Self {
        let mut filter = Self {
            filter_type: FilterType::Lowpass,
            cutoff_hz: 1000.0,
            resonance: 0.7,
            sample_rate: 44100.0,
            g: 0.0,
            k: 0.0,
            a1: 0.0,
            a2: 0.0,
            a3: 0.0,
            s1: 0.0,
            s2: 0.0,
        };
        filter.update_coefficients();
        filter
    }
    
    /// Prepares the filter for processing at the given sample rate.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz
    ///
    /// # Errors
    ///
    /// Returns `DspError::InvalidSampleRate` if sample rate is <= 0.
    pub fn prepare(&mut self, sample_rate: f32) -> Result<(), DspError> {
        if sample_rate <= 0.0 {
            return Err(DspError::InvalidSampleRate(sample_rate));
        }
        self.sample_rate = sample_rate;
        self.update_coefficients();
        Ok(())
    }
    
    /// Sets the filter type.
    ///
    /// # Arguments
    ///
    /// * `filter_type` - The desired filter type (Lowpass, Bandpass, or Highpass)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::state_variable::{StateVariableFilter, FilterType};
    ///
    /// let mut filter = StateVariableFilter::new();
    /// filter.set_type(FilterType::Highpass);
    /// ```
    pub fn set_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
    }
    
    /// Sets the cutoff frequency.
    ///
    /// # Arguments
    ///
    /// * `hz` - Cutoff frequency in Hz (should be less than Nyquist frequency)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::state_variable::StateVariableFilter;
    ///
    /// let mut filter = StateVariableFilter::new();
    /// filter.set_cutoff(2000.0);
    /// ```
    pub fn set_cutoff(&mut self, hz: f32) {
        self.cutoff_hz = hz.max(0.0);
        self.update_coefficients();
    }
    
    /// Sets the resonance (Q factor).
    ///
    /// # Arguments
    ///
    /// * `q` - Resonance value, typically 0.0 to 1.0
    ///   - 0.0: No resonance
    ///   - 1.0: Maximum resonance (near self-oscillation)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::state_variable::StateVariableFilter;
    ///
    /// let mut filter = StateVariableFilter::new();
    /// filter.set_resonance(0.5);
    /// ```
    pub fn set_resonance(&mut self, q: f32) {
        self.resonance = q.clamp(0.0, 1.0);
        self.update_coefficients();
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
    /// use nih_plug_dsp::state_variable::StateVariableFilter;
    ///
    /// let mut filter = StateVariableFilter::new();
    /// let output = filter.process_sample(1.0);
    /// ```
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // TPT algorithm
        let v0 = input;
        let v1 = self.a1 * self.s1 + self.a2 * (v0 - self.s2);
        let v2 = self.s2 + self.a2 * self.s1 + self.a3 * (v0 - self.s2);
        
        // Update state
        self.s1 = 2.0 * v1 - self.s1;
        self.s2 = 2.0 * v2 - self.s2;
        
        // Snap to zero to prevent denormals
        if self.s1.abs() < 1e-15 {
            self.s1 = 0.0;
        }
        if self.s2.abs() < 1e-15 {
            self.s2 = 0.0;
        }
        
        // Return appropriate output based on filter type
        match self.filter_type {
            FilterType::Lowpass => v2,
            FilterType::Bandpass => v1,
            FilterType::Highpass => v0 - self.k * v1 - v2,
        }
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
    /// use nih_plug_dsp::state_variable::StateVariableFilter;
    ///
    /// let mut filter = StateVariableFilter::new();
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
    
    /// Resets the filter's internal state to zero.
    ///
    /// This clears the state variables but preserves the filter parameters
    /// (type, cutoff, resonance, and coefficients).
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::state_variable::StateVariableFilter;
    ///
    /// let mut filter = StateVariableFilter::new();
    /// filter.set_cutoff(2000.0);
    /// 
    /// // Process some audio...
    /// let input = vec![1.0; 100];
    /// let mut output = vec![0.0; 100];
    /// filter.process(&input, &mut output);
    ///
    /// // Reset for a new stream (parameters are preserved)
    /// filter.reset();
    /// ```
    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
    
    /// Returns the current filter type.
    pub fn filter_type(&self) -> FilterType {
        self.filter_type
    }
    
    /// Returns the current cutoff frequency in Hz.
    pub fn cutoff(&self) -> f32 {
        self.cutoff_hz
    }
    
    /// Returns the current resonance value.
    pub fn resonance(&self) -> f32 {
        self.resonance
    }
    
    /// Updates the TPT coefficients based on current parameters.
    fn update_coefficients(&mut self) {
        // Clamp cutoff to valid range (avoid Nyquist issues)
        let nyquist = self.sample_rate * 0.5;
        let cutoff = self.cutoff_hz.min(nyquist * 0.99);
        
        // Calculate TPT coefficients
        self.g = (PI * cutoff / self.sample_rate).tan();
        self.k = 2.0 - 2.0 * self.resonance;
        
        let denominator = 1.0 + self.g * (self.g + self.k);
        self.a1 = 1.0 / denominator;
        self.a2 = self.g * self.a1;
        self.a3 = self.g * self.a2;
    }
}

impl Default for StateVariableFilter {
    fn default() -> Self {
        Self::new()
    }
}

// StateVariableFilter is Send because it contains only owned data
unsafe impl Send for StateVariableFilter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_filter() {
        let filter = StateVariableFilter::new();
        assert_eq!(filter.filter_type(), FilterType::Lowpass);
        assert_eq!(filter.cutoff(), 1000.0);
        assert_eq!(filter.resonance(), 0.7);
    }

    #[test]
    fn test_set_type() {
        let mut filter = StateVariableFilter::new();
        filter.set_type(FilterType::Highpass);
        assert_eq!(filter.filter_type(), FilterType::Highpass);
    }

    #[test]
    fn test_set_cutoff() {
        let mut filter = StateVariableFilter::new();
        filter.set_cutoff(2000.0);
        assert_eq!(filter.cutoff(), 2000.0);
    }

    #[test]
    fn test_set_resonance() {
        let mut filter = StateVariableFilter::new();
        filter.set_resonance(0.5);
        assert_eq!(filter.resonance(), 0.5);
    }

    #[test]
    fn test_resonance_clamping() {
        let mut filter = StateVariableFilter::new();
        filter.set_resonance(1.5);
        assert_eq!(filter.resonance(), 1.0);
        
        filter.set_resonance(-0.5);
        assert_eq!(filter.resonance(), 0.0);
    }

    #[test]
    fn test_prepare() {
        let mut filter = StateVariableFilter::new();
        assert!(filter.prepare(48000.0).is_ok());
    }

    #[test]
    fn test_prepare_invalid_sample_rate() {
        let mut filter = StateVariableFilter::new();
        assert!(filter.prepare(0.0).is_err());
        assert!(filter.prepare(-1.0).is_err());
    }

    #[test]
    fn test_process_sample() {
        let mut filter = StateVariableFilter::new();
        filter.prepare(44100.0).unwrap();
        
        let output = filter.process_sample(1.0);
        assert!(output.is_finite());
    }

    #[test]
    fn test_process_buffer() {
        let mut filter = StateVariableFilter::new();
        filter.prepare(44100.0).unwrap();
        
        let input = vec![1.0, 0.5, 0.25, 0.0];
        let mut output = vec![0.0; 4];
        filter.process(&input, &mut output);
        
        assert!(output.iter().all(|&x| x.is_finite()));
    }

    #[test]
    fn test_reset() {
        let mut filter = StateVariableFilter::new();
        filter.prepare(44100.0).unwrap();
        filter.set_cutoff(2000.0);
        filter.set_resonance(0.8);
        
        // Process some audio to populate state
        let input = vec![1.0; 10];
        let mut output = vec![0.0; 10];
        filter.process(&input, &mut output);
        
        // Store parameters before reset
        let cutoff_before = filter.cutoff();
        let resonance_before = filter.resonance();
        let type_before = filter.filter_type();
        
        // Reset should clear state but preserve parameters
        filter.reset();
        
        // Parameters should be unchanged
        assert_eq!(filter.cutoff(), cutoff_before);
        assert_eq!(filter.resonance(), resonance_before);
        assert_eq!(filter.filter_type(), type_before);
    }

    #[test]
    #[should_panic(expected = "Input and output buffers must have the same length")]
    fn test_process_mismatched_buffers() {
        let mut filter = StateVariableFilter::new();
        let input = vec![1.0; 10];
        let mut output = vec![0.0; 5];
        filter.process(&input, &mut output);
    }

    #[test]
    fn test_all_filter_types() {
        let mut filter = StateVariableFilter::new();
        filter.prepare(44100.0).unwrap();
        
        let input = vec![1.0; 10];
        let mut output = vec![0.0; 10];
        
        // Test lowpass
        filter.set_type(FilterType::Lowpass);
        filter.process(&input, &mut output);
        assert!(output.iter().all(|&x| x.is_finite()));
        
        // Test bandpass
        filter.reset();
        filter.set_type(FilterType::Bandpass);
        filter.process(&input, &mut output);
        assert!(output.iter().all(|&x| x.is_finite()));
        
        // Test highpass
        filter.reset();
        filter.set_type(FilterType::Highpass);
        filter.process(&input, &mut output);
        assert!(output.iter().all(|&x| x.is_finite()));
    }
}
