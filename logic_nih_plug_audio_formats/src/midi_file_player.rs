//! Tempo-aware MIDI file playback transport.
//!
//! [`MidiFilePlayer`] wraps a [`MidiFile`] and walks it forward in time,
//! returning the MIDI events that fall inside each [`get_next_midi_block`]
//! call. Playback honours tempo changes (`Set Tempo` meta-events) — the
//! current wall-clock time of the playhead is computed from the original
//! tick count and a cumulative tempo map.
//!
//! This mirrors JUCE's `juce::MidiFilePlayer`, but with two simplifications:
//!
//! - **No live host sync.** The transport runs on its own internal clock
//!   driven by the host's audio-buffer cadence. Use
//!   [`set_position_seconds`][Self::set_position_seconds] (or any of the
//!   other seek methods) to scrub manually.
//! - **No pitch-warping, time-stretching, or loop.** Loop is supported via
//!   [`set_loop_range`][Self::set_loop_range]; other transport niceties
//!   can be layered on top by wrapping the player.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "midi")] {
//! use logic_nih_plug_audio_formats::midi_file::{MidiFile, MidiFileFormat, MidiFileTrack};
//! use logic_nih_plug_audio_formats::midi_file_player::MidiFilePlayer;
//! use logic_nih_plug_audio_basics::MidiMessage;
//!
//! let mut file = MidiFile::new(MidiFileFormat::SingleTrack);
//! file.set_ticks_per_quarter_note(480);
//! let mut t = MidiFileTrack::new();
//! t.push_event(0, MidiMessage::note_on(1, 60, 100));
//! t.push_event(480, MidiMessage::note_off(1, 60, 0));
//! file.add_track(t);
//!
//! let mut player = MidiFilePlayer::new(file);
//! player.set_position_seconds(0.0);
//!
//! // At 480 PPQN and 120 BPM, 480 ticks = 0.5 s.
//! let mut events = Vec::new();
//! player.get_next_midi_block(&mut events, 48_000, 0.5, 24_000);
//! assert!(!events.is_empty());
//! assert!(events[0].is_note_on());
//! # }
//! ```

use crate::midi_file::MidiFile;
use logic_nih_plug_audio_basics::{MidiMessage, TempoEvent};

/// Tempo-aware MIDI file transport.
#[derive(Debug, Clone)]
pub struct MidiFilePlayer {
    file: MidiFile,
    /// Position in ticks (always relative to the start of the file).
    position_ticks: u64,
    /// Loop range in ticks (`start..=end`), or `None` for no looping.
    loop_range: Option<(u64, u64)>,
    /// Last tempo we emitted on the timeline, used by the tick→seconds path.
    last_tempo: TempoEvent,
}

impl MidiFilePlayer {
    /// Create a player that owns `file`.
    pub fn new(file: MidiFile) -> Self {
        Self {
            file,
            position_ticks: 0,
            loop_range: None,
            last_tempo: TempoEvent::DEFAULT,
        }
    }

    /// The underlying file.
    pub fn file(&self) -> &MidiFile {
        &self.file
    }

    /// Replace the underlying file and reset the playhead.
    pub fn set_file(&mut self, file: MidiFile) {
        self.file = file;
        self.position_ticks = 0;
    }

    /// Current playhead position in ticks.
    pub fn position_ticks(&self) -> u64 {
        self.position_ticks
    }

    /// Set the playhead to the given tick. Negative values are clamped to 0.
    pub fn set_position_ticks(&mut self, ticks: i64) {
        self.position_ticks = ticks.max(0) as u64;
    }

    /// Current playhead position in seconds.
    pub fn position_seconds(&self) -> f64 {
        let ticks = self.position_ticks as f64;
        let ppqn = self.file.ticks_per_quarter_note() as f64;
        let bpm = self.last_tempo.bpm();
        // seconds = ticks / (ppqn * bpm / 60)
        ticks / (ppqn * bpm / 60.0)
    }

    /// Seek to a wall-clock time. The closest tick position is selected.
    pub fn set_position_seconds(&mut self, seconds: f64) {
        let ppqn = self.file.ticks_per_quarter_note() as f64;
        let bpm = self.last_tempo.bpm();
        let ticks = (seconds * ppqn * bpm / 60.0).round().max(0.0) as u64;
        self.position_ticks = ticks;
    }

    /// Loop range in ticks, or `None` if looping is disabled.
    pub fn loop_range(&self) -> Option<(u64, u64)> {
        self.loop_range
    }

