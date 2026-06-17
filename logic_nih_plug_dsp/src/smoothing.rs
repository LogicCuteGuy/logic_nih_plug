//! Parameter smoothing utilities.
//!
//! This module provides parameter smoothing for audio applications, allowing
//! smooth transitions between parameter values to avoid clicks and discontinuities.

/// A smoothed value that interpolates between parameter changes over time.
///
/// This is ported from JUCE's SmoothedValue class and provides linear interpolation
/// between a current value and a target value over a specified time period.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::smoothing::SmoothedValue;
///
/// let mut smoother = SmoothedValue::<f32>::new(0.0);
/// smoother.reset(44100.0, 0.05); // 50ms smoothing time at 44.1kHz
/// smoother.set_target(1.0);
///
/// // Get smoothed values sample by sample
/// for _ in 0..100 {
///     let value = smoother.next();
///     // Use value...
/// }
/// ```
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance.
#[derive(Clone, Debug)]
pub struct SmoothedValue<T> {
    /// Current smoothed value
    current: T,
    /// Target value to smooth towards
    target: T,
    /// Number of steps remaining to reach target
    steps_remaining: i32,
    /// Increment per step
    step: T,
    /// Sample rate in Hz
    sample_rate: f32,
    /// Smoothing time in seconds
    smoothing_time: f32,
}

impl SmoothedValue<f32> {
    /// Creates a new SmoothedValue with the given initial value.
    ///
    /// The smoother is initialized with zero smoothing time. Call `reset` to
    /// configure the sample rate and smoothing time.
    ///
    /// # Arguments
    ///
    /// * `initial_value` - The starting value
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let smoother = SmoothedValue::<f32>::new(440.0);
    /// ```
    pub fn new(initial_value: f32) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            steps_remaining: 0,
            step: 0.0,
            sample_rate: 44100.0,
            smoothing_time: 0.0,
        }
    }

    /// Resets the smoother with a new sample rate and smoothing time.
    ///
    /// This recalculates the internal smoothing coefficient based on the
    /// sample rate and desired smoothing time. If currently smoothing towards
    /// a target, the smoothing will be recalculated with the new parameters.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (must be positive)
    /// * `smoothing_time_seconds` - Time to reach target in seconds (must be non-negative)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let mut smoother = SmoothedValue::<f32>::new(0.0);
    /// smoother.reset(48000.0, 0.1); // 100ms smoothing at 48kHz
    /// ```
    pub fn reset(&mut self, sample_rate: f32, smoothing_time_seconds: f32) {
        let was_smoothing = self.is_smoothing();
        let old_target = self.target;
        
        self.sample_rate = sample_rate;
        self.smoothing_time = smoothing_time_seconds;
        
        // If we were smoothing, recalculate with new sample rate
        if was_smoothing {
            self.set_target(old_target);
        } else {
            self.steps_remaining = 0;
            self.step = 0.0;
        }
    }

    /// Sets a new target value to smooth towards.
    ///
    /// The smoother will interpolate from the current value to the target
    /// over the configured smoothing time.
    ///
    /// # Arguments
    ///
    /// * `new_target` - The target value to smooth towards
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let mut smoother = SmoothedValue::<f32>::new(0.0);
    /// smoother.reset(44100.0, 0.05);
    /// smoother.set_target(1.0);
    /// ```
    pub fn set_target(&mut self, new_target: f32) {
        if self.target == new_target {
            return;
        }

        self.target = new_target;

        // Calculate number of steps based on sample rate and smoothing time
        let num_steps = (self.sample_rate * self.smoothing_time).max(1.0) as i32;
        self.steps_remaining = num_steps;

        // Calculate increment per step
        if num_steps > 0 {
            self.step = (self.target - self.current) / num_steps as f32;
        } else {
            self.step = 0.0;
        }
    }

    /// Returns the next smoothed value and advances the internal state.
    ///
    /// Call this once per sample to get the smoothed parameter value.
    ///
    /// # Returns
    ///
    /// The current smoothed value
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let mut smoother = SmoothedValue::<f32>::new(0.0);
    /// smoother.reset(44100.0, 0.05);
    /// smoother.set_target(1.0);
    ///
    /// let value = smoother.next();
    /// ```
    pub fn next(&mut self) -> f32 {
        if self.steps_remaining > 0 {
            self.current += self.step;
            self.steps_remaining -= 1;

            // Snap to target on last step to avoid accumulation errors
            if self.steps_remaining == 0 {
                self.current = self.target;
            }
        }

        self.current
    }

    /// Immediately sets the current value without smoothing.
    ///
    /// This skips the smoothing process and jumps directly to the new value.
    /// Useful for initialization or when discontinuities are acceptable.
    ///
    /// # Arguments
    ///
    /// * `new_value` - The value to set immediately
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let mut smoother = SmoothedValue::<f32>::new(0.0);
    /// smoother.skip(1.0); // Jump to 1.0 immediately
    /// ```
    pub fn skip(&mut self, new_value: f32) {
        self.current = new_value;
        self.target = new_value;
        self.steps_remaining = 0;
        self.step = 0.0;
    }

    /// Returns the current value without advancing the state.
    ///
    /// # Returns
    ///
    /// The current smoothed value
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let smoother = SmoothedValue::<f32>::new(0.5);
    /// assert_eq!(smoother.current(), 0.5);
    /// ```
    pub fn current(&self) -> f32 {
        self.current
    }

    /// Returns the target value.
    ///
    /// # Returns
    ///
    /// The target value being smoothed towards
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let mut smoother = SmoothedValue::<f32>::new(0.0);
    /// smoother.set_target(1.0);
    /// assert_eq!(smoother.target(), 1.0);
    /// ```
    pub fn target(&self) -> f32 {
        self.target
    }

    /// Returns whether the smoother is currently smoothing.
    ///
    /// # Returns
    ///
    /// `true` if the smoother is actively interpolating towards the target
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let mut smoother = SmoothedValue::<f32>::new(0.0);
    /// smoother.reset(44100.0, 0.05);
    /// smoother.set_target(1.0);
    /// assert!(smoother.is_smoothing());
    /// ```
    pub fn is_smoothing(&self) -> bool {
        self.steps_remaining > 0
    }

    /// Processes a block of samples, filling the output buffer with smoothed values.
    ///
    /// This is more efficient than calling `next()` in a loop.
    ///
    /// # Arguments
    ///
    /// * `output` - Buffer to fill with smoothed values
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::smoothing::SmoothedValue;
    ///
    /// let mut smoother = SmoothedValue::<f32>::new(0.0);
    /// smoother.reset(44100.0, 0.05);
    /// smoother.set_target(1.0);
    ///
    /// let mut buffer = vec![0.0; 128];
    /// smoother.process(&mut buffer);
    /// ```
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next();
        }
    }
}

