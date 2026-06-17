//! # nih_plug_audio_formats
//!
//! Audio file format support ported from JUCE.
//!
//! This crate provides readers and writers for common audio file formats:
//!
//! - **WAV**: Waveform Audio File Format
//! - **AIFF**: Audio Interchange File Format
//! - **FLAC**: Free Lossless Audio Codec
//! - **OGG**: Ogg Vorbis
//!
//! ## Examples
//!
//! ```ignore
//! use nih_plug_audio_formats::{AudioFileReader, AudioFormat};
//!
//! let mut reader = AudioFileReader::open("audio.wav").unwrap();
//! let samples = reader.read_all().unwrap();
//! ```

#![warn(missing_docs)]

pub mod error;

#[cfg(feature = "wav")]
pub mod wav;

#[cfg(feature = "aiff")]
pub mod aiff;

#[cfg(feature = "flac")]
pub mod flac;

#[cfg(feature = "ogg")]
pub mod ogg;

pub use error::{AudioFormatError, Result};

use std::path::Path;

/// Supported audio file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    /// WAV format
    Wav,
    /// AIFF format
    Aiff,
    /// FLAC format
    Flac,
    /// OGG Vorbis format
    Ogg,
}

impl AudioFormat {
    /// Detect the audio format from a file extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::AudioFormat;
    /// use std::path::Path;
    ///
    /// let format = AudioFormat::from_path(Path::new("audio.wav"));
    /// assert_eq!(format, Some(AudioFormat::Wav));
    /// ```
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext.to_lowercase().as_str() {
                "wav" => Some(AudioFormat::Wav),
                "aiff" | "aif" => Some(AudioFormat::Aiff),
                "flac" => Some(AudioFormat::Flac),
                "ogg" => Some(AudioFormat::Ogg),
                _ => None,
            })
    }

    /// Get the typical file extension for this format.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::AudioFormat;
    ///
    /// assert_eq!(AudioFormat::Wav.extension(), "wav");
    /// assert_eq!(AudioFormat::Aiff.extension(), "aiff");
    /// ```
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Aiff => "aiff",
            AudioFormat::Flac => "flac",
            AudioFormat::Ogg => "ogg",
        }
    }
}

/// Metadata about an audio file.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioMetadata {
    /// Sample rate in Hz
    pub sample_rate: f32,
    /// Number of audio channels
    pub num_channels: usize,
    /// Total number of sample frames
    pub num_frames: usize,
    /// Bit depth (if applicable)
    pub bit_depth: Option<u16>,
}

impl AudioMetadata {
    /// Create new audio metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::AudioMetadata;
    ///
    /// let metadata = AudioMetadata::new(44100.0, 2, 1000);
    /// assert_eq!(metadata.sample_rate, 44100.0);
    /// assert_eq!(metadata.num_channels, 2);
    /// ```
    pub fn new(sample_rate: f32, num_channels: usize, num_frames: usize) -> Self {
        Self {
            sample_rate,
            num_channels,
            num_frames,
            bit_depth: None,
        }
    }

    /// Create new audio metadata with bit depth.
    pub fn with_bit_depth(
        sample_rate: f32,
        num_channels: usize,
        num_frames: usize,
        bit_depth: u16,
    ) -> Self {
        Self {
            sample_rate,
            num_channels,
            num_frames,
            bit_depth: Some(bit_depth),
        }
    }

    /// Get the duration in seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::AudioMetadata;
    ///
    /// let metadata = AudioMetadata::new(44100.0, 2, 44100);
    /// assert!((metadata.duration_seconds() - 1.0).abs() < 0.001);
    /// ```
    pub fn duration_seconds(&self) -> f64 {
        self.num_frames as f64 / self.sample_rate as f64
    }
}

/// Sample format conversion utilities.
pub mod util {
    /// Convert an i16 sample to f32 in the range [-1.0, 1.0].
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::i16_to_f32;
    ///
    /// assert_eq!(i16_to_f32(0), 0.0);
    /// assert!((i16_to_f32(i16::MAX) - 1.0).abs() < 0.001);
    /// assert!((i16_to_f32(i16::MIN) - (-1.0)).abs() < 0.001);
    /// ```
    #[inline]
    pub fn i16_to_f32(sample: i16) -> f32 {
        if sample >= 0 {
            sample as f32 / i16::MAX as f32
        } else {
            sample as f32 / -(i16::MIN as f32)
        }
    }

