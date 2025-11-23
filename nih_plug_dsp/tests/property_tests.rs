//! Property-based tests for DSP operations.
//!
//! These tests use proptest to verify correctness properties across
//! a wide range of inputs.

use proptest::prelude::*;
use proptest::prop_oneof;

/// **Feature: juce-modules-integration, Property 1: Audio buffer processing preserves data**
/// **Validates: Requirements 1.3**
///
/// This property verifies that basic buffer operations preserve data correctly.
/// For any audio buffer with arbitrary channel count and sample count, processing
/// through DSP utility functions should preserve all sample values within numerical
/// precision limits.
mod buffer_processing_preserves_data {
    use super::*;
    use nih_plug_dsp::util::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that copying a buffer preserves all sample values exactly.
        #[test]
        fn prop_copy_buffer_preserves_data(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 1..1024)
        ) {
            let mut dst = vec![0.0; samples.len()];
            copy_buffer(&samples, &mut dst);
            
            // All values should be preserved exactly
            for (original, copied) in samples.iter().zip(dst.iter()) {
                prop_assert_eq!(*original, *copied);
            }
        }

        /// Test that clearing a buffer sets all values to zero.
        #[test]
        fn prop_clear_buffer_zeros_all_samples(
            mut samples in prop::collection::vec(-1.0f32..=1.0f32, 1..1024)
        ) {
            clear_buffer(&mut samples);
            
            // All values should be zero
            for sample in samples.iter() {
                prop_assert_eq!(*sample, 0.0);
            }
        }

        /// Test that adding buffers produces correct results.
        #[test]
        fn prop_add_buffer_preserves_sum(
            samples in prop::collection::vec((-0.5f32..=0.5f32, -0.5f32..=0.5f32), 1..1024)
        ) {
            let (src, dst): (Vec<f32>, Vec<f32>) = samples.into_iter().unzip();
            
            let original_dst = dst.clone();
            let mut result = dst;
            add_buffer(&src, &mut result);
            
            // Each sample should be the sum of the original values
            for i in 0..src.len() {
                let expected = original_dst[i] + src[i];
                let actual = result[i];
                // Allow for floating point precision
                prop_assert!((expected - actual).abs() < 1e-6);
            }
        }

        /// Test that scaling a buffer by a factor preserves relative values.
        #[test]
        fn prop_scale_buffer_preserves_ratios(
            mut samples in prop::collection::vec(-1.0f32..=1.0f32, 2..1024),
            gain in -10.0f32..=10.0f32
        ) {
            // Skip if gain is too close to zero to avoid division issues
            prop_assume!(gain.abs() > 0.001);
            
            let original = samples.clone();
            scale_buffer(&mut samples, gain);
            
            // Check that ratios between samples are preserved
            for i in 1..samples.len() {
                if original[i].abs() > 0.001 && original[i-1].abs() > 0.001 {
                    let original_ratio = original[i] / original[i-1];
                    let scaled_ratio = samples[i] / samples[i-1];
                    // Ratios should be preserved
                    prop_assert!((original_ratio - scaled_ratio).abs() < 0.01);
                }
            }
        }

        /// Test that lerp produces values within the input range.
        #[test]
        fn prop_lerp_stays_in_range(
            a in -100.0f32..=100.0f32,
            b in -100.0f32..=100.0f32,
            t in 0.0f32..=1.0f32
        ) {
            let result = lerp(a, b, t);
            
            let min = a.min(b);
            let max = a.max(b);
            
            // Result should be within the range [min, max]
            prop_assert!(result >= min - 1e-6);
            prop_assert!(result <= max + 1e-6);
        }

        /// Test that clamp keeps values within bounds.
        #[test]
        fn prop_clamp_enforces_bounds(
            value in -1000.0f32..=1000.0f32,
            min in -100.0f32..=0.0f32,
            max in 0.0f32..=100.0f32
        ) {
            prop_assume!(min <= max);
            
            let result = clamp(value, min, max);
            
            prop_assert!(result >= min);
            prop_assert!(result <= max);
            
            // If value was in range, it should be unchanged
            if value >= min && value <= max {
                prop_assert_eq!(result, value);
            }
        }

        /// Test that frequency/phase conversions are inverses.
        #[test]
        fn prop_frequency_phase_roundtrip(
            frequency in 20.0f32..=20000.0f32,
            sample_rate in 44100.0f32..=192000.0f32
        ) {
            let phase_inc = frequency_to_phase_increment(frequency, sample_rate);
            let freq_back = phase_increment_to_frequency(phase_inc, sample_rate);
            
            // Should round-trip within floating point precision
            // Use relative error for better handling of floating-point precision
            let relative_error = (frequency - freq_back).abs() / frequency;
            prop_assert!(relative_error < 1e-6, 
                "Relative error {} too large for frequency {} Hz at sample rate {} Hz",
                relative_error, frequency, sample_rate);
        }

        /// Test that buffer operations work with multi-channel data.
        #[test]
        fn prop_multichannel_buffer_operations(
            channels in prop::collection::vec(
                prop::collection::vec(-1.0f32..=1.0f32, 1..512),
                1..8
            )
        ) {
            // Ensure all channels have the same length
            let len = channels[0].len();
            prop_assume!(channels.iter().all(|ch| ch.len() == len));
            
            // Test that operations work independently on each channel
            for channel in channels.iter() {
                let mut dst = vec![0.0; channel.len()];
                copy_buffer(channel, &mut dst);
                
                for (original, copied) in channel.iter().zip(dst.iter()) {
                    prop_assert_eq!(*original, *copied);
                }
            }
        }

        /// Test that sample rate validation works correctly.
        #[test]
        fn prop_sample_rate_validation(
            sample_rate in prop_oneof![
                (1.0f32..=1000000.0f32).prop_map(|x| x),
                Just(0.0f32),
                Just(-1.0f32),
                Just(f32::NAN),
                Just(f32::INFINITY),
                Just(f32::NEG_INFINITY),
            ]
        ) {
            let result = validate_sample_rate(sample_rate);
            
            if sample_rate > 0.0 && sample_rate.is_finite() {
                prop_assert!(result.is_ok());
            } else {
                prop_assert!(result.is_err());
            }
        }

        /// Test that buffer size validation works correctly.
        #[test]
        fn prop_buffer_size_validation(
            buffer_size in prop_oneof![
                (1usize..=65536).prop_map(|x| x),
                Just(0usize),
            ]
        ) {
            let result = validate_buffer_size(buffer_size);
            
            if buffer_size > 0 {
                prop_assert!(result.is_ok());
            } else {
                prop_assert!(result.is_err());
            }
        }
    }
}