impl SmoothedValue<f64> {
    /// Creates a new SmoothedValue with the given initial value (f64 version).
    pub fn new(initial_value: f64) -> Self {
        Self {
            current: initial_value,
            target: initial_value,
            steps_remaining: 0,
            step: 0.0,
            sample_rate: 44100.0,
            smoothing_time: 0.0,
        }
    }

    /// Resets the smoother with a new sample rate and smoothing time (f64 version).
    pub fn reset(&mut self, sample_rate: f32, smoothing_time_seconds: f32) {
        let was_smoothing = self.is_smoothing();
        let old_target = self.target;
        
        self.sample_rate = sample_rate;
        self.smoothing_time = smoothing_time_seconds;
        
        // If we were smoothing, recalculate with new sample rate
        if was_smoothing {
            self.set_target(old_target);
        } else {
            self.steps_remaining = 0;
            self.step = 0.0;
        }
    }

    /// Sets a new target value to smooth towards (f64 version).
    pub fn set_target(&mut self, new_target: f64) {
        if self.target == new_target {
            return;
        }

        self.target = new_target;

        let num_steps = (self.sample_rate * self.smoothing_time).max(1.0) as i32;
        self.steps_remaining = num_steps;

        if num_steps > 0 {
            self.step = (self.target - self.current) / num_steps as f64;
        } else {
            self.step = 0.0;
        }
    }

    /// Returns the next smoothed value and advances the internal state (f64 version).
    pub fn next(&mut self) -> f64 {
        if self.steps_remaining > 0 {
            self.current += self.step;
            self.steps_remaining -= 1;

            if self.steps_remaining == 0 {
                self.current = self.target;
            }
        }

        self.current
    }

    /// Immediately sets the current value without smoothing (f64 version).
    pub fn skip(&mut self, new_value: f64) {
        self.current = new_value;
        self.target = new_value;
        self.steps_remaining = 0;
        self.step = 0.0;
    }

    /// Returns the current value without advancing the state (f64 version).
    pub fn current(&self) -> f64 {
        self.current
    }

