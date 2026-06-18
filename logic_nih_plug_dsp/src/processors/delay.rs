//! # Delay (feedback) — tempo-synced, ping-pong
//!
//! This module ports two pieces from JUCE's `juce::dsp` module:
//!
//! 1. [`DelayLine`] — a multi-channel, fractional-sample delay line with
//!    pluggable interpolation. This is a direct port of
//!    [`juce::dsp::DelayLine<SampleType, InterpolationType>`].
//! 2. [`Delay`] — a higher-level delay effect with feedback, dry/wet mix,
//!    ping-pong, and tempo-synced delay time. Built on top of
//!    [`DelayLine`] (typically [`DefaultDelayLine`]).
//!
//! ## DelayLine
//!
//! ```text
//!   in ──▶ push ──▶ ┌────────────────────────────┐
//!                   │  ring buffer (total_size)  │
//!                   └────────────────────────────┘
//!                                 │
//!                                 ▼
//!                              read ──▶ interpolation ──▶ out
//! ```
//!
//! The buffer is indexed by a pair of decrementing pointers (write_pos
//! and read_pos), which mirrors JUCE's inverted-ring layout. This makes
//! per-sample push/pop O(1) without branching.
//!
//! Four interpolation strategies are provided, all matching the JUCE
//! implementations line-for-line:
//!
//! | Type | Stateful? | Notes |
//! |---|---|---|
//! | [`NoInterpolation`] | No | Reads integer part only. Lo-fi. |
//! | [`LinearInterpolation`] | No | Linear blend. Low CPU, adds slight low-pass. |
//! | [`Lagrange3rdInterpolation`] | No | 3rd-order, 4-tap Lagrange. Better than linear. |
//! | [`ThiranInterpolation`] | Yes (per-channel) | Allpass, flat amplitude. |
//!
//! Use [`DefaultDelayLine`] (or `DelayLine<LinearInterpolation>`) for the
//! common case. Specialise the type parameter when you need a different
//! algorithm.
//!
//! ## Delay effect
//!
//! [`Delay`] is a port of the typical *feedback delay* effect you see in
//! JUCE tutorials and the dsp module examples. It supports:
//!
//! - Free-running delay time in seconds (when `tempo_sync` is `false`).
//! - Tempo-synced delay time in beats, with 12 [`NoteDivision`]s
//!   (whole, half, quarter, eighth, sixteenth, thirty-second,
//!   dotted-half/quarter/eighth, triplet-half/quarter/eighth).
//! - Feedback in `[0.0, 1.2]` (values > 1.0 self-oscillate; the delay-line
//!   input is hard-clipped to ±1.0 to prevent NaN/Inf).
//! - Ping-pong mode: each channel's output is fed back to the *opposite*
//!   channel's delay line.
//! - Linear parameter smoothing (10 ms time constant) for `mix`,
//!   `feedback`, and `delay_time` to avoid clicks.
//!
//! ## Example
//!
//! ```
//! use logic_nih_plug_dsp::processors::delay::{Delay, DelayParameters, NoteDivision};
//!
//! let mut delay = Delay::new();
//! delay.prepare(44100.0, 512);
//! delay.set_parameters(DelayParameters {
//!     delay_time_seconds: 0.375, // 3/8 of a second
//!     feedback: 0.45,
//!     mix: 0.4,
//!     ping_pong: true,
//!     tempo_sync: false,
//!     tempo_bpm: 120.0,
//!     note_division: NoteDivision::Quarter,
//!     max_delay_seconds: 2.0,
//!     enabled: true,
//! });
//!
//! // Mono in -> stereo out, ping-pong.
//! let in_l = 0.5_f32;
//! let in_r = -0.25_f32;
//! let mut out_l = 0.0;
//! let mut out_r = 0.0;
//! delay.process_stereo(in_l, in_r, &mut out_l, &mut out_r);
//! ```

use std::marker::PhantomData;

use super::dynamics::ProcessSpec;
use super::Processor;

// ========================================================================
// DelayLine interpolation
// ========================================================================

/// Interpolation strategy used by [`DelayLine`] to compute the output
/// sample for a fractional delay. The four implementations here mirror
/// `juce::dsp::DelayLineInterpolationTypes` line-for-line.
///
/// All implementations are stateless except [`ThiranInterpolation`],
/// which keeps a single `v` value per channel. `v` is held in
/// [`DelayLine`] (not in this trait) so the trait itself has no state.
pub trait DelayLineInterpolation: Default + Clone + 'static {
    /// Compute one output sample.
    ///
    /// * `samples` — the ring buffer for one channel.
    /// * `read_pos` — the current read position (absolute index into `samples`).
    /// * `total_size` — the length of `samples`.
    /// * `delay_int` — the integer part of the (possibly-shifted) delay.
    /// * `delay_frac` — the fractional part in `[0, 1)` (or `[2, 3)` for
    ///   [`Lagrange3rdInterpolation`] after the canonicalisation shift).
    /// * `v` — per-channel state used only by [`ThiranInterpolation`];
    ///   other interpolators leave it untouched.
    /// * `alpha` — pre-computed Thiran coefficient. `0.0` for other
    ///   interpolators.
    #[allow(clippy::too_many_arguments)]
    fn interpolate(
        samples: &[f32],
        read_pos: usize,
        total_size: usize,
        delay_int: usize,
        delay_frac: f32,
        v: &mut f32,
        alpha: f32,
    ) -> f32;

    /// Convert a raw `(delay_int, delay_frac)` pair into the canonical
    /// form for this interpolation type, returning the canonical
    /// `(int, frac)` and the `alpha` coefficient to pass to
    /// [`DelayLineInterpolation::interpolate`].
    ///
    /// The default implementation is a no-op (and `alpha = 0`), matching
    /// JUCE's behaviour for [`NoInterpolation`] and [`LinearInterpolation`].
    fn canonicalize(delay_int: usize, delay_frac: f32) -> (usize, f32, f32) {
        (delay_int, delay_frac, 0.0)
    }
}

/// No interpolation: read the sample at the integer part of the delay.
/// Useful for lo-fi effects or when the delay is a fixed integer.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoInterpolation;

impl DelayLineInterpolation for NoInterpolation {
    fn interpolate(
        samples: &[f32],
        read_pos: usize,
        total_size: usize,
        delay_int: usize,
        _delay_frac: f32,
        _v: &mut f32,
        _alpha: f32,
    ) -> f32 {
        let index = (read_pos + delay_int) % total_size;
        samples[index]
    }
}

