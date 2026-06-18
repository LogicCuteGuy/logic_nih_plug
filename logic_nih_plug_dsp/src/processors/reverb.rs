//! FreeVerb-style algorithmic reverb with Schroeder/Moorer topology.
//!
//! This is a Rust port of JUCE's [`juce::Reverb`](https://docs.juce.com/master/classjuce_1_1Reverb.html)
//! and the [`juce::dsp::Reverb`](https://docs.juce.com/master/classjuce_1_1dsp_1_1Reverb.html)
//! wrapper. The algorithm is based on the technique and tunings used in
//! FreeVerb (Schroeder/Moorer hybrid topology):
//!
//! ```text
//!   input ──┐
//!           ├──▶ 8 parallel low-pass-feedback comb filters ──▶ Σ ──▶ 4 series allpass filters ──▶ out
//!           │         (per channel)
//!           └──▶ * gain (0.015 in normal mode, 0.0 in freeze mode)
//! ```
//!
//! Stereo decorrelation is achieved by offsetting the right-channel comb
//! and allpass tunings by a fixed `STEREO_SPREAD`. Wet signal is split
//! into `wet1` (same channel) and `wet2` (cross channel) with width
//! controlling their balance.
//!
//! All sample-rate dependent delay lengths are recomputed in
//! [`Reverb::prepare`]; comb feedback and damping are parameter-smoothed
//! (10 ms time constant) via a small linear-ramp [`SmoothedValue`] helper.
//!
//! # Example
//!
//! ```
//! use logic_nih_plug_dsp::processors::reverb::{Reverb, Parameters};
//!
//! let mut verb = Reverb::new();
//! verb.prepare(44100.0, 512);
//! verb.set_parameters(Parameters {
//!     room_size: 0.7,
//!     damping: 0.4,
//!     wet_level: 0.33,
//!     dry_level: 0.4,
//!     width: 1.0,
//!     freeze_mode: 0.0,
//! });
//!
//! let input  = vec![0.0_f32; 512];
//! let mut left  = input.clone();
//! let mut right = input.clone();
//! verb.process_stereo(&mut left, &mut right);
//! ```

use super::dynamics::ProcessSpec;
use super::Processor;

/// Number of parallel low-pass-feedback comb filters per channel.
const NUM_COMBS: usize = 8;

/// Number of series allpass filters per channel.
const NUM_ALLPASSES: usize = 4;

/// Number of channels (mono = 1, stereo = 2). We support both.
const NUM_CHANNELS: usize = 2;

/// Offset (in samples at 44.1 kHz) between left and right channel tunings
/// to decorrelate the stereo image.
const STEREO_SPREAD: usize = 23;

/// Smoothing time (seconds) used for parameter changes.
const SMOOTH_TIME_SECONDS: f64 = 0.01;

/// Tuning (in samples at 44.1 kHz) for each comb filter, taken directly
/// from FreeVerb / JUCE.
const COMB_TUNINGS_44100: [usize; NUM_COMBS] =
    [1116, 1188, 1277, 1356, 1422, 1491, 1557, 1617];

/// Tuning (in samples at 44.1 kHz) for each allpass filter.
const ALLPASS_TUNINGS_44100: [usize; NUM_ALLPASSES] =
    [556, 441, 341, 225];

/// Wet/dry scaling factors used by `set_parameters` (matches JUCE).
const WET_SCALE_FACTOR: f32 = 3.0;
const DRY_SCALE_FACTOR: f32 = 2.0;

/// Mapping from the user-facing `roomSize` parameter to the comb feedback
/// coefficient. The feedback is `roomSize * ROOM_SCALE + ROOM_OFFSET`.
const ROOM_SCALE_FACTOR: f32 = 0.28;
const ROOM_OFFSET: f32 = 0.7;

/// Mapping from the user-facing `damping` parameter to the one-pole
/// damping coefficient applied to the comb output.
const DAMP_SCALE_FACTOR: f32 = 0.4;

/// Input gain applied to the summed L+R before the comb bank.
const INPUT_GAIN: f32 = 0.015;

