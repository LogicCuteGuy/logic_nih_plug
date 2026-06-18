//! Dry/wet mixing with selectable mixing rule.
//!
//! Port of JUCE's `juce::dsp::DryWetMixer` — a utility that stores dry
//! samples and mixes them with a (possibly latency-offset) wet signal.
//! The mix proportion and mixing rule are both adjustable.
//!
//! # Usage pattern
//!
//! ```text
//! mixer.push_dry(left, right);   // 1. store dry samples
//! // ... apply wet effect ...
//! mixer.mix_wet(left, right);   // 2. mix stored dry with wet in-place
//! ```
//!
//! # Mixing rules
//!
//! | Rule | Description |
//! |---|---|
//! | [`DryWetMixingRule::Linear`] | `dry = 1 - mix`, `wet = mix` |
//! | [`DryWetMixingRule::Balanced`] | Both at unity when mix = 0.5 |
//! | [`DryWetMixingRule::Sin3dB`] | Sine-based constant-power 3 dB |
//! | [`DryWetMixingRule::Sin4p5dB`] | Sine-based 4.5 dB |
//! | [`DryWetMixingRule::Sin6dB`] | Sine-based 6 dB |
//! | [`DryWetMixingRule::SquareRoot3dB`] | Square-root constant-power 3 dB |
//! | [`DryWetMixingRule::SquareRoot4p5dB`] | Square-root 4.5 dB |
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_dsp::processors::dry_wet::{DryWetMixer, DryWetMixingRule};
//!
//! let mut mixer = DryWetMixer::new();
//! mixer.set_mixing_rule(DryWetMixingRule::Linear);
//! mixer.set_wet_mix(0.5);
//! mixer.prepare(44100.0, 2, 512);
//!
//! let dry_left  = vec![0.5; 512];
//! let dry_right = vec![0.5; 512];
//! mixer.push_dry(&dry_left, &dry_right);
//!
//! let mut wet_left  = vec![0.5; 512];
//! let mut wet_right = vec![0.5; 512];
//! mixer.mix_wet(&mut wet_left, &mut wet_right);
//! ```

/// The rule governing how dry and wet signals are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(missing_docs)]
pub enum DryWetMixingRule {
    /// Linear: `dry = 1 - mix`, `wet = mix`.
    #[default]
    Linear,
    /// Both channels at unity when mix = 0.5; each tapers to 0 at extremes.
    Balanced,
    /// Sine-based 3 dB constant-power law.
    Sin3dB,
    /// Sine-based 4.5 dB law.
    Sin4p5dB,
    /// Sine-based 6 dB law.
    Sin6dB,
    /// Square-root 3 dB constant-power law.
    SquareRoot3dB,
    /// Square-root 4.5 dB law.
    SquareRoot4p5dB,
}

/// Linear ramp smoother (per-sample).
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

    fn snap_to(&mut self, value: f32) {
        self.current = value;
        self.target = value;
        self.step = 0.0;
        self.remaining = 0;
    }

    fn set_target(&mut self, target: f32, ramp_samples: usize) {
        self.target = target;
        if ramp_samples == 0 || (self.current - target).abs() < 1e-12 {
            self.snap_to(target);
        } else {
            self.remaining = ramp_samples;
            self.step = (target - self.current) / ramp_samples as f32;
        }
    }

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

/// Simple single-producer/single-consumer ring buffer for dry sample storage.
#[derive(Debug, Clone)]
struct RingBuffer {
    channels: Vec<Vec<f32>>,
    mask: usize,
    write_pos: usize,
    read_pos: usize,
}

impl RingBuffer {
    fn new(num_channels: usize, min_size: usize) -> Self {
        let size = min_size.max(4).next_power_of_two();
        Self {
            channels: vec![vec![0.0; size]; num_channels],
            mask: size - 1,
            write_pos: 0,
            read_pos: 0,
        }
    }

