//! Audio analysis tools.
//!
//! This module provides tools for analysing audio signals, including:
//!
//! * **Level metering** — [`LevelMeter`] (peak / RMS with ballistics).
//! * **Envelope following** — [`Follower`] (single-path attack/release smoother).
//! * **Loudness measurement** — [`LoudnessMeter`] (ITU-R BS.1770 K-weighting, momentary / short-term / integrated LUFS).
//! * **Waveform capture** — [`Oscilloscope`] (min/max circular buffer for display).
//! * **FFT** — [`FFT`] (complex) and [`RealFFT`] (real-only) frequency-domain transforms.
//! * **STFT** — [`STFT`] (windowed short-time Fourier transform with forward/inverse helpers).
//! * **Windows** — [`WindowingFunction`] (Hann, Hamming, Blackman, Kaiser, etc.).

pub mod fft;
pub mod follower;
pub mod level_meter;
pub mod loudness_meter;
pub mod oscilloscope;
pub mod real_fft;
pub mod stft;
pub mod windowing;

pub use fft::FFT;
pub use follower::Follower;
pub use level_meter::LevelMeter;
pub use loudness_meter::LoudnessMeter;
pub use oscilloscope::Oscilloscope;
pub use real_fft::RealFFT;
pub use stft::STFT;
pub use windowing::WindowingFunction;
