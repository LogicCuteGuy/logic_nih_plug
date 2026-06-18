//! Catmull-Rom cubic spline interpolation.

use super::Interpolator;

/// Catmull-Rom cubic spline interpolation using 4 control points.
/// Smoother than linear; good balance of quality and cost.
///
/// * Latency: 2 samples
/// * Memory: 4 samples
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CatmullRom;

impl Interpolator for CatmullRom {
    const ALGORITHMIC_LATENCY: f32 = 2.0;
    const MEMORY_SIZE: usize = 4;

    fn value_at_offset(&self, inputs: &[f32], offset: f32, write_pos: usize) -> f32 {
        // write_pos points to the oldest slot. Read forward oldest→newest.
        let p0 = inputs[(write_pos) % 4];
        let p1 = inputs[(write_pos + 1) % 4];
        let p2 = inputs[(write_pos + 2) % 4];
        let p3 = inputs[(write_pos + 3) % 4];

        // Standard Catmull-Rom: interpolates between p1 and p2
        // with p0 and p3 as outer control points.
        // At t = 0 → p1 (latency 2), at t = 1 → p2.
        let t = offset;
        let t2 = t * t;
        let t3 = t2 * t;

        0.5 * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::resampling::generic::GenericInterpolator;

    #[test]
    fn unity_passthrough() {
        let mut interp = GenericInterpolator::<CatmullRom>::new();
        let input: Vec<f32> = (0..64).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 64];
        interp.process(1.0, &input, &mut output);

        // Effective latency = 3 (algorithmic 2 + 1 for consumption order).
        for n in 4..64 {
            let diff = (output[n] - input[n - 3]).abs();
            assert!(
                diff < 0.5,
                "output[{n}] = {} but expected ≈{} (diff {})",
                output[n],
                input[n - 3],
                diff
            );
        }
    }

    #[test]
    fn smoothness() {
        // A ramp through CatmullRom should be smooth (no discontinuities).
        let mut interp = GenericInterpolator::<CatmullRom>::new();
        let input: Vec<f32> = (0..32).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 32];
        interp.process(1.0, &input, &mut output);

        // Check that consecutive differences are bounded.
        for n in 3..32 {
            let diff = (output[n] - output[n - 1]).abs();
            assert!(
                diff < 2.0,
                "discontinuity at output[{n}]: diff = {diff}"
            );
        }
    }
}
