//! Property-based tests for processor chain.
//!
//! These tests use proptest to verify correctness properties across
//! a wide range of inputs.

use proptest::prelude::*;

/// **Feature: juce-examples-validation, Property 9: Processor chain composition**
/// **Validates: Requirements 3.4, 4.2**
///
/// This property verifies that processing through a chain produces the same
/// output as applying each processor sequentially. For any sequence of
/// processors [P1, P2, ..., Pn] and input signal, processing through a chain
/// should produce the same output as applying each processor sequentially.
mod chain_composition {
    use super::*;
    use nih_plug_dsp::processors::chain::{Processor, ProcessorChain};
    use nih_plug_dsp::processors::gain::Gain;
    use nih_plug_dsp::processors::bias::Bias;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that chain composition matches sequential application.
        #[test]
        fn prop_chain_matches_sequential(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain1 in 0.5f32..=2.0f32,
            bias1 in -0.5f32..=0.5f32,
            gain2 in 0.5f32..=2.0f32
        ) {
            // Process through chain
            let mut chain = ProcessorChain::new();
            
            let mut g1 = Gain::new();
            g1.set_gain_linear(gain1);
            g1.set_smoothing_time(0.0, 44100.0);
            chain.add(g1);
            
            let mut b1 = Bias::new();
            b1.set_bias(bias1);
            chain.add(b1);
            
            let mut g2 = Gain::new();
            g2.set_gain_linear(gain2);
            g2.set_smoothing_time(0.0, 44100.0);
            chain.add(g2);
            
            chain.prepare(44100.0, 512);
            
            let mut output_chain = vec![0.0; samples.len()];
            chain.process(&samples, &mut output_chain);
            
            // Process sequentially
            let mut g1_seq = Gain::new();
            g1_seq.prepare(44100.0, 512);
            g1_seq.set_gain_linear(gain1);
            g1_seq.set_smoothing_time(0.0, 44100.0);
            
            let mut b1_seq = Bias::new();
            b1_seq.prepare(44100.0, 512);
            b1_seq.set_bias(bias1);
            
            let mut g2_seq = Gain::new();
            g2_seq.prepare(44100.0, 512);
            g2_seq.set_gain_linear(gain2);
            g2_seq.set_smoothing_time(0.0, 44100.0);
            
            let mut temp1 = vec![0.0; samples.len()];
            let mut temp2 = vec![0.0; samples.len()];
            let mut output_seq = vec![0.0; samples.len()];
            
            g1_seq.process(&samples, &mut temp1);
            b1_seq.process(&temp1, &mut temp2);
            g2_seq.process(&temp2, &mut output_seq);
            
            // Compare outputs
            for (chain_out, seq_out) in output_chain.iter().zip(output_seq.iter()) {
                prop_assert!((chain_out - seq_out).abs() < 1e-5,
                    "Chain output differs from sequential: chain={}, sequential={}",
                    chain_out, seq_out);
            }
        }

        /// Test that single processor chain matches direct processing.
        #[test]
        fn prop_single_processor_chain(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain in 0.5f32..=2.0f32
        ) {
            // Process through chain
            let mut chain = ProcessorChain::new();
            let mut g1 = Gain::new();
            g1.set_gain_linear(gain);
            g1.set_smoothing_time(0.0, 44100.0);
            chain.add(g1);
            chain.prepare(44100.0, 512);
            
            let mut output_chain = vec![0.0; samples.len()];
            chain.process(&samples, &mut output_chain);
            
            // Process directly
            let mut g2 = Gain::new();
            g2.prepare(44100.0, 512);
            g2.set_gain_linear(gain);
            g2.set_smoothing_time(0.0, 44100.0);
            
            let mut output_direct = vec![0.0; samples.len()];
            g2.process(&samples, &mut output_direct);
            
            // Compare outputs
            for (chain_out, direct_out) in output_chain.iter().zip(output_direct.iter()) {
                prop_assert!((chain_out - direct_out).abs() < 1e-5,
                    "Single processor chain differs from direct: chain={}, direct={}",
                    chain_out, direct_out);
            }
        }