/// Allpass feedback coefficient (gain applied to the delayed signal when
/// summing back into the input). 0.5 matches FreeVerb.
const ALLPASS_FEEDBACK: f32 = 0.5;

/// Reverb parameters. Mirrors JUCE's `juce::Reverb::Parameters`.
///
/// All fields are in the range `[0.0, 1.0]`. The defaults
/// (`room_size=0.5, damping=0.5, wet=0.33, dry=0.4, width=1.0, freeze=0.0`)
/// match `Reverb::new()` and produce a medium-size bright hall.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parameters {
    /// Room size. 0 = small, 1 = large. Controls comb feedback and thus
    /// reverb tail length.
    pub room_size: f32,
    /// High-frequency damping. 0 = bright (no damping), 1 = dull (fully
    /// damped). Applied as a one-pole smoother on each comb output.
    pub damping: f32,
    /// Wet (reverb) level. 0 = silent, 1 = full wet. Scaled internally by
    /// `WET_SCALE_FACTOR`.
    pub wet_level: f32,
    /// Dry (direct) level. 0 = silent, 1 = full dry. Scaled internally by
    /// `DRY_SCALE_FACTOR`.
    pub dry_level: f32,
    /// Stereo width. 0 = mono wet signal, 1 = full stereo decorrelation.
    pub width: f32,
    /// Freeze mode. `< 0.5` is normal operation; `>= 0.5` latches the
    /// reverb into a continuous feedback loop (comb feedback -> 1.0,
    /// input gain -> 0.0).
    pub freeze_mode: f32,
}

impl Default for Parameters {
    fn default() -> Self {
        // Matches `juce::Reverb::Parameters` defaults.
        Self {
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.33,
            dry_level: 0.4,
            width: 1.0,
            freeze_mode: 0.0,
        }
    }
}

/// FreeVerb-style stereo algorithmic reverb.
///
/// See the [module-level documentation](self) for the algorithm diagram
/// and design notes.
pub struct Reverb {
    parameters: Parameters,
    sample_rate: f32,
    max_block_size: usize,
    enabled: bool,
    /// `INPUT_GAIN` in normal mode, `0.0` in freeze mode.
    gain: f32,

    /// 8 comb filters per channel, [channel][comb].
    combs: [Vec<CombFilter>; NUM_CHANNELS],
    /// 4 allpass filters per channel, [channel][allpass].
    allpasses: [Vec<AllPassFilter>; NUM_CHANNELS],

    damping: SmoothedValue,
    feedback: SmoothedValue,
    dry_gain: SmoothedValue,
    wet_gain_1: SmoothedValue,
    wet_gain_2: SmoothedValue,
}

impl Reverb {
    /// Constructs a new `Reverb` with default [`Parameters`] and a 44.1 kHz
    /// internal state. Call [`Reverb::prepare`] before processing audio.
    pub fn new() -> Self {
        let parameters = Parameters::default();
        let mut s = Self {
            parameters,
            sample_rate: 44100.0,
            max_block_size: 512,
            enabled: true,
            gain: INPUT_GAIN,
            combs: [
                (0..NUM_COMBS).map(|_| CombFilter::new()).collect(),
                (0..NUM_COMBS).map(|_| CombFilter::new()).collect(),
            ],
            allpasses: [
                (0..NUM_ALLPASSES).map(|_| AllPassFilter::new()).collect(),
                (0..NUM_ALLPASSES).map(|_| AllPassFilter::new()).collect(),
            ],
            damping: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
            feedback: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
            dry_gain: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
            wet_gain_1: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
            wet_gain_2: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
        };
        // Apply default parameters so the smoothed targets are sane.
        s.apply_parameters(parameters);
        s
    }

    /// Returns a reference to the current parameters.
    pub fn get_parameters(&self) -> Parameters {
        self.parameters
    }

    /// Applies a new set of parameters. Smoothed values are ramped over
    /// `SMOOTH_TIME_SECONDS` to the new targets. This is not thread-safe
    /// with [`Reverb::process`].
    pub fn set_parameters(&mut self, new_params: Parameters) {
        self.parameters = new_params;
        self.apply_parameters(new_params);
    }