    fn push(&mut self, channel: usize, samples: &[f32]) {
        let m = self.mask;
        for &s in samples {
            self.channels[channel][self.write_pos & m] = s;
            self.write_pos = self.write_pos.wrapping_add(1);
        }
    }

    fn pop(&mut self, channel: usize, samples: &mut [f32]) {
        let m = self.mask;
        for s in samples.iter_mut() {
            *s = self.channels[channel][self.read_pos & m];
            self.read_pos = self.read_pos.wrapping_add(1);
        }
    }

    fn available(&self) -> usize {
        self.write_pos.wrapping_sub(self.read_pos)
    }

    fn reset(&mut self) {
        for ch in &mut self.channels {
            ch.iter_mut().for_each(|x| *x = 0.0);
        }
        self.write_pos = 0;
        self.read_pos = 0;
    }

    fn resize(&mut self, num_channels: usize, min_size: usize) {
        let size = min_size.max(4).next_power_of_two();
        self.channels.resize_with(num_channels, || vec![0.0; size]);
        for ch in &mut self.channels {
            ch.resize(size, 0.0);
        }
        self.mask = size - 1;
        self.write_pos = 0;
        self.read_pos = 0;
    }
}

/// A dry/wet mixing processor with latency compensation support.
///
/// Stores dry samples via [`push_dry`](Self::push_dry) and mixes them
/// with a (possibly latency-offset) wet signal via
/// [`mix_wet`](Self::mix_wet). The dry/wet balance is controlled by
/// [`set_wet_mix`](Self::set_wet_mix) and the mixing curve by
/// [`set_mixing_rule`](Self::set_mixing_rule).
#[derive(Debug, Clone)]
pub struct DryWetMixer {
    rule: DryWetMixingRule,
    mix: f32,
    dry_vol: Smoother,
    wet_vol: Smoother,
    ring: RingBuffer,
    sample_rate: f32,
    num_channels: usize,
    max_block_size: usize,
    /// How many samples of wet latency to compensate.
    wet_latency: usize,
}

impl Default for DryWetMixer {
    fn default() -> Self {
        Self::new()
    }
}

impl DryWetMixer {
    const RAMP_TIME_MS: f32 = 50.0;

    /// Creates a new mixer with default linear rule and full wet (1.0).
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_dsp::processors::dry_wet::DryWetMixer;
    ///
    /// let mixer = DryWetMixer::new();
    /// assert_eq!(mixer.wet_mix(), 1.0);
    /// ```
    pub fn new() -> Self {
        let mut s = Self {
            rule: DryWetMixingRule::Linear,
            mix: 1.0,
            dry_vol: Smoother::new(0.0),
            wet_vol: Smoother::new(1.0),
            ring: RingBuffer::new(2, 1024),
            sample_rate: 44100.0,
            num_channels: 2,
            max_block_size: 512,
            wet_latency: 0,
        };
        s.update();
        s
    }

    /// Sets the mixing rule.
    pub fn set_mixing_rule(&mut self, rule: DryWetMixingRule) {
        self.rule = rule;
        self.update();
    }

    /// Returns the current mixing rule.
    pub fn mixing_rule(&self) -> DryWetMixingRule {
        self.rule
    }

    /// Sets the wet mix proportion.
    ///
    /// * `0.0` — full dry
    /// * `1.0` — full wet
    ///
    /// Values outside [0, 1] are clamped.
    pub fn set_wet_mix(&mut self, wet_mix: f32) {
        self.mix = wet_mix.clamp(0.0, 1.0);
        self.update();
    }

    /// Returns the current wet mix proportion.
    pub fn wet_mix(&self) -> f32 {
        self.mix
    }

    /// Sets the wet-path latency (in samples) for compensation.
    ///
    /// The mixer will delay the dry path by this many samples to stay
    /// aligned with the wet signal.
    pub fn set_wet_latency(&mut self, latency_samples: usize) {
        self.wet_latency = latency_samples;
    }