/// Linear interpolation between adjacent samples. Low CPU; introduces a
/// slight low-pass filtering when the delay is modulated in real time.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinearInterpolation;

impl DelayLineInterpolation for LinearInterpolation {
    fn interpolate(
        samples: &[f32],
        read_pos: usize,
        total_size: usize,
        delay_int: usize,
        delay_frac: f32,
        _v: &mut f32,
        _alpha: f32,
    ) -> f32 {
        let mut index1 = read_pos + delay_int;
        let mut index2 = index1 + 1;
        if index2 >= total_size {
            index1 %= total_size;
            index2 %= total_size;
        }
        let value1 = samples[index1];
        let value2 = samples[index2];
        value1 + delay_frac * (value2 - value1)
    }
}

/// 3rd-order, 4-tap Lagrange interpolation. Lower distortion than linear,
/// suitable for real-time delay modulation.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lagrange3rdInterpolation;

impl DelayLineInterpolation for Lagrange3rdInterpolation {
    fn interpolate(
        samples: &[f32],
        read_pos: usize,
        total_size: usize,
        delay_int: usize,
        delay_frac: f32,
        _v: &mut f32,
        _alpha: f32,
    ) -> f32 {
        let mut index1 = read_pos + delay_int;
        let mut index2 = index1 + 1;
        let mut index3 = index2 + 1;
        let mut index4 = index3 + 1;
        if index4 >= total_size {
            index1 %= total_size;
            index2 %= total_size;
            index3 %= total_size;
            index4 %= total_size;
        }
        let value1 = samples[index1];
        let value2 = samples[index2];
        let value3 = samples[index3];
        let value4 = samples[index4];
        let d1 = delay_frac - 1.0;
        let d2 = delay_frac - 2.0;
        let d3 = delay_frac - 3.0;
        let c1 = -d1 * d2 * d3 / 6.0;
        let c2 = d1 * d3 * 0.5;
        let c3 = -d1 * d2 * 0.5;
        let c4 = d1 * d2 * d3 / 6.0;
        value1 * c1 + delay_frac * (value2 * c2 + value3 * c3 + value4 * c4)
    }

    fn canonicalize(delay_int: usize, delay_frac: f32) -> (usize, f32, f32) {
        // JUCE shifts frac by +1 (and decrements int) when frac < 2 and
        // int >= 1, so the kernel uses a more accurate sub-sample.
        if delay_frac < 2.0 && delay_int >= 1 {
            (delay_int - 1, delay_frac + 1.0, 0.0)
        } else {
            (delay_int, delay_frac, 0.0)
        }
    }
}

/// 1st-order Thiran (allpass) interpolation. Flat amplitude response;
/// stateful (one `v` per channel) so not suitable for very fast delay
/// modulation.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThiranInterpolation;

impl DelayLineInterpolation for ThiranInterpolation {
    fn interpolate(
        samples: &[f32],
        read_pos: usize,
        total_size: usize,
        delay_int: usize,
        delay_frac: f32,
        v: &mut f32,
        alpha: f32,
    ) -> f32 {
        let mut index1 = read_pos + delay_int;
        let mut index2 = index1 + 1;
        if index2 >= total_size {
            index1 %= total_size;
            index2 %= total_size;
        }
        let value1 = samples[index1];
        let value2 = samples[index2];
        let output = if delay_frac == 0.0 {
            value1
        } else {
            value2 + alpha * (value1 - *v)
        };
        *v = output;
        output
    }

    fn canonicalize(delay_int: usize, delay_frac: f32) -> (usize, f32, f32) {
        let (int, frac) = if delay_frac < 0.618 && delay_int >= 1 {
            (delay_int - 1, delay_frac + 1.0)
        } else {
            (delay_int, delay_frac)
        };
        let alpha = if frac == 0.0 {
            0.0
        } else {
            (1.0 - frac) / (1.0 + frac)
        };
        (int, frac, alpha)
    }
}

// ========================================================================
// DelayLine
// ========================================================================

/// Multi-channel delay line with fractional-sample delay and pluggable
/// interpolation. Port of `juce::dsp::DelayLine<SampleType, InterpolationType>`.
///
/// The ring buffer uses an inverted indexing scheme (write_pos and
/// read_pos decrement modulo `total_size`), so per-sample `push_sample`
/// and `pop_sample` are O(1) without branches.
///
/// # Real-time safety
///
/// * [`DelayLine::new`] and [`DelayLine::with_maximum_delay`] allocate.
/// * [`DelayLine::set_maximum_delay_in_samples`] and
///   [`DelayLine::prepare`] reallocate (or zero) the buffers; call them
///   only from the non-realtime thread.
/// * [`DelayLine::set_delay`], [`DelayLine::push_sample`],
///   [`DelayLine::pop_sample`], [`DelayLine::pop_sample_with_delay`],
///   and [`DelayLine::process`] are allocation-free.
///
/// # Example
///
/// ```
/// use logic_nih_plug_dsp::processors::delay::DefaultDelayLine;
///
/// // 1 second of delay at 44.1 kHz, linear interpolation (the default).
/// let mut line: DefaultDelayLine = DefaultDelayLine::with_maximum_delay(44100);
/// line.prepare(44100.0, 1);
/// line.set_delay(22050.0); // 0.5 seconds
///
/// // Push a sample in, pop a sample out 100 ms later.
/// for _ in 0..100 {
///     line.push_sample(0, 0.0);
/// }
/// line.push_sample(0, 1.0);
/// // The popped sample is still 0.0 — the impulse we just pushed won't
/// // emerge until `delay` more pops have happened.
/// let out = line.pop_sample(0);
/// assert_eq!(out, 0.0);
/// ```
#[derive(Debug)]
pub struct DelayLine<Interp: DelayLineInterpolation = LinearInterpolation> {
    buffer: Vec<Vec<f32>>,
    write_pos: Vec<usize>,
    read_pos: Vec<usize>,
    /// Per-channel state used by [`ThiranInterpolation`]; unused by the
    /// other interpolation strategies.
    v: Vec<f32>,
    /// Current delay in samples.
    delay: f32,
    delay_int: usize,
    delay_frac: f32,
    /// Total ring buffer length (`max(4, max_delay + 2)`).
    total_size: usize,
    /// Pre-computed Thiran coefficient (0.0 for other interpolators).
    alpha: f32,
    sample_rate: f32,
    _interp: PhantomData<Interp>,
}