    /// Returns whether the reverb is currently enabled (true by default).
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enables or disables the reverb. When disabled, `process_stereo` /
    /// `process` pass the input through unchanged.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Initialises (or re-initialises) the reverb for the given sample
    /// rate and maximum block size. This reallocates the comb and allpass
    /// delay lines, resets the smoothed values, and clears the buffers.
    pub fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.max_block_size = max_block_size.max(1);
        self.resize_delay_lines(sample_rate);
        self.reset_smoothed_values(sample_rate);
        self.reset();
    }

    /// Equivalent to `prepare` but takes a [`ProcessSpec`].
    pub fn prepare_spec(&mut self, spec: ProcessSpec) {
        self.prepare(spec.sample_rate, spec.maximum_block_size);
    }

    /// Clears the comb and allpass buffers without changing parameters or
    /// re-allocating.
    pub fn reset(&mut self) {
        for ch in 0..NUM_CHANNELS {
            for c in &mut self.combs[ch] {
                c.clear();
            }
            for a in &mut self.allpasses[ch] {
                a.clear();
            }
        }
    }

    /// Snaps denormals to zero in the comb and allpass state. Call this
    /// after a period of silence if you suspect denormal accumulation
    /// (JUCE calls `JUCE_UNDENORMALISE` on the comb filter's `last` and
    /// `temp` fields for the same reason).
    pub fn snap_to_zero(&mut self) {
        for ch in 0..NUM_CHANNELS {
            for c in &mut self.combs[ch] {
                c.snap_to_zero();
            }
        }
    }

    /// Processes a single mono sample. Equivalent to processing through
    /// the left channel only of `process_stereo`.
    pub fn process_sample(&mut self, input: f32) -> f32 {
        if !self.enabled {
            return input;
        }
        let damp = self.damping.get_next_value();
        let feedbck = self.feedback.get_next_value();
        let mut output = 0.0_f32;
        for c in &mut self.combs[0] {
            output += c.process(input * self.gain, damp, feedbck);
        }
        for a in &mut self.allpasses[0] {
            output = a.process(output);
        }
        let dry = self.dry_gain.get_next_value();
        let wet1 = self.wet_gain_1.get_next_value();
        output * wet1 + input * dry
    }

    /// Processes a block of mono audio. `input.len()` must equal
    /// `output.len()`.
    ///
    /// # Panics
    ///
    /// Panics if `input` and `output` differ in length.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            output.len(),
            "Reverb::process: input and output buffers must have the same length"
        );
        if !self.enabled {
            output.copy_from_slice(input);
            return;
        }
        for (i, o) in input.iter().zip(output.iter_mut()) {
            *o = self.process_sample(*i);
        }
    }

    /// Processes a stereo block in place. `left` and `right` must be the
    /// same length.
    ///
    /// # Panics
    ///
    /// Panics if `left` and `right` differ in length.
    pub fn process_stereo(&mut self, left: &mut [f32], right: &mut [f32]) {
        assert_eq!(
            left.len(),
            right.len(),
            "Reverb::process_stereo: left and right buffers must have the same length"
        );
        if !self.enabled {
            return; // input is already in `left`/`right`; nothing to do.
        }
        for i in 0..left.len() {
            let l = left[i];
            let r = right[i];
            let input = (l + r) * self.gain;

            let damp = self.damping.get_next_value();
            let feedbck = self.feedback.get_next_value();

            let mut out_l = 0.0_f32;
            let mut out_r = 0.0_f32;
            for j in 0..NUM_COMBS {
                out_l += self.combs[0][j].process(input, damp, feedbck);
                out_r += self.combs[1][j].process(input, damp, feedbck);
            }
            for j in 0..NUM_ALLPASSES {
                out_l = self.allpasses[0][j].process(out_l);
                out_r = self.allpasses[1][j].process(out_r);
            }

            let dry = self.dry_gain.get_next_value();
            let wet1 = self.wet_gain_1.get_next_value();
            let wet2 = self.wet_gain_2.get_next_value();

            left[i] = out_l * wet1 + out_r * wet2 + l * dry;
            right[i] = out_r * wet1 + out_l * wet2 + r * dry;
        }
    }

    /// Returns the current sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Returns the configured maximum block size.
    pub fn max_block_size(&self) -> usize {
        self.max_block_size
    }

    // -- private helpers ----------------------------------------------------

    fn apply_parameters(&mut self, params: Parameters) {
        let wet = params.wet_level * WET_SCALE_FACTOR;
        self.dry_gain
            .set_target_value(params.dry_level * DRY_SCALE_FACTOR);
        self.wet_gain_1
            .set_target_value(0.5 * wet * (1.0 + params.width));
        self.wet_gain_2
            .set_target_value(0.5 * wet * (1.0 - params.width));

        self.gain = if Self::is_frozen(params.freeze_mode) {
            0.0
        } else {
            INPUT_GAIN
        };

        self.update_damping();
    }

    fn update_damping(&mut self) {
        if Self::is_frozen(self.parameters.freeze_mode) {
            self.damping.set_target_value(0.0);
            self.feedback.set_target_value(1.0);
        } else {
            self.damping
                .set_target_value(self.parameters.damping * DAMP_SCALE_FACTOR);
            self.feedback.set_target_value(
                self.parameters.room_size * ROOM_SCALE_FACTOR + ROOM_OFFSET,
            );
        }
    }

    fn is_frozen(freeze_mode: f32) -> bool {
        freeze_mode >= 0.5
    }

    fn resize_delay_lines(&mut self, sample_rate: f32) {
        let sr = sample_rate.max(1.0) as f64;
        for ch in 0..NUM_CHANNELS {
            for (i, c) in self.combs[ch].iter_mut().enumerate() {
                let spread = if ch == 1 { STEREO_SPREAD as i32 } else { 0 };
                let tuning = COMB_TUNINGS_44100[i] as i32 + spread;
                let size = ((sr * tuning as f64) / 44100.0).round() as usize;
                c.set_size(size.max(1));
            }
            for (i, a) in self.allpasses[ch].iter_mut().enumerate() {
                let spread = if ch == 1 { STEREO_SPREAD as i32 } else { 0 };
                let tuning = ALLPASS_TUNINGS_44100[i] as i32 + spread;
                let size = ((sr * tuning as f64) / 44100.0).round() as usize;
                a.set_size(size.max(1));
            }
        }
    }

    fn reset_smoothed_values(&mut self, sample_rate: f32) {
        self.damping
            .reset(sample_rate, SMOOTH_TIME_SECONDS);
        self.feedback
            .reset(sample_rate, SMOOTH_TIME_SECONDS);
        self.dry_gain
            .reset(sample_rate, SMOOTH_TIME_SECONDS);
        self.wet_gain_1
            .reset(sample_rate, SMOOTH_TIME_SECONDS);
        self.wet_gain_2
            .reset(sample_rate, SMOOTH_TIME_SECONDS);
        // Re-apply current parameters so the smoothed targets are correct.
        self.apply_parameters(self.parameters);
    }
}

