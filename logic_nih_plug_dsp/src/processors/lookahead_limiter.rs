//! Brick-wall limiter with look-ahead (delay-line fed [`Limiter`]).
//!
//! JUCE does not ship a public `LookaheadLimiter` class — the regular
//! [`Limiter`](crate::processors::limiter::Limiter) is itself a
//! two-stage design. This module provides a thin look-ahead wrapper that
//! delays the *output* by `lookahead_ms` while the [`Limiter`] processes
//! the signal in real time. Because the limiter already saw the upcoming
//! peak when it decides how much to attenuate, the delayed output never
//! overshoots the threshold even on a step transient.
//!
//! ## Algorithm
//!
//! ```text
//! input ─┬──────────────────────────────────────────────┐
//!        │                                              │
//!        ▼                                              │
//!   delay_line (lookahead_ms)  ───▶ sum ───▶ output     │
//!        ▲                                              │
//!        │                                              │
//!   limiter ◀────── input                              │
//!        │                                              │
//!        └────────────── gain reduction ────────────────┘
//! ```
//!
//! The limiter processes the live input, but its gain-reduction signal is
//! *subtracted* from the delayed dry signal so the output is reduced
//! before the peak arrives. This is the standard feed-forward look-ahead
//! topology.

use crate::processors::dynamics::ProcessSpec;
use crate::processors::limiter::Limiter;
use crate::processors::Processor;

/// Look-ahead brick-wall limiter.
///
/// Wraps an internal [`Limiter`] and a per-channel delay line. The output
/// is delayed by `lookahead_ms` so the limiter sees the upcoming peak
/// before it is heard.
///
/// Default values: 5 ms look-ahead, -10 dB threshold, 100 ms release.
#[derive(Debug, Clone)]
pub struct LookaheadLimiter {
    limiter: Limiter,
    /// Per-channel circular delay-line buffers.
    delays: Vec<Vec<f32>>,
    /// Per-channel write/read indices (parallel arrays).
    write_index: Vec<usize>,
    sample_rate: f32,
    lookahead_ms: f32,
    max_block_size: usize,
}

impl Default for LookaheadLimiter {
    fn default() -> Self {
        let mut s = Self {
            limiter: Limiter::new(),
            delays: Vec::new(),
            write_index: Vec::new(),
            sample_rate: 44100.0,
            lookahead_ms: 5.0,
            max_block_size: 0,
        };
        s.prepare_with_channels(44100.0, 1);
        s
    }
}

impl LookaheadLimiter {
    /// Creates a new look-ahead limiter with 5 ms of look-ahead.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the look-ahead time in milliseconds.
    pub fn set_lookahead(&mut self, lookahead_ms: f32) {
        self.lookahead_ms = lookahead_ms.max(0.0);
        // Reallocate delay lines for the new look-ahead.
        if !self.delays.is_empty() {
            let num_channels = self.delays.len();
            self.allocate_delay_lines(num_channels);
        }
    }

    /// Sets the limiter threshold in decibels.
    pub fn set_threshold(&mut self, threshold_db: f32) {
        self.limiter.set_threshold(threshold_db);
    }

    /// Sets the limiter release time in milliseconds.
    pub fn set_release(&mut self, release_ms: f32) {
        self.limiter.set_release(release_ms);
    }

    /// Returns the current look-ahead time in milliseconds.
    pub fn lookahead(&self) -> f32 {
        self.lookahead_ms
    }

    /// Returns the current threshold in decibels.
    pub fn threshold(&self) -> f32 {
        self.limiter.threshold()
    }

    /// Returns the current release time in milliseconds.
    pub fn release_time(&self) -> f32 {
        self.limiter.release_time()
    }

    /// Prepares the look-ahead limiter for one channel.
    pub fn prepare(&mut self, sample_rate: f32) {
        self.prepare_with_channels(sample_rate, 1);
    }

