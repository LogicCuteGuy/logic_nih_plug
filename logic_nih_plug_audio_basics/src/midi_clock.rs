//! MIDI clock helpers — sample/tick conversions for the 24-tick-per-quarter
//! MIDI clock.
//!
//! The MIDI clock (`0xF8`) runs at exactly **24 ticks per quarter note**.
//! Given a sample rate and a tempo (in BPM), you can convert between
//! musical time (quarter notes) and the byte stream that needs to be sent
//! to a MIDI device.
//!
//! This module is a tiny utility class — it's not a beat tracker. It
//! gives you the math to convert between:
//!
//! - Samples ↔ ticks (`sample ↔ 24-tick MIDI clock`).
//! - Ticks ↔ quarter notes (`24 ticks = 1 quarter note`).
//! - Tempo in BPM ↔ seconds per quarter note ↔ samples per tick at a given
//!   sample rate.

/// Conversions for the 24-tick-per-quarter MIDI clock.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MidiClock {
    /// The sample rate in Hz (e.g. `48000`).
    pub sample_rate: f64,
    /// The tempo in beats (quarter notes) per minute.
    pub bpm: f64,
}

impl MidiClock {
    /// Construct a `MidiClock` with the given sample rate and BPM.
    ///
    /// # Panics
    ///
    /// Panics if `sample_rate <= 0` or `bpm <= 0`.
    pub fn new(sample_rate: f64, bpm: f64) -> Self {
        assert!(sample_rate > 0.0, "sample_rate must be > 0");
        assert!(bpm > 0.0, "bpm must be > 0");
        Self { sample_rate, bpm }
    }

    /// The number of seconds per quarter note.
    #[inline]
    pub fn seconds_per_quarter_note(&self) -> f64 {
        60.0 / self.bpm
    }

    /// The number of samples per quarter note.
    #[inline]
    pub fn samples_per_quarter_note(&self) -> f64 {
        self.sample_rate * self.seconds_per_quarter_note()
    }

    /// The number of samples per MIDI clock tick (24 ticks per QN).
    #[inline]
    pub fn samples_per_clock_tick(&self) -> f64 {
        self.samples_per_quarter_note() / 24.0
    }

    /// Convert a sample position to the corresponding MIDI clock tick
    /// number. The tick count is the count of `0xF8` messages that should
    /// have been emitted before the given sample.
    pub fn samples_to_ticks(&self, samples: f64) -> u64 {
        (samples / self.samples_per_clock_tick()).floor() as u64
    }

    /// Convert a MIDI clock tick number back to a sample position.
    pub fn ticks_to_samples(&self, ticks: u64) -> f64 {
        ticks as f64 * self.samples_per_clock_tick()
    }

    /// Given a tick delta (i.e. the number of ticks that have elapsed
    /// since the last emission), return how many `0xF8` clock messages
    /// to emit *right now* and how many samples to carry over until the
    /// next emission.
    pub fn split_tick_delta(&self, sample_delta: f64) -> (u64, f64) {
        let ticks = (sample_delta / self.samples_per_clock_tick()).floor();
        let consumed = ticks * self.samples_per_clock_tick();
        let carried = sample_delta - consumed;
        (ticks as u64, carried)
    }

    /// Convert a tempo in BPM to its microseconds-per-quarter-note
    /// representation, the inverse of [`bpm_from_micros`][Self::bpm_from_micros].
    pub fn micros_per_quarter_note(bpm: f64) -> u32 {
        (60_000_000.0 / bpm).round() as u32
    }

    /// Inverse of [`micros_per_quarter_note`][Self::micros_per_quarter_note].
    pub fn bpm_from_micros(micros: u32) -> f64 {
        60_000_000.0 / micros as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_per_quarter_note() {
        let clock = MidiClock::new(48_000.0, 120.0);
        assert!((clock.seconds_per_quarter_note() - 0.5).abs() < 1e-9);
        assert_eq!(clock.samples_per_quarter_note(), 24_000.0);
    }

    #[test]
    fn samples_per_clock_tick_at_120_bpm() {
        let clock = MidiClock::new(48_000.0, 120.0);
        // 120 BPM ⇒ 24_000 samples / QN ⇒ 1000 samples / tick.
        assert_eq!(clock.samples_per_clock_tick(), 1000.0);
    }

    #[test]
    fn samples_to_ticks_round_trip() {
        let clock = MidiClock::new(48_000.0, 120.0);
        for samples in [0.0, 500.0, 1000.0, 1500.0, 999.0, 24_000.0] {
            let ticks = clock.samples_to_ticks(samples);
            let back = clock.ticks_to_samples(ticks);
            // Floor rounding means we can be off by up to one tick.
            assert!(
                (back - samples).abs() < 1000.0,
                "samples={samples}, ticks={ticks}, back={back}"
            );
        }
    }

    #[test]
    fn split_tick_delta_at_120_bpm() {
        let clock = MidiClock::new(48_000.0, 120.0);
        // 2500 samples → 2 full ticks (2000 samples), 500 left over.
        let (ticks, carried) = clock.split_tick_delta(2500.0);
        assert_eq!(ticks, 2);
        assert!((carried - 500.0).abs() < 1e-9);
    }

    #[test]
    fn split_tick_delta_zero_or_small() {
        let clock = MidiClock::new(48_000.0, 120.0);
        assert_eq!(clock.split_tick_delta(0.0), (0, 0.0));
        assert_eq!(clock.split_tick_delta(999.0), (0, 999.0));
    }

    #[test]
    fn micros_per_quarter_note_round_trip() {
        for bpm in [60.0, 90.0, 120.0, 150.0, 200.0] {
            let micros = MidiClock::micros_per_quarter_note(bpm);
            let back = MidiClock::bpm_from_micros(micros);
            assert!((back - bpm).abs() < 1e-3, "bpm={bpm}, micros={micros}, back={back}");
        }
    }

    #[test]
    fn slow_tempo_at_48khz() {
        // 60 BPM at 48 kHz → 2000 samples per tick.
        let clock = MidiClock::new(48_000.0, 60.0);
        assert_eq!(clock.samples_per_clock_tick(), 2000.0);
    }

    #[test]
    fn fast_tempo_at_48khz() {
        // 240 BPM at 48 kHz → 500 samples per tick.
        let clock = MidiClock::new(48_000.0, 240.0);
        assert_eq!(clock.samples_per_clock_tick(), 500.0);
    }
}
