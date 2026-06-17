//! Property-based tests for gain processor.
//!
//! These tests use proptest to verify correctness properties across
//! a wide range of inputs.

use proptest::prelude::*;

/// **Feature: juce-examples-validation, Property 22: Decibel conversion accuracy**
/// **Validates: Requirements 12.2**
///
/// This property verifies that decibel to linear conversion and back preserves
/// the original value within acceptable tolerance. For any gain value in dB,
/// converting to linear gain and back should preserve the original value
/// (within 0.01 dB).
mod decibel_conversion_accuracy {
    use super::*;
    use nih_plug_dsp::processors::gain::{db_to_linear, linear_to_db};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that dB -> linear -> dB round-trip preserves the value.
        #[test]
        fn prop_db_to_linear_roundtrip(
            db in -60.0f32..=60.0f32
        ) {
            let linear = db_to_linear(db);
            let db_back = linear_to_db(linear);
            
            // Should preserve within 0.01 dB
            prop_assert!((db - db_back).abs() < 0.01,
                "Round-trip failed: {} dB -> {} linear -> {} dB (diff: {})",
                db, linear, db_back, (db - db_back).abs());
        }

        /// Test that linear -> dB -> linear round-trip preserves the value.
        #[test]
        fn prop_linear_to_db_roundtrip(
            linear in 0.001f32..=100.0f32
        ) {
            let db = linear_to_db(linear);
            let linear_back = db_to_linear(db);
            
            // Should preserve within 0.01% relative error
            let relative_error = ((linear - linear_back) / linear).abs();
            prop_assert!(relative_error < 0.0001,
                "Round-trip failed: {} linear -> {} dB -> {} linear (relative error: {})",
                linear, db, linear_back, relative_error);
        }

        /// Test that 0 dB equals unity gain (1.0 linear).
        #[test]
        fn prop_zero_db_is_unity_gain(
            _dummy in 0..1 // Just to make proptest happy
        ) {
            let linear = db_to_linear(0.0);
            prop_assert!((linear - 1.0).abs() < 1e-6,
                "0 dB should be 1.0 linear, got {}", linear);
        }

        /// Test that +6 dB is approximately 2x gain.
        #[test]
        fn prop_six_db_is_double_gain(
            _dummy in 0..1
        ) {
            let linear = db_to_linear(6.0);
            prop_assert!((linear - 2.0).abs() < 0.01,
                "+6 dB should be ~2.0 linear, got {}", linear);
        }

        /// Test that -6 dB is approximately 0.5x gain.
        #[test]
        fn prop_minus_six_db_is_half_gain(
            _dummy in 0..1
        ) {
            let linear = db_to_linear(-6.0);
            prop_assert!((linear - 0.5).abs() < 0.01,
                "-6 dB should be ~0.5 linear, got {}", linear);
        }

        /// Test that +20 dB is 10x gain.
        #[test]
        fn prop_twenty_db_is_ten_times_gain(
            _dummy in 0..1
        ) {
            let linear = db_to_linear(20.0);
            prop_assert!((linear - 10.0).abs() < 0.01,
                "+20 dB should be ~10.0 linear, got {}", linear);
        }

        /// Test that conversion is monotonic (higher dB = higher linear).
        #[test]
        fn prop_conversion_is_monotonic(
            db1 in -60.0f32..=60.0f32,
            db2 in -60.0f32..=60.0f32
        ) {
            prop_assume!(db1 != db2);
            
            let linear1 = db_to_linear(db1);
            let linear2 = db_to_linear(db2);
            
            if db1 < db2 {
                prop_assert!(linear1 < linear2,
                    "Conversion not monotonic: {} dB ({} linear) should be less than {} dB ({} linear)",
                    db1, linear1, db2, linear2);
            } else {
                prop_assert!(linear1 > linear2,
                    "Conversion not monotonic: {} dB ({} linear) should be greater than {} dB ({} linear)",
                    db1, linear1, db2, linear2);
            }
        }

        /// Test that negative dB values produce gain less than 1.0.
        #[test]
        fn prop_negative_db_attenuates(
            db in -60.0f32..=-0.01f32
        ) {
            let linear = db_to_linear(db);
            prop_assert!(linear < 1.0,
                "Negative dB {} should produce linear gain < 1.0, got {}", db, linear);
        }

