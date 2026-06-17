//! Property-based tests for FFT implementation.
//!
//! These tests verify correctness properties across many randomly generated inputs.

use nih_plug_dsp::analysis::FFT;
use num_complex::Complex;
use proptest::prelude::*;

/// **Feature: juce-examples-validation, Property 14: FFT round-trip**
/// **Validates: Requirements 6.2, 6.3**
///
/// For any input signal, performing forward FFT followed by inverse FFT
/// should reconstruct the original signal (within numerical precision tolerance of 1e-5).
#[test]
fn property_fft_round_trip() {
    proptest!(|(
        size_power in 1u32..=16,  // FFT sizes from 2^1 to 2^16 (2 to 65536)
        seed in any::<u64>(),
    )| {
        let size = 2usize.pow(size_power);
        
        // Skip sizes outside valid range
        if size < FFT::MIN_SIZE || size > FFT::MAX_SIZE {
            return Ok(());
        }

        let fft = FFT::new(size).unwrap();

        // Generate random input signal using seed for reproducibility
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};
        
        let mut hasher = RandomState::new().build_hasher();
        seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut input = Vec::with_capacity(size);
        let mut rng_state = hash;
        for _ in 0..size {
            // Simple LCG for deterministic random numbers
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = ((rng_state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            input.push(value);
        }

        // Forward FFT
        let mut freq_domain = vec![Complex::new(0.0, 0.0); size];
        fft.forward(&input, &mut freq_domain);

        // Inverse FFT
        let mut reconstructed = vec![0.0; size];
        fft.inverse(&freq_domain, &mut reconstructed);

        // Verify round-trip within tolerance
        for (i, (&original, &reconstructed_val)) in input.iter().zip(reconstructed.iter()).enumerate() {
            let error = (original - reconstructed_val).abs();
            prop_assert!(
                error < 1e-5,
                "Round-trip error at index {}: original={}, reconstructed={}, error={}",
                i, original, reconstructed_val, error
            );
        }
    });
}

/// Test FFT round-trip with specific common sizes
#[test]
fn test_fft_round_trip_common_sizes() {
    let common_sizes = [64, 128, 256, 512, 1024, 2048, 4096, 8192];

    for &size in &common_sizes {
        let fft = FFT::new(size).unwrap();

        // Create a test signal with multiple frequency components
        let mut input = vec![0.0; size];
        for i in 0..size {
            let t = i as f32 / size as f32;
            // Mix of frequencies
            input[i] = (2.0 * std::f32::consts::PI * 5.0 * t).sin()
                + 0.5 * (2.0 * std::f32::consts::PI * 10.0 * t).sin()
                + 0.25 * (2.0 * std::f32::consts::PI * 20.0 * t).sin();
        }

        // Forward FFT
        let mut freq_domain = vec![Complex::new(0.0, 0.0); size];
        fft.forward(&input, &mut freq_domain);

        // Inverse FFT
        let mut reconstructed = vec![0.0; size];
        fft.inverse(&freq_domain, &mut reconstructed);

        // Verify round-trip
        for (i, (&original, &reconstructed_val)) in
            input.iter().zip(reconstructed.iter()).enumerate()
        {
            let error = (original - reconstructed_val).abs();
            assert!(
                error < 1e-5,
                "Round-trip error at index {} for size {}: original={}, reconstructed={}, error={}",
                i, size, original, reconstructed_val, error
            );
        }
    }
}

/// **Feature: juce-examples-validation, Property 15: FFT magnitude spectrum**
/// **Validates: Requirements 6.4**
///
/// For any input signal, the frequency-only transform should produce
/// non-negative magnitude values.
#[test]
fn property_fft_magnitude_spectrum() {
    proptest!(|(
        size_power in 1u32..=16,  // FFT sizes from 2^1 to 2^16 (2 to 65536)
        seed in any::<u64>(),
    )| {
        let size = 2usize.pow(size_power);
        
        // Skip sizes outside valid range
        if size < FFT::MIN_SIZE || size > FFT::MAX_SIZE {
            return Ok(());
        }

        let fft = FFT::new(size).unwrap();

        // Generate random input signal using seed for reproducibility
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};
        
        let mut hasher = RandomState::new().build_hasher();
        seed.hash(&mut hasher);
        let hash = hasher.finish();
        
        let mut input = Vec::with_capacity(size);
        let mut rng_state = hash;
        for _ in 0..size {
            // Simple LCG for deterministic random numbers
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = ((rng_state >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            input.push(value);
        }

        // Get magnitude spectrum
        let mut magnitudes = vec![0.0; size];
        fft.forward_magnitude(&input, &mut magnitudes);

        // Verify all magnitudes are non-negative
        for (i, &mag) in magnitudes.iter().enumerate() {
            prop_assert!(
                mag >= 0.0,
                "Magnitude at index {} is negative: {}",
                i, mag
            );
            
            // Also verify it's not NaN or infinite
            prop_assert!(
                (mag as f32).is_finite(),
                "Magnitude at index {} is not finite: {}",
                i, mag
            );
        }
    });
}

/// Test magnitude spectrum with specific signals
#[test]
fn test_fft_magnitude_spectrum_specific() {
    let size = 1024;
    let fft = FFT::new(size).unwrap();

    // Test 1: DC signal (all ones)
    let dc_signal = vec![1.0; size];
    let mut magnitudes = vec![0.0; size];
    fft.forward_magnitude(&dc_signal, &mut magnitudes);

    // All magnitudes should be non-negative
    for &mag in &magnitudes {
        assert!(mag >= 0.0, "Magnitude should be non-negative");
        assert!((mag as f32).is_finite(), "Magnitude should be finite");
    }

    // DC component should be large
    assert!(magnitudes[0] > 100.0, "DC component should be large");

    // Test 2: Sine wave
    let mut sine_signal = vec![0.0; size];
    for i in 0..size {
        let t = i as f32 / size as f32;
        sine_signal[i] = (2.0 * std::f32::consts::PI * 10.0 * t).sin();
    }

    fft.forward_magnitude(&sine_signal, &mut magnitudes);

    // All magnitudes should be non-negative
    for &mag in &magnitudes {
        assert!(mag >= 0.0, "Magnitude should be non-negative");
        assert!((mag as f32).is_finite(), "Magnitude should be finite");
    }

    // Test 3: Zero signal
    let zero_signal = vec![0.0; size];
    fft.forward_magnitude(&zero_signal, &mut magnitudes);

    // All magnitudes should be zero or very close to zero
    for &mag in &magnitudes {
        assert!(mag >= 0.0, "Magnitude should be non-negative");
        assert!(mag < 1e-6, "Magnitude of zero signal should be near zero");
    }
}

/// **Feature: juce-examples-validation, Property 16: FFT power-of-2 sizes**
/// **Validates: Requirements 6.1**
///
/// For any power-of-2 size from 2 to 65536, FFT creation should succeed.
#[test]
fn property_fft_size_validation() {
    proptest!(|(
        size_power in 1u32..=16,  // FFT sizes from 2^1 to 2^16 (2 to 65536)
    )| {
        let size = 2usize.pow(size_power);
        
        // All power-of-2 sizes in valid range should succeed
        if size >= FFT::MIN_SIZE && size <= FFT::MAX_SIZE {
            let result = FFT::new(size);
            prop_assert!(
                result.is_ok(),
                "FFT creation should succeed for power-of-2 size {} in valid range",
                size
            );
            
            if let Ok(fft) = result {
                prop_assert_eq!(fft.size(), size, "FFT size should match requested size");
            }
        }
    });
}

/// Test that non-power-of-2 sizes are rejected
#[test]
fn property_fft_rejects_non_power_of_2() {
    proptest!(|(
        size in 3usize..=65536,  // Range of potential sizes
    )| {
        // Skip power-of-2 sizes
        if size.is_power_of_two() {
            return Ok(());
        }
        
        // Non-power-of-2 sizes should fail
        let result = FFT::new(size);
        prop_assert!(
            result.is_err(),
            "FFT creation should fail for non-power-of-2 size {}",
            size
        );
    });
}

/// Test specific power-of-2 sizes
#[test]
fn test_fft_power_of_2_sizes() {
    // Test all power-of-2 sizes from 2 to 65536
    for power in 1..=16 {
        let size = 2usize.pow(power);
        let result = FFT::new(size);
        assert!(
            result.is_ok(),
            "FFT creation should succeed for power-of-2 size {}",
            size
        );

        if let Ok(fft) = result {
            assert_eq!(fft.size(), size, "FFT size should match requested size");
        }
    }
}

/// Test that sizes outside valid range are rejected
#[test]
fn test_fft_rejects_out_of_range() {
    // Too small
    assert!(FFT::new(1).is_err(), "Size 1 should be rejected");

    // Too large
    assert!(
        FFT::new(131072).is_err(),
        "Size 131072 should be rejected"
    );
    assert!(
        FFT::new(262144).is_err(),
        "Size 262144 should be rejected"
    );
}

/// Test that common non-power-of-2 sizes are rejected
#[test]
fn test_fft_rejects_common_non_power_of_2() {
    let non_power_of_2_sizes = [3, 5, 7, 10, 100, 1000, 1023, 1025, 2047, 2049];

    for &size in &non_power_of_2_sizes {
        let result = FFT::new(size);
        assert!(
            result.is_err(),
            "FFT creation should fail for non-power-of-2 size {}",
            size
        );
    }
}