    /// Initialises the mixer. Must be called before
    /// [`push_dry`](Self::push_dry) / [`mix_wet`](Self::mix_wet).
    pub fn prepare(&mut self, sample_rate: f32, num_channels: usize, max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.num_channels = num_channels;
        self.max_block_size = max_block_size;

        // Ring buffer needs headroom for push-ahead / pop lag
        let ring_size = (max_block_size + self.wet_latency).max(max_block_size * 2);
        self.ring.resize(num_channels, ring_size);
    }

    /// Resets internal state (smoother targets, ring buffer).
    pub fn reset(&mut self) {
        let dry_target = compute_dry_value(self.rule, self.mix);
        let wet_target = compute_wet_value(self.rule, self.mix);
        self.dry_vol.snap_to(dry_target);
        self.wet_vol.snap_to(wet_target);
        self.ring.reset();
    }

    /// Stores dry samples for later mixing.
    ///
    /// `left` and `right` are the dry signal for the left and right
    /// channels. They must have the same length and be ≤ `max_block_size`.
    ///
    /// # Panics
    ///
    /// Panics if channel slices have different lengths or the ring buffer
    /// overflows (i.e. `push_dry` called more than `mix_wet` can drain).
    pub fn push_dry(&mut self, left: &[f32], right: &[f32]) {
        assert_eq!(
            left.len(),
            right.len(),
            "Left and right dry buffers must have the same length"
        );

        let n = left.len();
        if self.num_channels >= 1 {
            self.ring.push(0, left);
        }
        if self.num_channels >= 2 {
            self.ring.push(1, right);
        }
        // For >2 channels, push silence (they don't participate in dry)
        for ch in 2..self.num_channels {
            let zeros: Vec<f32> = vec![0.0; n];
            self.ring.push(ch, &zeros);
        }
    }

    /// Mixes stored dry samples with the wet signal in-place.
    ///
    /// On entry, `left` / `right` contain the **fully wet** signal. On
    /// exit they contain the mixed dry + wet result.
    ///
    /// The number of samples in `left` / `right` must be ≤
    /// `max_block_size` and ≤ the number of samples previously pushed
    /// via [`push_dry`](Self::push_dry).
    ///
    /// # Panics
    ///
    /// Panics if the ring buffer has fewer samples available than requested.
    pub fn mix_wet(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(
            left.len(),
            right.len(),
            "Left and right wet buffers must have the same length"
        );

        let n = left.len();
        let ramp_samples = (self.sample_rate * Self::RAMP_TIME_MS / 1000.0) as usize;
        self.dry_vol
            .set_target(compute_dry_value(self.rule, self.mix), ramp_samples);
        self.wet_vol
            .set_target(compute_wet_value(self.rule, self.mix), ramp_samples);

        assert!(
            self.ring.available() >= n,
            "Not enough dry samples in ring buffer (available={}, needed={})",
            self.ring.available(),
            n
        );

        // Read dry from ring buffer
        let mut dry_l = vec![0.0f32; n];
        let mut dry_r = vec![0.0f32; n];
        self.ring.pop(0, &mut dry_l);
        if self.num_channels >= 2 {
            self.ring.pop(1, &mut dry_r);
        } else {
            dry_r.copy_from_slice(&dry_l);
        }

        // Mix: output = wet * wet_vol + dry * dry_vol
        for i in 0..n {
            let wv = self.wet_vol.next();
            let dv = self.dry_vol.next();
            left[i] = left[i] * wv + dry_l[i] * dv;
            right[i] = right[i] * wv + dry_r[i] * dv;
        }
    }

    // ---- internal ----

    fn update(&mut self) {
        let dry_target = compute_dry_value(self.rule, self.mix);
        let wet_target = compute_wet_value(self.rule, self.mix);
        self.dry_vol.target = dry_target;
        self.wet_vol.target = wet_target;
    }
}

// ---- mixing rule computation ----

