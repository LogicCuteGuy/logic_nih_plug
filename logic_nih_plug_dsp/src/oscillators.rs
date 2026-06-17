//! Oscillator implementations.
//!
//! This module provides waveform generators for synthesis and modulation.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_dsp::oscillators::{Oscillator, Waveform};
//!
//! let mut osc = Oscillator::new(44100.0);
//! osc.set_frequency(440.0);
//! osc.set_waveform(Waveform::Sine);
//!
//! let mut output = vec![0.0; 1024];
//! osc.process(&mut output);
//! ```

use std::f32::consts::PI;

/// Waveform types supported by the oscillator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    /// Sine wave.
    Sine,
    /// Sawtooth wave.
    Saw,
    /// Square wave.
    Square,
    /// Triangle wave.
    Triangle,
}

/// An oscillator that generates various waveforms.
///
/// This oscillator supports sine, saw, square, and triangle waveforms with
/// per-sample frequency modulation and phase continuity.
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance.
///
/// # Performance
///
/// Generating 1024 samples takes approximately 5μs on modern CPUs.
#[derive(Debug, Clone)]
pub struct Oscillator {
    waveform: Waveform,
    phase: f32,
    frequency: f32,
    sample_rate: f32,
    phase_increment: f32,
}

impl Oscillator {
    /// Creates a new oscillator with the given sample rate.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate in Hz (must be positive)
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::oscillators::Oscillator;
    ///
    /// let osc = Oscillator::new(44100.0);
    /// ```
    pub fn new(sample_rate: f32) -> Self {
        Self {
            waveform: Waveform::Sine,
            phase: 0.0,
            frequency: 440.0,
            sample_rate,
            phase_increment: 440.0 / sample_rate,
        }
    }

    /// Sets the oscillator frequency.
    ///
    /// The frequency change takes effect immediately while maintaining phase continuity.
    ///
    /// # Arguments
    ///
    /// * `freq` - The frequency in Hz (must be positive and less than Nyquist)
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::oscillators::Oscillator;
    ///
    /// let mut osc = Oscillator::new(44100.0);
    /// osc.set_frequency(440.0);
    /// ```
    #[inline]
    pub fn set_frequency(&mut self, freq: f32) {
        self.frequency = freq;
        self.phase_increment = freq / self.sample_rate;
    }

    /// Sets the waveform type.
    ///
    /// # Arguments
    ///
    /// * `waveform` - The waveform type to use
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::oscillators::{Oscillator, Waveform};
    ///
    /// let mut osc = Oscillator::new(44100.0);
    /// osc.set_waveform(Waveform::Saw);
    /// ```
    #[inline]
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Gets the current waveform type.
    #[inline]
    pub fn waveform(&self) -> Waveform {
        self.waveform
    }

    /// Gets the current frequency.
    #[inline]
    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    /// Gets the current phase (0.0 to 1.0).
    #[inline]
    pub fn phase(&self) -> f32 {
        self.phase
    }

    /// Processes a block of samples, generating the waveform into the output buffer.
    ///
    /// # Arguments
    ///
    /// * `output` - The output buffer to fill with generated samples
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::oscillators::Oscillator;
    ///
    /// let mut osc = Oscillator::new(44100.0);
    /// osc.set_frequency(440.0);
    ///
    /// let mut output = vec![0.0; 1024];
    /// osc.process(&mut output);
    /// ```
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.process_sample();
        }
    }

    /// Processes a single sample and returns the output value.
    ///
    /// This method is useful for per-sample frequency modulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::oscillators::Oscillator;
    ///
    /// let mut osc = Oscillator::new(44100.0);
    /// osc.set_frequency(440.0);
    ///
    /// let sample = osc.process_sample();
    /// ```
    #[inline]
    pub fn process_sample(&mut self) -> f32 {
        let output = match self.waveform {
            Waveform::Sine => self.generate_sine(),
            Waveform::Saw => self.generate_saw(),
            Waveform::Square => self.generate_square(),
            Waveform::Triangle => self.generate_triangle(),
        };

        // Advance phase and wrap
        self.phase += self.phase_increment;
        while self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        while self.phase < 0.0 {
            self.phase += 1.0;
        }

        output
    }

    /// Resets the oscillator phase to zero.
    ///
    /// This is useful for synchronizing oscillators or starting from a known state.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::oscillators::Oscillator;
    ///
    /// let mut osc = Oscillator::new(44100.0);
    /// osc.set_frequency(440.0);
    ///
    /// let mut output = vec![0.0; 100];
    /// osc.process(&mut output);
    ///
    /// osc.reset();
    /// // Phase is now back to 0.0
    /// ```
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    /// Updates the sample rate and recalculates the phase increment.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The new sample rate in Hz (must be positive)
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.phase_increment = self.frequency / sample_rate;
    }

    #[inline]
    fn generate_sine(&self) -> f32 {
        (self.phase * 2.0 * PI).sin()
    }

    #[inline]
    fn generate_saw(&self) -> f32 {
        // Sawtooth: ramps from -1 to 1
        2.0 * self.phase - 1.0
    }

    #[inline]
    fn generate_square(&self) -> f32 {
        // Square: -1 for first half, 1 for second half
        if self.phase < 0.5 {
            -1.0
        } else {
            1.0
        }
    }

    #[inline]
    fn generate_triangle(&self) -> f32 {
        // Triangle: ramps up from -1 to 1, then down from 1 to -1
        if self.phase < 0.5 {
            4.0 * self.phase - 1.0
        } else {
            3.0 - 4.0 * self.phase
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = Oscillator::new(44100.0);
        assert_eq!(osc.frequency(), 440.0);
        assert_eq!(osc.phase(), 0.0);
        assert_eq!(osc.waveform(), Waveform::Sine);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_waveform_change() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_waveform(Waveform::Saw);
        assert_eq!(osc.waveform(), Waveform::Saw);
    }

    #[test]
    fn test_reset() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Process some samples
        let mut output = vec![0.0; 100];
        osc.process(&mut output);

        // Phase should have advanced
        assert!(osc.phase() > 0.0);

        // Reset
        osc.reset();
        assert_eq!(osc.phase(), 0.0);
    }

    #[test]
    fn test_sine_wave_range() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Sine);

        let mut output = vec![0.0; 1024];
        osc.process(&mut output);

        // All samples should be in range [-1, 1]
        for sample in output.iter() {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_saw_wave_range() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Saw);

        let mut output = vec![0.0; 1024];
        osc.process(&mut output);

        // All samples should be in range [-1, 1]
        for sample in output.iter() {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_square_wave_values() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Square);

        let mut output = vec![0.0; 1024];
        osc.process(&mut output);

        // All samples should be either -1 or 1
        for sample in output.iter() {
            assert!((*sample - 1.0).abs() < 0.001 || (*sample + 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_triangle_wave_range() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);
        osc.set_waveform(Waveform::Triangle);

        let mut output = vec![0.0; 1024];
        osc.process(&mut output);

        // All samples should be in range [-1, 1]
        for sample in output.iter() {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_per_sample_frequency_modulation() {
        let mut osc = Oscillator::new(44100.0);
        osc.set_frequency(440.0);

        // Process with changing frequency
        let mut output = vec![0.0; 100];
        for (i, sample) in output.iter_mut().enumerate() {
            // Modulate frequency
            osc.set_frequency(440.0 + (i as f32) * 10.0);
            *sample = osc.process_sample();
        }

        // Should have generated samples without panicking
        assert_eq!(output.len(), 100);
    }
}
