//! Common DSP utilities.
//!
//! This module provides utility functions for common DSP operations like
//! sample rate conversion and buffer manipulation.

use crate::error::DspError;

/// Validates that a sample rate is within reasonable bounds.
///
/// # Arguments
///
/// * `sample_rate` - The sample rate to validate (in Hz)
///
/// # Returns
///
/// Returns `Ok(())` if the sample rate is valid, otherwise returns an error.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::validate_sample_rate;
///
/// assert!(validate_sample_rate(44100.0).is_ok());
/// assert!(validate_sample_rate(48000.0).is_ok());
/// assert!(validate_sample_rate(0.0).is_err());
/// assert!(validate_sample_rate(-1.0).is_err());
/// ```
pub fn validate_sample_rate(sample_rate: f32) -> Result<(), DspError> {
    if sample_rate <= 0.0 || !sample_rate.is_finite() {
        return Err(DspError::InvalidSampleRate(sample_rate));
    }
    Ok(())
}

/// Validates that a buffer size is within reasonable bounds.
///
/// # Arguments
///
/// * `buffer_size` - The buffer size to validate (in samples)
///
/// # Returns
///
/// Returns `Ok(())` if the buffer size is valid, otherwise returns an error.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::validate_buffer_size;
///
/// assert!(validate_buffer_size(512).is_ok());
/// assert!(validate_buffer_size(1024).is_ok());
/// assert!(validate_buffer_size(0).is_err());
/// ```
pub fn validate_buffer_size(buffer_size: usize) -> Result<(), DspError> {
    if buffer_size == 0 {
        return Err(DspError::InvalidBufferSize(buffer_size));
    }
    Ok(())
}

/// Converts a frequency in Hz to a phase increment for a given sample rate.
///
/// # Arguments
///
/// * `frequency` - The frequency in Hz
/// * `sample_rate` - The sample rate in Hz
///
/// # Returns
///
/// The phase increment per sample (0.0 to 1.0 represents one full cycle).
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::frequency_to_phase_increment;
///
/// // 440 Hz at 44100 Hz sample rate
/// let phase_inc = frequency_to_phase_increment(440.0, 44100.0);
/// assert!((phase_inc - 0.00997).abs() < 0.0001);
/// ```
#[inline]
pub fn frequency_to_phase_increment(frequency: f32, sample_rate: f32) -> f32 {
    frequency / sample_rate
}

/// Converts a phase increment to a frequency in Hz for a given sample rate.
///
/// # Arguments
///
/// * `phase_increment` - The phase increment per sample
/// * `sample_rate` - The sample rate in Hz
///
/// # Returns
///
/// The frequency in Hz.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::phase_increment_to_frequency;
///
/// let freq = phase_increment_to_frequency(0.00997, 44100.0);
/// assert!((freq - 440.0).abs() < 1.0);
/// ```
#[inline]
pub fn phase_increment_to_frequency(phase_increment: f32, sample_rate: f32) -> f32 {
    phase_increment * sample_rate
}

/// Clears a buffer by setting all samples to zero.
///
/// # Arguments
///
/// * `buffer` - The buffer to clear
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::clear_buffer;
///
/// let mut buffer = vec![1.0, 2.0, 3.0];
/// clear_buffer(&mut buffer);
/// assert_eq!(buffer, vec![0.0, 0.0, 0.0]);
/// ```
#[inline]
pub fn clear_buffer(buffer: &mut [f32]) {
    buffer.fill(0.0);
}

/// Copies samples from one buffer to another.
///
/// # Arguments
///
/// * `src` - The source buffer
/// * `dst` - The destination buffer
///
/// # Panics
///
/// Panics if the buffers have different lengths.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::copy_buffer;
///
/// let src = vec![1.0, 2.0, 3.0];
/// let mut dst = vec![0.0, 0.0, 0.0];
/// copy_buffer(&src, &mut dst);
/// assert_eq!(dst, vec![1.0, 2.0, 3.0]);
/// ```
#[inline]
pub fn copy_buffer(src: &[f32], dst: &mut [f32]) {
    assert_eq!(
        src.len(),
        dst.len(),
        "Source and destination buffers must have the same length"
    );
    dst.copy_from_slice(src);
}