/// **Feature: juce-modules-integration, Property 2: Filter state persistence across process calls**
/// **Validates: Requirements 2.3**
///
/// This property verifies that filter state is correctly maintained across multiple
/// process calls. For any filter instance and sequence of audio blocks, processing
/// multiple blocks should maintain correct internal state such that the output
/// depends on all previous inputs.
mod filter_state_persistence {
    use super::*;
    use nih_plug_dsp::filters::IIRFilter;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that processing in multiple blocks produces the same result as
        /// processing in a single block.
        #[test]
        fn prop_filter_state_persists_across_blocks(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..200),
            split_point in 1usize..100
        ) {
            // Ensure we have valid coefficients
            let a_coeffs: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            prop_assume!(split_point < samples.len());
            
            // Process in one block
            let mut filter1 = IIRFilter::new();
            filter1.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            let mut output_single = vec![0.0; samples.len()];
            filter1.process(&samples, &mut output_single);
            
            // Process in two blocks
            let mut filter2 = IIRFilter::new();
            filter2.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            let mut output_split = vec![0.0; samples.len()];
            
            // First block
            filter2.process(&samples[..split_point], &mut output_split[..split_point]);
            // Second block
            filter2.process(&samples[split_point..], &mut output_split[split_point..]);
            
            // Results should be identical
            for (single, split) in output_single.iter().zip(output_split.iter()) {
                prop_assert!((single - split).abs() < 1e-5,
                    "Outputs differ: single={}, split={}", single, split);
            }
        }

        /// Test that filter state correctly accumulates over multiple process calls.
        #[test]
        fn prop_filter_state_accumulates(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..4),
            blocks in prop::collection::vec(
                prop::collection::vec(-0.5f32..=0.5f32, 10..50),
                2..5
            )
        ) {
            let a_coeffs: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            let mut filter = IIRFilter::new();
            filter.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            
            // Process each block
            for block in blocks.iter() {
                let mut output = vec![0.0; block.len()];
                filter.process(block, &mut output);
                
                // After processing, state should be non-zero (unless input was all zeros)
                if block.iter().any(|&x| x.abs() > 0.01) {
                    // At least one state value should be non-zero
                    prop_assert!(filter.state().iter().any(|&x| x.abs() > 1e-10));
                }
            }
        }

        /// Test that sample-by-sample processing matches block processing.
        #[test]
        fn prop_sample_by_sample_matches_block(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..4),
            samples in prop::collection::vec(-0.5f32..=0.5f32, 10..100)
        ) {
            let a_coeffs: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            // Block processing
            let mut filter1 = IIRFilter::new();
            filter1.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            let mut output_block = vec![0.0; samples.len()];
            filter1.process(&samples, &mut output_block);
            
            // Sample-by-sample processing
            let mut filter2 = IIRFilter::new();
            filter2.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            let output_sample: Vec<f32> = samples.iter()
                .map(|&s| filter2.process_sample(s))
                .collect();
            
            // Results should be identical
            for (block, sample) in output_block.iter().zip(output_sample.iter()) {
                prop_assert!((block - sample).abs() < 1e-5,
                    "Outputs differ: block={}, sample={}", block, sample);
            }
        }
    }
}

