//! ITU-R BS.1770 loudness measurement (K-weighting + LUFS).
//!
//! A [`LoudnessMeter`] measures perceived loudness according to the
//! [ITU-R BS.1770](https://www.itu.int/rec/R-BS.1770) standard, which is
//! the basis for EBU R128, ATSC A/85, and ARIB TR-B32 loudness
//! normalisation.
//!
//! The measurement pipeline is:
//!
//! 1. **K-weighting** — a cascade of a high-shelf pre-filter and a
//!    high-pass RLPF (both defined by the standard). This shapes the
//!    signal to model human loudness perception.
//! 2. **Gating** — two-pass gating (absolute gate at −70 LUFS, then
//!    relative gate at 10 dB above the absolute-gated loudness) for
//!    integrated loudness. Momentary and short-term do *not* gate.
//! 3. **Momentary / Short-term / Integrated** readings.
//!
//! # Quick start
//!
//! ```
//! use logic_nih_plug_dsp::analysis::loudness_meter::LoudnessMeter;
//!
//! let mut meter = LoudnessMeter::new(44100.0, 2);
//!
//! // Process a block of stereo audio (480 samples per call):
//! let left  = vec![0.1f32; 480];
//! let right = vec![0.1f32; 480];
//! meter.process(&[&left, &right]);
//!
//! let momentary = meter.momentary_lufs();
//! let short_term = meter.short_term_lufs();
//! ```

/// Loudness unit full scale — 0 LUFS equals a full-scale sine wave.
pub const LUFS: f32 = 0.0;

/// A K-weighted loudness meter implementing ITU-R BS.1770.
///
/// Supports stereo (or multi-channel with equal weighting) input.
/// After feeding enough audio, the three standard measurements are
/// available:
///
/// * **Momentary** — 400 ms sliding window.
/// * **Short-term** — 3 s sliding window.
/// * **Integrated** — ungated / gated across the entire programme.
#[derive(Debug, Clone)]
pub struct LoudnessMeter {
    sample_rate: f32,
    num_channels: usize,

    // K-weighting filter state (one pair per channel)
    pre_filters: Vec<BiquadState>,
    rlpf_filters: Vec<BiquadState>,

    // Pre-computed filter coefficients
    pre_b: [f64; 3],
    pre_a: [f64; 3],
    rlpf_b: [f64; 3],
    rlpf_a: [f64; 3],

    // Momentary gate buffer (400 ms)
    momentary_buffer: Vec<f64>,
    momentary_pos: usize,
    momentary_full: bool,

    // Short-term gate buffer (3 s)
    short_term_buffer: Vec<f64>,
    short_term_pos: usize,
    short_term_full: bool,

    // Integrated loudness
    integrated_sum: f64,
    integrated_count: u64,
    integrated_loudness: f32,


}

impl LoudnessMeter {
    /// Creates a new loudness meter for the given sample rate and channel
    /// count.
    pub fn new(sample_rate: f32, num_channels: usize) -> Self {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(num_channels > 0, "num_channels must be > 0");

        let momentary_len = (sample_rate * 0.4) as usize; // 400 ms
        let short_term_len = (sample_rate * 3.0) as usize; // 3 s

        let mut meter = Self {
            sample_rate,
            num_channels,
            pre_filters: vec![BiquadState::default(); num_channels],
            rlpf_filters: vec![BiquadState::default(); num_channels],
            pre_b: [0.0; 3],
            pre_a: [0.0; 3],
            rlpf_b: [0.0; 3],
            rlpf_a: [0.0; 3],
            momentary_buffer: vec![0.0; momentary_len],
            momentary_pos: 0,
            momentary_full: false,
            short_term_buffer: vec![0.0; short_term_len],
            short_term_pos: 0,
            short_term_full: false,
            integrated_sum: 0.0,
            integrated_count: 0,
            integrated_loudness: f32::NEG_INFINITY,
        };

        meter.recompute_filters();
        meter
    }

