//! 4th-order (5-point) Lagrange polynomial interpolation.

use super::Interpolator;

/// 5-point Lagrange polynomial interpolation.
/// Higher quality than Catmull-Rom; suitable for high-fidelity resampling.
///
/// * Latency: 2 samples
/// * Memory: 5 samples
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lagrange;

impl Interpolator for Lagrange {
    const ALGORITHMIC_LATENCY: f32 = 2.0;
    const MEMORY_SIZE: usize = 5;

    fn value_at_offset(&self, inputs: &[f32], offset: f32, write_pos: usize) -> f32 {
        // write_pos points to the oldest slot. Read forward oldest→newest.
        // v[0]=oldest … v[4]=newest. Nodes at x = 0, 1, 2, 3, 4.
        // Evaluate at x = offset + 2 → at offset=0 → middle (latency 2).
        let v = [
            inputs[(write_pos) % 5],
            inputs[(write_pos + 1) % 5],
            inputs[(write_pos + 2) % 5],
            inputs[(write_pos + 3) % 5],
            inputs[(write_pos + 4) % 5],
        ];

        let x = offset + 2.0;
        let x1 = x - 1.0;
        let x2 = x - 2.0;
        let x3 = x - 3.0;
        let x4 = x - 4.0;

        // Lagrange basis polynomials for nodes at 0, 1, 2, 3, 4.
        let l0 = x1 * x2 * x3 * x4 / 24.0;
        let l1 = x * x2 * x3 * x4 / -6.0;
        let l2 = x * x1 * x3 * x4 / 4.0;
        let l3 = x * x1 * x2 * x4 / -6.0;
        let l4 = x * x1 * x2 * x3 / 24.0;

        v[0] * l0 + v[1] * l1 + v[2] * l2 + v[3] * l3 + v[4] * l4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::resampling::generic::GenericInterpolator;

    #[test]
    fn unity_passthrough() {
        let mut interp = GenericInterpolator::<Lagrange>::new();
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
    fn basis_at_integer_positions() {
        // Verify the Lagrange basis passes through the sample values at
        // integer positions.
        let interp = Lagrange;
        // Buffer with write_pos = 0: oldest=10, 20, 30, 40, newest=50
        let inputs = [10.0, 20.0, 30.0, 40.0, 50.0];

        // At offset = 0 → x = 2 → middle sample (30).
        let val = interp.value_at_offset(&inputs, 0.0, 0);
        assert!(
            (val - 30.0).abs() < 0.01,
            "at offset 0 expected ≈30, got {val}"
        );

        // At offset = 1 → x = 3 → next sample (40).
        let val = interp.value_at_offset(&inputs, 1.0, 0);
        assert!(
            (val - 40.0).abs() < 0.01,
            "at offset 1 expected ≈40, got {val}"
        );

        // At offset = -1 → x = 1 → previous sample (20).
        let val = interp.value_at_offset(&inputs, -1.0, 0);
        assert!(
            (val - 20.0).abs() < 0.01,
            "at offset -1 expected ≈20, got {val}"
        );
    }
}