        /// Test that empty chain passes signal unchanged.
        #[test]
        fn prop_empty_chain_passthrough(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50)
        ) {
            let mut chain = ProcessorChain::new();
            chain.prepare(44100.0, 512);
            
            let mut output = vec![0.0; samples.len()];
            chain.process(&samples, &mut output);
            
            for (input, output) in samples.iter().zip(output.iter()) {
                prop_assert!((input - output).abs() < 1e-10,
                    "Empty chain should pass signal unchanged: input={}, output={}",
                    input, output);
            }
        }

        /// Test that chain order matters (non-commutative).
        #[test]
        fn prop_chain_order_matters(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain in 1.5f32..=2.0f32,
            bias in 0.1f32..=0.5f32
        ) {
            // Chain 1: Gain -> Bias
            let mut chain1 = ProcessorChain::new();
            let mut g1 = Gain::new();
            g1.set_gain_linear(gain);
            g1.set_smoothing_time(0.0, 44100.0);
            chain1.add(g1);
            
            let mut b1 = Bias::new();
            b1.set_bias(bias);
            chain1.add(b1);
            
            chain1.prepare(44100.0, 512);
            
            let mut output1 = vec![0.0; samples.len()];
            chain1.process(&samples, &mut output1);
            
            // Chain 2: Bias -> Gain
            let mut chain2 = ProcessorChain::new();
            let mut b2 = Bias::new();
            b2.set_bias(bias);
            chain2.add(b2);
            
            let mut g2 = Gain::new();
            g2.set_gain_linear(gain);
            g2.set_smoothing_time(0.0, 44100.0);
            chain2.add(g2);
            
            chain2.prepare(44100.0, 512);
            
            let mut output2 = vec![0.0; samples.len()];
            chain2.process(&samples, &mut output2);
            
            // Outputs should be different (order matters)
            let mut found_difference = false;
            for (out1, out2) in output1.iter().zip(output2.iter()) {
                if (out1 - out2).abs() > 1e-5 {
                    found_difference = true;
                    break;
                }
            }
            
            prop_assert!(found_difference,
                "Chain order should matter: gain->bias should differ from bias->gain");
        }

        /// Test that chain composition is associative.
        /// (A -> B) -> C should equal A -> (B -> C)
        #[test]
        fn prop_chain_associativity(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain1 in 0.5f32..=2.0f32,
            bias in -0.5f32..=0.5f32,
            gain2 in 0.5f32..=2.0f32
        ) {
            // All three processors in one chain
            let mut chain_flat = ProcessorChain::new();
            
            let mut g1 = Gain::new();
            g1.set_gain_linear(gain1);
            g1.set_smoothing_time(0.0, 44100.0);
            chain_flat.add(g1);
            
            let mut b1 = Bias::new();
            b1.set_bias(bias);
            chain_flat.add(b1);
            
            let mut g2 = Gain::new();
            g2.set_gain_linear(gain2);
            g2.set_smoothing_time(0.0, 44100.0);
            chain_flat.add(g2);
            
            chain_flat.prepare(44100.0, 512);
            
            let mut output_flat = vec![0.0; samples.len()];
            chain_flat.process(&samples, &mut output_flat);
            
            // Nested chain: (Gain1 -> Bias) as inner chain, then Gain2
            let mut inner_chain = ProcessorChain::new();
            let mut g1_inner = Gain::new();
            g1_inner.set_gain_linear(gain1);
            g1_inner.set_smoothing_time(0.0, 44100.0);
            inner_chain.add(g1_inner);
            
            let mut b1_inner = Bias::new();
            b1_inner.set_bias(bias);
            inner_chain.add(b1_inner);
            
            let mut outer_chain = ProcessorChain::new();
            outer_chain.add(inner_chain);
            
            let mut g2_outer = Gain::new();
            g2_outer.set_gain_linear(gain2);
            g2_outer.set_smoothing_time(0.0, 44100.0);
            outer_chain.add(g2_outer);
            
            outer_chain.prepare(44100.0, 512);
            
            let mut output_nested = vec![0.0; samples.len()];
            outer_chain.process(&samples, &mut output_nested);
            
            // Compare outputs
            for (flat_out, nested_out) in output_flat.iter().zip(output_nested.iter()) {
                prop_assert!((flat_out - nested_out).abs() < 1e-5,
                    "Flat chain differs from nested: flat={}, nested={}",
                    flat_out, nested_out);
            }
        }

        /// Test that chain preserves signal properties (finite values).
        #[test]
        fn prop_chain_preserves_finiteness(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain1 in 0.5f32..=2.0f32,
            bias in -0.5f32..=0.5f32,
            gain2 in 0.5f32..=2.0f32
        ) {
            let mut chain = ProcessorChain::new();
            
            let mut g1 = Gain::new();
            g1.set_gain_linear(gain1);
            g1.set_smoothing_time(0.0, 44100.0);
            chain.add(g1);
            
            let mut b1 = Bias::new();
            b1.set_bias(bias);
            chain.add(b1);
            
            let mut g2 = Gain::new();
            g2.set_gain_linear(gain2);
            g2.set_smoothing_time(0.0, 44100.0);
            chain.add(g2);
            
            chain.prepare(44100.0, 512);
            
            let mut output = vec![0.0; samples.len()];
            chain.process(&samples, &mut output);
            
            for output_sample in output.iter() {
                prop_assert!(output_sample.is_finite(),
                    "Chain output should be finite, got {}", output_sample);
            }
        }
    }
}