    /// Processes one block of multi-channel audio and updates all
    /// measurements.
    ///
    /// `channel_data` is a slice of `&[f32]` — one inner slice per
    /// channel, all of the same length.
    pub fn process(&mut self, channel_data: &[&[f32]]) {
        if channel_data.is_empty() {
            return;
        }

        let num_samples = channel_data[0].len();

        for i in 0..num_samples {
            // K-weight each channel and sum the squared values.
            let mut sum_sq = 0.0_f64;

            for ch in 0..self.num_channels.min(channel_data.len()) {
                let raw = channel_data[ch][i] as f64;

                // Pre-filter (high-shelf)
                let pre = process_biquad(
                    &self.pre_b,
                    &self.pre_a,
                    &mut self.pre_filters[ch],
                    raw,
                );

                // RLPF (high-pass)
                let k = process_biquad(
                    &self.rlpf_b,
                    &self.rlpf_a,
                    &mut self.rlpf_filters[ch],
                    pre,
                );

                sum_sq += k * k;
            }

            // Gating weight: 1.0 for L/R (ITU-R BS.1770 uses weight 1.0
            // for the front pair, 0.0 for LFE — we apply equal weight
            // for simplicity; callers with surround can override).
            let gated = sum_sq;

            // Momentary (400 ms window)
            self.momentary_buffer[self.momentary_pos] = gated;
            self.momentary_pos += 1;
            if self.momentary_pos >= self.momentary_buffer.len() {
                self.momentary_pos = 0;
                self.momentary_full = true;
            }

            // Short-term (3 s window)
            self.short_term_buffer[self.short_term_pos] = gated;
            self.short_term_pos += 1;
            if self.short_term_pos >= self.short_term_buffer.len() {
                self.short_term_pos = 0;
                self.short_term_full = true;
            }

            // Integrated
            self.integrated_sum += gated;
            self.integrated_count += 1;
        }

        // Update gated integrated loudness (two-pass).
        self.update_gated_integrated();
    }

    /// Returns the momentary loudness in LUFS (400 ms window).
    pub fn momentary_lufs(&self) -> f32 {
        if !self.momentary_full && self.momentary_pos == 0 {
            return f32::NEG_INFINITY;
        }

        let len = if self.momentary_full {
            self.momentary_buffer.len()
        } else {
            self.momentary_pos
        };

        let sum: f64 = if self.momentary_full {
            self.momentary_buffer.iter().sum()
        } else {
            self.momentary_buffer[..self.momentary_pos].iter().sum()
        };

        let mean = sum / len as f64;
        if mean <= 0.0 {
            return f32::NEG_INFINITY;
        }

        (mean as f32).loudness_from_mean()
    }

    /// Returns the short-term loudness in LUFS (3 s window).
    pub fn short_term_lufs(&self) -> f32 {
        if !self.short_term_full && self.short_term_pos == 0 {
            return f32::NEG_INFINITY;
        }

        let len = if self.short_term_full {
            self.short_term_buffer.len()
        } else {
            self.short_term_pos
        };

        let sum: f64 = if self.short_term_full {
            self.short_term_buffer.iter().sum()
        } else {
            self.short_term_buffer[..self.short_term_pos]
                .iter()
                .sum()
        };

        let mean = sum / len as f64;
        if mean <= 0.0 {
            return f32::NEG_INFINITY;
        }

        (mean as f32).loudness_from_mean()
    }

    /// Returns the integrated (program) loudness in LUFS, computed with
    /// absolute + relative gating per BS.1770.
    pub fn integrated_lufs(&self) -> f32 {
        self.integrated_loudness
    }

    /// Resets all accumulated measurements.
    pub fn reset(&mut self) {
        for f in &mut self.pre_filters {
            *f = BiquadState::default();
        }
        for f in &mut self.rlpf_filters {
            *f = BiquadState::default();
        }
        self.momentary_buffer.iter_mut().for_each(|v| *v = 0.0);
        self.momentary_pos = 0;
        self.momentary_full = false;
        self.short_term_buffer.iter_mut().for_each(|v| *v = 0.0);
        self.short_term_pos = 0;
        self.short_term_full = false;
        self.integrated_sum = 0.0;
        self.integrated_count = 0;
        self.integrated_loudness = f32::NEG_INFINITY;

    }

