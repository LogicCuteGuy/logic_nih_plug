//! Linear interpolation between adjacent samples.

use super::Interpolator;

/// Linear interpolation: blends between adjacent samples.
/// Low CPU cost; introduces slight low-pass filtering when the speed
/// ratio changes in real time.
///
/// * Latency: 1 sample
/// * Memory: 2 samples
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Linear;

impl Interpolator for Linear {
    const ALGORITHMIC_LATENCY: f32 = 1.0;
    const MEMORY_SIZE: usize = 2;

    fn value_at_offset(&self, inputs: &[f32], offset: f32, write_pos: usize) -> f32 {
        // write_pos points to the oldest slot (next to be overwritten).
        // offset = 0 → older sample (latency 1), offset = 1 → newer sample.
        let a = inputs[(write_pos) % 2];
        let b = inputs[(write_pos + 1) % 2];
        a + offset * (b - a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::resampling::generic::GenericInterpolator;

    #[test]
    fn unity_passthrough() {
        let mut interp = GenericInterpolator::<Linear>::new();
        let input: Vec<f32> = (0..64).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 64];
        interp.process(1.0, &input, &mut output);

        // Effective latency = 2 (algorithmic 1 + 1 for consumption order).
        for n in 2..64 {
            let diff = (output[n] - input[n - 2]).abs();
            assert!(
                diff < 1e-5,
                "output[{n}] = {} but expected {}",
                output[n],
                input[n - 2]
            );
        }
    }

    #[test]
    fn interpolation_quality() {
        // Test half-speed interpolation produces intermediate values.
        let mut interp = GenericInterpolator::<Linear>::new();
        // Long enough input to fill the buffer and produce interpolated output.
        let input = [0.0f32, 0.5, 1.0, 1.0];
        let mut output = vec![0.0f32; 8];
        interp.process(0.5, &input, &mut output);

        // At half speed, each input sample is held for 2 output samples.
        // After the initial fill, we should see intermediate values
        // between the input samples.
        assert!(
            output[5] > 0.0 && output[5] < 1.0,
            "expected interpolated value between 0 and 1, got {}",
            output[5]
        );
    }
}
