//! Pitch and time processing — phase vocoder, pitch shifting, and time stretching.
//!
//! This module provides a complete phase-vocoder-based pitch and time
//! manipulation toolkit:
//!
//! * [`PhaseVocoder`] — the core STFT analysis → phase processing →
//!   synthesis engine. You can drive it directly for custom phase
//!   manipulation, or let the higher-level processors use it.
//! * [`PitchShift`] — real-time pitch shifting (±24 semitones, or any
//!   fractional ratio) without changing the playback duration.
//! * [`TimeStretching`] — time stretching / compressing without changing
//!   the pitch.
//!
//! Both higher-level processors use a [phase vocoder][PhaseVocoder] with
//! overlap-add synthesis and Hann windows.
//!
//! # Quick start — pitch shifting
//!
//! ```ignore
//! use logic_nih_plug_dsp::processors::pitch::{PitchShift, PitchShiftParameters};
//!
//! let mut shifter = PitchShift::new();
//! shifter.prepare(44100.0, 512);
//! shifter.set_parameters(PitchShiftParameters {
//!     pitch_ratio: 1.5,   // +7 semitones
//!     enabled: true,
//! });
//!
//! let input  = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 512];
//! shifter.process(&input, &mut output);
//! ```
//!
//! # Quick start — time stretching
//!
//! ```ignore
//! use logic_nih_plug_dsp::processors::pitch::{TimeStretching, TimeStretchParameters};
//!
//! let mut stretcher = TimeStretching::new();
//! stretcher.prepare(44100.0, 512);
//! stretcher.set_parameters(TimeStretchParameters {
//!     time_ratio: 0.5,   // 2× faster
//!     enabled: true,
//! });
//!
//! let input  = vec![0.5_f32; 512];
//! let mut output = vec![0.0_f32; 512];
//! stretcher.process(&input, &mut output);
//! ```

pub mod phase_vocoder;
pub mod pitch_shift;
pub mod time_stretch;

pub use crate::analysis::windowing::WindowingFunction;
pub use phase_vocoder::PhaseVocoder;
pub use pitch_shift::{PitchShift, PitchShiftParameters};
pub use time_stretch::{TimeStretching, TimeStretchParameters};