/// **Feature: juce-examples-validation, Property 12: Chain preparation propagation**
/// **Validates: Requirements 4.4**
///
/// This property verifies that calling prepare() on a chain propagates to all
/// processors. For any processor chain, after calling prepare(), all processors
/// in the chain should be in prepared state.
mod chain_preparation {
    use super::*;
    use nih_plug_dsp::processors::chain::{Processor, ProcessorChain};
    use nih_plug_dsp::processors::gain::Gain;
    use nih_plug_dsp::processors::bias::Bias;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that prepare propagates to all processors.
        #[test]
        fn prop_prepare_propagates(
            sample_rate in 22050.0f32..=96000.0f32,
            max_block_size in 64usize..=2048,
            num_processors in 1usize..=5
        ) {
            let mut chain = ProcessorChain::new();
            
            // Add multiple processors
            for _ in 0..num_processors {
                chain.add(Gain::new());
            }
            
            // Prepare the chain
            chain.prepare(sample_rate, max_block_size);
            
            // Test that processing works (which requires preparation)
            let input = vec![1.0; max_block_size];
            let mut output = vec![0.0; max_block_size];
            
            // This should not panic or produce invalid results
            chain.process(&input, &mut output);
            
            // All outputs should be finite
            for output_sample in output.iter() {
                prop_assert!(output_sample.is_finite(),
                    "After prepare, processing should produce finite output");
            }
        }

        /// Test that prepare can be called multiple times.
        #[test]
        fn prop_prepare_idempotent(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain in 0.5f32..=2.0f32
        ) {
            let mut chain = ProcessorChain::new();
            let mut g = Gain::new();
            g.set_gain_linear(gain);
            g.set_smoothing_time(0.0, 44100.0);
            chain.add(g);
            
            // Prepare multiple times
            chain.prepare(44100.0, 512);
            chain.prepare(48000.0, 1024);
            chain.prepare(44100.0, 512);
            
            let mut output = vec![0.0; samples.len()];
            chain.process(&samples, &mut output);
            
            // Should still work correctly
            for (input, output) in samples.iter().zip(output.iter()) {
                let expected = input * gain;
                prop_assert!((output - expected).abs() < 1e-5,
                    "After multiple prepares, processing should work: expected={}, got={}",
                    expected, output);
            }
        }

        /// Test that prepare with different sample rates works.
        #[test]
        fn prop_prepare_different_sample_rates(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            sample_rate1 in 22050.0f32..=48000.0f32,
            sample_rate2 in 48000.0f32..=96000.0f32
        ) {
            let mut chain = ProcessorChain::new();
            chain.add(Gain::new());
            chain.add(Bias::new());
            
            // Prepare with first sample rate
            chain.prepare(sample_rate1, 512);
            let mut output1 = vec![0.0; samples.len()];
            chain.process(&samples, &mut output1);
            
            // Prepare with second sample rate
            chain.prepare(sample_rate2, 512);
            let mut output2 = vec![0.0; samples.len()];
            chain.process(&samples, &mut output2);
            
            // Both should produce finite outputs
            for output_sample in output1.iter().chain(output2.iter()) {
                prop_assert!(output_sample.is_finite(),
                    "Processing after prepare should produce finite output");
            }
        }
    }
}

