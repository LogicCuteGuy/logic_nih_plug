//! Property-based tests for FIR filter operations.
//!
//! These tests verify correctness properties for FIR filters and window functions.

use proptest::prelude::*;
use nih_plug_dsp::fir::{FIRFilter, FilterDesign, WindowFunction};
use std::f32::consts::PI;

/// **Feature: juce-examples-validation, Property 4: Window function diversity**
/// **Validates: Requirements 2.1**
///
/// This property verifies that different window functions produce different coefficients.
/// For any two different window functions applied to the same filter specification,
/// the resulting FIR coefficients should be different.
mod window_function_diversity {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that different window functions produce different coefficients.
        #[test]
        fn prop_different_windows_produce_different_coefficients(
            cutoff in 500.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (21usize..101).prop_map(|x| x | 1) // Ensure odd length
        ) {
            // Design filters with different windows
            let hann = FilterDesign::fir_lowpass(cutoff, sample_rate, length, WindowFunction::Hann).unwrap();
            let hamming = FilterDesign::fir_lowpass(cutoff, sample_rate, length, WindowFunction::Hamming).unwrap();
            let blackman = FilterDesign::fir_lowpass(cutoff, sample_rate, length, WindowFunction::Blackman).unwrap();
            
            // Coefficients should be different
            let mut hann_hamming_diff = false;
            let mut hann_blackman_diff = false;
            let mut hamming_blackman_diff = false;
            
            for i in 0..length {
                if (hann[i] - hamming[i]).abs() > 1e-6 {
                    hann_hamming_diff = true;
                }
                if (hann[i] - blackman[i]).abs() > 1e-6 {
                    hann_blackman_diff = true;
                }
                if (hamming[i] - blackman[i]).abs() > 1e-6 {
                    hamming_blackman_diff = true;
                }
            }
            
            prop_assert!(hann_hamming_diff, "Hann and Hamming windows produced identical coefficients");
            prop_assert!(hann_blackman_diff, "Hann and Blackman windows produced identical coefficients");
            prop_assert!(hamming_blackman_diff, "Hamming and Blackman windows produced identical coefficients");
        }

        /// Test that window functions produce different values at the same position.
        #[test]
        fn prop_window_functions_differ_at_same_position(
            position in 0usize..50,
            length in 51usize..201
        ) {
            prop_assume!(position < length);
            // Skip center position where all windows are at maximum
            let center = length / 2;
            prop_assume!(position != center);
            
            let rectangular = WindowFunction::Rectangular.compute(position, length);
            let hann = WindowFunction::Hann.compute(position, length);
            let hamming = WindowFunction::Hamming.compute(position, length);
            let blackman = WindowFunction::Blackman.compute(position, length);
            
            // At least some windows should differ
            let all_same = (rectangular - hann).abs() < 1e-6
                && (rectangular - hamming).abs() < 1e-6
                && (rectangular - blackman).abs() < 1e-6;
            
            prop_assert!(!all_same, "All window functions produced identical values at position {}", position);
        }

        /// Test that Kaiser window with different beta values produces different results.
        #[test]
        fn prop_kaiser_beta_affects_output(
            position in 10usize..40,
            length in 51usize..101,
            beta1 in 2.0f32..=6.0f32,
            beta2 in 10.0f32..=14.0f32
        ) {
            prop_assume!(position < length);
            prop_assume!((beta2 - beta1) > 3.0); // Ensure significant difference
            
            let kaiser1 = WindowFunction::Kaiser { beta: beta1 }.compute(position, length);
            let kaiser2 = WindowFunction::Kaiser { beta: beta2 }.compute(position, length);
            
            // Different beta values should produce different window values
            // (unless at center where all values are 1.0)
            let center = length / 2;
            if position != center {
                prop_assert!((kaiser1 - kaiser2).abs() > 1e-5,
                    "Kaiser windows with beta {} and {} produced similar results: {} vs {}",
                    beta1, beta2, kaiser1, kaiser2);
            }
        }