    /// Convert an f32 sample in the range [-1.0, 1.0] to i16.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::f32_to_i16;
    ///
    /// assert_eq!(f32_to_i16(0.0), 0);
    /// assert_eq!(f32_to_i16(1.0), i16::MAX);
    /// assert_eq!(f32_to_i16(-1.0), i16::MIN);
    /// ```
    #[inline]
    pub fn f32_to_i16(sample: f32) -> i16 {
        let clamped = sample.clamp(-1.0, 1.0);
        if clamped >= 0.0 {
            (clamped * i16::MAX as f32) as i16
        } else {
            (clamped * -(i16::MIN as f32)) as i16
        }
    }

    /// Convert an i24 sample (stored as i32) to f32 in the range [-1.0, 1.0].
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::i24_to_f32;
    ///
    /// assert_eq!(i24_to_f32(0), 0.0);
    /// ```
    #[inline]
    pub fn i24_to_f32(sample: i32) -> f32 {
        const MAX_24BIT: f32 = 8388607.0; // 2^23 - 1
        const MIN_24BIT: f32 = 8388608.0; // 2^23
        
        if sample >= 0 {
            sample as f32 / MAX_24BIT
        } else {
            sample as f32 / MIN_24BIT
        }
    }

    /// Convert an f32 sample in the range [-1.0, 1.0] to i24 (stored as i32).
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::f32_to_i24;
    ///
    /// assert_eq!(f32_to_i24(0.0), 0);
    /// ```
    #[inline]
    pub fn f32_to_i24(sample: f32) -> i32 {
        const MAX_24BIT: f32 = 8388607.0; // 2^23 - 1
        const MIN_24BIT: f32 = 8388608.0; // 2^23
        
        let clamped = sample.clamp(-1.0, 1.0);
        if clamped >= 0.0 {
            (clamped * MAX_24BIT) as i32
        } else {
            (clamped * MIN_24BIT) as i32
        }
    }

    /// Convert an i32 sample to f32 in the range [-1.0, 1.0].
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::i32_to_f32;
    ///
    /// assert_eq!(i32_to_f32(0), 0.0);
    /// ```
    #[inline]
    pub fn i32_to_f32(sample: i32) -> f32 {
        if sample >= 0 {
            sample as f32 / i32::MAX as f32
        } else {
            sample as f32 / -(i32::MIN as f32)
        }
    }

    /// Convert an f32 sample in the range [-1.0, 1.0] to i32.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::f32_to_i32;
    ///
    /// assert_eq!(f32_to_i32(0.0), 0);
    /// ```
    #[inline]
    pub fn f32_to_i32(sample: f32) -> i32 {
        let clamped = sample.clamp(-1.0, 1.0);
        if clamped >= 0.0 {
            (clamped * i32::MAX as f32) as i32
        } else {
            (clamped * -(i32::MIN as f32)) as i32
        }
    }

    /// Interleave multi-channel audio samples.
    ///
    /// Takes a slice of channel buffers and produces an interleaved buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::interleave;
    ///
    /// let left = vec![1.0, 2.0, 3.0];
    /// let right = vec![4.0, 5.0, 6.0];
    /// let channels = vec![left, right];
    /// let interleaved = interleave(&channels);
    /// assert_eq!(interleaved, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    /// ```
    pub fn interleave(channels: &[Vec<f32>]) -> Vec<f32> {
        if channels.is_empty() {
            return Vec::new();
        }

        let num_channels = channels.len();
        let num_frames = channels[0].len();
        let mut interleaved = Vec::with_capacity(num_channels * num_frames);

        for frame in 0..num_frames {
            for channel in channels.iter() {
                interleaved.push(channel[frame]);
            }
        }

        interleaved
    }

    /// Deinterleave multi-channel audio samples.
    ///
    /// Takes an interleaved buffer and produces separate channel buffers.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_audio_formats::util::deinterleave;
    ///
    /// let interleaved = vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0];
    /// let channels = deinterleave(&interleaved, 2);
    /// assert_eq!(channels[0], vec![1.0, 2.0, 3.0]);
    /// assert_eq!(channels[1], vec![4.0, 5.0, 6.0]);
    /// ```
    pub fn deinterleave(interleaved: &[f32], num_channels: usize) -> Vec<Vec<f32>> {
        if num_channels == 0 || interleaved.is_empty() {
            return Vec::new();
        }

        let num_frames = interleaved.len() / num_channels;
        let mut channels = vec![Vec::with_capacity(num_frames); num_channels];

        for (i, &sample) in interleaved.iter().enumerate() {
            let channel = i % num_channels;
            channels[channel].push(sample);
        }

        channels
    }
}