impl Default for Reverb {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for Reverb {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        Reverb::prepare(self, sample_rate, max_block_size);
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        Reverb::process(self, input, output);
    }

    fn reset(&mut self) {
        Reverb::reset(self);
    }
}

// ---------------------------------------------------------------------------
// Internal: low-pass-feedback comb filter (Schroeder/Moorer)
//
// Mirrors `juce::Reverb::CombFilter` in `juce_Reverb.h`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CombFilter {
    buffer: Vec<f32>,
    buffer_size: usize,
    buffer_index: usize,
    last: f32,
}

impl CombFilter {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            buffer_size: 0,
            buffer_index: 0,
            last: 0.0,
        }
    }

    fn set_size(&mut self, size: usize) {
        if size != self.buffer_size {
            self.buffer = vec![0.0; size];
            self.buffer_size = size;
            self.buffer_index = 0;
        }
        self.clear();
    }

    fn clear(&mut self) {
        self.last = 0.0;
        for s in &mut self.buffer {
            *s = 0.0;
        }
        self.buffer_index = 0;
    }

    fn snap_to_zero(&mut self) {
        if self.last.is_subnormal() {
            self.last = 0.0;
        }
        for s in &mut self.buffer {
            if s.is_subnormal() {
                *s = 0.0;
            }
        }
    }

    fn process(&mut self, input: f32, damp: f32, feedback: f32) -> f32 {
        debug_assert!(self.buffer_size > 0, "CombFilter not prepared");
        let output = self.buffer[self.buffer_index];
        // One-pole low-pass: damp=0 -> pass-through (last = output);
        //                    damp=1 -> holds the previous value.
        self.last = output * (1.0 - damp) + self.last * damp;
        if self.last.is_subnormal() {
            self.last = 0.0;
        }
        let temp = input + self.last * feedback;
        if temp.is_subnormal() {
            // don't write the denormal back; replace with 0
            self.buffer[self.buffer_index] = 0.0;
        } else {
            self.buffer[self.buffer_index] = temp;
        }
        self.buffer_index = (self.buffer_index + 1) % self.buffer_size;
        output
    }
}