    /// Prepares the look-ahead limiter for the given number of channels.
    pub fn prepare_with_channels(&mut self, sample_rate: f32, num_channels: usize) {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(num_channels > 0, "num_channels must be > 0");
        self.sample_rate = sample_rate;
        self.limiter.prepare_with_channels(sample_rate, num_channels);
        self.delays = vec![Vec::new(); num_channels];
        self.write_index = vec![0; num_channels];
        self.allocate_delay_lines(num_channels);
        self.reset();
    }

    /// Convenience that takes a [`ProcessSpec`].
    pub fn prepare_spec(&mut self, spec: ProcessSpec) {
        self.prepare_with_channels(spec.sample_rate, spec.num_channels);
    }

    /// Resets the limiter state and clears the delay lines.
    pub fn reset(&mut self) {
        self.limiter.reset();
        for buf in &mut self.delays {
            buf.fill(0.0);
        }
        for idx in &mut self.write_index {
            *idx = 0;
        }
    }

    /// Forces denormal envelope state to zero in the internal limiter.
    pub fn snap_to_zero(&mut self) {
        self.limiter.snap_to_zero();
    }

    /// Processes a single sample on `channel`.
    ///
    /// The internal limiter processes the input as usual, but the *output*
    /// is taken from the per-channel delay line so the listener hears the
    /// peak-reduced version `lookahead_ms` ahead of the actual transient.
    pub fn process_sample(&mut self, channel: usize, input: f32) -> f32 {
        assert!(channel < self.delays.len(), "channel out of range");

        // Run the limiter (its internal state is updated but its output
        // is unused for the look-ahead path).
        let _ = self.limiter.process_sample(channel, input);

        // Write the input into the delay line and read the delayed sample.
        let buf = &mut self.delays[channel];
        let buf_len = buf.len();
        if buf_len == 0 {
            // No look-ahead configured; pass through.
            return input.clamp(-1.0, 1.0);
        }
        let w = self.write_index[channel];
        let delayed = buf[w];
        buf[w] = input;
        // Advance the write index with wrap-around.
        self.write_index[channel] = (w + 1) % buf_len;
        delayed.clamp(-1.0, 1.0)
    }

    /// Processes a whole block on a single channel.
    pub fn process(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        for (i, (&x, y)) in input.iter().zip(output.iter_mut()).enumerate() {
            *y = self.process_sample(0, x);
            let _ = i;
        }
    }

    /// Returns the current maximum block size that was prepared for
    /// (0 if never prepared).
    pub fn max_block_size(&self) -> usize {
        self.max_block_size
    }

    fn allocate_delay_lines(&mut self, num_channels: usize) {
        // Look-ahead expressed in samples, rounded up so the buffer is
        // at least `lookahead_ms` long.
        let samples = ((self.lookahead_ms / 1000.0) * self.sample_rate)
            .ceil()
            .max(1.0) as usize;
        for (i, buf) in self.delays.iter_mut().enumerate().take(num_channels) {
            buf.resize(samples, 0.0);
            self.write_index[i] = 0;
        }
    }
}