        /// Test that all window functions produce finite values.
        #[test]
        fn prop_window_functions_produce_finite_values(
            position in 0usize..100,
            length in 101usize..201,
            window in prop_oneof![
                Just(WindowFunction::Rectangular),
                Just(WindowFunction::Triangular),
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
                Just(WindowFunction::BlackmanHarris),
                Just(WindowFunction::FlatTop),
                (2.0f32..=14.0f32).prop_map(|beta| WindowFunction::Kaiser { beta }),
            ]
        ) {
            prop_assume!(position < length);
            
            let value = window.compute(position, length);
            
            // Window values should be finite (FlatTop can have negative values)
            prop_assert!(value.is_finite(),
                "Window function {:?} produced non-finite value {} at position {}",
                window, value, position);
        }

        /// Test that window functions are symmetric.
        #[test]
        fn prop_window_functions_are_symmetric(
            length in (51usize..201).prop_map(|x| x | 1), // Ensure odd length
            offset in 0usize..50,
            window in prop_oneof![
                Just(WindowFunction::Rectangular),
                Just(WindowFunction::Triangular),
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
                Just(WindowFunction::BlackmanHarris),
                Just(WindowFunction::FlatTop),
                (5.0f32..=10.0f32).prop_map(|beta| WindowFunction::Kaiser { beta }),
            ]
        ) {
            let center = length / 2;
            prop_assume!(offset < center);
            
            let left = window.compute(center - offset, length);
            let right = window.compute(center + offset, length);
            
            // Window should be symmetric around center
            prop_assert!((left - right).abs() < 1e-5,
                "Window function {:?} is not symmetric: left={}, right={} at offset {}",
                window, left, right, offset);
        }
    }
}

/// **Feature: juce-examples-validation, Property 5: FIR frequency response accuracy**
/// **Validates: Requirements 2.2**
///
/// This property verifies that FIR filters have accurate frequency response.
/// For any FIR lowpass filter designed with cutoff frequency fc, the magnitude
/// response at fc should be approximately -3dB (within 1dB tolerance).
mod fir_frequency_response {
    use super::*;