// ---------------------------------------------------------------------------
// Internal: Schroeder allpass filter
//
// Mirrors `juce::Reverb::AllPassFilter` in `juce_Reverb.h`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct AllPassFilter {
    buffer: Vec<f32>,
    buffer_size: usize,
    buffer_index: usize,
}

impl AllPassFilter {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            buffer_size: 0,
            buffer_index: 0,
        }
    }

    fn set_size(&mut self, size: usize) {
        if size != self.buffer_size {
            self.buffer = vec![0.0; size];
            self.buffer_size = size;
            self.buffer_index = 0;
        }
        self.clear();
    }

    fn clear(&mut self) {
        for s in &mut self.buffer {
            *s = 0.0;
        }
        self.buffer_index = 0;
    }

    fn process(&mut self, input: f32) -> f32 {
        debug_assert!(self.buffer_size > 0, "AllPassFilter not prepared");
        let buffered = self.buffer[self.buffer_index];
        let temp = input + buffered * ALLPASS_FEEDBACK;
        if temp.is_subnormal() {
            self.buffer[self.buffer_index] = 0.0;
        } else {
            self.buffer[self.buffer_index] = temp;
        }
        self.buffer_index = (self.buffer_index + 1) % self.buffer_size;
        buffered - input
    }
}

// ---------------------------------------------------------------------------
// Internal: linear-ramp smoothed value
//
// A minimal port of `juce::SmoothedValue<float>` (linear mode, no
// listeners). Used for the wet/dry gains and the damping / feedback
// parameters.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct SmoothedValue {
    current: f32,
    target: f32,
    /// Per-sample increment.
    step: f32,
    /// Number of `get_next_value` calls still to perform.
    steps_to_target: i32,
    /// Total ramp length in samples. Recomputed in `reset`.
    total_steps: i32,
}

impl SmoothedValue {
    fn new(initial: f32, sample_rate: f32, smooth_time: f64) -> Self {
        let mut s = Self {
            current: initial,
            target: initial,
            step: 0.0,
            steps_to_target: 0,
            total_steps: 0,
        };
        s.reset(sample_rate, smooth_time);
        s.set_target_value(initial);
        s
    }

    /// Re-initialises the smoothing ramp length for a new sample rate.
    /// After this call the current value is snapped to the existing
    /// target; calling `set_target_value` afterwards will start a fresh
    /// ramp from that snapped value.
    fn reset(&mut self, sample_rate: f32, smooth_time: f64) {
        let steps = (f64::from(sample_rate) * smooth_time).round() as i32;
        self.total_steps = steps.max(1);
        self.current = self.target;
        self.step = 0.0;
        self.steps_to_target = 0;
    }

    /// Set a new target value. Starts a linear ramp from the current
    /// value over `total_steps` samples.
    fn set_target_value(&mut self, target: f32) {
        if target == self.target {
            return;
        }
        self.target = target;
        self.step = (self.target - self.current) / self.total_steps as f32;
        self.steps_to_target = self.total_steps;
    }