/// **Feature: juce-modules-integration, Property 3: Reset restores initial state**
/// **Validates: Requirements 2.4**
///
/// This property verifies that resetting a filter restores it to its initial state.
/// For any stateful DSP object (filter, oscillator, envelope), resetting it should
/// produce the same state as a freshly constructed instance.
mod filter_reset_restores_initial_state {
    use super::*;
    use nih_plug_dsp::filters::IIRFilter;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that reset produces the same output as a fresh filter.
        #[test]
        fn prop_reset_produces_fresh_filter_output(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            input_before in prop::collection::vec(-1.0f32..=1.0f32, 10..100),
            input_after in prop::collection::vec(-1.0f32..=1.0f32, 10..100)
        ) {
            let a_coeffs: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            // Create two filters with same coefficients
            let mut filter1 = IIRFilter::new();
            filter1.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            
            let mut filter2 = IIRFilter::new();
            filter2.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            
            // Process some audio with filter2 then reset
            let mut temp = vec![0.0; input_before.len()];
            filter2.process(&input_before, &mut temp);
            filter2.reset();
            
            // Both filters should now produce identical output
            let mut output1 = vec![0.0; input_after.len()];
            let mut output2 = vec![0.0; input_after.len()];
            filter1.process(&input_after, &mut output1);
            filter2.process(&input_after, &mut output2);
            
            for (a, b) in output1.iter().zip(output2.iter()) {
                prop_assert!((a - b).abs() < 1e-6,
                    "Outputs differ after reset: filter1={}, filter2={}", a, b);
            }
        }

        /// Test that reset clears all internal state to zero.
        #[test]
        fn prop_reset_clears_state_to_zero(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            input in prop::collection::vec(-1.0f32..=1.0f32, 10..100)
        ) {
            let a_coeffs: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            let mut filter = IIRFilter::new();
            filter.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            
            // Process some audio to populate state
            let mut output = vec![0.0; input.len()];
            filter.process(&input, &mut output);
            
            // Reset should clear state
            filter.reset();
            
            // All state values should be zero
            for &state_val in filter.state().iter() {
                prop_assert_eq!(state_val, 0.0);
            }
        }

        /// Test that reset_to sets state to the specified value.
        #[test]
        fn prop_reset_to_sets_specific_value(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            input in prop::collection::vec(-1.0f32..=1.0f32, 10..100),
            reset_value in -1.0f32..=1.0f32
        ) {
            let a_coeffs: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            let mut filter = IIRFilter::new();
            filter.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            
            // Process some audio to populate state
            let mut output = vec![0.0; input.len()];
            filter.process(&input, &mut output);
            
            // Reset to specific value
            filter.reset_to(reset_value);
            
            // All state values should be the reset value
            for &state_val in filter.state().iter() {
                prop_assert_eq!(state_val, reset_value);
            }
        }