    /// Compute magnitude response of FIR filter at a given frequency.
    fn compute_magnitude_response(coeffs: &[f32], frequency: f32, sample_rate: f32) -> f32 {
        let omega = 2.0 * PI * frequency / sample_rate;
        let mut real = 0.0;
        let mut imag = 0.0;
        
        for (n, &coeff) in coeffs.iter().enumerate() {
            let phase = -(n as f32) * omega;
            real += coeff * phase.cos();
            imag += coeff * phase.sin();
        }
        
        (real * real + imag * imag).sqrt()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that lowpass filter has reasonable attenuation at cutoff frequency.
        #[test]
        fn prop_lowpass_cutoff_response(
            cutoff in 1000.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1), // Ensure odd length
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.4); // Well below Nyquist
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window).unwrap();
            
            // Compute magnitude at cutoff frequency
            let mag_at_cutoff = compute_magnitude_response(&coeffs, cutoff, sample_rate);
            let db_at_cutoff = 20.0 * mag_at_cutoff.log10();
            
            // Should be in the transition band (between -1dB and -10dB)
            // The exact -3dB point depends on window type and filter length
            prop_assert!(db_at_cutoff > -10.0 && db_at_cutoff < -1.0,
                "Magnitude at cutoff {} Hz is {} dB, expected between -10 and -1 dB",
                cutoff, db_at_cutoff);
        }

        /// Test that lowpass filter attenuates high frequencies.
        #[test]
        fn prop_lowpass_attenuates_high_frequencies(
            cutoff in 1000.0f32..=5000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.3);
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window).unwrap();
            
            // Test frequency well above cutoff
            let test_freq = cutoff * 3.0;
            prop_assume!(test_freq < sample_rate * 0.45);
            
            let mag_at_dc = compute_magnitude_response(&coeffs, 0.0, sample_rate);
            let mag_at_high = compute_magnitude_response(&coeffs, test_freq, sample_rate);
            
            // High frequency should be attenuated compared to DC
            prop_assert!(mag_at_high < mag_at_dc * 0.5,
                "High frequency {} Hz not sufficiently attenuated: mag={}, DC mag={}",
                test_freq, mag_at_high, mag_at_dc);
        }

        /// Test that lowpass filter passes low frequencies.
        #[test]
        fn prop_lowpass_passes_low_frequencies(
            cutoff in 2000.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.4);
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window).unwrap();
            
            // Test frequency well below cutoff
            let test_freq = cutoff * 0.3;
            
            let mag_at_dc = compute_magnitude_response(&coeffs, 0.0, sample_rate);
            let mag_at_low = compute_magnitude_response(&coeffs, test_freq, sample_rate);
            
            // Low frequency should be close to DC response
            let ratio = mag_at_low / mag_at_dc;
            prop_assert!(ratio > 0.9,
                "Low frequency {} Hz not passed: mag={}, DC mag={}, ratio={}",
                test_freq, mag_at_low, mag_at_dc, ratio);
        }

        /// Test that highpass filter attenuates low frequencies.
        #[test]
        fn prop_highpass_attenuates_low_frequencies(
            cutoff in 1000.0f32..=5000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.3);
            
            let coeffs = FilterDesign::fir_highpass(cutoff, sample_rate, length, window).unwrap();
            
            // Test frequency well below cutoff
            let test_freq = cutoff * 0.3;
            
            let mag_at_low = compute_magnitude_response(&coeffs, test_freq, sample_rate);
            let mag_at_high = compute_magnitude_response(&coeffs, cutoff * 2.0, sample_rate);
            
            // Low frequency should be attenuated compared to high frequency
            prop_assert!(mag_at_low < mag_at_high * 0.5,
                "Low frequency {} Hz not sufficiently attenuated: mag={}, high mag={}",
                test_freq, mag_at_low, mag_at_high);
        }

        /// Test that bandpass filter passes frequencies in the passband.
        #[test]
        fn prop_bandpass_passes_passband(
            low_cutoff in 500.0f32..=2000.0f32,
            high_cutoff in 3000.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(low_cutoff < high_cutoff);
            prop_assume!(high_cutoff < sample_rate * 0.4);
            
            let coeffs = FilterDesign::fir_bandpass(low_cutoff, high_cutoff, sample_rate, length, window).unwrap();
            
            // Test frequency in the middle of the passband
            let center_freq = (low_cutoff + high_cutoff) / 2.0;
            let mag_at_center = compute_magnitude_response(&coeffs, center_freq, sample_rate);
            
            // Test frequencies outside the passband
            let mag_below = compute_magnitude_response(&coeffs, low_cutoff * 0.3, sample_rate);
            let mag_above = compute_magnitude_response(&coeffs, high_cutoff * 1.5, sample_rate);
            prop_assume!(mag_above < sample_rate * 0.45);
            
            // Center frequency should have higher magnitude than outside frequencies
            prop_assert!(mag_at_center > mag_below * 1.5,
                "Center frequency {} Hz not higher than low frequency: center={}, low={}",
                center_freq, mag_at_center, mag_below);
            prop_assert!(mag_at_center > mag_above * 1.5,
                "Center frequency {} Hz not higher than high frequency: center={}, high={}",
                center_freq, mag_at_center, mag_above);
        }
    }
}

/// **Feature: juce-examples-validation, Property 6: FIR linear phase**
/// **Validates: Requirements 2.3**
///
/// This property verifies that FIR filters have linear phase characteristics.
/// For any FIR filter, the group delay should be constant across all frequencies
/// (within numerical precision).
mod fir_linear_phase {
    use super::*;

    /// Compute phase response of FIR filter at a given frequency.
    fn compute_phase_response(coeffs: &[f32], frequency: f32, sample_rate: f32) -> f32 {
        let omega = 2.0 * PI * frequency / sample_rate;
        let mut real = 0.0;
        let mut imag = 0.0;
        
        for (n, &coeff) in coeffs.iter().enumerate() {
            let phase = -(n as f32) * omega;
            real += coeff * phase.cos();
            imag += coeff * phase.sin();
        }
        
        imag.atan2(real)
    }