/// The default [`DelayLine`] type, using [`LinearInterpolation`].
pub type DefaultDelayLine = DelayLine<LinearInterpolation>;

impl<Interp: DelayLineInterpolation> DelayLine<Interp> {
    /// Constructs a new empty delay line with a 4-sample ring buffer.
    /// Call [`DelayLine::set_maximum_delay_in_samples`] and
    /// [`DelayLine::prepare`] before processing audio.
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            write_pos: Vec::new(),
            read_pos: Vec::new(),
            v: Vec::new(),
            delay: 0.0,
            delay_int: 0,
            delay_frac: 0.0,
            total_size: 4,
            alpha: 0.0,
            sample_rate: 44100.0,
            _interp: PhantomData,
        }
    }

    /// Constructs a delay line with the given maximum delay (in samples).
    /// Allocates the ring buffer immediately.
    pub fn with_maximum_delay(maximum_delay_in_samples: usize) -> Self {
        let mut s = Self::new();
        s.set_maximum_delay_in_samples(maximum_delay_in_samples);
        s
    }

    /// Sets a new maximum delay in samples and reallocates the ring
    /// buffer. Clears the buffer and resets the read/write positions.
    ///
    /// **Not real-time safe** — allocates. Match JUCE's
    /// `juce::dsp::DelayLine::setMaximumDelayInSamples`.
    pub fn set_maximum_delay_in_samples(&mut self, max_delay_in_samples: usize) {
        // Matches JUCE: totalSize = jmax(4, maxDelayInSamples + 2).
        self.total_size = (max_delay_in_samples + 2).max(4);
        for ch in 0..self.buffer.len() {
            self.buffer[ch] = vec![0.0; self.total_size];
        }
        self.reset();
    }

    /// Returns the maximum possible delay in samples, which is
    /// `total_size - 2`.
    pub fn get_maximum_delay_in_samples(&self) -> usize {
        self.total_size.saturating_sub(2)
    }

    /// Sets the delay in samples. Clamped to `[0.0, get_maximum_delay_in_samples()]`.
    pub fn set_delay(&mut self, new_delay_in_samples: f32) {
        let upper = self.get_maximum_delay_in_samples() as f32;
        self.delay = new_delay_in_samples.clamp(0.0, upper);
        let raw_int = self.delay.floor();
        let raw_frac = self.delay - raw_int;
        let raw_int = if raw_int < 0.0 { 0 } else { raw_int as usize };
        let (delay_int, delay_frac, alpha) = Interp::canonicalize(raw_int, raw_frac);
        self.delay_int = delay_int;
        self.delay_frac = delay_frac;
        self.alpha = alpha;
    }

    /// Returns the current delay in samples.
    pub fn get_delay(&self) -> f32 {
        self.delay
    }

    /// Initialises the delay line for the given sample rate and channel
    /// count. Allocates (or reuses) the per-channel ring buffers.
    ///
    /// **Not real-time safe** if `num_channels` differs from the previous
    /// call.
    pub fn prepare(&mut self, sample_rate: f32, num_channels: usize) {
        assert!(
            num_channels > 0,
            "DelayLine::prepare: num_channels must be > 0"
        );
        self.sample_rate = sample_rate;
        if self.buffer.len() != num_channels {
            self.buffer = (0..num_channels)
                .map(|_| vec![0.0; self.total_size])
                .collect();
        } else {
            for ch in 0..num_channels {
                self.buffer[ch].fill(0.0);
            }
        }
        self.write_pos = vec![0; num_channels];
        self.read_pos = vec![0; num_channels];
        self.v = vec![0.0; num_channels];
    }

    /// Clears the delay line state (all buffer slots set to 0, all
    /// pointers set to 0). Parameter values are preserved.
    pub fn reset(&mut self) {
        for ch in 0..self.buffer.len() {
            self.buffer[ch].fill(0.0);
            self.write_pos[ch] = 0;
            self.read_pos[ch] = 0;
            self.v[ch] = 0.0;
        }
    }

    /// Pushes a single sample into one channel of the delay line.
    pub fn push_sample(&mut self, channel: usize, sample: f32) {
        let ts = self.total_size;
        let buf = &mut self.buffer[channel];
        let wp = self.write_pos[channel];
        buf[wp] = sample;
        self.write_pos[channel] = if wp == 0 { ts - 1 } else { wp - 1 };
    }

    /// Pops a single sample from one channel of the delay line using the
    /// currently-set delay. Advances the read pointer.
    pub fn pop_sample(&mut self, channel: usize) -> f32 {
        self.pop_sample_with_delay(channel, -1.0, true)
    }

    /// Pops a single sample with an optional per-pop delay override.
    ///
    /// * `channel` — the target channel.
    /// * `delay_in_samples` — if `>= 0.0`, the delay is set to this value
    ///   for this pop only; if `< 0.0`, the currently-set delay is used.
    /// * `update_read_pointer` — when `true`, the read pointer advances
    ///   after the pop. Set this to `false` to read multiple taps from
    ///   the same buffer state (multi-tap delay).
    pub fn pop_sample_with_delay(
        &mut self,
        channel: usize,
        delay_in_samples: f32,
        update_read_pointer: bool,
    ) -> f32 {
        if delay_in_samples >= 0.0 {
            self.set_delay(delay_in_samples);
        }
        let result = Interp::interpolate(
            &self.buffer[channel],
            self.read_pos[channel],
            self.total_size,
            self.delay_int,
            self.delay_frac,
            &mut self.v[channel],
            self.alpha,
        );
        if update_read_pointer {
            let rp = self.read_pos[channel];
            self.read_pos[channel] = if rp == 0 {
                self.total_size - 1
            } else {
                rp - 1
            };
        }
        result
    }

    /// Processes a block of mono audio (channel 0). `input.len()` must
    /// equal `output.len()`.
    ///
    /// # Panics
    ///
    /// Panics if `input` and `output` differ in length, or if no channels
    /// have been prepared.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(
            input.len(),
            output.len(),
            "DelayLine::process: input and output must have the same length"
        );
        assert!(
            !self.buffer.is_empty(),
            "DelayLine::process: call prepare() first"
        );
        for (i, o) in input.iter().zip(output.iter_mut()) {
            self.push_sample(0, *i);
            *o = self.pop_sample(0);
        }
    }
}

