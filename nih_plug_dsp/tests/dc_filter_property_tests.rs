//! Property-based tests for DC filter.
//!
//! These tests use proptest to verify correctness properties across
//! a wide range of inputs.

use proptest::prelude::*;

/// **Feature: juce-examples-validation, Property 25: DC removal**
/// **Validates: Requirements 11.2**
///
/// This property verifies that the DC filter removes DC offset from audio signals.
/// For any input signal with DC offset, processing through DC filter should reduce
/// DC component to near zero while preserving AC components above 20Hz.
mod dc_removal {
    use super::*;
    use nih_plug_dsp::processors::dc_filter::DCFilter;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that DC filter removes constant DC offset.
        #[test]
        fn prop_removes_dc_offset(
            dc_offset in -1.0f32..=1.0f32,
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0, 96000.0])
        ) {
            let mut dc_filter = DCFilter::new();
            dc_filter.prepare(sample_rate, 512);
            
            // Create a signal with DC offset - need enough samples for filter to settle
            // At 5 Hz cutoff, we need about 200ms to settle (5 time constants)
            let num_samples = (sample_rate * 0.3) as usize; // 300ms
            let input = vec![dc_offset; num_samples];
            let mut output = vec![0.0; num_samples];
            dc_filter.process(&input, &mut output);
            
            // Check the last 20% of samples after the filter has settled
            let start_idx = (num_samples as f32 * 0.8) as usize;
            let avg_output: f32 = output[start_idx..].iter().sum::<f32>() 
                / (num_samples - start_idx) as f32;
            
            // DC should be reduced to near zero (within 5% of original offset)
            let dc_reduction = avg_output.abs() / dc_offset.abs().max(0.01);
            prop_assert!(dc_reduction < 0.05,
                "DC offset not removed: input={}, output_avg={}, reduction={}",
                dc_offset, avg_output, dc_reduction);
        }

        /// Test that DC filter preserves AC signals well above cutoff.
        #[test]
        fn prop_preserves_ac_signal(
            frequency in 100.0f32..=1000.0f32,
            amplitude in 0.1f32..=1.0f32,
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0, 96000.0])
        ) {
            use std::f32::consts::PI;
            
            let mut dc_filter = DCFilter::new();
            dc_filter.prepare(sample_rate, 512);
            
            // Create a sine wave at the given frequency
            let num_samples = (sample_rate * 0.1) as usize; // 100ms
            let input: Vec<f32> = (0..num_samples)
                .map(|i| amplitude * (2.0 * PI * frequency * i as f32 / sample_rate).sin())
                .collect();
            
            let mut output = vec![0.0; num_samples];
            dc_filter.process(&input, &mut output);
            
            // After settling (skip first 50%), check that amplitude is preserved
            let start_idx = num_samples / 2;
            let input_rms: f32 = input[start_idx..].iter()
                .map(|x| x * x)
                .sum::<f32>() / (num_samples - start_idx) as f32;
            let output_rms: f32 = output[start_idx..].iter()
                .map(|x| x * x)
                .sum::<f32>() / (num_samples - start_idx) as f32;
            
            let input_rms = input_rms.sqrt();
            let output_rms = output_rms.sqrt();
            
            // RMS should be preserved within 10% for signals well above cutoff
            let ratio = output_rms / input_rms;
            prop_assert!(ratio > 0.9 && ratio < 1.1,
                "AC signal not preserved: freq={} Hz, input_rms={}, output_rms={}, ratio={}",
                frequency, input_rms, output_rms, ratio);
        }

        /// Test that DC filter removes DC from AC+DC signals.
        #[test]
        fn prop_removes_dc_from_ac_plus_dc(
            frequency in 100.0f32..=1000.0f32,
            amplitude in 0.1f32..=0.5f32,
            dc_offset in -0.5f32..=0.5f32,
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0])
        ) {
            use std::f32::consts::PI;
            
            let mut dc_filter = DCFilter::new();
            dc_filter.prepare(sample_rate, 512);
            
            // Create a sine wave with DC offset
            let num_samples = (sample_rate * 0.3) as usize; // 300ms
            let input: Vec<f32> = (0..num_samples)
                .map(|i| {
                    amplitude * (2.0 * PI * frequency * i as f32 / sample_rate).sin() + dc_offset
                })
                .collect();
            
            let mut output = vec![0.0; num_samples];
            dc_filter.process(&input, &mut output);
            
            // After settling, check that DC is removed but AC is preserved
            let start_idx = (num_samples as f32 * 0.8) as usize;
            
            // Check DC removal
            let avg_output: f32 = output[start_idx..].iter().sum::<f32>() 
                / (num_samples - start_idx) as f32;
            prop_assert!(avg_output.abs() < 0.1,
                "DC not removed from AC+DC signal: dc_offset={}, avg_output={}",
                dc_offset, avg_output);
            
            // Check AC preservation (RMS should be similar to input AC component)
            let output_rms: f32 = output[start_idx..].iter()
                .map(|x| x * x)
                .sum::<f32>() / (num_samples - start_idx) as f32;
            let output_rms = output_rms.sqrt();
            
            // Expected RMS is approximately the amplitude / sqrt(2) for sine wave
            let expected_rms = amplitude / 2.0_f32.sqrt();
            let ratio = output_rms / expected_rms;
            prop_assert!(ratio > 0.8 && ratio < 1.2,
                "AC component not preserved: expected_rms={}, output_rms={}, ratio={}",
                expected_rms, output_rms, ratio);
        }

        /// Test that DC filter output remains finite for all inputs.
        #[test]
        fn prop_output_is_finite(
            samples in prop::collection::vec(-10.0f32..=10.0f32, 100..500),
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0, 96000.0])
        ) {
            let mut dc_filter = DCFilter::new();
            dc_filter.prepare(sample_rate, 512);
            
            let mut output = vec![0.0; samples.len()];
            dc_filter.process(&samples, &mut output);
            
            for (i, &out) in output.iter().enumerate() {
                prop_assert!(out.is_finite(),
                    "Output not finite at index {}: input={}, output={}",
                    i, samples[i], out);
            }
        }

        /// Test that DC filter handles zero input correctly.
        #[test]
        fn prop_zero_input_produces_zero_output(
            length in 100usize..500,
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0])
        ) {
            let mut dc_filter = DCFilter::new();
            dc_filter.prepare(sample_rate, 512);
            
            let input = vec![0.0; length];
            let mut output = vec![0.0; length];
            dc_filter.process(&input, &mut output);
            
            // After settling, output should be zero
            let start_idx = length / 2;
            for (i, &out) in output[start_idx..].iter().enumerate() {
                prop_assert!(out.abs() < 1e-6,
                    "Zero input should produce zero output at index {}: output={}",
                    start_idx + i, out);
            }
        }

        /// Test that reset clears filter state.
        #[test]
        fn prop_reset_clears_state(
            dc_offset in -1.0f32..=1.0f32,
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0])
        ) {
            let mut dc_filter = DCFilter::new();
            dc_filter.prepare(sample_rate, 512);
            
            // Process some samples to populate state
            let input = vec![dc_offset; 100];
            let mut output = vec![0.0; 100];
            dc_filter.process(&input, &mut output);
            
            // Reset and process again
            dc_filter.reset();
            let mut output2 = vec![0.0; 100];
            dc_filter.process(&input, &mut output2);
            
            // First few samples should match (state was cleared)
            for i in 0..10 {
                prop_assert!((output[i] - output2[i]).abs() < 1e-5,
                    "Reset should clear state: sample {}, first_run={}, second_run={}",
                    i, output[i], output2[i]);
            }
        }

        /// Test that sample-by-sample processing matches buffer processing.
        #[test]
        fn prop_sample_by_sample_matches_buffer(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 50..100),
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0])
        ) {
            // Buffer processing
            let mut dc_filter1 = DCFilter::new();
            dc_filter1.prepare(sample_rate, 512);
            
            let mut output_buffer = vec![0.0; samples.len()];
            dc_filter1.process(&samples, &mut output_buffer);
            
            // Sample-by-sample processing
            let mut dc_filter2 = DCFilter::new();
            dc_filter2.prepare(sample_rate, 512);
            
            let output_sample: Vec<f32> = samples.iter()
                .map(|&s| dc_filter2.process_sample(s))
                .collect();
            
            for (i, (buffer, sample)) in output_buffer.iter().zip(output_sample.iter()).enumerate() {
                prop_assert!((buffer - sample).abs() < 1e-5,
                    "Buffer and sample-by-sample processing differ at index {}: buffer={}, sample={}",
                    i, buffer, sample);
            }
        }

        /// Test that DC filter is stable (no oscillation or runaway).
        #[test]
        fn prop_filter_is_stable(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 500..1000),
            sample_rate in prop::sample::select(&[44100.0f32, 48000.0, 96000.0])
        ) {
            let mut dc_filter = DCFilter::new();
            dc_filter.prepare(sample_rate, 512);
            
            let mut output = vec![0.0; samples.len()];
            dc_filter.process(&samples, &mut output);
            
            // Check that output doesn't exceed reasonable bounds
            // For input in [-1, 1], output should stay within reasonable bounds
            for (i, &out) in output.iter().enumerate() {
                prop_assert!(out.abs() <= 10.0,
                    "Filter unstable at index {}: input={}, output={}",
                    i, samples[i], out);
            }
        }
    }
}