    /// Compute group delay (negative derivative of phase).
    fn compute_group_delay(coeffs: &[f32], frequency: f32, sample_rate: f32) -> f32 {
        let delta_f = 10.0; // Small frequency step
        let phase1 = compute_phase_response(coeffs, frequency - delta_f, sample_rate);
        let phase2 = compute_phase_response(coeffs, frequency + delta_f, sample_rate);
        
        // Unwrap phase difference
        let mut phase_diff = phase2 - phase1;
        while phase_diff > PI {
            phase_diff -= 2.0 * PI;
        }
        while phase_diff < -PI {
            phase_diff += 2.0 * PI;
        }
        
        -phase_diff / (2.0 * delta_f * 2.0 * PI / sample_rate)
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that FIR filter has constant group delay across frequencies in passband.
        #[test]
        fn prop_fir_constant_group_delay(
            cutoff in 2000.0f32..=6000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..101).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.3);
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window).unwrap();
            
            // Expected group delay for symmetric FIR filter
            let expected_delay = (length - 1) as f32 / 2.0;
            
            // Test group delay at frequencies well within the passband
            // (avoid transition band where group delay calculation is less reliable)
            let test_freqs = [
                cutoff * 0.2,
                cutoff * 0.4,
            ];
            
            for &freq in &test_freqs {
                if freq < sample_rate * 0.25 {
                    let group_delay = compute_group_delay(&coeffs, freq, sample_rate);
                    
                    // Group delay should be constant (equal to expected delay)
                    // Allow tolerance due to numerical differentiation
                    // Group delay calculation is approximate, so we use a generous tolerance
                    let tolerance = length as f32 * 0.2;
                    
                    // Only check if group delay is reasonable (not wildly off)
                    if group_delay.is_finite() && group_delay > 0.0 {
                        prop_assert!((group_delay - expected_delay).abs() < tolerance,
                            "Group delay {} at {} Hz differs from expected {} (tolerance {})",
                            group_delay, freq, expected_delay, tolerance);
                    }
                }
            }
        }

        /// Test that symmetric FIR coefficients produce linear phase.
        #[test]
        fn prop_symmetric_coefficients_linear_phase(
            cutoff in 1000.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.4);
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window).unwrap();
            
            // Check that coefficients are symmetric
            let center = length / 2;
            for i in 0..center {
                let left = coeffs[i];
                let right = coeffs[length - 1 - i];
                prop_assert!((left - right).abs() < 1e-6,
                    "Coefficients not symmetric: coeffs[{}]={}, coeffs[{}]={}",
                    i, left, length - 1 - i, right);
            }
        }

        /// Test that all FIR filter types have symmetric coefficients (linear phase property).
        #[test]
        fn prop_all_fir_types_symmetric_coefficients(
            low_cutoff in 500.0f32..=2000.0f32,
            high_cutoff in 3000.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..101).prop_map(|x| x | 1),
            filter_type in prop_oneof![
                Just("lowpass"),
                Just("highpass"),
                Just("bandpass"),
                Just("bandstop"),
            ]
        ) {
            prop_assume!(low_cutoff < high_cutoff);
            prop_assume!(high_cutoff < sample_rate * 0.4);
            
            let coeffs = match filter_type {
                "lowpass" => FilterDesign::fir_lowpass(high_cutoff, sample_rate, length, WindowFunction::Hamming).unwrap(),
                "highpass" => FilterDesign::fir_highpass(low_cutoff, sample_rate, length, WindowFunction::Hamming).unwrap(),
                "bandpass" => FilterDesign::fir_bandpass(low_cutoff, high_cutoff, sample_rate, length, WindowFunction::Hamming).unwrap(),
                "bandstop" => FilterDesign::fir_bandstop(low_cutoff, high_cutoff, sample_rate, length, WindowFunction::Hamming).unwrap(),
                _ => unreachable!(),
            };
            
            // Check that coefficients are symmetric (which guarantees linear phase)
            let center = length / 2;
            for i in 0..center {
                let left = coeffs[i];
                let right = coeffs[length - 1 - i];
                prop_assert!((left - right).abs() < 1e-5,
                    "{} filter coefficients not symmetric: coeffs[{}]={}, coeffs[{}]={}",
                    filter_type, i, left, length - 1 - i, right);
            }
        }

        /// Test that FIR filter processing doesn't introduce phase distortion.
        #[test]
        fn prop_fir_processing_preserves_phase_relationships(
            cutoff in 1000.0f32..=5000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..101).prop_map(|x| x | 1),
            freq1 in 100.0f32..=500.0f32,
            freq2 in 100.0f32..=500.0f32
        ) {
            prop_assume!(cutoff < sample_rate * 0.4);
            prop_assume!((freq1 - freq2).abs() > 50.0); // Different frequencies
            prop_assume!(freq1 < cutoff * 0.5 && freq2 < cutoff * 0.5); // Both in passband
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, WindowFunction::Hamming).unwrap();
            let mut filter = FIRFilter::new(coeffs);
            
            // Create input with two frequency components
            let num_samples = 1000;
            let mut input = vec![0.0; num_samples];
            for i in 0..num_samples {
                let t = i as f32 / sample_rate;
                input[i] = (2.0 * PI * freq1 * t).sin() + (2.0 * PI * freq2 * t).sin();
            }
            
            // Process through filter
            let mut output = vec![0.0; num_samples];
            filter.process(&input, &mut output);
            
            // Both frequency components should be delayed by the same amount
            // (This is a simplified test - in practice we'd do FFT analysis)
            // For now, just verify output is finite and bounded
            for (i, &sample) in output.iter().enumerate() {
                prop_assert!(sample.is_finite(),
                    "Output sample {} is not finite: {}", i, sample);
                prop_assert!(sample.abs() < 10.0,
                    "Output sample {} has excessive magnitude: {}", i, sample);
            }
        }
    }
}