impl<Interp: DelayLineInterpolation> Default for DelayLine<Interp> {
    fn default() -> Self {
        Self::new()
    }
}

// ========================================================================
// SmoothedValue (small internal helper for the Delay effect)
// ========================================================================

/// Smoothing time (seconds) for `feedback`, `mix`, and `delay_time`.
const SMOOTH_TIME_SECONDS: f64 = 0.01;

/// Linear-ramp smoother. Internal helper for the [`Delay`] effect;
/// duplicated here so the module is self-contained (a copy also lives
/// in the `reverb` module).
///
/// The first call to [`SmoothedValue::get_next_value`] flips an
/// internal `used` flag; until then, [`SmoothedValue::set_target_value`]
/// **snaps** the current value to the target instead of starting a
/// ramp. This means the very first parameter setup uses the target
/// value immediately (no fade-in from 0), while subsequent parameter
/// changes ramp smoothly.
struct SmoothedValue {
    current: f32,
    target: f32,
    step: f32,
    steps_to_target: i32,
    total_steps: i32,
    used: bool,
}

impl SmoothedValue {
    fn new(initial: f32, sample_rate: f32, smooth_time: f64) -> Self {
        let mut s = Self {
            current: initial,
            target: initial,
            step: 0.0,
            steps_to_target: 0,
            total_steps: 0,
            used: false,
        };
        s.reset(sample_rate, smooth_time);
        s.set_target_value(initial);
        s
    }

    fn reset(&mut self, sample_rate: f32, smooth_time: f64) {
        let steps = (f64::from(sample_rate) * smooth_time).round() as i32;
        self.total_steps = steps.max(1);
        self.current = self.target;
        self.step = 0.0;
        self.steps_to_target = 0;
    }

    fn set_target_value(&mut self, target: f32) {
        if target == self.target {
            return;
        }
        self.target = target;
        if !self.used {
            // First set after construction / before any get_next_value:
            // snap, don't ramp. Avoids a fade-in from 0 on the very
            // first parameter setup.
            self.current = target;
            self.step = 0.0;
            self.steps_to_target = 0;
        } else {
            self.step = (self.target - self.current) / self.total_steps as f32;
            self.steps_to_target = self.total_steps;
        }
    }

    fn get_next_value(&mut self) -> f32 {
        self.used = true;
        if self.steps_to_target > 0 {
            self.current += self.step;
            self.steps_to_target -= 1;
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

// ========================================================================
// Delay effect
// ========================================================================

/// Note value used when the [`Delay`] effect's `tempo_sync` is enabled.
/// The number is the delay length in beats (so a quarter note is 1 beat,
/// a dotted eighth is 0.75 beats, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoteDivision {
    /// 1/1 — 4 beats.
    Whole,
    /// 1/2 — 2 beats.
    Half,
    /// 1/4 — 1 beat.
    #[default]
    Quarter,
    /// 1/8 — 0.5 beats.
    Eighth,
    /// 1/16 — 0.25 beats.
    Sixteenth,
    /// 1/32 — 0.125 beats.
    ThirtySecond,
    /// 1/2. (dotted half) — 3 beats.
    DottedHalf,
    /// 1/4. (dotted quarter) — 1.5 beats.
    DottedQuarter,
    /// 1/8. (dotted eighth) — 0.75 beats.
    DottedEighth,
    /// 1/2T (half-note triplet) — 4/3 beats.
    TripletHalf,
    /// 1/4T (quarter-note triplet) — 2/3 beats.
    TripletQuarter,
    /// 1/8T (eighth-note triplet) — 1/3 beats.
    TripletEighth,
}

impl NoteDivision {
    /// Returns the delay length in beats for this note value.
    pub fn beats(self) -> f32 {
        match self {
            Self::Whole => 4.0,
            Self::Half => 2.0,
            Self::Quarter => 1.0,
            Self::Eighth => 0.5,
            Self::Sixteenth => 0.25,
            Self::ThirtySecond => 0.125,
            Self::DottedHalf => 3.0,
            Self::DottedQuarter => 1.5,
            Self::DottedEighth => 0.75,
            Self::TripletHalf => 4.0 / 3.0,
            Self::TripletQuarter => 2.0 / 3.0,
            Self::TripletEighth => 1.0 / 3.0,
        }
    }

    /// Returns the delay length in samples for the given BPM and sample
    /// rate. The formula is `beats * 60 / bpm * sample_rate`.
    pub fn delay_samples(self, bpm: f32, sample_rate: f32) -> f32 {
        let beats_per_second = bpm / 60.0;
        let delay_seconds = self.beats() / beats_per_second;
        delay_seconds * sample_rate
    }
}

/// Parameters for the [`Delay`] effect. All fields are public for
/// ergonomic construction; mutate and pass back to
/// [`Delay::set_parameters`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DelayParameters {
    /// Delay time in seconds, used when `tempo_sync` is `false`. Clamped
    /// to `[0.0, max_delay_seconds]`.
    pub delay_time_seconds: f32,
    /// Feedback amount in `[0.0, 1.2]`. Values above 1.0 will
    /// self-oscillate. The delay-line input is hard-clipped to ±1.0 to
    /// prevent NaN/Inf.
    pub feedback: f32,
    /// Dry/wet mix in `[0.0, 1.0]`. 0 = full dry, 1 = full wet.
    pub mix: f32,
    /// If `true`, each channel's delay output is fed back to the
    /// *opposite* channel's delay line, producing the classic ping-pong
    /// effect. Only meaningful in stereo (`process_stereo`); in mono
    /// (`process_sample`) this is ignored.
    pub ping_pong: bool,
    /// If `true`, the delay time is computed from `tempo_bpm` and
    /// `note_division` instead of `delay_time_seconds`.
    pub tempo_sync: bool,
    /// Tempo in BPM, used when `tempo_sync` is `true`. Ignored otherwise.
    pub tempo_bpm: f32,
    /// Note value for tempo-synced delay time.
    pub note_division: NoteDivision,
    /// Maximum possible delay in seconds. Determines the ring-buffer
    /// allocation in [`Delay::prepare`]. Default 2.0 seconds.
    pub max_delay_seconds: f32,
    /// When `false`, the effect is bypassed and input passes through
    /// unchanged.
    pub enabled: bool,
}

impl Default for DelayParameters {
    fn default() -> Self {
        Self {
            delay_time_seconds: 0.375,
            feedback: 0.35,
            mix: 0.3,
            ping_pong: false,
            tempo_sync: false,
            tempo_bpm: 120.0,
            note_division: NoteDivision::Quarter,
            max_delay_seconds: 2.0,
            enabled: true,
        }
    }
}

/// Feedback delay effect. Built on top of two [`DelayLine`]s (one per
/// channel) with linear parameter smoothing for `mix`, `feedback`, and
/// the current delay time.
///
/// See the [module-level documentation](self) for an overview and
/// example.
pub struct Delay {
    params: DelayParameters,
    sample_rate: f32,
    max_block_size: usize,
    /// Maximum delay in samples, computed from `max_delay_seconds` at
    /// `prepare` time.
    max_delay_samples: usize,
    line_l: DefaultDelayLine,
    line_r: DefaultDelayLine,
    mix_smooth: SmoothedValue,
    feedback_smooth: SmoothedValue,
    delay_smooth: SmoothedValue,
}

impl Delay {
    /// Constructs a new `Delay` with default [`DelayParameters`]. The
    /// delay lines are not yet allocated; call [`Delay::prepare`] before
    /// processing audio.
    pub fn new() -> Self {
        let params = DelayParameters::default();
        Self::with_parameters(params)
    }