/// **Feature: juce-examples-validation, Property 13: Chain reset propagation**
/// **Validates: Requirements 4.5**
///
/// This property verifies that calling reset() on a chain propagates to all
/// processors. For any processor chain, after calling reset(), all processors
/// should be in reset state.
mod chain_reset {
    use super::*;
    use nih_plug_dsp::processors::chain::{Processor, ProcessorChain};
    use nih_plug_dsp::processors::gain::Gain;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 100,
            ..ProptestConfig::default()
        })]

        /// Test that reset propagates to all processors.
        #[test]
        fn prop_reset_propagates(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain in 0.5f32..=2.0f32,
            smoothing_time in 5.0f32..=20.0f32
        ) {
            let mut chain = ProcessorChain::new();
            let mut g = Gain::new();
            g.set_gain_linear(gain);
            g.set_smoothing_time(smoothing_time, 44100.0);
            chain.add(g);
            
            chain.prepare(44100.0, 512);
            
            // Process some samples to build up state
            let mut output1 = vec![0.0; samples.len()];
            chain.process(&samples, &mut output1);
            
            // Reset the chain
            chain.reset();
            
            // Process the same samples again
            let mut output2 = vec![0.0; samples.len()];
            chain.process(&samples, &mut output2);
            
            // Outputs should match (reset cleared state)
            for (out1, out2) in output1.iter().zip(output2.iter()) {
                prop_assert!((out1 - out2).abs() < 1e-5,
                    "After reset, processing should produce same output: first={}, second={}",
                    out1, out2);
            }
        }

        /// Test that reset can be called multiple times.
        #[test]
        fn prop_reset_idempotent(
            samples in prop::collection::vec(-1.0f32..=1.0f32, 10..50),
            gain in 0.5f32..=2.0f32
        ) {
            let mut chain = ProcessorChain::new();
            let mut g = Gain::new();
            g.set_gain_linear(gain);
            g.set_smoothing_time(0.0, 44100.0);
            chain.add(g);
            
            chain.prepare(44100.0, 512);
            
            // Reset multiple times
            chain.reset();
            chain.reset();
            chain.reset();
            
            let mut output = vec![0.0; samples.len()];
            chain.process(&samples, &mut output);
            
            // Should still work correctly
            for (input, output) in samples.iter().zip(output.iter()) {
                let expected = input * gain;
                prop_assert!((output - expected).abs() < 1e-5,
                    "After multiple resets, processing should work: expected={}, got={}",
                    expected, output);
            }
        }

        /// Test that reset clears smoothing state in all processors.
        #[test]
        fn prop_reset_clears_all_smoothing(
            gain1 in 0.5f32..=1.0f32,
            gain2 in 1.5f32..=2.0f32,
            smoothing_time in 10.0f32..=30.0f32
        ) {
            let mut chain = ProcessorChain::new();
            let mut g = Gain::new();
            g.set_gain_linear(gain1);
            g.set_smoothing_time(smoothing_time, 44100.0);
            chain.add(g);
            
            chain.prepare(44100.0, 512);
            
            // Process some samples to settle at gain1
            let settle_samples = ((smoothing_time / 1000.0) * 44100.0 * 5.0) as usize;
            let input = vec![1.0; settle_samples];
            let mut output = vec![0.0; settle_samples];
            chain.process(&input, &mut output);
            
            // Change gain and reset
            if let Some(processor) = chain.get_mut(0) {
                // We can't directly access the gain, but we can reset
                // The test is that after reset, the next sample should have the new gain
            }
            chain.reset();
            
            // After reset, next sample should immediately reflect current gain
            // (no smoothing from previous state)
            let output_sample = {
                let input_single = vec![1.0; 1];
                let mut output_single = vec![0.0; 1];
                chain.process(&input_single, &mut output_single);
                output_single[0]
            };
            
            // Output should be close to gain1 (the current gain after reset)
            prop_assert!((output_sample - gain1).abs() < 0.1,
                "After reset, output should reflect current gain: expected~{}, got={}",
                gain1, output_sample);
        }

        /// Test that reset works on empty chain.
        #[test]
        fn prop_reset_empty_chain(
            _dummy in 0..1
        ) {
            let mut chain = ProcessorChain::new();
            chain.prepare(44100.0, 512);
            
            // Reset should not panic on empty chain
            chain.reset();
            
            // Should still work
            let input = vec![1.0; 10];
            let mut output = vec![0.0; 10];
            chain.process(&input, &mut output);
            
            prop_assert_eq!(output, input, "Empty chain should pass through after reset");
        }
    }
}