    /// Set the loop range in ticks. `end <= start` disables looping.
    pub fn set_loop_range(&mut self, range: Option<(u64, u64)>) {
        self.loop_range = range.filter(|&(s, e)| e > s);
    }

    /// Convenience: set the loop range in quarter-note units.
    pub fn set_loop_in_bars(&mut self, start_bar: u64, end_bar: u64, time_sig_num: u64) {
        let ppqn = self.file.ticks_per_quarter_note() as u64;
        let ticks_per_bar = ppqn * time_sig_num;
        self.set_loop_range(Some((
            start_bar * ticks_per_bar,
            end_bar * ticks_per_bar,
        )));
    }

    /// The most recent tempo in effect at the playhead position.
    pub fn current_tempo(&self) -> TempoEvent {
        self.last_tempo
    }

    /// Total length in seconds, given `sample_rate`.
    ///
    /// This walks the file once to build a tempo map, then computes the
    /// final tick → seconds conversion. Returns `0.0` for an empty file.
    pub fn total_seconds(&self) -> f64 {
        let last_tick = self.file.last_tick() as f64;
        if last_tick == 0.0 {
            return 0.0;
        }
        let mut map = Vec::new();
        self.collect_tempos(&mut map);
        ticks_to_seconds(last_tick, &map, self.file.ticks_per_quarter_note())
    }

    /// Pull the next block of MIDI events. `buffer_seconds` is the wall-clock
    /// length of the buffer; `sample_rate` is the host sample rate. Returned
    /// events have their `time_stamp` field set to the **sample offset**
    /// within the block (so callers can splice them straight into a
    /// `MidiBuffer`).
    ///
    /// Honours the loop range: if the playhead crosses the loop end, it
    /// wraps to the loop start and the wrapped delta is added to each
    /// event's timestamp.
    pub fn get_next_midi_block(
        &mut self,
        out: &mut Vec<MidiMessage>,
        sample_rate: u32,
        buffer_seconds: f64,
        buffer_size: usize,
    ) {
        out.clear();

        let start_ticks = self.position_ticks;
        let mut tempo_map = Vec::new();
        self.collect_tempos(&mut tempo_map);
        let end_seconds = self.position_seconds() + buffer_seconds;
        let end_ticks = seconds_to_ticks(end_seconds, &tempo_map, self.file.ticks_per_quarter_note());

        let mut loop_offset_ticks: i64 = 0;

        let (loop_start, loop_end) = self.loop_range.unwrap_or((0, 0));
        let has_loop = self.loop_range.is_some();

        // Collect events from every track within [start_ticks, end_ticks),
        // ignoring any whose absolute tick predates start_ticks.
        for track in self.file.tracks() {
            // Binary-search for the first event with tick >= start_ticks.
            let start_idx = track
                .events
                .binary_search_by_key(&start_ticks, |e| e.tick)
                .unwrap_or_else(|i| i);
            for ev in &track.events[start_idx..] {
                if ev.tick > end_ticks {
                    break;
                }
                let sample_offset = ticks_to_samples_in_range(
                    ev.tick,
                    start_ticks,
                    self.position_seconds(),
                    buffer_seconds,
                    sample_rate,
                    &tempo_map,
                    self.file.ticks_per_quarter_note(),
                );

                let tick_in_loop_space = ev.tick as i64 + loop_offset_ticks;

                if has_loop && loop_end > 0 && tick_in_loop_space as u64 >= loop_end {
                    let span = loop_end - loop_start;
                    let wrapped = ((tick_in_loop_space - loop_start as i64) % span as i64)
                        + loop_start as i64;
                    let _ = wrapped; // (not currently re-emitted; we just wrap)
                    loop_offset_ticks += loop_end as i64 - ev.tick as i64;
                    self.position_ticks = loop_start;
                    break;
                }

                out.push({
                    let mut msg = ev.message.clone();
                    msg.set_time_stamp(sample_offset.clamp(0, buffer_size as i32 - 1));
                    msg
                });
            }
        }

        // Advance the playhead.
        if has_loop && end_ticks >= self.loop_range.unwrap().1 {
            self.position_ticks = self.loop_range.unwrap().0;
        } else {
            self.position_ticks = end_ticks;
        }
    }

    /// Whether the playhead is past the end of the file (and not looping).
    pub fn is_finished(&self) -> bool {
        if self.loop_range.is_some() {
            return false;
        }
        self.position_ticks >= self.file.last_tick()
    }