    /// Returns the target value (f64 version).
    pub fn target(&self) -> f64 {
        self.target
    }

    /// Returns whether the smoother is currently smoothing (f64 version).
    pub fn is_smoothing(&self) -> bool {
        self.steps_remaining > 0
    }

    /// Processes a block of samples (f64 version).
    pub fn process(&mut self, output: &mut [f64]) {
        for sample in output.iter_mut() {
            *sample = self.next();
        }
    }
}

impl Default for SmoothedValue<f32> {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Default for SmoothedValue<f64> {
    fn default() -> Self {
        Self::new(0.0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_smoother() {
        let smoother = SmoothedValue::<f32>::new(0.5);
        assert_eq!(smoother.current(), 0.5);
        assert_eq!(smoother.target(), 0.5);
        assert!(!smoother.is_smoothing());
    }

    #[test]
    fn test_skip_to_value() {
        let mut smoother = SmoothedValue::<f32>::new(0.0);
        smoother.skip(1.0);
        assert_eq!(smoother.current(), 1.0);
        assert_eq!(smoother.target(), 1.0);
        assert!(!smoother.is_smoothing());
    }

    #[test]
    fn test_smoothing_basic() {
        let mut smoother = SmoothedValue::<f32>::new(0.0);
        smoother.reset(100.0, 0.1); // 100 Hz, 0.1 seconds = 10 samples
        smoother.set_target(1.0);

        assert!(smoother.is_smoothing());
        assert_eq!(smoother.target(), 1.0);

        // Should take 10 steps to reach target
        for _ in 0..9 {
            let val = smoother.next();
            assert!(val > 0.0 && val < 1.0);
            assert!(smoother.is_smoothing());
        }

        // Last step should reach target exactly
        let final_val = smoother.next();
        assert_eq!(final_val, 1.0);
        assert!(!smoother.is_smoothing());
    }

    #[test]
    fn test_smoothing_incremental() {
        let mut smoother = SmoothedValue::<f32>::new(0.0);
        smoother.reset(100.0, 0.1);
        smoother.set_target(1.0);

        let mut last_val = 0.0;
        for _ in 0..10 {
            let val = smoother.next();
            assert!(val >= last_val); // Should be monotonically increasing
            last_val = val;
        }

        assert_eq!(smoother.current(), 1.0);
    }

    #[test]
    fn test_sample_rate_change() {
        let mut smoother = SmoothedValue::<f32>::new(0.0);
        smoother.reset(44100.0, 0.05);
        smoother.set_target(1.0);

        // Advance a bit
        let val1 = smoother.next();
        let val2 = smoother.next();
        assert!(val1 < val2); // Should be increasing

        // Change sample rate mid-smoothing - should recalculate
        smoother.reset(48000.0, 0.05);

        // Should still be smoothing towards same target
        assert!(smoother.is_smoothing());
        assert_eq!(smoother.target(), 1.0);
        
        // Verify it completes smoothing
        for _ in 0..10000 {
            smoother.next();
            if !smoother.is_smoothing() {
                break;
            }
        }
        assert_eq!(smoother.current(), 1.0);
    }

    #[test]
    fn test_process_block() {
        let mut smoother = SmoothedValue::<f32>::new(0.0);
        smoother.reset(100.0, 0.1);
        smoother.set_target(1.0);

        let mut buffer = vec![0.0; 10];
        smoother.process(&mut buffer);

        // Last value should be target
        assert_eq!(buffer[9], 1.0);

        // Values should be increasing
        for i in 1..buffer.len() {
            assert!(buffer[i] >= buffer[i - 1]);
        }
    }

    #[test]
    fn test_immediate_target_change() {
        let mut smoother = SmoothedValue::<f32>::new(0.0);
        smoother.reset(100.0, 0.1);
        smoother.set_target(1.0);

        // Change target mid-smoothing
        smoother.next();
        smoother.next();
        smoother.set_target(0.5);

        // Should now smooth towards new target
        assert!(smoother.is_smoothing());
        assert_eq!(smoother.target(), 0.5);
    }

    #[test]
    fn test_f64_version() {
        let mut smoother = SmoothedValue::<f64>::new(0.0);
        smoother.reset(100.0, 0.1);
        smoother.set_target(1.0);

        assert!(smoother.is_smoothing());

        for _ in 0..10 {
            smoother.next();
        }

        assert_eq!(smoother.current(), 1.0);
        assert!(!smoother.is_smoothing());
    }
}