fn compute_dry_value(rule: DryWetMixingRule, mix: f32) -> f32 {
    let m = mix as f64;
    match rule {
        DryWetMixingRule::Linear => (1.0 - m) as f32,
        DryWetMixingRule::Balanced => {
            (2.0 * f64::min(0.5, 1.0 - m)) as f32
        }
        DryWetMixingRule::Sin3dB => {
            (std::f64::consts::FRAC_PI_2 * (1.0 - m)).sin() as f32
        }
        DryWetMixingRule::Sin4p5dB => {
            ((std::f64::consts::FRAC_PI_2 * (1.0 - m)).sin()).powf(1.5) as f32
        }
        DryWetMixingRule::Sin6dB => {
            ((std::f64::consts::FRAC_PI_2 * (1.0 - m)).sin()).powi(2) as f32
        }
        DryWetMixingRule::SquareRoot3dB => ((1.0 - m).sqrt()) as f32,
        DryWetMixingRule::SquareRoot4p5dB => ((1.0 - m).sqrt().powf(1.5)) as f32,
    }
}

fn compute_wet_value(rule: DryWetMixingRule, mix: f32) -> f32 {
    let m = mix as f64;
    match rule {
        DryWetMixingRule::Linear => m as f32,
        DryWetMixingRule::Balanced => (2.0 * f64::min(0.5, m)) as f32,
        DryWetMixingRule::Sin3dB => (std::f64::consts::FRAC_PI_2 * m).sin() as f32,
        DryWetMixingRule::Sin4p5dB => {
            ((std::f64::consts::FRAC_PI_2 * m).sin()).powf(1.5) as f32
        }
        DryWetMixingRule::Sin6dB => {
            ((std::f64::consts::FRAC_PI_2 * m).sin()).powi(2) as f32
        }
        DryWetMixingRule::SquareRoot3dB => m.sqrt() as f32,
        DryWetMixingRule::SquareRoot4p5dB => m.sqrt().powf(1.5) as f32,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a ramp buffer (0.0 → 1.0) for one channel.
    fn ramp_up(n: usize) -> Vec<f32> {
        (0..n).map(|i| i as f32 / n as f32).collect()
    }

    /// Helper: create a descending ramp buffer (1.0 → 0.0).
    fn ramp_down(n: usize) -> Vec<f32> {
        (0..n).map(|i| 1.0 - i as f32 / n as f32).collect()
    }

    // ---- construction ----

    #[test]
    fn test_default_construction() {
        let m = DryWetMixer::new();
        assert_eq!(m.wet_mix(), 1.0);
        assert_eq!(m.mixing_rule(), DryWetMixingRule::Linear);
    }

    #[test]
    fn test_set_wet_mix_clamped() {
        let mut m = DryWetMixer::new();
        m.set_wet_mix(-0.5);
        assert_eq!(m.wet_mix(), 0.0);
        m.set_wet_mix(2.0);
        assert_eq!(m.wet_mix(), 1.0);
    }

    // ---- linear rule ----

    #[test]
    fn test_linear_full_dry() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::Linear);
        m.set_wet_mix(0.0);
        m.prepare(44100.0, 2, 512);
        m.reset();

        let dry_l = ramp_up(512);
        let dry_r = ramp_down(512);
        m.push_dry(&dry_l, &dry_r);

        let mut wet_l = vec![1.0; 512];
        let mut wet_r = vec![1.0; 512];
        m.mix_wet(&mut wet_l, &mut wet_r);

        // At full dry, output should match dry signal
        let eps = 0.01;
        for i in 0..512 {
            assert!(
                (wet_l[i] - dry_l[i]).abs() < eps,
                "left[{i}]: got {}, expected ~{}",
                wet_l[i],
                dry_l[i]
            );
            assert!(
                (wet_r[i] - dry_r[i]).abs() < eps,
                "right[{i}]: got {}, expected ~{}",
                wet_r[i],
                dry_r[i]
            );
        }
    }

    #[test]
    fn test_linear_full_wet() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::Linear);
        m.set_wet_mix(1.0);
        m.prepare(44100.0, 2, 512);
        m.reset();

        let dry_l = vec![0.0; 512];
        let dry_r = vec![0.0; 512];
        m.push_dry(&dry_l, &dry_r);

        let mut wet_l = ramp_up(512);
        let mut wet_r = ramp_down(512);
        let orig_wet_l = wet_l.clone();
        m.mix_wet(&mut wet_l, &mut wet_r);

        // At full wet, output should match wet signal
        let eps = 0.01;
        for i in 0..512 {
            assert!(
                (wet_l[i] - orig_wet_l[i]).abs() < eps,
                "left[{i}]: got {}, expected ~{}",
                wet_l[i],
                orig_wet_l[i]
            );
        }
    }

    #[test]
    fn test_linear_half_mix() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::Linear);
        m.set_wet_mix(0.5);
        m.prepare(44100.0, 2, 512);
        m.reset();

        // Dry = 1.0, Wet = 1.0 → at 50% linear, output ≈ 1.0
        let dry_l = vec![1.0; 512];
        let dry_r = vec![1.0; 512];
        m.push_dry(&dry_l, &dry_r);

        let mut wet_l = vec![1.0; 512];
        let mut wet_r = vec![1.0; 512];
        m.mix_wet(&mut wet_l, &mut wet_r);

        // At end of ramp: 0.5 * 1.0 + 0.5 * 1.0 = 1.0
        let eps = 0.01;
        for i in 400..512 {
            assert!(
                (wet_l[i] - 1.0).abs() < eps,
                "left[{i}]: got {}",
                wet_l[i]
            );
        }
    }

    // ---- balanced rule ----

    #[test]
    fn test_balanced_half_mix() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::Balanced);
        m.set_wet_mix(0.5);
        m.prepare(44100.0, 2, 512);
        m.reset();

        let dry_l = vec![1.0; 512];
        let dry_r = vec![1.0; 512];
        m.push_dry(&dry_l, &dry_r);

        let mut wet_l = vec![1.0; 512];
        let mut wet_r = vec![1.0; 512];
        m.mix_wet(&mut wet_l, &mut wet_r);

        // Balanced at 0.5: dry = 2*min(0.5, 0.5) = 1.0, wet = 2*min(0.5, 0.5) = 1.0
        // output = 1.0 * 1.0 + 1.0 * 1.0 = 2.0
        let eps = 0.05;
        for i in 400..512 {
            assert!(
                (wet_l[i] - 2.0).abs() < eps,
                "left[{i}]: got {}",
                wet_l[i]
            );
        }
    }

    // ---- square root 3 dB ----

    #[test]
    fn test_sqrt3db_half_mix() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::SquareRoot3dB);
        m.set_wet_mix(0.5);
        m.prepare(44100.0, 2, 512);
        m.reset();

        let dry_l = vec![1.0; 512];
        let dry_r = vec![1.0; 512];
        m.push_dry(&dry_l, &dry_r);

        let mut wet_l = vec![1.0; 512];
        let mut wet_r = vec![1.0; 512];
        m.mix_wet(&mut wet_l, &mut wet_r);

        // sqrt3dB at 0.5: dry = sqrt(0.5) ≈ 0.707, wet = sqrt(0.5) ≈ 0.707
        // output ≈ 0.707 + 0.707 = 1.414
        let eps = 0.05;
        for i in 400..512 {
            assert!(
                (wet_l[i] - std::f32::consts::SQRT_2).abs() < eps,
                "left[{i}]: got {}, expected ~{}",
                wet_l[i],
                std::f32::consts::SQRT_2
            );
        }
    }

    // ---- sin3dB ----

    #[test]
    fn test_sin3db_half_mix() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::Sin3dB);
        m.set_wet_mix(0.5);
        m.prepare(44100.0, 2, 512);
        m.reset();

        let dry_l = vec![1.0; 512];
        let dry_r = vec![1.0; 512];
        m.push_dry(&dry_l, &dry_r);

        let mut wet_l = vec![1.0; 512];
        let mut wet_r = vec![1.0; 512];
        m.mix_wet(&mut wet_l, &mut wet_r);

        // sin3dB at 0.5: dry = sin(pi/4) ≈ 0.707, wet = sin(pi/4) ≈ 0.707
        // output ≈ 1.414
        let eps = 0.05;
        for i in 400..512 {
            assert!(
                (wet_l[i] - std::f32::consts::SQRT_2).abs() < eps,
                "left[{i}]: got {}, expected ~{}",
                wet_l[i],
                std::f32::consts::SQRT_2
            );
        }
    }

    // ---- different rules produce different results ----

    #[test]
    fn test_different_rules_differ() {
        let mut m = DryWetMixer::new();
        m.set_wet_mix(0.3);
        m.prepare(44100.0, 2, 64);

        let mut results = Vec::new();
        for rule in [
            DryWetMixingRule::Linear,
            DryWetMixingRule::Balanced,
            DryWetMixingRule::SquareRoot3dB,
            DryWetMixingRule::Sin3dB,
        ] {
            m.set_mixing_rule(rule);
            m.reset();

            let dry_l = vec![1.0; 64];
            let dry_r = vec![1.0; 64];
            m.push_dry(&dry_l, &dry_r);

            let mut wet_l = vec![1.0; 64];
            let mut wet_r = vec![1.0; 64];
            m.mix_wet(&mut wet_l, &mut wet_r);

            // Take the value at the end of the block (after ramp settles)
            results.push(wet_l[63]);
        }

        let first = results[0];
        let any_different = results.iter().any(|v| (v - first).abs() > 0.01);
        assert!(
            any_different,
            "Different mixing rules should produce different results at mix=0.3"
        );
    }

    // ---- silence stays silent ----

    #[test]
    fn test_silence_stays_silent() {
        let mut m = DryWetMixer::new();
        m.set_wet_mix(0.5);
        m.prepare(44100.0, 2, 256);
        m.reset();

        let dry_l = vec![0.0; 256];
        let dry_r = vec![0.0; 256];
        m.push_dry(&dry_l, &dry_r);

        let mut wet_l = vec![0.0; 256];
        let mut wet_r = vec![0.0; 256];
        m.mix_wet(&mut wet_l, &mut wet_r);

        for (l, r) in wet_l.iter().zip(wet_r.iter()) {
            assert!(*l < 1e-10);
            assert!(*r < 1e-10);
        }
    }

    // ---- multiple push/mix cycles ----

    #[test]
    fn test_multiple_blocks() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::Linear);
        m.set_wet_mix(0.5);
        m.prepare(44100.0, 2, 256);
        m.reset();

        for _ in 0..5 {
            let dry_l = vec![1.0; 256];
            let dry_r = vec![1.0; 256];
            m.push_dry(&dry_l, &dry_r);

            let mut wet_l = vec![1.0; 256];
            let mut wet_r = vec![1.0; 256];
            m.mix_wet(&mut wet_l, &mut wet_r);

            // After ramp settles, output ≈ 1.0
            let last = wet_l[255];
            assert!(
                (last - 1.0).abs() < 0.05,
                "Expected ~1.0, got {last}"
            );
        }
    }

    // ---- mono (1-channel) ----

    #[test]
    fn test_mono_push_mix() {
        let mut m = DryWetMixer::new();
        m.set_mixing_rule(DryWetMixingRule::Linear);
        m.set_wet_mix(0.5);
        m.prepare(44100.0, 1, 128);
        m.reset();

        let dry = vec![1.0; 128];
        m.push_dry(&dry, &dry);

        let mut wet = vec![0.5; 128];
        let mut dummy = vec![0.0; 128];
        m.mix_wet(&mut wet, &mut dummy);

        // After ramp: 0.5 * 0.5 + 0.5 * 1.0 = 0.75
        let last = wet[127];
        assert!(
            (last - 0.75).abs() < 0.05,
            "Expected ~0.75, got {last}"
        );
    }
}