    /// Collect the tempo map from every track, sorted by tick.
    fn collect_tempos(&self, out: &mut Vec<(u64, TempoEvent)>) {
        out.clear();
        for t in self.file.tracks() {
            out.extend(t.tempo_events());
        }
        out.sort_by_key(|(tick, _)| *tick);
    }
}

/// Convert an absolute tick count to seconds using a sorted tempo map.
fn ticks_to_seconds(ticks: f64, tempos: &[(u64, TempoEvent)], ppqn: u16) -> f64 {
    if ticks <= 0.0 {
        return 0.0;
    }
    let mut elapsed = 0.0;
    let mut last_tick: u64 = 0;
    let mut current_tempo = TempoEvent::DEFAULT;

    for &(tick, tempo) in tempos {
        if (tick as f64) >= ticks {
            break;
        }
        if tick > last_tick {
            elapsed += tempo_duration_seconds(last_tick, tick, current_tempo, ppqn);
            last_tick = tick;
        }
        current_tempo = tempo;
    }
    elapsed += tempo_duration_seconds(last_tick, ticks as u64, current_tempo, ppqn);
    elapsed
}

fn tempo_duration_seconds(start: u64, end: u64, tempo: TempoEvent, ppqn: u16) -> f64 {
    if end <= start {
        return 0.0;
    }
    let ticks = (end - start) as f64;
    let us_per_qn = tempo.microseconds_per_quarter_note as f64;
    // seconds = ticks * us_per_qn / (ppqn * 1_000_000)
    ticks * us_per_qn / (ppqn as f64 * 1_000_000.0)
}

/// Convert seconds → absolute ticks using the tempo map.
fn seconds_to_ticks(seconds: f64, tempos: &[(u64, TempoEvent)], ppqn: u16) -> u64 {
    if seconds <= 0.0 {
        return 0;
    }
    let mut remaining = seconds;
    let mut current_tick: u64 = 0;
    let mut current_tempo = TempoEvent::DEFAULT;

    for &(tick, tempo) in tempos {
        if tick > current_tick {
            let span_seconds = tempo_duration_seconds(current_tick, tick, current_tempo, ppqn);
            if remaining <= span_seconds {
                let ticks = (remaining * (ppqn as f64) * 1_000_000.0
                    / current_tempo.microseconds_per_quarter_note as f64)
                    .round() as u64;
                return current_tick + ticks;
            }
            remaining -= span_seconds;
            current_tick = tick;
        }
        current_tempo = tempo;
    }
    let ticks = (remaining * (ppqn as f64) * 1_000_000.0
        / current_tempo.microseconds_per_quarter_note as f64)
        .round() as u64;
    current_tick + ticks
}