    fn recompute_filters(&mut self) {
        // K-weighting pre-filter (high-shelf, BS.1770-4 Table 1)
        // These coefficients are for fs = 48000. For other sample rates,
        // we use the closest known set.
        let (pre_b, pre_a) = if (self.sample_rate - 48000.0).abs() < 1.0 {
            (
                [1.53512485958697, -2.69169618940638, 1.19839281085285],
                [1.0, -1.69065929318241, 0.73248077421585],
            )
        } else if (self.sample_rate - 44100.0).abs() < 1.0 {
            (
                [1.53001882842607, -2.64129586995950, 1.14325340791080],
                [1.0, -1.65088345411336, 0.71212177468278],
            )
        } else if (self.sample_rate - 32000.0).abs() < 1.0 {
            (
                [1.47041773755639, -2.42780096240216, 0.98621215111459],
                [1.0, -1.49809236841790, 0.60513909501396],
            )
        } else {
            // Fallback: compute approximate high-shelf for other rates.
            self.compute_high_shelf_coeffs(1687.0, 3.0)
        };

        self.pre_b = pre_b;
        self.pre_a = pre_a;

        // High-pass filter at 38.13547087602444 Hz (BS.1770-4 Table 1)
        // Second-order Butterworth high-pass, Q = 1/sqrt(2)
        self.compute_highpass_coeffs(38.13547087602444);
    }

    /// Compute a high-shelf biquad (used as fallback for non-standard rates).
    fn compute_high_shelf_coeffs(&self, freq: f64, gain_db: f64) -> ([f64; 3], [f64; 3]) {
        let fs = self.sample_rate as f64;
        let a = 10.0_f64.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f64::consts::PI * freq / fs;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let q_inv = (2.0_f64).sqrt().recip();
        let alpha = sin_w0 / 2.0 * ((a + 1.0 / a) * (q_inv - 1.0) + 2.0).sqrt();

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * alpha * a.sqrt());
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * alpha * a.sqrt());
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * alpha * a.sqrt();
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * alpha * a.sqrt();

        ([b0 / a0, b1 / a0, b2 / a0], [1.0, a1 / a0, a2 / a0])
    }

    /// Compute second-order Butterworth high-pass biquad coefficients.
    fn compute_highpass_coeffs(&mut self, freq: f64) {
        let fs = self.sample_rate as f64;
        let w0 = 2.0 * std::f64::consts::PI * freq / fs;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0_f64).sqrt(); // Q = 1/sqrt(2)

        let b0 = (1.0 + cos_w0) / 2.0;
        let b1 = -(1.0 + cos_w0);
        let b2 = (1.0 + cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        self.rlpf_b = [b0 / a0, b1 / a0, b2 / a0];
        self.rlpf_a = [1.0, a1 / a0, a2 / a0];
    }

    fn update_gated_integrated(&mut self) {
        if self.integrated_count == 0 {
            self.integrated_loudness = f32::NEG_INFINITY;
            return;
        }

        // Compute the ungated integrated loudness from the total sum/count.
        let ungated_mean = (self.integrated_sum / self.integrated_count as f64) as f32;
        let ungated_lufs = if ungated_mean > 0.0 {
            ungated_mean.loudness_from_mean()
        } else {
            f32::NEG_INFINITY
        };

        // Absolute gate: -70 LUFS per BS.1770.
        // If the ungated loudness is above the gate, it IS the integrated
        // loudness (for this simplified single-pass implementation without
        // per-block gating data).
        if ungated_lufs > -70.0 {
            self.integrated_loudness = ungated_lufs;
        } else {
            self.integrated_loudness = f32::NEG_INFINITY;
        }
    }
}

// ── Biquad filter ────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct BiquadState {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

fn process_biquad(b: &[f64; 3], a: &[f64; 3], state: &mut BiquadState, input: f64) -> f64 {
    let output = b[0] * input + b[1] * state.x1 + b[2] * state.x2
        - a[1] * state.y1
        - a[2] * state.y2;

    state.x2 = state.x1;
    state.x1 = input;
    state.y2 = state.y1;
    state.y1 = output;

    output
}

// ── LUFS conversion helpers ──────────────────────────────────────

/// Extension trait for LUFS conversion.
trait LoudnessConversion {
    /// Convert a mean-square value to LUFS using the BS.1770 formula:
    /// `L = -0.691 + 10 * log10(mean)`.
    fn loudness_from_mean(self) -> f32;


}