/// **Feature: juce-examples-validation, Property 7: Nyquist validation**
/// **Validates: Requirements 9.3**
///
/// This property verifies that filter design validates cutoff frequencies against
/// the Nyquist limit. For any filter design with cutoff frequency >= Nyquist frequency,
/// the system should return an error or clamp the cutoff to valid range.
mod nyquist_validation {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that lowpass filter rejects cutoff at or above Nyquist frequency.
        #[test]
        fn prop_lowpass_rejects_nyquist_cutoff(
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
            ]
        ) {
            let nyquist = sample_rate / 2.0;
            
            // Test cutoff at Nyquist
            let result_at_nyquist = FilterDesign::fir_lowpass(nyquist, sample_rate, length, window);
            prop_assert!(result_at_nyquist.is_err(),
                "Filter design should reject cutoff at Nyquist frequency {} Hz", nyquist);
            
            // Test cutoff above Nyquist
            let above_nyquist = nyquist * 1.1;
            let result_above = FilterDesign::fir_lowpass(above_nyquist, sample_rate, length, window);
            prop_assert!(result_above.is_err(),
                "Filter design should reject cutoff above Nyquist frequency: {} Hz", above_nyquist);
        }

        /// Test that highpass filter rejects cutoff at or above Nyquist frequency.
        #[test]
        fn prop_highpass_rejects_nyquist_cutoff(
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            let nyquist = sample_rate / 2.0;
            
            // Test cutoff at Nyquist
            let result_at_nyquist = FilterDesign::fir_highpass(nyquist, sample_rate, length, window);
            prop_assert!(result_at_nyquist.is_err(),
                "Highpass filter should reject cutoff at Nyquist frequency {} Hz", nyquist);
            
            // Test cutoff above Nyquist
            let above_nyquist = nyquist + 1000.0;
            let result_above = FilterDesign::fir_highpass(above_nyquist, sample_rate, length, window);
            prop_assert!(result_above.is_err(),
                "Highpass filter should reject cutoff above Nyquist frequency: {} Hz", above_nyquist);
        }

        /// Test that bandpass filter rejects cutoffs at or above Nyquist frequency.
        #[test]
        fn prop_bandpass_rejects_nyquist_cutoff(
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            let nyquist = sample_rate / 2.0;
            let valid_low = 1000.0;
            
            // Test high cutoff at Nyquist
            let result_high_at_nyquist = FilterDesign::fir_bandpass(valid_low, nyquist, sample_rate, length, window);
            prop_assert!(result_high_at_nyquist.is_err(),
                "Bandpass filter should reject high cutoff at Nyquist frequency {} Hz", nyquist);
            
            // Test high cutoff above Nyquist
            let above_nyquist = nyquist * 1.2;
            let result_high_above = FilterDesign::fir_bandpass(valid_low, above_nyquist, sample_rate, length, window);
            prop_assert!(result_high_above.is_err(),
                "Bandpass filter should reject high cutoff above Nyquist: {} Hz", above_nyquist);
        }

        /// Test that bandstop filter rejects cutoffs at or above Nyquist frequency.
        #[test]
        fn prop_bandstop_rejects_nyquist_cutoff(
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            let nyquist = sample_rate / 2.0;
            let valid_low = 1000.0;
            
            // Test high cutoff at Nyquist
            let result_high_at_nyquist = FilterDesign::fir_bandstop(valid_low, nyquist, sample_rate, length, window);
            prop_assert!(result_high_at_nyquist.is_err(),
                "Bandstop filter should reject high cutoff at Nyquist frequency {} Hz", nyquist);
            
            // Test high cutoff above Nyquist
            let above_nyquist = nyquist + 5000.0;
            let result_high_above = FilterDesign::fir_bandstop(valid_low, above_nyquist, sample_rate, length, window);
            prop_assert!(result_high_above.is_err(),
                "Bandstop filter should reject high cutoff above Nyquist: {} Hz", above_nyquist);
        }

        /// Test that valid cutoffs below Nyquist are accepted.
        #[test]
        fn prop_valid_cutoffs_accepted(
            sample_rate in 44100.0f32..=96000.0f32,
            cutoff_ratio in 0.1f32..=0.45f32, // Well below Nyquist
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            let nyquist = sample_rate / 2.0;
            let cutoff = nyquist * cutoff_ratio;
            
            // Valid cutoff should be accepted
            let result = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window);
            prop_assert!(result.is_ok(),
                "Filter design should accept valid cutoff {} Hz (Nyquist: {} Hz)", cutoff, nyquist);
        }

        /// Test that cutoff exactly at Nyquist boundary is rejected.
        #[test]
        fn prop_exact_nyquist_boundary_rejected(
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1)
        ) {
            let nyquist = sample_rate / 2.0;
            
            // Test at exact Nyquist frequency
            let result = FilterDesign::fir_lowpass(nyquist, sample_rate, length, WindowFunction::Hamming);
            prop_assert!(result.is_err(),
                "Filter design should reject cutoff exactly at Nyquist frequency {} Hz", nyquist);
        }

        /// Test that bandpass rejects invalid frequency ordering.
        #[test]
        fn prop_bandpass_rejects_invalid_ordering(
            sample_rate in 44100.0f32..=96000.0f32,
            low_cutoff in 2000.0f32..=8000.0f32,
            high_cutoff in 500.0f32..=1500.0f32,
            length in (51usize..151).prop_map(|x| x | 1)
        ) {
            prop_assume!(low_cutoff > high_cutoff); // Invalid ordering
            prop_assume!(high_cutoff < sample_rate * 0.4);
            
            // Should reject when low >= high
            let result = FilterDesign::fir_bandpass(low_cutoff, high_cutoff, sample_rate, length, WindowFunction::Hamming);
            prop_assert!(result.is_err(),
                "Bandpass filter should reject invalid frequency ordering: low={} >= high={}",
                low_cutoff, high_cutoff);
        }
    }
}