/// **Feature: juce-examples-validation, Property 26: DC filter sample rate adaptation**
/// **Validates: Requirements 11.3, 11.5**
///
/// This property verifies that the DC filter adapts to sample rate changes.
/// For any two sample rates, the DC filter cutoff frequency in Hz should remain
/// constant when sample rate changes.
mod sample_rate_adaptation {
    use super::*;
    use nih_plug_dsp::processors::dc_filter::DCFilter;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that DC filter adapts to different sample rates.
        #[test]
        fn prop_adapts_to_sample_rate(
            dc_offset in -1.0f32..=1.0f32,
            sample_rate1 in prop::sample::select(&[44100.0f32, 48000.0]),
            sample_rate2 in prop::sample::select(&[88200.0f32, 96000.0])
        ) {
            // Test at first sample rate
            let mut dc_filter1 = DCFilter::new();
            dc_filter1.prepare(sample_rate1, 512);
            
            let num_samples1 = (sample_rate1 * 0.3) as usize;
            let input1 = vec![dc_offset; num_samples1];
            let mut output1 = vec![0.0; num_samples1];
            dc_filter1.process(&input1, &mut output1);
            
            // Check DC removal at first sample rate
            let start_idx1 = (num_samples1 as f32 * 0.8) as usize;
            let avg1: f32 = output1[start_idx1..].iter().sum::<f32>() 
                / (num_samples1 - start_idx1) as f32;
            
            // Test at second sample rate
            let mut dc_filter2 = DCFilter::new();
            dc_filter2.prepare(sample_rate2, 512);
            
            let num_samples2 = (sample_rate2 * 0.3) as usize;
            let input2 = vec![dc_offset; num_samples2];
            let mut output2 = vec![0.0; num_samples2];
            dc_filter2.process(&input2, &mut output2);
            
            // Check DC removal at second sample rate
            let start_idx2 = (num_samples2 as f32 * 0.8) as usize;
            let avg2: f32 = output2[start_idx2..].iter().sum::<f32>() 
                / (num_samples2 - start_idx2) as f32;
            
            // Both should remove DC to similar levels (within 10%)
            prop_assert!(avg1.abs() < 0.05,
                "DC not removed at {} Hz: avg={}", sample_rate1, avg1);
            prop_assert!(avg2.abs() < 0.05,
                "DC not removed at {} Hz: avg={}", sample_rate2, avg2);
        }