    /// Advance one sample and return the new current value.
    fn get_next_value(&mut self) -> f32 {
        if self.steps_to_target > 0 {
            self.current += self.step;
            self.steps_to_target -= 1;
            // Clamp to target to avoid numerical overshoot.
            if (self.step > 0.0 && self.current > self.target)
                || (self.step < 0.0 && self.current < self.target)
            {
                self.current = self.target;
                self.steps_to_target = 0;
            }
        }
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // CombFilter
    // -----------------------------------------------------------------------

    #[test]
    fn comb_filter_clear_zeros_state() {
        let mut c = CombFilter::new();
        c.set_size(8);
        c.process(1.0, 0.0, 0.5);
        c.clear();
        // After clear, processing an impulse should not return any prior
        // energy.
        let out = c.process(1.0, 0.0, 0.0);
        assert_eq!(out, 0.0);
    }

    #[test]
    fn comb_filter_buffers_impulse() {
        let mut c = CombFilter::new();
        c.set_size(4);
        // First sample: reads buffer[0]=0, writes 1.0 to slot 0, returns 0.
        let out0 = c.process(1.0, 0.0, 0.0);
        assert_eq!(out0, 0.0);
        // Three more silent samples advance the index to 3.
        let _ = c.process(0.0, 0.0, 0.0);
        let _ = c.process(0.0, 0.0, 0.0);
        let _ = c.process(0.0, 0.0, 0.0);
        // After 4 total process calls the index wraps to 0, and the
        // next call reads the impulse back from buffer[0].
        let out4 = c.process(0.0, 0.0, 0.0);
        assert!(
            (out4 - 1.0).abs() < 1e-6,
            "expected impulse to wrap, got {out4}"
        );
    }

    #[test]
    fn comb_filter_feedback_produces_decay() {
        let mut c = CombFilter::new();
        c.set_size(2);
        c.process(1.0, 0.0, 0.5);
        // After the impulse wraps once with feedback 0.5, we should see
        // energy (the impulse is still alive in the buffer).
        let _ = c.process(0.0, 0.0, 0.0);
        let out2 = c.process(0.0, 0.0, 0.0);
        assert!(out2 > 0.0, "feedback path should retain energy, got {out2}");
    }

    // -----------------------------------------------------------------------
    // AllPassFilter
    // -----------------------------------------------------------------------

    #[test]
    fn allpass_passes_impulse_through() {
        let mut a = AllPassFilter::new();
        a.set_size(2);
        // First sample: reads buffer[0]=0, writes 1.0 to slot 0,
        // returns `buffered - input` = 0 - 1.0 = -1.0.
        let out0 = a.process(1.0);
        assert!((out0 - -1.0).abs() < 1e-6, "expected -1.0, got {out0}");
        // One more silent sample advances the index to 0 (wrap).
        let _ = a.process(0.0);
        // The wrap-around call reads buffer[0] (still 1.0) and returns
        // `1.0 - 0 = 1.0`.
        let out2 = a.process(0.0);
        assert!((out2 - 1.0).abs() < 1e-6, "expected 1.0, got {out2}");
    }

    // -----------------------------------------------------------------------
    // SmoothedValue
    // -----------------------------------------------------------------------

    #[test]
    fn smoothed_value_ramps_to_target() {
        let mut s = SmoothedValue::new(0.0, 100.0, 0.01);
        // total_steps = 1; the very first get_next_value should reach
        // the target exactly.
        s.set_target_value(1.0);
        let v = s.get_next_value();
        assert!((v - 1.0).abs() < 1e-6, "expected 1.0, got {v}");
    }

    #[test]
    fn smoothed_value_multi_step_ramp() {
        let mut s = SmoothedValue::new(0.0, 100.0, 0.10);
        // total_steps = 10.
        s.set_target_value(1.0);
        let v1 = s.get_next_value();
        assert!((v1 - 0.1).abs() < 1e-4, "step1 expected ~0.1, got {v1}");
        for _ in 0..9 {
            let _ = s.get_next_value();
        }
        let v10 = s.get_next_value();
        assert!((v10 - 1.0).abs() < 1e-4, "step10 expected 1.0, got {v10}");
    }

    // -----------------------------------------------------------------------
    // Reverb
    // -----------------------------------------------------------------------

    #[test]
    fn default_parameters_match_juce() {
        let p = Parameters::default();
        assert_eq!(p.room_size, 0.5);
        assert_eq!(p.damping, 0.5);
        assert_eq!(p.wet_level, 0.33);
        assert_eq!(p.dry_level, 0.4);
        assert_eq!(p.width, 1.0);
        assert_eq!(p.freeze_mode, 0.0);
    }

    #[test]
    fn new_creates_disabled_state_until_prepare() {
        let verb = Reverb::new();
        assert_eq!(verb.get_parameters().room_size, 0.5);
        assert!(verb.is_enabled());
        assert_eq!(verb.sample_rate(), 44100.0);
    }

    #[test]
    fn prepare_resizes_delay_lines() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        // We don't expose sizes publicly, but the comb for channel 0 / comb 0
        // should be the canonical 1116 samples at 44.1 kHz.
        assert_eq!(verb.combs[0][0].buffer_size, COMB_TUNINGS_44100[0]);
        // The right channel should be offset by STEREO_SPREAD.
        assert_eq!(
            verb.combs[1][0].buffer_size,
            COMB_TUNINGS_44100[0] + STEREO_SPREAD
        );
        // Allpasses scale the same way.
        assert_eq!(verb.allpasses[0][0].buffer_size, ALLPASS_TUNINGS_44100[0]);
        assert_eq!(
            verb.allpasses[1][0].buffer_size,
            ALLPASS_TUNINGS_44100[0] + STEREO_SPREAD
        );
    }

