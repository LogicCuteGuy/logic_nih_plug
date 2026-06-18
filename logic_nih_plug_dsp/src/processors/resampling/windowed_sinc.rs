//! Highest-quality windowed sinc interpolation.

use std::fmt;
use std::sync::Arc;

use super::Interpolator;

/// Number of sub-sample entries in the sinc kernel lookup table.
const SINC_TABLE_SIZE: usize = 1000;

/// Highest-quality interpolation using a Hann-windowed sinc kernel.
/// Precomputes a lookup table for fast sub-sample evaluation.
///
/// * Latency: 100 samples
/// * Memory: 200 samples
#[derive(Clone)]
pub struct WindowedSinc {
    table: Arc<[f32]>,
}

impl WindowedSinc {
    /// Create a new `WindowedSinc` interpolator, precomputing the kernel
    /// lookup table.  This allocates ~800 KB and takes a few milliseconds;
    /// subsequent [`clone()`](Clone::clone) calls are cheap (reference
    /// counted).
    pub fn new() -> Self {
        let n = Self::MEMORY_SIZE;
        let half_n = n as f32 / 2.0;
        let mut table = vec![0.0f32; SINC_TABLE_SIZE * n];

        for i in 0..SINC_TABLE_SIZE {
            let offset = i as f32 / SINC_TABLE_SIZE as f32;
            let mut sum = 0.0f32;

            for k in 0..n {
                let x = k as f32 - half_n + offset;

                // sinc(x) = sin(πx) / (πx), with sinc(0) = 1.
                let sinc_val = if x.abs() < 1e-6 {
                    1.0
                } else {
                    (std::f32::consts::PI * x).sin() / (std::f32::consts::PI * x)
                };

                // Hann window centred at half_n.
                let w = if x.abs() <= half_n {
                    0.5 * (1.0 + (std::f32::consts::PI * x / half_n).cos())
                } else {
                    0.0
                };

                let val = sinc_val * w;
                table[i * n + k] = val;
                sum += val;
            }

            // Normalise so the weights sum to 1.0 (unity gain at DC).
            if sum.abs() > 1e-10 {
                let inv_sum = 1.0 / sum;
                for k in 0..n {
                    table[i * n + k] *= inv_sum;
                }
            }
        }

        Self {
            table: Arc::from(table.into_boxed_slice()),
        }
    }
}

impl Default for WindowedSinc {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for WindowedSinc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WindowedSinc")
            .field("table_entries", &SINC_TABLE_SIZE)
            .field("memory_size", &Self::MEMORY_SIZE)
            .finish()
    }
}

impl Interpolator for WindowedSinc {
    const ALGORITHMIC_LATENCY: f32 = 100.0;
    const MEMORY_SIZE: usize = 200;

    fn value_at_offset(&self, inputs: &[f32], offset: f32, write_pos: usize) -> f32 {
        let table_idx = ((offset * SINC_TABLE_SIZE as f32) as usize).min(SINC_TABLE_SIZE - 1);
        let base = table_idx * Self::MEMORY_SIZE;

        let mut result = 0.0f32;
        for k in 0..Self::MEMORY_SIZE {
            let buf_idx = (write_pos + k) % Self::MEMORY_SIZE;
            result += unsafe {
                // SAFETY: base + k < SINC_TABLE_SIZE * MEMORY_SIZE and
                // buf_idx < MEMORY_SIZE, both within bounds.
                *self.table.get_unchecked(base + k) * *inputs.get_unchecked(buf_idx)
            };
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::resampling::generic::GenericInterpolator;

    #[test]
    fn unity_passthrough() {
        let mut interp = GenericInterpolator::<WindowedSinc>::new();
        // Use a large enough signal to fill the 200-sample buffer.
        let input: Vec<f32> = (0..300).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 300];
        interp.process(1.0, &input, &mut output);

        // After the buffer is full (200 samples) the output should track
        // the input with ~100-sample latency.
        for n in 210..300 {
            let expected = input[n - 100] as f32;
            let diff = (output[n] - expected).abs();
            assert!(
                diff < 0.5,
                "output[{n}] = {} but expected ≈{expected} (diff {diff})",
                output[n]
            );
        }
    }

    #[test]
    fn half_speed() {
        let mut interp = GenericInterpolator::<WindowedSinc>::new();
        // Use enough input samples to fully fill the 200-sample buffer.
        let input: Vec<f32> = (0..500).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 1000];
        interp.process(0.5, &input, &mut output);

        // After full ramp-up, output should be non-zero and rising.
        let last = *output.last().unwrap();
        assert!(
            last > 350.0,
            "last output ({last}) should be near end of input"
        );
    }

    #[test]
    fn table_normalisation() {
        // Verify that the kernel weights sum to approximately 1.0 at each
        // offset.
        let sinc = WindowedSinc::new();
        for table_idx in 0..SINC_TABLE_SIZE {
            let base = table_idx * WindowedSinc::MEMORY_SIZE;
            let sum: f32 = sinc.table[base..base + WindowedSinc::MEMORY_SIZE]
                .iter()
                .sum();
            assert!(
                (sum - 1.0).abs() < 0.01,
                "kernel sum at table_idx {table_idx} = {sum}, expected ≈1.0"
            );
        }
    }
}