/// Adds samples from one buffer to another (dst += src).
///
/// # Arguments
///
/// * `src` - The source buffer
/// * `dst` - The destination buffer
///
/// # Panics
///
/// Panics if the buffers have different lengths.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::add_buffer;
///
/// let src = vec![1.0, 2.0, 3.0];
/// let mut dst = vec![1.0, 1.0, 1.0];
/// add_buffer(&src, &mut dst);
/// assert_eq!(dst, vec![2.0, 3.0, 4.0]);
/// ```
#[inline]
pub fn add_buffer(src: &[f32], dst: &mut [f32]) {
    assert_eq!(
        src.len(),
        dst.len(),
        "Source and destination buffers must have the same length"
    );
    for (s, d) in src.iter().zip(dst.iter_mut()) {
        *d += *s;
    }
}

/// Multiplies all samples in a buffer by a gain factor.
///
/// # Arguments
///
/// * `buffer` - The buffer to scale
/// * `gain` - The gain factor to apply
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::scale_buffer;
///
/// let mut buffer = vec![1.0, 2.0, 3.0];
/// scale_buffer(&mut buffer, 2.0);
/// assert_eq!(buffer, vec![2.0, 4.0, 6.0]);
/// ```
#[inline]
pub fn scale_buffer(buffer: &mut [f32], gain: f32) {
    for sample in buffer.iter_mut() {
        *sample *= gain;
    }
}

/// Linear interpolation between two values.
///
/// # Arguments
///
/// * `a` - The start value
/// * `b` - The end value
/// * `t` - The interpolation factor (0.0 to 1.0)
///
/// # Returns
///
/// The interpolated value.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::lerp;
///
/// assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
/// assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
/// assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
/// ```
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamps a value between a minimum and maximum.
///
/// # Arguments
///
/// * `value` - The value to clamp
/// * `min` - The minimum value
/// * `max` - The maximum value
///
/// # Returns
///
/// The clamped value.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::util::clamp;
///
/// assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
/// assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
/// assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
/// ```
#[inline]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_sample_rate() {
        assert!(validate_sample_rate(44100.0).is_ok());
        assert!(validate_sample_rate(48000.0).is_ok());
        assert!(validate_sample_rate(96000.0).is_ok());
        assert!(validate_sample_rate(0.0).is_err());
        assert!(validate_sample_rate(-1.0).is_err());
        assert!(validate_sample_rate(f32::NAN).is_err());
        assert!(validate_sample_rate(f32::INFINITY).is_err());
    }

    #[test]
    fn test_validate_buffer_size() {
        assert!(validate_buffer_size(1).is_ok());
        assert!(validate_buffer_size(512).is_ok());
        assert!(validate_buffer_size(1024).is_ok());
        assert!(validate_buffer_size(0).is_err());
    }

    #[test]
    fn test_frequency_conversions() {
        let freq = 440.0;
        let sample_rate = 44100.0;
        let phase_inc = frequency_to_phase_increment(freq, sample_rate);
        let freq_back = phase_increment_to_frequency(phase_inc, sample_rate);
        assert!((freq - freq_back).abs() < 0.001);
    }

    #[test]
    fn test_clear_buffer() {
        let mut buffer = vec![1.0, 2.0, 3.0];
        clear_buffer(&mut buffer);
        assert_eq!(buffer, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_copy_buffer() {
        let src = vec![1.0, 2.0, 3.0];
        let mut dst = vec![0.0, 0.0, 0.0];
        copy_buffer(&src, &mut dst);
        assert_eq!(dst, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_add_buffer() {
        let src = vec![1.0, 2.0, 3.0];
        let mut dst = vec![1.0, 1.0, 1.0];
        add_buffer(&src, &mut dst);
        assert_eq!(dst, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_scale_buffer() {
        let mut buffer = vec![1.0, 2.0, 3.0];
        scale_buffer(&mut buffer, 2.0);
        assert_eq!(buffer, vec![2.0, 4.0, 6.0]);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }
}