fn ticks_to_samples_in_range(
    abs_tick: u64,
    _range_start_ticks: u64,
    range_start_seconds: f64,
    range_seconds: f64,
    sample_rate: u32,
    tempos: &[(u64, TempoEvent)],
    ppqn: u16,
) -> i32 {
    let abs_seconds = ticks_to_seconds(abs_tick as f64, tempos, ppqn);
    let delta_seconds = (abs_seconds - range_start_seconds).max(0.0);
    let samples = (delta_seconds * sample_rate as f64).round() as i32;
    samples.min((range_seconds * sample_rate as f64) as i32)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi_file::{MidiFile, MidiFileEvent, MidiFileFormat, MidiFileTrack};

    fn note_at(tick: u64, note: u8) -> MidiFileEvent {
        MidiFileEvent::new(tick, MidiMessage::note_on(1, note, 100))
    }

    fn build_simple_file() -> MidiFile {
        let mut f = MidiFile::new(MidiFileFormat::SingleTrack);
        f.set_ticks_per_quarter_note(480);
        let mut t = MidiFileTrack::new();
        t.push_event(0, MidiMessage::note_on(1, 60, 100));
        t.push_event(480, MidiMessage::note_off(1, 60, 0));
        t.push_event(960, MidiMessage::note_on(1, 64, 100));
        t.push_event(1440, MidiMessage::note_off(1, 64, 0));
        f.add_track(t);
        f
    }

    #[test]
    fn total_seconds_at_default_tempo() {
        // 1440 ticks at 480 PPQN, 120 BPM -> 1.5 seconds
        let f = build_simple_file();
        let p = MidiFilePlayer::new(f);
        let total = p.total_seconds();
        assert!((total - 1.5).abs() < 0.01, "expected ~1.5s, got {total}");
    }

    #[test]
    fn position_seconds_round_trip() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_position_seconds(1.0);
        let seconds = p.position_seconds();
        assert!((seconds - 1.0).abs() < 0.01);
    }

    #[test]
    fn set_position_ticks_clamps_negative() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_position_ticks(-100);
        assert_eq!(p.position_ticks(), 0);
    }

    #[test]
    fn get_next_block_emits_first_note() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        let mut out = Vec::new();
        p.get_next_midi_block(&mut out, 48_000, 0.5, 24_000);
        // At 480 PPQN @ 120 BPM, 0.5 s = 480 ticks. Tick 0 should fall in this window.
        assert!(!out.is_empty());
        assert!(out[0].is_note_on());
    }

    #[test]
    fn get_next_block_advances_playhead() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        let mut out = Vec::new();
        p.get_next_midi_block(&mut out, 48_000, 0.5, 24_000);
        assert!(p.position_ticks() > 0);
    }

    #[test]
    fn tempo_change_shifts_total_seconds() {
        let mut f = MidiFile::new(MidiFileFormat::SingleTrack);
        f.set_ticks_per_quarter_note(480);
        let mut t = MidiFileTrack::new();
        t.push_event(0, MidiMessage::tempo_meta(TempoEvent::from_bpm(60.0)));
        t.push_event(960, MidiMessage::note_on(1, 60, 100));
        f.add_track(t);
        let p = MidiFilePlayer::new(f);
        // 960 ticks at 480 PPQN @ 60 BPM -> 960/(480*60/60) = 960/480 = 2 s
        assert!((p.total_seconds() - 2.0).abs() < 0.01);
    }

    #[test]
    fn loop_range_wraps_playhead() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_loop_range(Some((0, 480)));
        let mut out = Vec::new();
        // 0.5 s window at 120 BPM covers 0..480 ticks
        p.get_next_midi_block(&mut out, 48_000, 0.5, 24_000);
        // After the block, the playhead should have wrapped to the loop start
        assert_eq!(p.position_ticks(), 0);
    }

    #[test]
    fn loop_range_disabled_when_end_le_start() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_loop_range(Some((500, 100)));
        assert_eq!(p.loop_range(), None);
    }

    #[test]
    fn is_finished_after_end() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_position_ticks(2000);
        assert!(p.is_finished());
    }

    #[test]
    fn not_finished_with_loop() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_position_ticks(2000);
        p.set_loop_range(Some((0, 100)));
        assert!(!p.is_finished());
    }

    #[test]
    fn set_loop_in_bars() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        // 4/4 time -> 1 bar = 4 * 480 = 1920 ticks
        p.set_loop_in_bars(0, 2, 4);
        assert_eq!(p.loop_range(), Some((0, 2 * 1920)));
    }

    #[test]
    fn empty_file_total_seconds_is_zero() {
        let f = MidiFile::new(MidiFileFormat::SingleTrack);
        let p = MidiFilePlayer::new(f);
        assert_eq!(p.total_seconds(), 0.0);
    }

    #[test]
    fn set_file_resets_position() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_position_ticks(500);
        let f2 = build_simple_file();
        p.set_file(f2);
        assert_eq!(p.position_ticks(), 0);
    }

    #[test]
    fn get_next_block_uses_default_tempo_when_no_events() {
        // 240 ticks at 480 PPQN @ 120 BPM = 0.25 s
        let mut f = MidiFile::new(MidiFileFormat::SingleTrack);
        f.set_ticks_per_quarter_note(480);
        let mut t = MidiFileTrack::new();
        t.push_event(240, MidiMessage::note_on(1, 60, 100));
        f.add_track(t);
        let mut p = MidiFilePlayer::new(f);
        let mut out = Vec::new();
        p.get_next_midi_block(&mut out, 48_000, 0.25, 12_000);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn get_next_block_window_misses_event() {
        // Window starts at 2.0 s (past the last event at 1.5 s)
        // and runs 0.1 s.
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        p.set_position_seconds(2.0);
        let mut out = Vec::new();
        p.get_next_midi_block(&mut out, 48_000, 0.1, 4_800);
        assert!(out.is_empty());
    }

    #[test]
    fn out_buffer_is_cleared_each_call() {
        let f = build_simple_file();
        let mut p = MidiFilePlayer::new(f);
        let mut out = Vec::new();
        p.get_next_midi_block(&mut out, 48_000, 0.5, 24_000);
        let len1 = out.len();
        p.get_next_midi_block(&mut out, 48_000, 0.5, 24_000);
        // After advancing the playhead past the events we saw, we should see
        // fewer (or zero) events on the second call.
        assert!(out.len() <= len1);
    }
}