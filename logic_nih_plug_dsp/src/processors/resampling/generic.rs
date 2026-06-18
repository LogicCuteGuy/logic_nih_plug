//! Generic interpolator wrapper that manages a circular sample buffer
//! and delegates sub-sample interpolation to a specific [`Interpolator`]
//! implementation.

/// Trait for sub-sample interpolation strategies used by
/// [`GenericInterpolator`].
///
/// Each implementation specifies its algorithmic latency and memory
/// requirements, and provides the core interpolation function.
/// Stateless interpolators can safely ignore `&self`.
pub trait Interpolator: Default + Clone + 'static {
    /// Algorithmic latency introduced by this interpolator, in samples.
    const ALGORITHMIC_LATENCY: f32;

    /// Number of samples required in the circular buffer.
    const MEMORY_SIZE: usize;

    /// Compute the interpolated value at the given sub-sample offset.
    ///
    /// * `inputs` — circular buffer of at least [`MEMORY_SIZE`](Self::MEMORY_SIZE)
    ///   elements.
    /// * `offset` — fractional position in `[0, 1)`.
    /// * `write_pos` — current write position (next slot to be written to).
    fn value_at_offset(&self, inputs: &[f32], offset: f32, write_pos: usize) -> f32;
}

/// Manages a circular sample buffer and delegates interpolation to a specific
/// [`Interpolator`] strategy.
///
/// One instance is needed **per channel**. The [`process`](Self::process)
/// method consumes input samples at a given speed ratio and produces
/// interpolated output samples.
///
/// # Type parameters
///
/// * `T` — the interpolation strategy (e.g. [`Linear`](super::Linear),
///   [`Lagrange`](super::Lagrange), [`WindowedSinc`](super::WindowedSinc)).
///
/// # Example
///
/// ```rust
/// use logic_nih_plug_dsp::processors::resampling::*;
///
/// let mut interp = GenericInterpolator::<Linear>::new();
/// let input = [0.0f32, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
/// let mut output = [0.0f32; 16];
/// interp.process(0.5, &input, &mut output);
/// ```
pub struct GenericInterpolator<T: Interpolator> {
    interpolator: T,
    last_input_samples: Vec<f32>,
    sub_sample_pos: f64,
    write_pos: usize,
}

impl<T: Interpolator> GenericInterpolator<T> {
    /// Create a new interpolator with all state cleared.
    pub fn new() -> Self {
        Self {
            interpolator: T::default(),
            last_input_samples: vec![0.0; T::MEMORY_SIZE],
            sub_sample_pos: 0.0,
            write_pos: 0,
        }
    }

    /// Process a block of input samples, filling `output` with interpolated
    /// samples at the given `speed_ratio`.
    ///
    /// * `speed_ratio` — resampling ratio. `1.0` = same rate (passthrough
    ///   with latency), `0.5` = half speed (time-stretch ×2),
    ///   `2.0` = double speed (time-compress ×2).
    /// * `input` — input samples to consume.
    /// * `output` — buffer to fill with interpolated output.
    pub fn process(&mut self, speed_ratio: f64, input: &[f32], output: &mut [f32]) {
        self.process_internal(speed_ratio, input, output, input.len(), false);
    }

    /// Like [`process`](Self::process), but adds the interpolated result
    /// (scaled by `gain`) to `output` instead of overwriting it.
    pub fn process_adding(
        &mut self,
        speed_ratio: f64,
        input: &[f32],
        output: &mut [f32],
        gain: f32,
    ) {
        let input_len = input.len();
        let mut input_pos = 0usize;

        for out in output.iter_mut() {
            while self.sub_sample_pos >= 1.0 && input_pos < input_len {
                self.last_input_samples[self.write_pos] = input[input_pos];
                self.write_pos = (self.write_pos + 1) % T::MEMORY_SIZE;
                self.sub_sample_pos -= 1.0;
                input_pos += 1;
            }

            *out += self.interpolator.value_at_offset(
                &self.last_input_samples,
                self.sub_sample_pos as f32,
                self.write_pos,
            ) * gain;

            self.sub_sample_pos += speed_ratio;
        }
    }