        /// Test that positive dB values produce gain greater than 1.0.
        #[test]
        fn prop_positive_db_amplifies(
            db in 0.01f32..=60.0f32
        ) {
            let linear = db_to_linear(db);
            prop_assert!(linear > 1.0,
                "Positive dB {} should produce linear gain > 1.0, got {}", db, linear);
        }
    }
}

/// **Feature: juce-examples-validation, Property 23: Gain application**
/// **Validates: Requirements 12.3**
///
/// This property verifies that gain is correctly applied to audio signals.
/// For any input signal and linear gain g, the output should equal input * g
/// for all samples (after smoothing settles).
mod gain_application {
    use super::*;
    use nih_plug_dsp::processors::gain::Gain;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that gain is correctly applied to all samples.
        #[test]
        fn prop_gain_multiplies_samples(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..100),
            gain_linear in 0.1f32..=10.0f32
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain_linear);
            gain.set_smoothing_time(0.0, 44100.0); // No smoothing
            
            let mut output = vec![0.0; samples.len()];
            gain.process(&samples, &mut output);
            
            for (input, output) in samples.iter().zip(output.iter()) {
                let expected = input * gain_linear;
                prop_assert!((output - expected).abs() < 1e-5,
                    "Gain not applied correctly: input={}, expected={}, got={}",
                    input, expected, output);
            }
        }

        /// Test that unity gain (0 dB) passes signal unchanged.
        #[test]
        fn prop_unity_gain_passes_unchanged(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..100)
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_db(0.0);
            gain.set_smoothing_time(0.0, 44100.0); // No smoothing
            
            let mut output = vec![0.0; samples.len()];
            gain.process(&samples, &mut output);
            
            for (input, output) in samples.iter().zip(output.iter()) {
                prop_assert!((input - output).abs() < 1e-5,
                    "Unity gain should pass signal unchanged: input={}, output={}",
                    input, output);
            }
        }

        /// Test that gain preserves signal polarity.
        #[test]
        fn prop_gain_preserves_polarity(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..100),
            gain_linear in 0.1f32..=10.0f32
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain_linear);
            gain.set_smoothing_time(0.0, 44100.0); // No smoothing
            
            let mut output = vec![0.0; samples.len()];
            gain.process(&samples, &mut output);
            
            for (input, output) in samples.iter().zip(output.iter()) {
                if input.abs() > 1e-6 {
                    let same_sign = (*input > 0.0) == (*output > 0.0);
                    prop_assert!(same_sign,
                        "Gain should preserve polarity: input={}, output={}",
                        input, output);
                }
            }
        }

        /// Test that zero input produces zero output regardless of gain.
        #[test]
        fn prop_zero_input_produces_zero_output(
            length in 10usize..100,
            gain_linear in 0.1f32..=10.0f32
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain_linear);
            gain.set_smoothing_time(0.0, 44100.0); // No smoothing
            
            let samples = vec![0.0; length];
            let mut output = vec![0.0; length];
            gain.process(&samples, &mut output);
            
            for output_sample in output.iter() {
                prop_assert_eq!(*output_sample, 0.0,
                    "Zero input should produce zero output");
            }
        }

        /// Test that gain scales signal amplitude proportionally.
        #[test]
        fn prop_gain_scales_amplitude(
            amplitude in 0.1f32..=1.0f32,
            gain_linear in 0.5f32..=5.0f32,
            length in 10usize..50
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain_linear);
            gain.set_smoothing_time(0.0, 44100.0); // No smoothing
            
            // Create a simple sine-like pattern
            let samples: Vec<f32> = (0..length)
                .map(|i| amplitude * (i as f32 * 0.1).sin())
                .collect();
            
            let mut output = vec![0.0; length];
            gain.process(&samples, &mut output);
            
            // Find max amplitude in input and output
            let max_input = samples.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
            let max_output = output.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
            
            let expected_max = max_input * gain_linear;
            prop_assert!((max_output - expected_max).abs() < 0.01,
                "Output amplitude not scaled correctly: expected max={}, got max={}",
                expected_max, max_output);
        }

        /// Test that sample-by-sample processing matches buffer processing.
        #[test]
        fn prop_sample_by_sample_matches_buffer(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain_linear in 0.1f32..=10.0f32
        ) {
            // Buffer processing
            let mut gain1 = Gain::new();
            gain1.prepare(44100.0, 512);
            gain1.set_gain_linear(gain_linear);
            gain1.set_smoothing_time(0.0, 44100.0); // No smoothing
            
            let mut output_buffer = vec![0.0; samples.len()];
            gain1.process(&samples, &mut output_buffer);
            
            // Sample-by-sample processing
            let mut gain2 = Gain::new();
            gain2.prepare(44100.0, 512);
            gain2.set_gain_linear(gain_linear);
            gain2.set_smoothing_time(0.0, 44100.0); // No smoothing
            
            let output_sample: Vec<f32> = samples.iter()
                .map(|&s| gain2.process_sample(s))
                .collect();
            
            for (buffer, sample) in output_buffer.iter().zip(output_sample.iter()) {
                prop_assert!((buffer - sample).abs() < 1e-5,
                    "Buffer and sample-by-sample processing differ: buffer={}, sample={}",
                    buffer, sample);
            }
        }
    }
}