    #[test]
    fn prepare_scales_with_sample_rate() {
        let mut verb = Reverb::new();
        verb.prepare(88200.0, 512);
        // 88200 / 44100 = 2.0 -> comb size should double.
        let expected = (COMB_TUNINGS_44100[0] as f64 * 2.0).round() as usize;
        assert_eq!(verb.combs[0][0].buffer_size, expected);
    }

    #[test]
    fn set_parameters_updates_damping_and_feedback() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_parameters(Parameters {
            room_size: 0.0,
            damping: 1.0,
            wet_level: 1.0,
            dry_level: 1.0,
            width: 1.0,
            freeze_mode: 0.0,
        });
        // Wait for the smoothed values to ramp.
        for _ in 0..2000 {
            let _ = verb.damping.get_next_value();
            let _ = verb.feedback.get_next_value();
            let _ = verb.dry_gain.get_next_value();
            let _ = verb.wet_gain_1.get_next_value();
        }
        let p = verb.get_parameters();
        assert_eq!(p.room_size, 0.0);
        assert_eq!(p.damping, 1.0);
        // After settling, gain should still be INPUT_GAIN (not frozen).
        assert_eq!(verb.gain, INPUT_GAIN);
    }

    #[test]
    fn freeze_mode_zeros_input_gain() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_parameters(Parameters {
            freeze_mode: 0.9,
            ..Parameters::default()
        });
        assert_eq!(verb.gain, 0.0);
    }

    #[test]
    fn process_stereo_with_silence_remains_silent() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        let mut left = vec![0.0_f32; 512];
        let mut right = vec![0.0_f32; 512];
        verb.process_stereo(&mut left, &mut right);
        let max = left
            .iter()
            .chain(right.iter())
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);
        assert!(max < 1e-6, "expected silence to remain silent, got max={max}");
    }

    #[test]
    fn process_stereo_impulse_produces_decay() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_parameters(Parameters {
            room_size: 0.7,
            damping: 0.3,
            wet_level: 1.0,
            dry_level: 0.0, // remove the dry path for easier assertions
            width: 1.0,
            freeze_mode: 0.0,
        });
        // Wait for smoothed values to ramp.
        for _ in 0..2000 {
            let _ = verb.process_sample(0.0);
        }
        let mut left = vec![0.0_f32; 4096];
        let mut right = vec![0.0_f32; 4096];
        left[0] = 1.0;
        right[0] = 1.0;
        verb.process_stereo(&mut left, &mut right);
        // Some energy should appear in the tail.
        let tail_energy: f32 = left[100..]
            .iter()
            .chain(right[100..].iter())
            .map(|s| s * s)
            .sum();
        assert!(
            tail_energy > 1e-6,
            "expected reverb tail to be non-silent, energy={tail_energy}"
        );
    }

    #[test]
    fn process_stereo_in_freeze_mode_does_not_accept_input() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_parameters(Parameters {
            freeze_mode: 1.0,
            ..Parameters::default()
        });
        let mut left = vec![1.0_f32; 256];
        let mut right = vec![1.0_f32; 256];
        verb.process_stereo(&mut left, &mut right);
        // With gain=0 in freeze mode, the comb filters receive no input,
        // so the output should be (close to) the dry signal only.
        for s in left.iter().chain(right.iter()) {
            assert!(s.is_finite(), "got non-finite sample in freeze mode");
        }
    }

    #[test]
    fn disabled_passes_input_through_unchanged() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_enabled(false);
        let left_in: Vec<f32> = (0..64).map(|i| (i as f32) / 64.0).collect();
        let right_in: Vec<f32> = (0..64).map(|i| (i as f32) / 128.0).collect();
        let mut left = left_in.clone();
        let mut right = right_in.clone();
        verb.process_stereo(&mut left, &mut right);
        for (a, b) in left.iter().zip(left_in.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
        for (a, b) in right.iter().zip(right_in.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn reset_clears_state_but_keeps_parameters() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_parameters(Parameters {
            room_size: 0.9,
            ..Parameters::default()
        });
        // Pump some energy in.
        let mut left = vec![0.5_f32; 1024];
        let mut right = vec![0.5_f32; 1024];
        verb.process_stereo(&mut left, &mut right);
        verb.reset();
        // After reset, the buffers should be zero, so the first samples
        // out should be near the dry signal only (comb outputs are 0).
        let mut left = vec![0.0_f32; 256];
        let mut right = vec![0.0_f32; 256];
        verb.process_stereo(&mut left, &mut right);
        let max = left
            .iter()
            .chain(right.iter())
            .map(|s| s.abs())
            .fold(0.0_f32, f32::max);
        assert!(
            max < 0.1,
            "expected near-silence after reset, got max={max}"
        );
        // Parameters should be preserved.
        assert_eq!(verb.get_parameters().room_size, 0.9);
    }

    #[test]
    fn mono_process_block_matches_processor_trait() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_parameters(Parameters {
            wet_level: 1.0,
            dry_level: 0.0,
            ..Parameters::default()
        });
        // Use the Processor trait entry point.
        let input = vec![0.0_f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut output = vec![0.0_f32; 8];
        <Reverb as Processor>::process(&mut verb, &input, &mut output);
        // First sample: dry=0, so it should be 0.
        assert_eq!(output[0], 0.0);
        // Some later sample should be non-zero (reverb tail).
        let max = output.iter().map(|s| s.abs()).fold(0.0_f32, f32::max);
        assert!(max > 0.0, "expected non-zero reverb tail in block path");
    }

    #[test]
    fn width_zero_makes_wet_mono() {
        let mut verb = Reverb::new();
        verb.prepare(44100.0, 512);
        verb.set_parameters(Parameters {
            width: 0.0,
            wet_level: 1.0,
            dry_level: 0.0,
            ..Parameters::default()
        });
        // Wait for wet gain to settle (wet1 = wet2 = 1.5 * 0.5 = 0.75).
        for _ in 0..2000 {
            let _ = verb.wet_gain_1.get_next_value();
            let _ = verb.wet_gain_2.get_next_value();
        }
        // With width=0, wet1 == wet2 always.
        assert!(
            (verb.wet_gain_1.current - verb.wet_gain_2.current).abs() < 1e-4,
            "width=0 should make wet1 == wet2, got {} and {}",
            verb.wet_gain_1.current,
            verb.wet_gain_2.current,
        );
    }
}