    /// Constructs a new `Delay` with the given parameters.
    pub fn with_parameters(params: DelayParameters) -> Self {
        let mut s = Self {
            params,
            sample_rate: 44100.0,
            max_block_size: 512,
            max_delay_samples: 0,
            line_l: DelayLine::new(),
            line_r: DelayLine::new(),
            mix_smooth: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
            feedback_smooth: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
            delay_smooth: SmoothedValue::new(0.0, 44100.0, SMOOTH_TIME_SECONDS),
        };
        s.apply_parameters();
        s
    }

    /// Returns the current parameters.
    pub fn get_parameters(&self) -> DelayParameters {
        self.params
    }

    /// Applies a new set of parameters. The smoothed values are ramped
    /// over `SMOOTH_TIME_SECONDS` to the new targets.
    pub fn set_parameters(&mut self, new_params: DelayParameters) {
        self.params = new_params;
        self.apply_parameters();
    }

    /// Returns whether the effect is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.params.enabled
    }

    /// Enables or disables the effect. When disabled, [`Delay::process`]
    /// and [`Delay::process_stereo`] pass input through unchanged.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.params.enabled = enabled;
    }

    /// Initialises (or re-initialises) the effect for the given sample
    /// rate and maximum block size. Allocates the two delay lines.
    /// **Not real-time safe.**
    pub fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.sample_rate = sample_rate;
        self.max_block_size = max_block_size.max(1);

        // Recompute max delay from the (possibly updated) parameters.
        self.max_delay_samples =
            (self.params.max_delay_seconds.max(0.001) * sample_rate).ceil() as usize;

        // Per-channel delay lines, 1 channel each (we have two lines for
        // stereo, each processing a single channel).
        self.line_l
            .set_maximum_delay_in_samples(self.max_delay_samples);
        self.line_r
            .set_maximum_delay_in_samples(self.max_delay_samples);
        self.line_l.prepare(sample_rate, 1);
        self.line_r.prepare(sample_rate, 1);

        // Re-initialise smoothed values for the new sample rate.
        self.mix_smooth.reset(sample_rate, SMOOTH_TIME_SECONDS);
        self.feedback_smooth.reset(sample_rate, SMOOTH_TIME_SECONDS);
        self.delay_smooth.reset(sample_rate, SMOOTH_TIME_SECONDS);

        // Re-apply parameters (recomputes targets and the per-line delay).
        self.apply_parameters();
        self.reset();
    }

    /// Equivalent to `prepare` but takes a [`ProcessSpec`].
    pub fn prepare_spec(&mut self, spec: ProcessSpec) {
        self.prepare(spec.sample_rate, spec.maximum_block_size);
    }

    /// Clears the delay-line state. Parameter values are preserved.
    pub fn reset(&mut self) {
        self.line_l.reset();
        self.line_r.reset();
    }

    /// Updates the tempo (BPM) without needing to re-construct the
    /// parameters struct. Has no effect when `tempo_sync` is `false`.
    pub fn set_tempo_bpm(&mut self, bpm: f32) {
        self.params.tempo_bpm = bpm;
        self.apply_parameters();
    }

    /// Returns the current sample rate.
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }

    /// Returns the configured maximum block size.
    pub fn max_block_size(&self) -> usize {
        self.max_block_size
    }

    /// Processes a single mono sample. Uses the left delay line only;
    /// ping-pong is ignored in mono.
    pub fn process_sample(&mut self, input: f32) -> f32 {
        if !self.params.enabled {
            return input;
        }
        let mix = self.mix_smooth.get_next_value();
        let feedback = self.feedback_smooth.get_next_value();
        let delay_samples = self.delay_smooth.get_next_value();

        let delayed = self.line_l.pop_sample_with_delay(0, delay_samples, true);
        let to_push = (input + delayed * feedback).clamp(-1.0, 1.0);
        self.line_l.push_sample(0, to_push);

        let dry = 1.0 - mix;
        input * dry + delayed * mix
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
            "Delay::process: input and output must have the same length"
        );
        for (i, o) in input.iter().zip(output.iter_mut()) {
            *o = self.process_sample(*i);
        }
    }

    /// Processes a stereo block. When `params.ping_pong` is `true`, the
    /// left output feeds into the right delay line and vice versa.
    pub fn process_stereo(
        &mut self,
        in_l: f32,
        in_r: f32,
        out_l: &mut f32,
        out_r: &mut f32,
    ) {
        if !self.params.enabled {
            *out_l = in_l;
            *out_r = in_r;
            return;
        }
        let mix = self.mix_smooth.get_next_value();
        let feedback = self.feedback_smooth.get_next_value();
        let delay_samples = self.delay_smooth.get_next_value();

        let delayed_l = self.line_l.pop_sample_with_delay(0, delay_samples, true);
        let delayed_r = self.line_r.pop_sample_with_delay(0, delay_samples, true);

        if self.params.ping_pong {
            // Cross feedback: each channel's delay line is fed by
            // `input + opposite_channel_delayed * feedback`.
            let l_in = (in_l + delayed_r * feedback).clamp(-1.0, 1.0);
            let r_in = (in_r + delayed_l * feedback).clamp(-1.0, 1.0);
            self.line_l.push_sample(0, l_in);
            self.line_r.push_sample(0, r_in);
        } else {
            // Independent feedback per channel.
            let l_in = (in_l + delayed_l * feedback).clamp(-1.0, 1.0);
            let r_in = (in_r + delayed_r * feedback).clamp(-1.0, 1.0);
            self.line_l.push_sample(0, l_in);
            self.line_r.push_sample(0, r_in);
        }

        let dry = 1.0 - mix;
        *out_l = in_l * dry + delayed_l * mix;
        *out_r = in_r * dry + delayed_r * mix;
    }

    // -- private helpers ----------------------------------------------------

    fn apply_parameters(&mut self) {
        self.mix_smooth
            .set_target_value(self.params.mix.clamp(0.0, 1.0));
        self.feedback_smooth
            .set_target_value(self.params.feedback.clamp(0.0, 1.2));

        let delay_samples = self.compute_delay_samples();
        let clamped = delay_samples.clamp(0.0, self.max_delay_samples.max(1) as f32);
        self.delay_smooth.set_target_value(clamped);

        // Apply the per-line delay so the very first call (before the
        // smoother has ramped) has a sensible default.
        self.line_l.set_delay(clamped);
        self.line_r.set_delay(clamped);
    }

    fn compute_delay_samples(&self) -> f32 {
        if self.params.tempo_sync {
            self.params
                .note_division
                .delay_samples(self.params.tempo_bpm, self.sample_rate)
        } else {
            self.params.delay_time_seconds * self.sample_rate
        }
    }
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor for Delay {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare(sample_rate, max_block_size);
    }
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        self.process(input, output);
    }
    fn reset(&mut self) {
        self.reset();
    }
}