        /// Test that cutoff frequency remains constant across sample rates.
        #[test]
        fn prop_cutoff_frequency_constant(
            frequency in 50.0f32..=200.0f32,
            amplitude in 0.1f32..=1.0f32,
            sample_rate1 in prop::sample::select(&[44100.0f32, 48000.0]),
            sample_rate2 in prop::sample::select(&[88200.0f32, 96000.0])
        ) {
            use std::f32::consts::PI;
            
            // Test at first sample rate
            let mut dc_filter1 = DCFilter::new();
            dc_filter1.prepare(sample_rate1, 512);
            
            let num_samples1 = (sample_rate1 * 0.1) as usize;
            let input1: Vec<f32> = (0..num_samples1)
                .map(|i| amplitude * (2.0 * PI * frequency * i as f32 / sample_rate1).sin())
                .collect();
            
            let mut output1 = vec![0.0; num_samples1];
            dc_filter1.process(&input1, &mut output1);
            
            // Measure attenuation at first sample rate
            let start_idx1 = num_samples1 / 2;
            let input_rms1: f32 = input1[start_idx1..].iter()
                .map(|x| x * x)
                .sum::<f32>() / (num_samples1 - start_idx1) as f32;
            let output_rms1: f32 = output1[start_idx1..].iter()
                .map(|x| x * x)
                .sum::<f32>() / (num_samples1 - start_idx1) as f32;
            let attenuation1 = output_rms1.sqrt() / input_rms1.sqrt();
            
            // Test at second sample rate
            let mut dc_filter2 = DCFilter::new();
            dc_filter2.prepare(sample_rate2, 512);
            
            let num_samples2 = (sample_rate2 * 0.1) as usize;
            let input2: Vec<f32> = (0..num_samples2)
                .map(|i| amplitude * (2.0 * PI * frequency * i as f32 / sample_rate2).sin())
                .collect();
            
            let mut output2 = vec![0.0; num_samples2];
            dc_filter2.process(&input2, &mut output2);
            
            // Measure attenuation at second sample rate
            let start_idx2 = num_samples2 / 2;
            let input_rms2: f32 = input2[start_idx2..].iter()
                .map(|x| x * x)
                .sum::<f32>() / (num_samples2 - start_idx2) as f32;
            let output_rms2: f32 = output2[start_idx2..].iter()
                .map(|x| x * x)
                .sum::<f32>() / (num_samples2 - start_idx2) as f32;
            let attenuation2 = output_rms2.sqrt() / input_rms2.sqrt();
            
            // Attenuation should be similar at both sample rates (within 15%)
            let ratio = attenuation1 / attenuation2;
            prop_assert!(ratio > 0.85 && ratio < 1.15,
                "Cutoff frequency not constant: freq={} Hz, sr1={} (att={}), sr2={} (att={}), ratio={}",
                frequency, sample_rate1, attenuation1, sample_rate2, attenuation2, ratio);
        }