    /// Process with explicit control over available input count and circular
    /// buffer mode.
    ///
    /// * `speed_ratio` — resampling ratio.
    /// * `input` — input sample buffer.
    /// * `output` — output buffer to fill.
    /// * `num_input_samples_available` — how many input samples to consume
    ///   (may be less than `input.len()`).
    /// * `wrap_around` — if `true`, reads from `input[write_pos % input.len()]`
    ///   instead of sequentially (useful when `input` is itself a circular
    ///   buffer).
    pub fn process_advanced(
        &mut self,
        speed_ratio: f64,
        input: &[f32],
        output: &mut [f32],
        num_input_samples_available: usize,
        wrap_around: bool,
    ) {
        self.process_internal(
            speed_ratio,
            input,
            output,
            num_input_samples_available,
            wrap_around,
        );
    }

    fn process_internal(
        &mut self,
        speed_ratio: f64,
        input: &[f32],
        output: &mut [f32],
        num_input_samples_available: usize,
        wrap_around: bool,
    ) {
        let input_len = input.len();
        let mut input_pos = 0usize;
        let mut remaining = num_input_samples_available;

        for out in output.iter_mut() {
            while self.sub_sample_pos >= 1.0 && remaining > 0 {
                if wrap_around {
                    self.last_input_samples[self.write_pos] =
                        input[self.write_pos % input_len];
                } else {
                    self.last_input_samples[self.write_pos] = input[input_pos];
                    input_pos += 1;
                }
                self.write_pos = (self.write_pos + 1) % T::MEMORY_SIZE;
                self.sub_sample_pos -= 1.0;
                remaining -= 1;
            }

            *out = self.interpolator.value_at_offset(
                &self.last_input_samples,
                self.sub_sample_pos as f32,
                self.write_pos,
            );

            self.sub_sample_pos += speed_ratio;
        }
    }

    /// Reset all internal state (circular buffer, fractional position,
    /// write pointer).
    pub fn reset(&mut self) {
        for sample in &mut self.last_input_samples {
            *sample = 0.0;
        }
        self.sub_sample_pos = 0.0;
        self.write_pos = 0;
    }

    /// Returns the algorithmic latency of the interpolation strategy, in
    /// samples.
    pub fn get_base_latency(&self) -> f32 {
        T::ALGORITHMIC_LATENCY
    }

    /// Returns the current sub-sample position within the input stream.
    pub fn get_current_sub_sample(&self) -> f64 {
        self.sub_sample_pos
    }
}

impl<T: Interpolator> Default for GenericInterpolator<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::processors::resampling::linear::Linear;

    #[test]
    fn unity_passthrough() {
        // At speed_ratio = 1.0 the output should be a delayed copy of the input.
        let mut interp = GenericInterpolator::<Linear>::new();
        let input: Vec<f32> = (0..64).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 64];
        interp.process(1.0, &input, &mut output);

        // Linear has algorithmic latency 1; effective latency is 2 due to
        // the consumption-before-interpolation order.
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
    fn half_speed_doubles_length() {
        let mut interp = GenericInterpolator::<Linear>::new();
        // Use non-zero-starting input so early outputs are non-zero.
        let input: Vec<f32> = (1..33).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; 64];
        interp.process(0.5, &input, &mut output);

        // Output should be roughly twice as long.
        // After the initial fill, outputs should be non-zero.
        assert!(output[4] > 0.0, "output should contain non-zero values");
        // Last output should be near the last input value.
        let last = *output.last().unwrap();
        assert!(
            last > 20.0,
            "last output ({last}) should be near end of input"
        );
    }

    #[test]
    fn reset_clears_state() {
        let mut interp = GenericInterpolator::<Linear>::new();
        let input = [1.0f32; 16];
        let mut output = [0.0f32; 4];
        interp.process(1.0, &input, &mut output);
        // After processing, sub_sample_pos should be non-zero.
        assert!(interp.get_current_sub_sample() > 0.0);

        interp.reset();
        assert_eq!(interp.get_current_sub_sample(), 0.0);
        assert_eq!(interp.get_base_latency(), 1.0);
    }
}