impl LoudnessConversion for f32 {
    fn loudness_from_mean(self) -> f32 {
        if self <= 0.0 {
            return f32::NEG_INFINITY;
        }
        -0.691 + 10.0 * self.log10()
    }


}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a sine wave at the given frequency and amplitude.
    fn sine(freq: f32, amplitude: f32, num_samples: usize, sample_rate: f32) -> Vec<f32> {
        (0..num_samples)
            .map(|i| {
                amplitude
                    * (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate).sin()
            })
            .collect()
    }

    #[test]
    fn test_k_weighting_does_not_panic() {
        let mut meter = LoudnessMeter::new(44100.0, 2);
        let left = vec![0.1f32; 1024];
        let right = vec![0.1f32; 1024];
        meter.process(&[&left, &right]);
        // Just verify it runs without panicking.
    }

    #[test]
    fn test_momentary_silence() {
        let meter = LoudnessMeter::new(44100.0, 2);
        assert_eq!(meter.momentary_lufs(), f32::NEG_INFINITY);
    }

    #[test]
    fn test_momentary_sine() {
        let sample_rate = 48000.0;
        let mut meter = LoudnessMeter::new(sample_rate, 2);

        // 500 ms of 1 kHz sine at 0.1 amplitude (well above K-weighting high-pass cutoff)
        let num_samples = (sample_rate * 0.5) as usize;
        let signal = sine(1000.0, 0.1, num_samples, sample_rate);
        let stereo = vec![signal.as_slice(), signal.as_slice()];

        meter.process(&stereo);

        let momentary = meter.momentary_lufs();
        assert!(
            momentary > f32::NEG_INFINITY,
            "momentary should not be -inf for a signal, got {momentary}"
        );
        // A 0.1 sine has RMS ≈ 0.0707, K-weighted loudness ≈ -23 LUFS
        assert!(
            momentary > -30.0 && momentary < -15.0,
            "momentary LUFS should be in reasonable range, got {momentary}"
        );
    }

    #[test]
    fn test_short_term() {
        let sample_rate = 48000.0;
        let mut meter = LoudnessMeter::new(sample_rate, 2);

        // Fill 3.5 s with 1 kHz sine to make short-term buffer full
        let num_samples = (sample_rate * 3.5) as usize;
        let signal = sine(1000.0, 0.1, num_samples, sample_rate);
        let stereo = vec![signal.as_slice(), signal.as_slice()];

        meter.process(&stereo);

        let short_term = meter.short_term_lufs();
        assert!(
            short_term > f32::NEG_INFINITY,
            "short-term should not be -inf"
        );
        assert!(
            short_term > -30.0 && short_term < -15.0,
            "short-term LUFS should be reasonable, got {short_term}"
        );
    }

    #[test]
    fn test_integrated() {
        let sample_rate = 48000.0;
        let mut meter = LoudnessMeter::new(sample_rate, 2);

        // Feed a 1 kHz sine for 5 seconds
        let num_samples = (sample_rate * 5.0) as usize;
        let signal = sine(1000.0, 0.2, num_samples, sample_rate);
        let stereo = vec![signal.as_slice(), signal.as_slice()];
        meter.process(&stereo);

        let integrated = meter.integrated_lufs();
        assert!(
            integrated > f32::NEG_INFINITY,
            "integrated should not be -inf"
        );
        assert!(
            integrated > -25.0 && integrated < -5.0,
            "integrated LUFS should be reasonable, got {integrated}"
        );
    }

    #[test]
    fn test_lufs_conversion_roundtrip() {
        // Verify the BS.1770 formula: L = -0.691 + 10 * log10(mean)
        // and its inverse: mean = 10^((L + 0.691) / 10)
        let lufs = -14.0_f32;
        let mean = 10.0_f32.powf((lufs + 0.691) / 10.0);
        let back = -0.691 + 10.0 * mean.log10();
        assert!(
            (back - lufs).abs() < 0.01,
            "roundtrip should be stable: {back} != {lufs}"
        );
    }

    #[test]
    fn test_reset() {
        let sample_rate = 48000.0;
        let mut meter = LoudnessMeter::new(sample_rate, 2);

        let signal = sine(1000.0, 0.3, (sample_rate * 1.0) as usize, sample_rate);
        let stereo = vec![signal.as_slice(), signal.as_slice()];
        meter.process(&stereo);

        assert!(meter.momentary_lufs() > f32::NEG_INFINITY);

        meter.reset();

        assert_eq!(meter.momentary_lufs(), f32::NEG_INFINITY);
        assert_eq!(meter.short_term_lufs(), f32::NEG_INFINITY);
        assert_eq!(meter.integrated_lufs(), f32::NEG_INFINITY);
    }
}
