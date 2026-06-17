//! Property-based tests for wave shaper operations.
//!
//! These tests verify correctness properties for wave shapers.

use proptest::prelude::*;
use logic_nih_plug_dsp::processors::waveshaper::{WaveShaper, transfer_functions};

/// **Feature: juce-examples-validation, Property 8: Transfer function application**
/// **Validates: Requirements 3.2**
///
/// This property verifies that wave shaper correctly applies transfer functions.
/// For any input sample x and transfer function f, the wave shaper output should
/// equal f(x).
#[cfg(feature = "processors")]
mod transfer_function_application {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        #[test]
        fn prop_waveshaper_applies_identity(
            input in -10.0f32..=10.0f32
        ) {
            let identity = WaveShaper::new(|x| x);
            let output = identity.process_sample(input);
            prop_assert!((output - input).abs() < 1e-6);
        }

        #[test]
        fn prop_tanh_correct(
            input in -5.0f32..=5.0f32
        ) {
            let shaper = WaveShaper::new(transfer_functions::tanh);
            let output = shaper.process_sample(input);
            let expected = input.tanh();
            prop_assert!((output - expected).abs() < 1e-6);
        }

        #[test]
        fn prop_buffer_matches_sample(
            samples in prop::collection::vec(-5.0f32..=5.0f32, 10..200)
        ) {
            let shaper = WaveShaper::new(transfer_functions::tanh);
            let mut output_buffer = vec![0.0; samples.len()];
            shaper.process(&samples, &mut output_buffer);
            
            let output_samples: Vec<f32> = samples.iter()
                .map(|&s| shaper.process_sample(s))
                .collect();
            
            for (i, (buffer, sample)) in output_buffer.iter().zip(output_samples.iter()).enumerate() {
                prop_assert!((buffer - sample).abs() < 1e-6,
                    "Differ at {}: buffer={}, sample={}", i, buffer, sample);
            }
        }
    }
}