/// **Feature: juce-examples-validation, Property 24: Gain smoothing continuity**
/// **Validates: Requirements 12.4**
///
/// This property verifies that gain smoothing prevents discontinuities.
/// For any gain change, the output signal should not contain discontinuities
/// larger than the smoothing step size.
mod gain_smoothing_continuity {
    use super::*;
    use nih_plug_dsp::processors::gain::Gain;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that smoothing prevents large discontinuities.
        #[test]
        fn prop_smoothing_prevents_discontinuities(
            gain1 in 0.5f32..=2.0f32,
            gain2 in 0.5f32..=2.0f32,
            smoothing_time in 1.0f32..=50.0f32
        ) {
            prop_assume!((gain1 - gain2).abs() > 0.1); // Ensure meaningful change
            
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain1);
            gain.set_smoothing_time(smoothing_time, 44100.0);
            
            // Process enough samples to settle at first gain
            let settle_samples = ((smoothing_time / 1000.0) * 44100.0 * 5.0) as usize;
            let input = vec![1.0; settle_samples];
            let mut output = vec![0.0; settle_samples];
            gain.process(&input, &mut output);
            
            // Change gain
            gain.set_gain_linear(gain2);
            
            // Process more samples and check for discontinuities
            let test_samples = 200;
            let input2 = vec![1.0; test_samples];
            let mut output2 = vec![0.0; test_samples];
            gain.process(&input2, &mut output2);
            
