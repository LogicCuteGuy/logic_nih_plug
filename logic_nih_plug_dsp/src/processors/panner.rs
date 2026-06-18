//! Stereo panner with selectable pan law.
//!
//! Port of JUCE's `juce::dsp::Panner` — a processor that distributes a mono
//! or stereo signal across the stereo field using one of several pan-law
//! curves. Pan values range from **-1.0** (full left) to **+1.0** (full
//! right), with **0.0** being centre.
//!
//! # Pan laws
//!
//! | Law | Description |
//! |---|---|
//! | [`PanLaw::Linear`] | Classic 6 dB law — perceived level stays constant when summed to mono |
//! | [`PanLaw::Balanced`] | Both channels at unity when centred, tapering to 0 at the extremes |
//! | [`PanLaw::Sin3dB`] | Sine-based constant-power 3 dB law |
//! | [`PanLaw::Sin4p5dB`] | Sine-based 4.5 dB law (compromise between 3 dB and 6 dB) |
//! | [`PanLaw::Sin6dB`] | Sine-based 6 dB law |
//! | [`PanLaw::SquareRoot3dB`] | Square-root constant-power 3 dB law |
//! | [`PanLaw::SquareRoot4p5dB`] | Square-root 4.5 dB law |
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_dsp::processors::panner::{Panner, PanLaw};
//!
//! let mut panner = Panner::new();
//! panner.set_law(PanLaw::SquareRoot3dB);
//! panner.prepare(44100.0, 512);
//! panner.set_pan(0.5); // panned right
//!
//! let mut left  = vec![0.5; 512];
//! let mut right = vec![0.5; 512];
//! panner.process_stereo(&mut left, &mut right);
//! ```

/// The pan law governing how signal is distributed across the stereo field.
///
/// Each law trades off mono compatibility, perceived loudness constancy, and
/// stereo width differently. See the [module-level](self) documentation for
/// details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(missing_docs)]
pub enum PanLaw {
    /// 6 dB / linear — `L = 1 - pan`, `R = pan`. Mono-sum-safe.
    Linear,
    /// Both channels at 1.0 when centred, tapering to 0 at extremes.
    #[default]
    Balanced,
    /// Sine-based constant-power 3 dB law.
    Sin3dB,
    /// Sine-based 4.5 dB law (compromise).
    Sin4p5dB,
    /// Sine-based 6 dB law.
    Sin6dB,
    /// Square-root constant-power 3 dB law.
    SquareRoot3dB,
    /// Square-root 4.5 dB law.
    SquareRoot4p5dB,
}

/// Linear ramp smoother for parameter smoothing.
///
/// When the target changes, the output ramps linearly from the current value
/// to the target over a configurable number of samples.
#[derive(Debug, Clone)]
struct Smoother {
    current: f32,
    target: f32,
    step: f32,
    remaining: usize,
}

impl Smoother {
    fn new(initial: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            step: 0.0,
            remaining: 0,
        }
    }

    /// Snap directly to a value (no ramp).
    fn snap_to(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.remaining = 0;
    }

    /// Set a new target, ramping over `ramp_samples` steps.
    fn set_target(&mut self, target: f32, ramp_samples: usize) {
        self.target = target;
        if ramp_samples == 0 || (self.current - target).abs() < 1e-12 {
            self.snap_to(target);
        } else {
            self.remaining = ramp_samples;
            self.step = (target - self.current) / ramp_samples as f32;
        }
    }

    /// Return the next smoothed sample.
    #[inline]
    fn next(&mut self) -> f32 {
        if self.remaining > 0 {
            self.current += self.step;
            self.remaining -= 1;
            if self.remaining == 0 {
                self.current = self.target;
            }
        }
        self.current
    }
}

/// A stereo panner with selectable pan law.
///
/// Takes a mono or stereo signal and distributes it across the stereo field
/// according to the chosen [`PanLaw`]. L/R volume coefficients are smoothed
/// to avoid clicks on parameter changes.
///
/// Pan values range from **-1.0** (full left) through **0.0** (centre) to
/// **+1.0** (full right).
#[derive(Debug, Clone)]
pub struct Panner {
    law: PanLaw,
    pan: f32,
    left_smoother: Smoother,
    right_smoother: Smoother,
    sample_rate: f32,
    ramp_time_ms: f32,
}

impl Default for Panner {
    fn default() -> Self {
        Self::new()
    }
}

impl Panner {
    /// Ramp time used by JUCE's `SmoothedValue::reset(sampleRate, 0.05)`.
    const RAMP_TIME_MS: f32 = 50.0;

