//! Zero-order hold interpolation — the simplest (lo-fi) strategy.

use super::Interpolator;

/// Zero-order hold interpolation: returns the nearest sample without
/// smoothing.  Produces a staircase waveform; useful for lo-fi effects
/// or when the delay is always an integer.
///
/// * Latency: 0 samples
/// * Memory: 1 sample
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroOrderHold;

impl Interpolator for ZeroOrderHold {
    const ALGORITHMIC_LATENCY: f32 = 0.0;
    const MEMORY_SIZE: usize = 1;

    fn value_at_offset(&self, inputs: &[f32], _offset: f32, _write_pos: usize) -> f32 {
        inputs[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::resampling::generic::GenericInterpolator;

    #[test]
    fn unity_passthrough() {
        let mut interp = GenericInterpolator::<ZeroOrderHold>::new();
        let input: Vec<f32> = (0..32).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 32];
        interp.process(1.0, &input, &mut output);

        // ZeroOrderHold has latency 0: output[0] = 0 (initial), then
        // output[n] = input[n-1] for n >= 1.
        for n in 1..32 {
            let diff = (output[n] - input[n - 1]).abs();
            assert!(
                diff < 1e-5,
                "output[{n}] = {} but expected {}",
                output[n],
                input[n - 1]
            );
        }
    }

    #[test]
    fn half_speed() {
        let mut interp = GenericInterpolator::<ZeroOrderHold>::new();
        let input: Vec<f32> = (0..16).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 32];
        interp.process(0.5, &input, &mut output);

        // At half speed, output should be roughly 2× the input length.
        // Each input sample is held for two output samples.
        let last = *output.last().unwrap();
        assert!(
            last > 10.0,
            "last output ({last}) should be near end of input"
        );
    }
}