            // Check that transitions are smooth (no large jumps)
            let max_jump = output2.windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0f32, f32::max);
            
            // Calculate expected maximum step size based on smoothing time
            // The maximum jump occurs on the first sample after the gain change
            let tau = smoothing_time / 1000.0;
            let smoothing_coeff = 1.0 - (-1.0 / (tau * 44100.0)).exp();
            // The jump is smoothing_coeff * (target - current) * input_amplitude
            // Since we're settled at gain1, current = gain1, target = gain2, input = 1.0
            let max_expected_jump = smoothing_coeff * (gain2 - gain1).abs() * 2.0; // Allow margin for numerical precision
            
            prop_assert!(max_jump <= max_expected_jump,
                "Discontinuity too large: max_jump={}, expected<={}, smoothing_time={}ms, coeff={}",
                max_jump, max_expected_jump, smoothing_time, smoothing_coeff);
        }

        /// Test that longer smoothing times produce smoother transitions.
        #[test]
        fn prop_longer_smoothing_is_smoother(
            gain1 in 0.5f32..=2.0f32,
            gain2 in 0.5f32..=2.0f32
        ) {
            prop_assume!((gain1 - gain2).abs() > 0.2); // Ensure meaningful change
            
            let short_time = 5.0;
            let long_time = 50.0;
            
            // Test with short smoothing - settle first, then change
            let mut gain_short = Gain::new();
            gain_short.prepare(44100.0, 512);
            gain_short.set_gain_linear(gain1);
            gain_short.set_smoothing_time(short_time, 44100.0);
            
            // Settle at gain1
            let settle_samples = ((short_time / 1000.0) * 44100.0 * 5.0) as usize;
            let input_settle = vec![1.0; settle_samples];
            let mut output_settle = vec![0.0; settle_samples];
            gain_short.process(&input_settle, &mut output_settle);
            
            // Change gain and measure jumps
            gain_short.set_gain_linear(gain2);
            let input = vec![1.0; 100];
            let mut output_short = vec![0.0; 100];
            gain_short.process(&input, &mut output_short);
            
            let max_jump_short = output_short.windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0f32, f32::max);
            
            // Test with long smoothing - settle first, then change
            let mut gain_long = Gain::new();
            gain_long.prepare(44100.0, 512);
            gain_long.set_gain_linear(gain1);
            gain_long.set_smoothing_time(long_time, 44100.0);
            
            // Settle at gain1
            let settle_samples_long = ((long_time / 1000.0) * 44100.0 * 5.0) as usize;
            let input_settle_long = vec![1.0; settle_samples_long];
            let mut output_settle_long = vec![0.0; settle_samples_long];
            gain_long.process(&input_settle_long, &mut output_settle_long);
            
            // Change gain and measure jumps
            gain_long.set_gain_linear(gain2);
            let mut output_long = vec![0.0; 100];
            gain_long.process(&input, &mut output_long);
            
            let max_jump_long = output_long.windows(2)
                .map(|w| (w[1] - w[0]).abs())
                .fold(0.0f32, f32::max);
            
            // Longer smoothing should produce smaller jumps (smaller smoothing coefficient)
            prop_assert!(max_jump_long < max_jump_short,
                "Longer smoothing should be smoother: short_jump={}, long_jump={}",
                max_jump_short, max_jump_long);
        }

        /// Test that zero smoothing time produces instant changes.
        #[test]
        fn prop_zero_smoothing_is_instant(
            gain1 in 0.5f32..=2.0f32,
            gain2 in 0.5f32..=2.0f32
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain1);
            gain.set_smoothing_time(0.0, 44100.0); // Instant
            
            // Process one sample
            let output1 = gain.process_sample(1.0);
            
            // Change gain
            gain.set_gain_linear(gain2);
            
            // Next sample should immediately reflect new gain
            let output2 = gain.process_sample(1.0);
            
            prop_assert!((output1 - gain1).abs() < 1e-5,
                "First sample should have gain1: expected={}, got={}", gain1, output1);
            prop_assert!((output2 - gain2).abs() < 1e-5,
                "Second sample should have gain2: expected={}, got={}", gain2, output2);
        }

        /// Test that smoothing eventually reaches target gain.
        #[test]
        fn prop_smoothing_reaches_target(
            gain1 in 0.5f32..=2.0f32,
            gain2 in 0.5f32..=2.0f32,
            smoothing_time in 5.0f32..=20.0f32
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain1);
            gain.set_smoothing_time(smoothing_time, 44100.0);
            
            // Process some samples to settle
            let input = vec![1.0; 50];
            let mut output = vec![0.0; 50];
            gain.process(&input, &mut output);
            
            // Change gain
            gain.set_gain_linear(gain2);
            
            // Process enough samples for smoothing to settle (5x time constant)
            let settle_samples = ((smoothing_time / 1000.0) * 44100.0 * 5.0) as usize;
            let input_long = vec![1.0; settle_samples];
            let mut output_long = vec![0.0; settle_samples];
            gain.process(&input_long, &mut output_long);
            
            // Final samples should be very close to target gain
            let final_output = output_long[output_long.len() - 1];
            prop_assert!((final_output - gain2).abs() < 0.01,
                "Smoothing should reach target: expected={}, got={}, diff={}",
                gain2, final_output, (final_output - gain2).abs());
        }

        /// Test that reset clears smoothing state.
        #[test]
        fn prop_reset_clears_smoothing(
            gain1 in 0.5f32..=2.0f32,
            gain2 in 0.5f32..=2.0f32,
            smoothing_time in 5.0f32..=20.0f32
        ) {
            let mut gain = Gain::new();
            gain.prepare(44100.0, 512);
            gain.set_gain_linear(gain1);
            gain.set_smoothing_time(smoothing_time, 44100.0);
            
            // Process some samples
            let input = vec![1.0; 50];
            let mut output = vec![0.0; 50];
            gain.process(&input, &mut output);
            
            // Change gain and reset
            gain.set_gain_linear(gain2);
            gain.reset();
            
            // Next sample should immediately have new gain (no smoothing)
            let output_sample = gain.process_sample(1.0);
            prop_assert!((output_sample - gain2).abs() < 1e-5,
                "After reset, gain should be immediate: expected={}, got={}",
                gain2, output_sample);
        }
    }
}