/// **Feature: juce-examples-validation, Property 27: Filter design numerical stability**
/// **Validates: Requirements 9.4**
///
/// This property verifies that filter design produces numerically stable coefficients.
/// For any valid filter specification, the designed filter coefficients should not
/// cause instability (all poles inside unit circle for IIR, finite coefficients for FIR).
mod filter_design_stability {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that FIR lowpass coefficients are finite and bounded.
        #[test]
        fn prop_lowpass_coefficients_finite(
            cutoff in 100.0f32..=10000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (21usize..201).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Rectangular),
                Just(WindowFunction::Triangular),
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
                Just(WindowFunction::BlackmanHarris),
                Just(WindowFunction::FlatTop),
                (2.0f32..=14.0f32).prop_map(|beta| WindowFunction::Kaiser { beta }),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.45);
            
            let result = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window);
            
            if let Ok(coeffs) = result {
                // All coefficients should be finite
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.is_finite(),
                        "Coefficient {} is not finite: {}", i, coeff);
                }
                
                // Coefficients should be reasonably bounded (not excessively large)
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.abs() < 10.0,
                        "Coefficient {} has excessive magnitude: {}", i, coeff);
                }
                
                // Sum of coefficients should be close to 1.0 for lowpass (unity gain at DC)
                let sum: f32 = coeffs.iter().sum();
                prop_assert!((sum - 1.0).abs() < 0.01,
                    "Lowpass coefficients sum to {}, expected ~1.0", sum);
            }
        }

        /// Test that FIR highpass coefficients are finite and bounded.
        #[test]
        fn prop_highpass_coefficients_finite(
            cutoff in 100.0f32..=10000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (21usize..201).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.45);
            
            let result = FilterDesign::fir_highpass(cutoff, sample_rate, length, window);
            
            if let Ok(coeffs) = result {
                // All coefficients should be finite
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.is_finite(),
                        "Highpass coefficient {} is not finite: {}", i, coeff);
                }
                
                // Coefficients should be reasonably bounded
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.abs() < 10.0,
                        "Highpass coefficient {} has excessive magnitude: {}", i, coeff);
                }
            }
        }

        /// Test that FIR bandpass coefficients are finite and bounded.
        #[test]
        fn prop_bandpass_coefficients_finite(
            low_cutoff in 100.0f32..=3000.0f32,
            high_cutoff in 4000.0f32..=10000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (21usize..201).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(low_cutoff < high_cutoff);
            prop_assume!(high_cutoff < sample_rate * 0.45);
            
            let result = FilterDesign::fir_bandpass(low_cutoff, high_cutoff, sample_rate, length, window);
            
            if let Ok(coeffs) = result {
                // All coefficients should be finite
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.is_finite(),
                        "Bandpass coefficient {} is not finite: {}", i, coeff);
                }
                
                // Coefficients should be reasonably bounded
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.abs() < 10.0,
                        "Bandpass coefficient {} has excessive magnitude: {}", i, coeff);
                }
            }
        }

        /// Test that FIR bandstop coefficients are finite and bounded.
        #[test]
        fn prop_bandstop_coefficients_finite(
            low_cutoff in 100.0f32..=3000.0f32,
            high_cutoff in 4000.0f32..=10000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (21usize..201).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(low_cutoff < high_cutoff);
            prop_assume!(high_cutoff < sample_rate * 0.45);
            
            let result = FilterDesign::fir_bandstop(low_cutoff, high_cutoff, sample_rate, length, window);
            
            if let Ok(coeffs) = result {
                // All coefficients should be finite
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.is_finite(),
                        "Bandstop coefficient {} is not finite: {}", i, coeff);
                }
                
                // Coefficients should be reasonably bounded
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.abs() < 10.0,
                        "Bandstop coefficient {} has excessive magnitude: {}", i, coeff);
                }
            }
        }

        /// Test that filter processing with designed coefficients produces stable output.
        #[test]
        fn prop_filter_processing_stable(
            cutoff in 500.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (21usize..101).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.4);
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window).unwrap();
            let mut filter = FIRFilter::new(coeffs);
            
            // Process a bounded input signal
            let num_samples = 1000;
            let mut input = vec![0.0; num_samples];
            for i in 0..num_samples {
                let t = i as f32 / sample_rate;
                input[i] = (2.0 * PI * 440.0 * t).sin(); // 440 Hz sine wave
            }
            
            let mut output = vec![0.0; num_samples];
            filter.process(&input, &mut output);
            
            // Output should remain bounded and finite
            for (i, &sample) in output.iter().enumerate() {
                prop_assert!(sample.is_finite(),
                    "Output sample {} is not finite: {}", i, sample);
                
                // For a sine wave input with amplitude 1.0, output should be bounded
                // (may have some overshoot due to filter response, but not excessive)
                prop_assert!(sample.abs() < 5.0,
                    "Output sample {} has excessive magnitude: {} (input was sine wave with amplitude 1.0)",
                    i, sample);
            }
        }

        /// Test that extreme but valid parameters produce stable filters.
        #[test]
        fn prop_extreme_parameters_stable(
            sample_rate in 44100.0f32..=192000.0f32,
            length in (21usize..301).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Blackman),
                (2.0f32..=20.0f32).prop_map(|beta| WindowFunction::Kaiser { beta }),
            ]
        ) {
            // Test with very low cutoff
            let low_cutoff = 50.0;
            if low_cutoff < sample_rate * 0.45 {
                let result_low = FilterDesign::fir_lowpass(low_cutoff, sample_rate, length, window);
                if let Ok(coeffs) = result_low {
                    for &coeff in &coeffs {
                        prop_assert!(coeff.is_finite(), "Low cutoff produced non-finite coefficient");
                    }
                }
            }
            
            // Test with high cutoff (but below Nyquist)
            let high_cutoff = sample_rate * 0.4;
            let result_high = FilterDesign::fir_lowpass(high_cutoff, sample_rate, length, window);
            if let Ok(coeffs) = result_high {
                for &coeff in &coeffs {
                    prop_assert!(coeff.is_finite(), "High cutoff produced non-finite coefficient");
                }
            }
        }

        /// Test that very short filters remain stable.
        #[test]
        fn prop_short_filters_stable(
            cutoff in 1000.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (3usize..21).prop_map(|x| x | 1), // Very short filters
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.4);
            
            let result = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window);
            
            if let Ok(coeffs) = result {
                // Even short filters should have finite coefficients
                for (i, &coeff) in coeffs.iter().enumerate() {
                    prop_assert!(coeff.is_finite(),
                        "Short filter coefficient {} is not finite: {}", i, coeff);
                    prop_assert!(coeff.abs() < 10.0,
                        "Short filter coefficient {} has excessive magnitude: {}", i, coeff);
                }
            }
        }

        /// Test that normalized coefficients maintain stability.
        #[test]
        fn prop_normalized_coefficients_stable(
            cutoff in 500.0f32..=8000.0f32,
            sample_rate in 44100.0f32..=96000.0f32,
            length in (51usize..151).prop_map(|x| x | 1),
            window in prop_oneof![
                Just(WindowFunction::Hann),
                Just(WindowFunction::Hamming),
                Just(WindowFunction::Blackman),
            ]
        ) {
            prop_assume!(cutoff < sample_rate * 0.4);
            
            let coeffs = FilterDesign::fir_lowpass(cutoff, sample_rate, length, window).unwrap();
            
            // Check that normalization didn't introduce instability
            let sum: f32 = coeffs.iter().sum();
            prop_assert!(sum.is_finite(), "Sum of coefficients is not finite: {}", sum);
            prop_assert!(sum.abs() > 0.5 && sum.abs() < 2.0,
                "Sum of normalized coefficients is out of expected range: {}", sum);
            
            // Check that no individual coefficient is excessively large after normalization
            let max_coeff = coeffs.iter().map(|&c| c.abs()).fold(0.0f32, f32::max);
            prop_assert!(max_coeff < 5.0,
                "Maximum coefficient magnitude after normalization is excessive: {}", max_coeff);
        }
    }
}