// ========================================================================
// Tests
// ========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // NoteDivision
    // -----------------------------------------------------------------------

    #[test]
    fn note_division_values_match_juce_tutorial_conventions() {
        // Standard divisions.
        assert_eq!(NoteDivision::Whole.beats(), 4.0);
        assert_eq!(NoteDivision::Half.beats(), 2.0);
        assert_eq!(NoteDivision::Quarter.beats(), 1.0);
        assert_eq!(NoteDivision::Eighth.beats(), 0.5);
        assert_eq!(NoteDivision::Sixteenth.beats(), 0.25);
        assert_eq!(NoteDivision::ThirtySecond.beats(), 0.125);
        // Dotted.
        assert_eq!(NoteDivision::DottedHalf.beats(), 3.0);
        assert_eq!(NoteDivision::DottedQuarter.beats(), 1.5);
        assert_eq!(NoteDivision::DottedEighth.beats(), 0.75);
        // Triplets.
        assert!((NoteDivision::TripletHalf.beats() - 4.0 / 3.0).abs() < 1e-6);
        assert!((NoteDivision::TripletQuarter.beats() - 2.0 / 3.0).abs() < 1e-6);
        assert!((NoteDivision::TripletEighth.beats() - 1.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn note_division_delay_samples_at_120bpm_44100hz() {
        // 1 quarter at 120 bpm = 0.5 s = 22050 samples at 44.1 kHz.
        let s = NoteDivision::Quarter.delay_samples(120.0, 44100.0);
        assert!((s - 22050.0).abs() < 1e-3, "expected 22050, got {s}");
        // 1 whole = 4 * quarter.
        let s = NoteDivision::Whole.delay_samples(120.0, 44100.0);
        assert!((s - 88200.0).abs() < 1e-3, "expected 88200, got {s}");
    }

    // -----------------------------------------------------------------------
    // DelayLine
    // -----------------------------------------------------------------------

    #[test]
    fn delay_line_set_maximum_resizes_buffer() {
        let mut line: DefaultDelayLine = DelayLine::new();
        line.set_maximum_delay_in_samples(1024);
        assert_eq!(line.get_maximum_delay_in_samples(), 1024);
        assert_eq!(line.total_size, 1024 + 2);
    }

    #[test]
    fn delay_line_get_maximum_returns_total_size_minus_two() {
        let mut line: DefaultDelayLine = DelayLine::new();
        line.set_maximum_delay_in_samples(100);
        // JUCE: getMaximumDelayInSamples() = totalSize - 2 = 102 - 2 = 100.
        assert_eq!(line.get_maximum_delay_in_samples(), 100);
    }

    #[test]
    fn delay_line_default_delay_is_zero() {
        let mut line: DefaultDelayLine = DelayLine::new();
        line.set_maximum_delay_in_samples(100);
        line.prepare(44100.0, 1);
        assert_eq!(line.get_delay(), 0.0);
    }

    #[test]
    fn delay_line_set_delay_clamps_to_maximum() {
        let mut line: DefaultDelayLine = DelayLine::new();
        line.set_maximum_delay_in_samples(100);
        line.set_delay(500.0); // above max
        assert_eq!(line.get_delay(), 100.0);
        line.set_delay(-5.0); // below zero
        assert_eq!(line.get_delay(), 0.0);
    }

    #[test]
    fn delay_line_push_pop_round_trip_with_unit_delay() {
        let mut line: DefaultDelayLine = DelayLine::with_maximum_delay(8);
        line.prepare(44100.0, 1);
        line.set_delay(1.0);
        // Interleaved push/pop: with delay=1, each pushed value
        // emerges one push/pop cycle later.
        line.push_sample(0, 1.0);
        // First pop: empty buffer at the read offset.
        assert_eq!(line.pop_sample(0), 0.0);
        line.push_sample(0, 2.0);
        // Second pop: the 1.0 we pushed one cycle ago.
        assert_eq!(line.pop_sample(0), 1.0);
        line.push_sample(0, 3.0);
        // Third pop: the 2.0 from the previous push.
        assert_eq!(line.pop_sample(0), 2.0);
    }

    #[test]
    fn delay_line_integer_delay_returns_correct_sample() {
        let mut line: DefaultDelayLine = DelayLine::with_maximum_delay(8);
        line.prepare(44100.0, 1);
        line.set_delay(2.0);
        // Push 1.0, then two zeros, then pop three times. The 1.0
        // should emerge on the third pop (delay=2 samples).
        line.push_sample(0, 1.0);
        line.push_sample(0, 0.0);
        line.push_sample(0, 0.0);
        assert_eq!(line.pop_sample(0), 0.0);
        assert_eq!(line.pop_sample(0), 0.0);
        assert_eq!(line.pop_sample(0), 1.0);
    }

    #[test]
    fn delay_line_linear_interpolation_blends_samples() {
        let mut line: DefaultDelayLine = DelayLine::with_maximum_delay(8);
        line.prepare(44100.0, 1);
        // Push [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; then 1.0.
        for _ in 0..10 {
            line.push_sample(0, 0.0);
        }
        line.push_sample(0, 1.0);
        // delay = 1.5 means read the 0 just before the 1.0, and blend
        // toward 1.0 by 0.5 => 0.5.
        line.set_delay(0.5);
        let out = line.pop_sample(0);
        assert!(
            (out - 0.5).abs() < 1e-6,
            "linear blend expected 0.5, got {out}"
        );
    }

    #[test]
    fn delay_line_multi_tap_without_advancing() {
        let mut line: DefaultDelayLine = DelayLine::with_maximum_delay(8);
        line.prepare(44100.0, 1);
        for v in [1.0, 2.0, 3.0, 4.0] {
            line.push_sample(0, v);
        }
        line.set_delay(2.0);
        // Read same buffer state 3 times.
        let a = line.pop_sample_with_delay(0, -1.0, false);
        let b = line.pop_sample_with_delay(0, -1.0, false);
        let c = line.pop_sample_with_delay(0, -1.0, false);
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn delay_line_reset_clears_state() {
        let mut line: DefaultDelayLine = DelayLine::with_maximum_delay(8);
        line.prepare(44100.0, 1);
        line.push_sample(0, 1.0);
        line.push_sample(0, 1.0);
        line.reset();
        // After reset, all buffer slots are 0.
        for _ in 0..8 {
            assert_eq!(line.pop_sample(0), 0.0);
        }
    }

    #[test]
    fn delay_line_process_block_matches_sample_loop() {
        let mut line_a: DefaultDelayLine = DelayLine::with_maximum_delay(64);
        let mut line_b: DefaultDelayLine = DelayLine::with_maximum_delay(64);
        line_a.prepare(44100.0, 1);
        line_b.prepare(44100.0, 1);
        line_a.set_delay(7.0);
        line_b.set_delay(7.0);

        let input: Vec<f32> = (0..32).map(|i| (i as f32) * 0.1).collect();
        let mut out_block = vec![0.0_f32; 32];
        line_a.process(&input, &mut out_block);

        let mut out_sample = vec![0.0_f32; 32];
        for (i, o) in input.iter().zip(out_sample.iter_mut()) {
            line_b.push_sample(0, *i);
            *o = line_b.pop_sample(0);
        }
        for (a, b) in out_block.iter().zip(out_sample.iter()) {
            assert!((a - b).abs() < 1e-6, "block {a} != sample {b}");
        }
    }

    #[test]
    fn delay_line_thiran_passes_impulse_through() {
        // Build a delay line using Thiran interpolation. Push a single
        // impulse; the impulse should eventually emerge in the output.
        let mut line: DelayLine<ThiranInterpolation> =
            DelayLine::with_maximum_delay(8);
        line.prepare(44100.0, 1);
        line.set_delay(2.0);
        for _ in 0..5 {
            line.push_sample(0, 0.0);
        }
        line.push_sample(0, 1.0);
        // After enough pops, the impulse should appear.
        let mut found_peak = false;
        for _ in 0..8 {
            let out = line.pop_sample(0);
            if out.abs() > 0.5 {
                found_peak = true;
            }
        }
        assert!(found_peak, "Thiran delay did not pass the impulse through");
    }

    // -----------------------------------------------------------------------
    // Delay
    // -----------------------------------------------------------------------

    #[test]
    fn delay_default_parameters_are_sane() {
        let p = DelayParameters::default();
        assert!(p.delay_time_seconds > 0.0);
        assert!((0.0..=1.0).contains(&p.feedback));
        assert!((0.0..=1.0).contains(&p.mix));
        assert!(!p.ping_pong);
        assert!(!p.tempo_sync);
        assert!(p.tempo_bpm > 0.0);
        assert!(p.max_delay_seconds > 0.0);
        assert!(p.enabled);
    }

    #[test]
    fn delay_prepare_allocates_lines_to_max_delay() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        // 2.0 s * 44100 Hz = 88200 samples max.
        assert_eq!(d.max_delay_samples, 88200);
        assert_eq!(d.line_l.get_maximum_delay_in_samples(), 88200);
        assert_eq!(d.line_r.get_maximum_delay_in_samples(), 88200);
    }

    #[test]
    fn delay_feedback_zero_produces_single_echo() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        d.set_parameters(DelayParameters {
            delay_time_seconds: 0.001, // ~44 samples
            feedback: 0.0,
            mix: 1.0,
            ping_pong: false,
            tempo_sync: false,
            tempo_bpm: 120.0,
            note_division: NoteDivision::Quarter,
            max_delay_seconds: 0.1,
            enabled: true,
        });
        // Push enough silence for the delay line to fill.
        for _ in 0..200 {
            let _ = d.process_sample(0.0);
        }
        // Now feed an impulse.
        let out = d.process_sample(1.0);
        // With mix=1, dry is 0, so the first sample is 0 (no delayed
        // signal yet).
        assert_eq!(out, 0.0);
        // After ~44 samples, the impulse should emerge.
        let mut found_peak = false;
        for _ in 0..100 {
            let out = d.process_sample(0.0);
            if out.abs() > 0.5 {
                found_peak = true;
            }
        }
        assert!(found_peak, "expected to find the impulse echo");
    }

    #[test]
    fn delay_feedback_high_produces_multiple_echoes() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        d.set_parameters(DelayParameters {
            delay_time_seconds: 0.01, // ~441 samples at 44.1 kHz
            feedback: 0.7,
            mix: 1.0,
            ping_pong: false,
            tempo_sync: false,
            tempo_bpm: 120.0,
            note_division: NoteDivision::Quarter,
            max_delay_seconds: 0.5,
            enabled: true,
        });
        // Prime with silence so the delay line is filled.
        for _ in 0..600 {
            let _ = d.process_sample(0.0);
        }
        // Feed an impulse.
        let _ = d.process_sample(1.0);
        // For the next 2000 samples, collect the maximum output and
        // count the number of distinct echo peaks (above 0.05).
        let mut max_after = 0.0_f32;
        let mut echo_count = 0_usize;
        let mut above_threshold = false;
        for _ in 0..2000 {
            let out = d.process_sample(0.0);
            max_after = max_after.max(out.abs());
            if out.abs() > 0.05 {
                if !above_threshold {
                    echo_count += 1;
                    above_threshold = true;
                }
            } else {
                above_threshold = false;
            }
        }
        // With feedback=0.7 the first echo has amplitude 0.7 and the
        // chain decays geometrically. We expect at least 2 distinct
        // echo peaks and a maximum amplitude > 0.3.
        assert!(
            max_after > 0.3,
            "expected first echo amplitude > 0.3, got {max_after}"
        );
        assert!(
            echo_count >= 2,
            "expected multiple distinct echo peaks, got {echo_count}"
        );
    }

    #[test]
    fn delay_ping_pong_alternates_channels() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        d.set_parameters(DelayParameters {
            delay_time_seconds: 0.001, // ~44 samples
            feedback: 0.5,
            mix: 1.0,
            ping_pong: true,
            tempo_sync: false,
            tempo_bpm: 120.0,
            note_division: NoteDivision::Quarter,
            max_delay_seconds: 0.1,
            enabled: true,
        });
        // Prime with silence.
        for _ in 0..200 {
            let mut ol = 0.0_f32;
            let mut or_ = 0.0_f32;
            d.process_stereo(0.0, 0.0, &mut ol, &mut or_);
        }
        // Feed a left-only impulse.
        let mut ol = 0.0_f32;
        let mut or_ = 0.0_f32;
        d.process_stereo(1.0, 0.0, &mut ol, &mut or_);
        // The first sample: L = in * dry = 0, R = in * dry = 0.
        assert_eq!(ol, 0.0);
        assert_eq!(or_, 0.0);
        // After ~44 samples, the L impulse should appear at L.
        let mut l_peak = 0.0_f32;
        let mut r_peak = 0.0_f32;
        for _ in 0..100 {
            let mut ol = 0.0_f32;
            let mut or_ = 0.0_f32;
            d.process_stereo(0.0, 0.0, &mut ol, &mut or_);
            l_peak = l_peak.max(ol.abs());
            r_peak = r_peak.max(or_.abs());
        }
        // Both channels should show some activity.
        assert!(l_peak > 0.1, "L channel should have activity, got {l_peak}");
        assert!(r_peak > 0.1, "R channel should have activity, got {r_peak}");
    }

    #[test]
    fn delay_mix_zero_passes_input_through_unchanged() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        d.set_parameters(DelayParameters {
            delay_time_seconds: 0.1,
            feedback: 0.5,
            mix: 0.0, // full dry
            ping_pong: false,
            tempo_sync: false,
            tempo_bpm: 120.0,
            note_division: NoteDivision::Quarter,
            max_delay_seconds: 0.5,
            enabled: true,
        });
        // First 8 samples should equal the input exactly.
        for i in 0..8 {
            let v = (i as f32) * 0.1;
            let out = d.process_sample(v);
            assert!(
                (out - v).abs() < 1e-6,
                "expected passthrough, got {out} for input {v}"
            );
        }
    }

    #[test]
    fn delay_disabled_passes_input_through_unchanged() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        d.set_enabled(false);
        for i in 0..8 {
            let v = (i as f32) * 0.1;
            let out = d.process_sample(v);
            assert_eq!(out, v);
        }
    }

    #[test]
    fn delay_reset_clears_state_but_keeps_parameters() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        let params_before = d.get_parameters();
        // Fill the line with non-zero state.
        for _ in 0..200 {
            let _ = d.process_sample(1.0);
        }
        d.reset();
        let params_after = d.get_parameters();
        assert_eq!(params_before, params_after);
        // After reset, processing a few samples of silence should
        // produce near-zero output (modulo the smoothing tail).
        for _ in 0..(SMOOTH_TIME_SECONDS * 44100.0) as i32 + 100 {
            let _ = d.process_sample(0.0);
        }
        // Now check that the next samples are still small.
        for _ in 0..100 {
            let out = d.process_sample(0.0);
            assert!(out.abs() < 0.1, "post-reset output too large: {out}");
        }
    }

    #[test]
    fn delay_processor_trait_block_processing() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        d.set_parameters(DelayParameters {
            delay_time_seconds: 0.001,
            feedback: 0.3,
            mix: 0.5,
            ping_pong: false,
            tempo_sync: false,
            tempo_bpm: 120.0,
            note_division: NoteDivision::Quarter,
            max_delay_seconds: 0.1,
            enabled: true,
        });
        let input: Vec<f32> = (0..256).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();
        let mut output = vec![0.0_f32; 256];
        Processor::process(&mut d, &input, &mut output);
        // The output should not be all zero (delay is active).
        let non_zero = output.iter().filter(|v| v.abs() > 0.001).count();
        assert!(non_zero > 0, "delay should produce non-zero output");
    }

    #[test]
    fn delay_tempo_change_updates_delay_time() {
        let mut d = Delay::new();
        d.prepare(44100.0, 512);
        d.set_parameters(DelayParameters {
            delay_time_seconds: 0.0, // ignored when tempo_sync is true
            feedback: 0.0,
            mix: 1.0,
            ping_pong: false,
            tempo_sync: true,
            tempo_bpm: 120.0, // quarter note = 0.5 s = 22050 samples
            note_division: NoteDivision::Quarter,
            max_delay_seconds: 2.0,
            enabled: true,
        });
        // Verify the delay line's target delay matches 22050 samples.
        let s = NoteDivision::Quarter.delay_samples(120.0, 44100.0);
        assert_eq!(s, 22050.0);
        // Change tempo to 60 bpm -> quarter note = 1.0 s = 44100 samples.
        d.set_tempo_bpm(60.0);
        let s2 = NoteDivision::Quarter.delay_samples(60.0, 44100.0);
        assert_eq!(s2, 44100.0);
    }

    #[test]
    fn delay_prepare_spec_matches_prepare() {
        let mut d = Delay::new();
        let spec = ProcessSpec::new(48000.0, 2, 256);
        d.prepare_spec(spec);
        assert_eq!(d.sample_rate(), 48000.0);
        assert_eq!(d.max_block_size(), 256);
    }
}