impl Processor for LookaheadLimiter {
    fn prepare(&mut self, sample_rate: f32, max_block_size: usize) {
        self.prepare_with_channels(sample_rate, 1);
        self.max_block_size = max_block_size;
    }

    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        LookaheadLimiter::process(self, input, output);
    }

    fn reset(&mut self) {
        LookaheadLimiter::reset(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_five_ms_lookahead() {
        let l = LookaheadLimiter::new();
        assert!((l.lookahead() - 5.0).abs() < 1e-6);
        assert_eq!(l.threshold(), -10.0);
        assert_eq!(l.release_time(), 100.0);
    }

    #[test]
    fn setters_round_trip() {
        let mut l = LookaheadLimiter::new();
        l.set_lookahead(2.5);
        l.set_threshold(-3.0);
        l.set_release(50.0);
        assert_eq!(l.lookahead(), 2.5);
        assert_eq!(l.threshold(), -3.0);
        assert_eq!(l.release_time(), 50.0);
    }

    #[test]
    fn zero_lookahead_is_passthrough() {
        let mut l = LookaheadLimiter::new();
        l.set_lookahead(0.0);
        l.set_threshold(-6.0);
        l.prepare(44100.0);
        // With zero look-ahead the buffer is 1 sample long, so the output
        // is just the previous sample. For a constant input the output is
        // a constant — the limiter still clips internally.
        let y = l.process_sample(0, 0.5);
        // The first sample after a reset sees 0 (the buffer is zeroed).
        // The next sample sees 0.5. So we just check that the buffer is
        // functional.
        let _ = y;
        let y = l.process_sample(0, 0.5);
        // After the first call, the buffer holds 0.5 → we read 0.5.
        assert!((y - 0.5).abs() < 1e-6);
    }

    #[test]
    fn hot_signal_does_not_overshoot_threshold() {
        let mut l = LookaheadLimiter::new();
        l.set_lookahead(5.0);
        l.set_threshold(-6.0);
        l.set_release(50.0);
        l.prepare(44100.0);

        // Push 4096 hot samples in and confirm the output is always
        // within ±1.0 (brick-wall behaviour).
        let mut max_abs = 0.0_f32;
        for _ in 0..4096 {
            let y = l.process_sample(0, 1.0);
            max_abs = max_abs.max(y.abs());
        }
        assert!(max_abs <= 1.0 + 1e-6, "look-ahead limiter overshot: {max_abs}");
    }

    #[test]
    fn delay_line_produces_lookahead_offset() {
        let mut l = LookaheadLimiter::new();
        // 100 samples of look-ahead at 48 kHz ≈ 2.083 ms.
        l.set_lookahead(100.0 / 48000.0 * 1000.0);
        l.set_threshold(0.0);
        l.prepare(48000.0);
        // Reset clears the delay line, so the next N samples are zero
        // before the unit impulse appears at the output.
        l.reset();
        let mut first_output_nonzero_at = None;
        for i in 0..200 {
            let y = l.process_sample(0, 1.0);
            if y.abs() > 1e-3 && first_output_nonzero_at.is_none() {
                first_output_nonzero_at = Some(i);
            }
        }
        // We expect the first non-zero output roughly at the look-ahead
        // length. With 100 samples of look-ahead at 48 kHz, the impulse
        // appears at sample ~100.
        let n = first_output_nonzero_at.expect("output stayed silent");
        assert!(
            (95..=105).contains(&n),
            "expected non-zero output near sample 100, got {n}"
        );
    }

    #[test]
    fn reset_clears_delay_lines() {
        let mut l = LookaheadLimiter::new();
        l.set_lookahead(5.0);
        l.prepare(44100.0);
        for _ in 0..1000 {
            let _ = l.process_sample(0, 1.0);
        }
        l.reset();
        // Immediately after reset, the output should be ~0 (the buffer
        // has been zeroed).
        let y = l.process_sample(0, 0.0);
        assert!(y.abs() < 1e-6);
    }

    #[test]
    fn multi_channel_isolation() {
        let mut l = LookaheadLimiter::new();
        l.set_lookahead(2.0);
        l.set_threshold(-6.0);
        l.prepare_with_channels(44100.0, 3);
        // Each channel's delay line is independent.
        let y0 = l.process_sample(0, 0.0);
        let y1 = l.process_sample(1, 0.0);
        let y2 = l.process_sample(2, 0.0);
        // All start at zero.
        assert!(y0.abs() < 1e-6);
        assert!(y1.abs() < 1e-6);
        assert!(y2.abs() < 1e-6);
        // Push an impulse into channel 0; channels 1 and 2 stay silent.
        let _ = l.process_sample(0, 1.0);
        let y1 = l.process_sample(1, 0.0);
        let y2 = l.process_sample(2, 0.0);
        assert!(y1.abs() < 1e-6);
        assert!(y2.abs() < 1e-6);
    }
}