        /// Test that multiple resets produce consistent results.
        #[test]
        fn prop_multiple_resets_are_consistent(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            input in prop::collection::vec(-1.0f32..=1.0f32, 10..100)
        ) {
            let a_coeffs: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            let mut filter = IIRFilter::new();
            filter.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            
            // Process and reset multiple times
            let mut outputs = Vec::new();
            for _ in 0..3 {
                let mut output = vec![0.0; input.len()];
                filter.process(&input, &mut output);
                outputs.push(output);
                filter.reset();
            }
            
            // All outputs should be identical
            for i in 1..outputs.len() {
                for (a, b) in outputs[0].iter().zip(outputs[i].iter()) {
                    prop_assert!((a - b).abs() < 1e-6,
                        "Outputs differ after multiple resets");
                }
            }
        }
    }
}

/// **Feature: juce-modules-integration, Property 13: Filter coefficient validation**
/// **Validates: Requirements 2.2**
///
/// This property verifies that filter coefficient validation works correctly.
/// For any filter, setting invalid coefficients should return an error rather
/// than producing undefined behavior.
mod filter_coefficient_validation {
    use super::*;
    use nih_plug_dsp::filters::IIRFilter;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that valid coefficients are accepted.
        #[test]
        fn prop_valid_coefficients_accepted(
            b_coeffs in prop::collection::vec(-10.0f32..=10.0f32, 2..8),
            a0 in 0.1f32..=10.0f32
        ) {
            let a_coeffs: Vec<f32> = vec![a0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            let mut filter = IIRFilter::new();
            let result = filter.set_coefficients(&b_coeffs, &a_coeffs);
            
            prop_assert!(result.is_ok());
            prop_assert_eq!(filter.order(), a_coeffs.len() - 1);
        }

        /// Test that empty coefficient arrays are rejected.
        #[test]
        fn prop_empty_coefficients_rejected(
            empty_b in prop::bool::ANY,
            empty_a in prop::bool::ANY
        ) {
            prop_assume!(empty_b || empty_a);
            
            let mut filter = IIRFilter::new();
            
            let b_coeffs = if empty_b { vec![] } else { vec![1.0, 0.5] };
            let a_coeffs = if empty_a { vec![] } else { vec![1.0, -0.5] };
            
            let result = filter.set_coefficients(&b_coeffs, &a_coeffs);
            prop_assert!(result.is_err());
        }

        /// Test that mismatched coefficient lengths are rejected.
        #[test]
        fn prop_mismatched_lengths_rejected(
            b_len in 2usize..8,
            a_len in 2usize..8
        ) {
            prop_assume!(b_len != a_len);
            
            let b_coeffs = vec![1.0; b_len];
            let a_coeffs = vec![1.0; a_len];
            
            let mut filter = IIRFilter::new();
            let result = filter.set_coefficients(&b_coeffs, &a_coeffs);
            
            prop_assert!(result.is_err());
        }

        /// Test that zero or near-zero a0 is rejected.
        #[test]
        fn prop_zero_a0_rejected(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            a0 in prop_oneof![
                Just(0.0f32),
                (-1e-11f32..=1e-11f32).prop_map(|x| x),
            ]
        ) {
            let a_coeffs: Vec<f32> = vec![a0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            let mut filter = IIRFilter::new();
            let result = filter.set_coefficients(&b_coeffs, &a_coeffs);
            
            prop_assert!(result.is_err());
        }

        /// Test that coefficient normalization works correctly.
        #[test]
        fn prop_coefficient_normalization(
            b_coeffs in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            a0 in 0.1f32..=10.0f32
        ) {
            prop_assume!(a0.abs() > 0.01);
            
            let a_coeffs: Vec<f32> = vec![a0].into_iter()
                .chain(b_coeffs[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            // Create two filters with different a0 values but proportional coefficients
            let mut filter1 = IIRFilter::new();
            filter1.set_coefficients(&b_coeffs, &a_coeffs).unwrap();
            
            let scale = 2.0;
            let b_coeffs_scaled: Vec<f32> = b_coeffs.iter().map(|&x| x * scale).collect();
            let a_coeffs_scaled: Vec<f32> = a_coeffs.iter().map(|&x| x * scale).collect();
            
            let mut filter2 = IIRFilter::new();
            filter2.set_coefficients(&b_coeffs_scaled, &a_coeffs_scaled).unwrap();
            
            // Both filters should produce identical output (coefficients are normalized)
            let input = vec![1.0, 0.5, 0.25, 0.0, -0.25, -0.5];
            let mut output1 = vec![0.0; input.len()];
            let mut output2 = vec![0.0; input.len()];
            
            filter1.process(&input, &mut output1);
            filter2.process(&input, &mut output2);
            
            for (a, b) in output1.iter().zip(output2.iter()) {
                prop_assert!((a - b).abs() < 1e-5,
                    "Normalized filters produce different outputs: {} vs {}", a, b);
            }
        }

        /// Test that changing coefficients updates filter order correctly.
        #[test]
        fn prop_changing_coefficients_updates_order(
            b_coeffs1 in prop::collection::vec(-1.0f32..=1.0f32, 2..5),
            b_coeffs2 in prop::collection::vec(-1.0f32..=1.0f32, 3..7)
        ) {
            let a_coeffs1: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs1[1..].iter().map(|&x| x * 0.5))
                .collect();
            let a_coeffs2: Vec<f32> = vec![1.0].into_iter()
                .chain(b_coeffs2[1..].iter().map(|&x| x * 0.5))
                .collect();
            
            let mut filter = IIRFilter::new();
            
            filter.set_coefficients(&b_coeffs1, &a_coeffs1).unwrap();
            let order1 = filter.order();
            prop_assert_eq!(order1, a_coeffs1.len() - 1);
            
            filter.set_coefficients(&b_coeffs2, &a_coeffs2).unwrap();
            let order2 = filter.order();
            prop_assert_eq!(order2, a_coeffs2.len() - 1);
        }
    }
}

/// **Feature: juce-modules-integration, Property 12: Oscillator phase continuity**
/// **Validates: Requirements 3.4**
///
/// This property verifies that oscillator phase continuity is maintained when
/// changing frequency. For any oscillator, changing frequency should maintain
/// phase continuity without clicks or discontinuities.
mod oscillator_phase_continuity {
    use super::*;
    use nih_plug_dsp::oscillators::{Oscillator, Waveform};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that frequency changes don't cause discontinuities in the output.
        #[test]
        fn prop_frequency_change_maintains_continuity(
            sample_rate in 44100.0f32..=96000.0f32,
            freq1 in 100.0f32..=1000.0f32,
            freq2 in 100.0f32..=1000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ]
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(freq1);
            osc.set_waveform(waveform);
            
            // Generate some samples at first frequency
            let mut output = vec![0.0; 200];
            for i in 0..100 {
                output[i] = osc.process_sample();
            }
            
            // Change frequency mid-buffer
            osc.set_frequency(freq2);
            
            // Generate more samples at second frequency
            for i in 100..200 {
                output[i] = osc.process_sample();
            }
            
            // Check for discontinuities (large jumps between adjacent samples)
            // For sine and triangle waves, the maximum rate of change is bounded
            // For square waves, we expect discontinuities at transitions
            // For saw waves, we expect discontinuities at the reset point
            
            let max_expected_jump = match waveform {
                Waveform::Sine | Waveform::Triangle => {
                    // Maximum slope for sine/triangle at these frequencies
                    let max_freq = freq1.max(freq2);
                    let max_slope = 2.0 * std::f32::consts::PI * max_freq / sample_rate;
                    max_slope * 3.0 // Allow some margin
                }
                Waveform::Square | Waveform::Saw => {
                    // These waveforms have intentional discontinuities
                    2.0 // Full range jump is expected
                }
            };
            
            // Count discontinuities
            let mut discontinuity_count = 0;
            for i in 1..output.len() {
                let diff = (output[i] - output[i-1]).abs();
                if diff > max_expected_jump {
                    discontinuity_count += 1;
                }
            }
            
            // For sine and triangle, we should have very few discontinuities
            // (only at the frequency change point, if any)
            if waveform == Waveform::Sine || waveform == Waveform::Triangle {
                prop_assert!(discontinuity_count <= 2,
                    "Too many discontinuities for {:?}: {}", waveform, discontinuity_count);
            }
        }

        /// Test that phase is continuous when processing sample-by-sample.
        #[test]
        fn prop_phase_advances_smoothly(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=1000.0f32,
            num_samples in 10usize..200
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(frequency);
            osc.set_waveform(Waveform::Sine);
            
            let mut phases = Vec::new();
            phases.push(osc.phase());
            
            // Generate samples and record phase after each
            for _ in 0..num_samples {
                osc.process_sample();
                phases.push(osc.phase());
            }
            
            // Phase should always advance (or wrap around)
            for i in 1..phases.len() {
                let phase_diff = phases[i] - phases[i-1];
                
                // Phase should either advance or wrap (negative diff means wrap)
                if phase_diff < 0.0 {
                    // Wrapped around - check that it's close to wrapping
                    prop_assert!(phases[i-1] > 0.9,
                        "Phase wrapped but previous phase was not near 1.0: {}", phases[i-1]);
                    prop_assert!(phases[i] < 0.1,
                        "Phase wrapped but new phase is not near 0.0: {}", phases[i]);
                } else {
                    // Normal advance - should be positive and reasonable
                    let expected_increment = frequency / sample_rate;
                    prop_assert!((phase_diff - expected_increment).abs() < 0.001,
                        "Phase increment {} doesn't match expected {}", phase_diff, expected_increment);
                }
            }
        }

        /// Test that changing frequency per-sample doesn't cause phase jumps.
        #[test]
        fn prop_per_sample_frequency_modulation_is_continuous(
            sample_rate in 44100.0f32..=96000.0f32,
            base_freq in 200.0f32..=500.0f32,
            mod_depth in 10.0f32..=100.0f32,
            num_samples in 50usize..150
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_waveform(Waveform::Sine);
            
            let mut output = Vec::new();
            
            // Modulate frequency per sample
            for i in 0..num_samples {
                let t = i as f32 / num_samples as f32;
                let freq = base_freq + mod_depth * (t * 2.0 * std::f32::consts::PI).sin();
                osc.set_frequency(freq);
                output.push(osc.process_sample());
            }
            
            // Check that output is continuous (no large jumps)
            let max_expected_jump = 2.0 * std::f32::consts::PI * (base_freq + mod_depth) / sample_rate * 3.0;
            
            for i in 1..output.len() {
                let diff = (output[i] - output[i-1]).abs();
                prop_assert!(diff < max_expected_jump,
                    "Discontinuity detected at sample {}: diff={}, max={}", i, diff, max_expected_jump);
            }
        }

        /// Test that phase wrapping doesn't cause discontinuities.
        #[test]
        fn prop_phase_wrapping_is_continuous(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 1000.0f32..=5000.0f32, // High frequency to ensure wrapping
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Triangle),
            ]
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(frequency);
            osc.set_waveform(waveform);
            
