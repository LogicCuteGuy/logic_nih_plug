//! Property-based tests for SIMD optimizations.
//!
//! **Feature: juce-examples-validation, Property 17: SIMD equivalence**
//!
//! These tests verify that SIMD-optimized implementations produce identical
//! results to scalar implementations (within floating-point precision).

use proptest::prelude::*;

#[cfg(feature = "simd")]
use nih_plug_dsp::simd::optimizations::{
    SimdStateVariableFilter, SimdFIRFilter, interleave_channels, deinterleave_channels,
};
#[cfg(feature = "simd")]
use nih_plug_dsp::state_variable::{StateVariableFilter, FilterType};
#[cfg(feature = "simd")]
use nih_plug_dsp::fir::{FIRFilter, FilterDesign, WindowFunction};

/// Floating-point comparison tolerance for SIMD vs scalar equivalence.
/// SIMD operations may have slightly different rounding behavior.
const EPSILON: f32 = 1e-5;

/// Checks if two floating-point values are approximately equal.
fn approx_equal(a: f32, b: f32, epsilon: f32) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    if a.is_infinite() && b.is_infinite() && a.signum() == b.signum() {
        return true;
    }
    (a - b).abs() <= epsilon
}

#[cfg(feature = "simd")]
proptest! {
    /// **Property 17: SIMD equivalence**
    /// **Validates: Requirements 7.2, 7.3**
    ///
    /// For any input signal, processing with SIMD-optimized code should produce
    /// identical results to scalar code (within floating-point precision).
    ///
    /// This property tests state variable filters with random parameters and inputs.
    #[test]
    fn prop_simd_state_variable_filter_equivalence(
        cutoff_hz in 100.0f32..10000.0f32,
        resonance in 0.0f32..1.0f32,
        filter_type_idx in 0usize..3,
        input_samples in prop::collection::vec(-1.0f32..1.0f32, 100..1000),
    ) {
        let sample_rate = 44100.0;
        
        // Map index to filter type
        let filter_type = match filter_type_idx {
            0 => FilterType::Lowpass,
            1 => FilterType::Bandpass,
            _ => FilterType::Highpass,
        };
        
        // Create scalar filter
        let mut scalar_filter = StateVariableFilter::new();
        scalar_filter.prepare(sample_rate).unwrap();
        scalar_filter.set_type(filter_type);
        scalar_filter.set_cutoff(cutoff_hz);
        scalar_filter.set_resonance(resonance);
        
        // Create SIMD filter with same parameters
        let mut simd_filter = SimdStateVariableFilter::new();
        simd_filter.prepare(sample_rate).unwrap();
        simd_filter.set_type(filter_type);
        simd_filter.set_cutoff(cutoff_hz);
        simd_filter.set_resonance(resonance);
        
        // Process with both filters
        let mut scalar_output = vec![0.0; input_samples.len()];
        let mut simd_output = vec![0.0; input_samples.len()];
        
        scalar_filter.process(&input_samples, &mut scalar_output);
        simd_filter.process(&input_samples, &mut simd_output);
        
        // Verify outputs are approximately equal
        for (i, (&scalar, &simd)) in scalar_output.iter().zip(simd_output.iter()).enumerate() {
            prop_assert!(
                approx_equal(scalar, simd, EPSILON),
                "Output mismatch at sample {}: scalar={}, simd={}, diff={}",
                i, scalar, simd, (scalar - simd).abs()
            );
        }
    }
    
    /// **Property 17: SIMD equivalence**
    /// **Validates: Requirements 7.2, 7.3**
    ///
    /// For any FIR filter coefficients and input signal, SIMD processing should
    /// produce identical results to scalar processing.
    #[test]
    fn prop_simd_fir_filter_equivalence(
        cutoff_hz in 500.0f32..5000.0f32,
        filter_length in prop::sample::select(vec![16, 32, 64, 128]),
        input_samples in prop::collection::vec(-1.0f32..1.0f32, 100..1000),
    ) {
        let sample_rate = 44100.0;
        
        // Design filter coefficients
        let coefficients = FilterDesign::fir_lowpass(
            cutoff_hz,
            sample_rate,
            filter_length,
            WindowFunction::Hamming,
        ).unwrap();
        
        // Create scalar filter
        let mut scalar_filter = FIRFilter::new(coefficients.clone());
        
        // Create SIMD filter
        let mut simd_filter = SimdFIRFilter::new(coefficients);
        
        // Process with both filters
        let mut scalar_output = vec![0.0; input_samples.len()];
        let mut simd_output = vec![0.0; input_samples.len()];
        
        scalar_filter.process(&input_samples, &mut scalar_output);
        simd_filter.process(&input_samples, &mut simd_output);
        
        // Verify outputs are approximately equal
        for (i, (&scalar, &simd)) in scalar_output.iter().zip(simd_output.iter()).enumerate() {
            prop_assert!(
                approx_equal(scalar, simd, EPSILON),
                "Output mismatch at sample {}: scalar={}, simd={}, diff={}",
                i, scalar, simd, (scalar - simd).abs()
            );
        }
    }
    
    /// **Property 17: SIMD equivalence**
    /// **Validates: Requirements 7.2, 7.3**
    ///
    /// Channel interleaving and deinterleaving should be lossless round-trip operations.
    #[test]
    fn prop_channel_interleaving_round_trip(
        num_channels in 1usize..8,
        num_samples in 10usize..1000,
    ) {
        // Generate random channel data
        let mut channels: Vec<Vec<f32>> = Vec::new();
        for _ in 0..num_channels {
            let channel: Vec<f32> = (0..num_samples)
                .map(|i| (i as f32 * 0.1).sin())
                .collect();
            channels.push(channel);
        }
        
        // Create references for interleaving
        let channel_refs: Vec<&[f32]> = channels.iter().map(|c| c.as_slice()).collect();
        
        // Interleave
        let mut interleaved = vec![0.0; num_channels * num_samples];
        interleave_channels(&channel_refs, &mut interleaved);
        
        // Deinterleave
        let mut deinterleaved: Vec<Vec<f32>> = vec![vec![0.0; num_samples]; num_channels];
        let mut deinterleaved_refs: Vec<&mut [f32]> = deinterleaved
            .iter_mut()
            .map(|c| c.as_mut_slice())
            .collect();
        deinterleave_channels(&interleaved, &mut deinterleaved_refs);
        
        // Verify round-trip preserves data
        for (original, recovered) in channels.iter().zip(deinterleaved.iter()) {
            for (i, (&orig, &recov)) in original.iter().zip(recovered.iter()).enumerate() {
                prop_assert!(
                    approx_equal(orig, recov, 1e-10),
                    "Round-trip mismatch at sample {}: original={}, recovered={}",
                    i, orig, recov
                );
            }
        }
    }
    
    /// **Property 17: SIMD equivalence**
    /// **Validates: Requirements 7.2, 7.3**
    ///
    /// SIMD filters should maintain state consistency across multiple process calls,
    /// producing the same results as scalar filters.
    #[test]
    fn prop_simd_state_consistency(
        cutoff_hz in 500.0f32..5000.0f32,
        resonance in 0.0f32..0.9f32,
        chunk_sizes in prop::collection::vec(10usize..100, 5..10),
    ) {
        let sample_rate = 44100.0;
        
        // Create filters
        let mut scalar_filter = StateVariableFilter::new();
        scalar_filter.prepare(sample_rate).unwrap();
        scalar_filter.set_type(FilterType::Lowpass);
        scalar_filter.set_cutoff(cutoff_hz);
        scalar_filter.set_resonance(resonance);
        
        let mut simd_filter = SimdStateVariableFilter::new();
        simd_filter.prepare(sample_rate).unwrap();
        simd_filter.set_type(FilterType::Lowpass);
        simd_filter.set_cutoff(cutoff_hz);
        simd_filter.set_resonance(resonance);
        
        // Process in chunks
        for chunk_size in chunk_sizes {
            let input: Vec<f32> = (0..chunk_size).map(|i| (i as f32 * 0.01).sin()).collect();
            let mut scalar_output = vec![0.0; chunk_size];
            let mut simd_output = vec![0.0; chunk_size];
            
            scalar_filter.process(&input, &mut scalar_output);
            simd_filter.process(&input, &mut simd_output);
            
            // Verify outputs match
            for (i, (&scalar, &simd)) in scalar_output.iter().zip(simd_output.iter()).enumerate() {
                prop_assert!(
                    approx_equal(scalar, simd, EPSILON),
                    "State consistency mismatch at sample {}: scalar={}, simd={}",
                    i, scalar, simd
                );
            }
        }
    }
    
    /// **Property 17: SIMD equivalence**
    /// **Validates: Requirements 7.2, 7.3**
    ///
    /// SIMD filters should handle edge cases (zero, very small, very large values)
    /// identically to scalar filters.
    #[test]
    fn prop_simd_edge_cases(
        cutoff_hz in 100.0f32..10000.0f32,
        resonance in 0.0f32..1.0f32,
    ) {
        let sample_rate = 44100.0;
        
        // Test with various edge case inputs
        let edge_cases = vec![
            vec![0.0; 100],                           // All zeros
            vec![1e-10; 100],                         // Very small values
            vec![0.999; 100],                         // Near maximum
            vec![-0.999; 100],                        // Near minimum
            (0..100).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect(), // Alternating
        ];
        
        for input in edge_cases {
            // Create filters
            let mut scalar_filter = StateVariableFilter::new();
            scalar_filter.prepare(sample_rate).unwrap();
            scalar_filter.set_type(FilterType::Lowpass);
            scalar_filter.set_cutoff(cutoff_hz);
            scalar_filter.set_resonance(resonance);
            
            let mut simd_filter = SimdStateVariableFilter::new();
            simd_filter.prepare(sample_rate).unwrap();
            simd_filter.set_type(FilterType::Lowpass);
            simd_filter.set_cutoff(cutoff_hz);
            simd_filter.set_resonance(resonance);
            
            // Process
            let mut scalar_output = vec![0.0; input.len()];
            let mut simd_output = vec![0.0; input.len()];
            
            scalar_filter.process(&input, &mut scalar_output);
            simd_filter.process(&input, &mut simd_output);
            
            // Verify outputs match
            for (i, (&scalar, &simd)) in scalar_output.iter().zip(simd_output.iter()).enumerate() {
                prop_assert!(
                    approx_equal(scalar, simd, EPSILON),
                    "Edge case mismatch at sample {}: scalar={}, simd={}",
                    i, scalar, simd
                );
            }
        }
    }
}

#[cfg(not(feature = "simd"))]
#[test]
fn simd_feature_not_enabled() {
    // This test exists to ensure the test file compiles even without the simd feature
    println!("SIMD feature not enabled, skipping SIMD property tests");
}