        /// Test that changing sample rate updates filter coefficients.
        #[test]
        fn prop_sample_rate_change_updates_coefficients(
            dc_offset in -1.0f32..=1.0f32,
            sample_rate1 in prop::sample::select(&[44100.0f32, 48000.0]),
            sample_rate2 in prop::sample::select(&[88200.0f32, 96000.0])
        ) {
            let mut dc_filter = DCFilter::new();
            
            // Prepare at first sample rate and process
            dc_filter.prepare(sample_rate1, 512);
            let num_samples1 = (sample_rate1 * 0.3) as usize;
            let input1 = vec![dc_offset; num_samples1];
            let mut output1 = vec![0.0; num_samples1];
            dc_filter.process(&input1, &mut output1);
            
            // Change sample rate and reset
            dc_filter.reset();
            dc_filter.prepare(sample_rate2, 512);
            
            // Process at new sample rate
            let num_samples2 = (sample_rate2 * 0.3) as usize;
            let input2 = vec![dc_offset; num_samples2];
            let mut output2 = vec![0.0; num_samples2];
            dc_filter.process(&input2, &mut output2);
            
            // Both should remove DC effectively
            let start_idx1 = (num_samples1 as f32 * 0.8) as usize;
            let avg1: f32 = output1[start_idx1..].iter().sum::<f32>() 
                / (num_samples1 - start_idx1) as f32;
            
            let start_idx2 = (num_samples2 as f32 * 0.8) as usize;
            let avg2: f32 = output2[start_idx2..].iter().sum::<f32>() 
                / (num_samples2 - start_idx2) as f32;
            
            prop_assert!(avg1.abs() < 0.05,
                "DC not removed after first prepare: avg={}", avg1);
            prop_assert!(avg2.abs() < 0.05,
                "DC not removed after sample rate change: avg={}", avg2);
        }

        /// Test that custom cutoff frequency is preserved across sample rate changes.
        #[test]
        fn prop_custom_cutoff_preserved(
            cutoff_hz in 3.0f32..=15.0f32,
            sample_rate1 in prop::sample::select(&[44100.0f32, 48000.0]),
            sample_rate2 in prop::sample::select(&[88200.0f32, 96000.0])
        ) {
            let mut dc_filter = DCFilter::with_cutoff(cutoff_hz);
            
            // Prepare at first sample rate
            dc_filter.prepare(sample_rate1, 512);
            prop_assert!((dc_filter.cutoff() - cutoff_hz).abs() < 1e-5,
                "Cutoff not preserved at first sample rate: expected={}, got={}",
                cutoff_hz, dc_filter.cutoff());
            
            // Change sample rate
            dc_filter.prepare(sample_rate2, 512);
            prop_assert!((dc_filter.cutoff() - cutoff_hz).abs() < 1e-5,
                "Cutoff not preserved after sample rate change: expected={}, got={}",
                cutoff_hz, dc_filter.cutoff());
        }
    }
}