            // Generate enough samples to wrap phase multiple times
            let samples_per_cycle = sample_rate / frequency;
            let num_samples = (samples_per_cycle * 3.0) as usize;
            
            let mut output = vec![0.0; num_samples];
            for i in 0..num_samples {
                output[i] = osc.process_sample();
            }
            
            // Check for discontinuities at phase wrapping points
            let max_expected_jump = 2.0 * std::f32::consts::PI * frequency / sample_rate * 3.0;
            
            let mut large_jumps = 0;
            for i in 1..output.len() {
                let diff = (output[i] - output[i-1]).abs();
                if diff > max_expected_jump {
                    large_jumps += 1;
                }
            }
            
            // Should have very few large jumps (only at actual discontinuities, not phase wraps)
            prop_assert!(large_jumps <= 2,
                "Too many discontinuities for {:?}: {}", waveform, large_jumps);
        }

        /// Test that all waveforms produce output in the expected range.
        #[test]
        fn prop_waveforms_stay_in_range(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=2000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ]
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(frequency);
            osc.set_waveform(waveform);
            
            let mut output = vec![0.0; 1000];
            osc.process(&mut output);
            
            // All samples should be in range [-1, 1]
            for (i, &sample) in output.iter().enumerate() {
                prop_assert!(sample >= -1.0 && sample <= 1.0,
                    "Sample {} out of range for {:?}: {}", i, waveform, sample);
            }
        }

        /// Test that block processing matches sample-by-sample processing.
        #[test]
        fn prop_block_matches_sample_by_sample(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=1000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ],
            num_samples in 10usize..200
        ) {
            // Block processing
            let mut osc1 = Oscillator::new(sample_rate);
            osc1.set_frequency(frequency);
            osc1.set_waveform(waveform);
            let mut output_block = vec![0.0; num_samples];
            osc1.process(&mut output_block);
            
            // Sample-by-sample processing
            let mut osc2 = Oscillator::new(sample_rate);
            osc2.set_frequency(frequency);
            osc2.set_waveform(waveform);
            let output_sample: Vec<f32> = (0..num_samples)
                .map(|_| osc2.process_sample())
                .collect();
            
            // Results should be identical
            for (i, (block, sample)) in output_block.iter().zip(output_sample.iter()).enumerate() {
                prop_assert!((block - sample).abs() < 1e-6,
                    "Outputs differ at sample {}: block={}, sample={}", i, block, sample);
            }
        }
    }
}