    /// Creates a new panner with default (balanced) law and centre pan.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::panner::Panner;
    ///
    /// let panner = Panner::new();
    /// assert_eq!(panner.pan(), 0.0);
    /// ```
    pub fn new() -> Self {
        let mut s = Self {
            law: PanLaw::Balanced,
            pan: 0.0,
            left_smoother: Smoother::new(0.5),
            right_smoother: Smoother::new(0.5),
            sample_rate: 44100.0,
            ramp_time_ms: Self::RAMP_TIME_MS,
        };
        s.update();
        s
    }

    /// Sets the pan law.
    pub fn set_law(&mut self, law: PanLaw) {
        self.law = law;
        self.update();
    }

    /// Returns the current pan law.
    pub fn law(&self) -> PanLaw {
        self.law
    }

    /// Sets the pan position.
    ///
    /// # Arguments
    ///
    /// * `pan` — Pan value in the range **-1.0** (full left) to **+1.0** (full right).
    ///   Values outside this range are clamped.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::panner::Panner;
    ///
    /// let mut panner = Panner::new();
    /// panner.set_pan(0.75); // panned right
    /// assert!((panner.pan() - 0.75).abs() < 1e-6);
    /// ```
    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
        self.update();
    }

    /// Returns the current pan value.
    pub fn pan(&self) -> f32 {
        self.pan
    }

    /// Initialises the processor. Must be called before [`process_stereo`](Self::process_stereo).
    pub fn prepare(&mut self, sample_rate: f32, _max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.left_smoother.snap_to(self.left_smoother.target);
        self.right_smoother.snap_to(self.right_smoother.target);
    }

    /// Resets internal state, snapping smoothed values to their targets.
    pub fn reset(&mut self) {
        self.left_smoother.snap_to(self.left_smoother.target);
        self.right_smoother.snap_to(self.right_smoother.target);
    }

    /// Processes a stereo buffer pair in-place.
    ///
    /// `left` and `right` must have the same length. Mono content can be
    /// processed by passing the same slice for both channels and then
    /// reading the result from `left` / `right`.
    ///
    /// # Panics
    ///
    /// Panics if the two slices have different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::panner::{Panner, PanLaw};
    ///
    /// let mut panner = Panner::new();
    /// panner.prepare(44100.0, 512);
    /// panner.set_pan(0.0); // centre
    ///
    /// let mut left  = vec![1.0; 64];
    /// let mut right = vec![1.0; 64];
    /// panner.process_stereo(&mut left, &mut right);
    /// ```
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(
            left.len(),
            right.len(),
            "Left and right buffers must have the same length"
        );

        let ramp_samples = (self.sample_rate * self.ramp_time_ms / 1000.0) as usize;
        self.left_smoother
            .set_target(self.left_smoother.target, ramp_samples);
        self.right_smoother
            .set_target(self.right_smoother.target, ramp_samples);

        for (l, r) in left.iter_mut().zip(right.iter_mut()) {
            let lv = self.left_smoother.next();
            let rv = self.right_smoother.next();
            *l *= lv;
            *r *= rv;
        }
    }

    // ---- internal ----

    /// Recomputes target L/R volumes from the current pan value and law.
    fn update(&mut self) {
        let normalised_pan = 0.5 * (self.pan + 1.0);

        let (left_val, right_val, boost_val) = match self.law {
            PanLaw::Balanced => (
                f32::min(0.5, 1.0 - normalised_pan),
                f32::min(0.5, normalised_pan),
                2.0,
            ),
            PanLaw::Linear => (
                1.0 - normalised_pan,
                normalised_pan,
                2.0,
            ),
            PanLaw::Sin3dB => {
                let half_pi = std::f32::consts::FRAC_PI_2;
                (
                    (half_pi * (1.0 - normalised_pan)).sin(),
                    (half_pi * normalised_pan).sin(),
                    std::f32::consts::SQRT_2,
                )
            }
            PanLaw::Sin4p5dB => {
                let half_pi = std::f32::consts::FRAC_PI_2;
                (
                    (half_pi * (1.0 - normalised_pan)).sin().powf(1.5),
                    (half_pi * normalised_pan).sin().powf(1.5),
                    2.0f32.powf(0.75),
                )
            }
            PanLaw::Sin6dB => {
                let half_pi = std::f32::consts::FRAC_PI_2;
                (
                    (half_pi * (1.0 - normalised_pan)).sin().powi(2),
                    (half_pi * normalised_pan).sin().powi(2),
                    2.0,
                )
            }
            PanLaw::SquareRoot3dB => (
                (1.0 - normalised_pan).sqrt(),
                normalised_pan.sqrt(),
                std::f32::consts::SQRT_2,
            ),
            PanLaw::SquareRoot4p5dB => (
                (1.0 - normalised_pan).sqrt().powf(1.5),
                normalised_pan.sqrt().powf(1.5),
                2.0f32.powf(0.75),
            ),
        };

        self.left_smoother.target = left_val * boost_val;
        self.right_smoother.target = right_val * boost_val;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helpers ----

    fn make_dc_block(length: usize) -> (Vec<f32>, Vec<f32>) {
        (vec![1.0; length], vec![1.0; length])
    }

    fn rms(buf: &[f32]) -> f32 {
        let sum: f32 = buf.iter().map(|x| x * x).sum();
        (sum / buf.len() as f32).sqrt()
    }

    // ---- construction ----

    #[test]
    fn test_default_construction() {
        let p = Panner::new();
        assert_eq!(p.pan(), 0.0);
        assert_eq!(p.law(), PanLaw::Balanced);
    }

    #[test]
    fn test_set_pan_clamped() {
        let mut p = Panner::new();
        p.set_pan(5.0);
        assert_eq!(p.pan(), 1.0);
        p.set_pan(-5.0);
        assert_eq!(p.pan(), -1.0);
    }

    // ---- balanced law ----

    #[test]
    fn test_balanced_centre() {
        let mut p = Panner::new();
        p.set_law(PanLaw::Balanced);
        p.prepare(44100.0, 512);
        p.set_pan(0.0);

        let (mut left, mut right) = make_dc_block(512);
        p.process_stereo(&mut left, &mut right);

        // Balanced: both = min(0.5, 0.5) * 2.0 = 1.0
        let expected = 1.0;
        for (l, r) in left.iter().zip(right.iter()) {
            assert!((l - expected).abs() < 0.01, "left={l}");
            assert!((r - expected).abs() < 0.01, "right={r}");
        }
    }

    #[test]
    fn test_balanced_full_left() {
        let mut p = Panner::new();
        p.set_law(PanLaw::Balanced);
        p.set_pan(-1.0);
        p.prepare(44100.0, 512);
        p.reset();

        let (mut left, mut right) = make_dc_block(512);
        p.process_stereo(&mut left, &mut right);

        // After reset, smoothers start at target: left = min(0.5, 1.0)*2 = 1.0, right = min(0.5, 0.0)*2 = 0.0
        let l_rms = rms(&left);
        let r_rms = rms(&right);
        assert!(l_rms > 0.9, "left rms={l_rms}");
        assert!(r_rms < 0.05, "right rms={r_rms}");
    }

    #[test]
    fn test_balanced_full_right() {
        let mut p = Panner::new();
        p.set_law(PanLaw::Balanced);
        p.set_pan(1.0);
        p.prepare(44100.0, 512);
        p.reset();

        let (mut left, mut right) = make_dc_block(512);
        p.process_stereo(&mut left, &mut right);

        let l_rms = rms(&left);
        let r_rms = rms(&right);
        assert!(l_rms < 0.05, "left rms={l_rms}");
        assert!(r_rms > 0.9, "right rms={r_rms}");
    }

    // ---- linear law ----

    #[test]
    fn test_linear_centre() {
        let mut p = Panner::new();
        p.set_law(PanLaw::Linear);
        p.prepare(44100.0, 512);
        p.set_pan(0.0);

        let (mut left, mut right) = make_dc_block(512);
        p.process_stereo(&mut left, &mut right);

        // Linear: both = 0.5 * 2.0 = 1.0
        let l_rms = rms(&left);
        let r_rms = rms(&right);
        assert!((l_rms - 1.0).abs() < 0.01, "left rms={l_rms}");
        assert!((r_rms - 1.0).abs() < 0.01, "right rms={r_rms}");
    }

    #[test]
    fn test_linear_full_left() {
        let mut p = Panner::new();
        p.set_law(PanLaw::Linear);
        p.set_pan(-1.0);
        p.prepare(44100.0, 512);
        p.reset();

        let (mut left, mut right) = make_dc_block(512);
        p.process_stereo(&mut left, &mut right);

        // Linear: left = (1-0)*2 = 2.0, right = 0*2 = 0.0
        let l_rms = rms(&left);
        let r_rms = rms(&right);
        assert!(l_rms > 1.9, "left rms={l_rms}");
        assert!(r_rms < 0.05, "right rms={r_rms}");
    }

    // ---- square-root 3 dB (constant power) ----

    #[test]
    fn test_sqrt3db_centre() {
        let mut p = Panner::new();
        p.set_law(PanLaw::SquareRoot3dB);
        p.prepare(44100.0, 512);
        p.set_pan(0.0);

        let (mut left, mut right) = make_dc_block(512);
        p.process_stereo(&mut left, &mut right);

        // sqrt(0.5)*sqrt(2) = 1.0
        let l_rms = rms(&left);
        let r_rms = rms(&right);
        assert!((l_rms - 1.0).abs() < 0.01, "left rms={l_rms}");
        assert!((r_rms - 1.0).abs() < 0.01, "right rms={r_rms}");
    }

    #[test]
    fn test_sqrt3db_constant_power() {
        // Constant-power law should maintain approximately the same
        // total energy regardless of pan position.
        let mut p = Panner::new();
        p.set_law(PanLaw::SquareRoot3dB);
        p.prepare(44100.0, 1024);

        let mut energies = Vec::new();
        for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            p.set_pan(pan);
            p.reset();
            let (mut left, mut right) = make_dc_block(1024);
            p.process_stereo(&mut left, &mut right);
            let energy: f32 = left.iter().zip(right.iter()).map(|(l, r)| l * l + r * r).sum();
            energies.push(energy / 1024.0);
        }

        // All energies should be approximately equal
        for (i, e) in energies.iter().enumerate() {
            assert!(
                (e - energies[2]).abs() < 0.05,
                "energy at pan index {i} = {e}, expected ~{}",
                energies[2]
            );
        }
    }

    // ---- sin3dB ----

    #[test]
    fn test_sin3db_centre() {
        let mut p = Panner::new();
        p.set_law(PanLaw::Sin3dB);
        p.prepare(44100.0, 512);
        p.set_pan(0.0);

        let (mut left, mut right) = make_dc_block(512);
        p.process_stereo(&mut left, &mut right);

        // sin(pi/2 * 0.5) * sqrt(2) = sin(pi/4) * sqrt(2) = 1.0
        let l_rms = rms(&left);
        let r_rms = rms(&right);
        assert!((l_rms - 1.0).abs() < 0.02, "left rms={l_rms}");
        assert!((r_rms - 1.0).abs() < 0.02, "right rms={r_rms}");
    }

    // ---- reset ----

    #[test]
    fn test_reset() {
        let mut p = Panner::new();
        p.prepare(44100.0, 512);
        p.set_pan(0.8);
        // Process a few samples to move the smoother away from target
        let (mut left, mut right) = make_dc_block(10);
        p.process_stereo(&mut left, &mut right);
        // Reset should snap smoothed values to target
        p.reset();
        // After reset, next process should start from the target
        let (mut left, mut right) = make_dc_block(4);
        p.process_stereo(&mut left, &mut right);
        // Should be close to target (no remaining ramp)
        let l_rms = rms(&left);
        let r_rms = rms(&right);
        assert!(l_rms > 0.0 || r_rms > 0.0);
    }

    // ---- edge: both channels at full left / full right ----

    #[test]
    fn test_silence_stays_silent() {
        let mut p = Panner::new();
        p.prepare(44100.0, 256);
        p.set_pan(0.5);

        let mut left = vec![0.0; 256];
        let mut right = vec![0.0; 256];
        p.process_stereo(&mut left, &mut right);

        for (l, r) in left.iter().zip(right.iter()) {
            assert!(*l < 1e-10);
            assert!(*r < 1e-10);
        }
    }

    // ---- multiple laws produce different results ----

    #[test]
    fn test_different_laws_differ() {
        let mut p = Panner::new();
        p.prepare(44100.0, 64);
        p.set_pan(0.3);

        let mut results = Vec::new();
        for law in [
            PanLaw::Linear,
            PanLaw::Balanced,
            PanLaw::SquareRoot3dB,
            PanLaw::Sin3dB,
        ] {
            p.set_law(law);
            p.reset();
            let (mut left, mut right) = make_dc_block(64);
            p.process_stereo(&mut left, &mut right);
            let l = rms(&left);
            let r = rms(&right);
            results.push((l, r));
        }

        // At least some laws should produce different results
        let first = results[0];
        let any_different = results.iter().any(|(l, r)| {
            (l - first.0).abs() > 0.01 || (r - first.1).abs() > 0.01
        });
        assert!(any_different, "Different pan laws should produce different results");
    }

}