/// **Feature: juce-modules-integration, Property 3: Reset restores initial state (Oscillators)**
/// **Validates: Requirements 3.5**
///
/// This property verifies that resetting an oscillator restores it to its initial state.
/// For any oscillator, resetting it should produce the same state as a freshly
/// constructed instance.
mod oscillator_reset_restores_initial_state {
    use super::*;
    use nih_plug_dsp::oscillators::{Oscillator, Waveform};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that reset produces the same output as a fresh oscillator.
        #[test]
        fn prop_reset_produces_fresh_oscillator_output(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=2000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ],
            samples_before in 10usize..200,
            samples_after in 10usize..200
        ) {
            // Create two oscillators with same settings
            let mut osc1 = Oscillator::new(sample_rate);
            osc1.set_frequency(frequency);
            osc1.set_waveform(waveform);
            
            let mut osc2 = Oscillator::new(sample_rate);
            osc2.set_frequency(frequency);
            osc2.set_waveform(waveform);
            
            // Process some audio with osc2 then reset
            let mut temp = vec![0.0; samples_before];
            osc2.process(&mut temp);
            osc2.reset();
            
            // Both oscillators should now produce identical output
            let mut output1 = vec![0.0; samples_after];
            let mut output2 = vec![0.0; samples_after];
            osc1.process(&mut output1);
            osc2.process(&mut output2);
            
            for (i, (a, b)) in output1.iter().zip(output2.iter()).enumerate() {
                prop_assert!((a - b).abs() < 1e-6,
                    "Outputs differ after reset at sample {}: osc1={}, osc2={}", i, a, b);
            }
        }

        /// Test that reset sets phase to zero.
        #[test]
        fn prop_reset_sets_phase_to_zero(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=2000.0f32,
            num_samples in 10usize..200
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(frequency);
            
            // Process some samples to advance phase
            let mut output = vec![0.0; num_samples];
            osc.process(&mut output);
            
            // Phase should have advanced
            prop_assert!(osc.phase() > 0.0,
                "Phase should have advanced after processing");
            
            // Reset should set phase to zero
            osc.reset();
            prop_assert_eq!(osc.phase(), 0.0,
                "Phase should be zero after reset");
        }

        /// Test that reset doesn't affect frequency or waveform settings.
        #[test]
        fn prop_reset_preserves_settings(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=2000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ],
            num_samples in 10usize..200
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(frequency);
            osc.set_waveform(waveform);
            
            // Process some samples
            let mut output = vec![0.0; num_samples];
            osc.process(&mut output);
            
            // Reset
            osc.reset();
            
            // Settings should be preserved
            prop_assert_eq!(osc.frequency(), frequency,
                "Frequency should be preserved after reset");
            prop_assert_eq!(osc.waveform(), waveform,
                "Waveform should be preserved after reset");
        }

        /// Test that multiple resets produce consistent results.
        #[test]
        fn prop_multiple_resets_are_consistent(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=2000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ],
            num_samples in 10usize..100
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(frequency);
            osc.set_waveform(waveform);
            
            // Process and reset multiple times
            let mut outputs = Vec::new();
            for _ in 0..3 {
                let mut output = vec![0.0; num_samples];
                osc.process(&mut output);
                outputs.push(output);
                osc.reset();
            }
            
            // All outputs should be identical
            for i in 1..outputs.len() {
                for (j, (a, b)) in outputs[0].iter().zip(outputs[i].iter()).enumerate() {
                    prop_assert!((a - b).abs() < 1e-6,
                        "Outputs differ after multiple resets at sample {}: {} vs {}", j, a, b);
                }
            }
        }

        /// Test that reset works correctly after frequency changes.
        #[test]
        fn prop_reset_after_frequency_change(
            sample_rate in 44100.0f32..=96000.0f32,
            freq1 in 100.0f32..=1000.0f32,
            freq2 in 100.0f32..=1000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ],
            num_samples in 10usize..100
        ) {
            // Create reference oscillator
            let mut osc_ref = Oscillator::new(sample_rate);
            osc_ref.set_frequency(freq2);
            osc_ref.set_waveform(waveform);
            
            // Create test oscillator with different initial frequency
            let mut osc_test = Oscillator::new(sample_rate);
            osc_test.set_frequency(freq1);
            osc_test.set_waveform(waveform);
            
            // Process some samples
            let mut temp = vec![0.0; num_samples];
            osc_test.process(&mut temp);
            
            // Change frequency and reset
            osc_test.set_frequency(freq2);
            osc_test.reset();
            
            // Both oscillators should now produce identical output
            let mut output_ref = vec![0.0; num_samples];
            let mut output_test = vec![0.0; num_samples];
            osc_ref.process(&mut output_ref);
            osc_test.process(&mut output_test);
            
            for (i, (a, b)) in output_ref.iter().zip(output_test.iter()).enumerate() {
                prop_assert!((a - b).abs() < 1e-6,
                    "Outputs differ after frequency change and reset at sample {}: {} vs {}", i, a, b);
            }
        }

        /// Test that reset works correctly after waveform changes.
        #[test]
        fn prop_reset_after_waveform_change(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=2000.0f32,
            waveform1 in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
            ],
            waveform2 in prop_oneof![
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ],
            num_samples in 10usize..100
        ) {
            // Create reference oscillator
            let mut osc_ref = Oscillator::new(sample_rate);
            osc_ref.set_frequency(frequency);
            osc_ref.set_waveform(waveform2);
            
            // Create test oscillator with different initial waveform
            let mut osc_test = Oscillator::new(sample_rate);
            osc_test.set_frequency(frequency);
            osc_test.set_waveform(waveform1);
            
            // Process some samples
            let mut temp = vec![0.0; num_samples];
            osc_test.process(&mut temp);
            
            // Change waveform and reset
            osc_test.set_waveform(waveform2);
            osc_test.reset();
            
            // Both oscillators should now produce identical output
            let mut output_ref = vec![0.0; num_samples];
            let mut output_test = vec![0.0; num_samples];
            osc_ref.process(&mut output_ref);
            osc_test.process(&mut output_test);
            
            for (i, (a, b)) in output_ref.iter().zip(output_test.iter()).enumerate() {
                prop_assert!((a - b).abs() < 1e-6,
                    "Outputs differ after waveform change and reset at sample {}: {} vs {}", i, a, b);
            }
        }

        /// Test that reset followed by immediate processing produces expected first sample.
        #[test]
        fn prop_reset_produces_expected_first_sample(
            sample_rate in 44100.0f32..=96000.0f32,
            frequency in 100.0f32..=2000.0f32,
            waveform in prop_oneof![
                Just(Waveform::Sine),
                Just(Waveform::Saw),
                Just(Waveform::Square),
                Just(Waveform::Triangle),
            ]
        ) {
            let mut osc = Oscillator::new(sample_rate);
            osc.set_frequency(frequency);
            osc.set_waveform(waveform);
            
            // Process some samples
            let mut temp = vec![0.0; 50];
            osc.process(&mut temp);
            
            // Reset and get first sample
            osc.reset();
            let first_sample = osc.process_sample();
            
            // First sample should match the waveform at phase 0
            let expected = match waveform {
                Waveform::Sine => 0.0, // sin(0) = 0
                Waveform::Saw => -1.0, // Saw starts at -1
                Waveform::Square => -1.0, // Square starts at -1
                Waveform::Triangle => -1.0, // Triangle starts at -1
            };
            
            prop_assert!((first_sample - expected).abs() < 1e-6,
                "First sample after reset doesn't match expected for {:?}: got {}, expected {}",
                waveform, first_sample, expected);
        }
    }
}
