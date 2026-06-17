//! WAV file format support.
//!
//! This module provides reading and writing of WAV (Waveform Audio File Format) files.
//! It supports various bit depths (8, 16, 24, 32-bit integer and 32-bit float) and
//! arbitrary sample rates and channel counts.
//!
//! # Examples
//!
//! ## Reading a WAV file
//!
//! ```no_run
//! use logic_nih_plug_audio_formats::wav::WavReader;
//!
//! let mut reader = WavReader::open("audio.wav").unwrap();
//! let samples = reader.read_all().unwrap();
//! println!("Read {} channels with {} frames", samples.len(), samples[0].len());
//! ```
//!
//! ## Writing a WAV file
//!
//! ```no_run
//! use logic_nih_plug_audio_formats::wav::WavWriter;
//!
//! let left = vec![0.0, 0.5, 1.0, 0.5, 0.0];
//! let right = vec![0.0, -0.5, -1.0, -0.5, 0.0];
//! let samples = vec![left, right];
//!
//! let mut writer = WavWriter::create("output.wav", 44100.0, 2, 16).unwrap();
//! writer.write_samples(&samples).unwrap();
//! ```

use crate::error::{AudioFormatError, Result};
use crate::util::{deinterleave, f32_to_i16, f32_to_i24, i16_to_f32, i24_to_f32, i32_to_f32, interleave};
use crate::AudioMetadata;
use hound::{SampleFormat, WavSpec, WavWriter as HoundWriter, WavReader as HoundReader};
use std::path::Path;

/// A WAV file reader.
///
/// This reader supports various bit depths and automatically converts samples to f32
/// in the range [-1.0, 1.0].
pub struct WavReader {
    reader: HoundReader<std::io::BufReader<std::fs::File>>,
    metadata: AudioMetadata,
}

impl WavReader {
    /// Open a WAV file for reading.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the WAV file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file is not a valid WAV file
    /// - The file format is unsupported
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::wav::WavReader;
    ///
    /// let reader = WavReader::open("audio.wav").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        
        let reader = HoundReader::open(path_ref).map_err(|e| {
            if path_ref.exists() {
                AudioFormatError::InvalidData(format!("Failed to open WAV file: {}", e))
            } else {
                AudioFormatError::FileNotFound(path_ref.display().to_string())
            }
        })?;

        let spec = reader.spec();
        
        // Validate sample rate
        if spec.sample_rate == 0 {
            return Err(AudioFormatError::InvalidSampleRate(0.0));
        }
        
        // Validate channel count
        if spec.channels == 0 {
            return Err(AudioFormatError::InvalidChannelCount(0));
        }

        let num_frames = reader.duration() as usize;
        
        let metadata = AudioMetadata::with_bit_depth(
            spec.sample_rate as f32,
            spec.channels as usize,
            num_frames,
            spec.bits_per_sample,
        );

        Ok(Self { reader, metadata })
    }

    /// Get the metadata for this WAV file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::wav::WavReader;
    ///
    /// let reader = WavReader::open("audio.wav").unwrap();
    /// let metadata = reader.metadata();
    /// println!("Sample rate: {} Hz", metadata.sample_rate);
    /// ```
    pub fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }

    /// Get the sample rate in Hz.
    pub fn sample_rate(&self) -> f32 {
        self.metadata.sample_rate
    }

    /// Get the number of channels.
    pub fn num_channels(&self) -> usize {
        self.metadata.num_channels
    }

    /// Get the total number of sample frames.
    pub fn num_frames(&self) -> usize {
        self.metadata.num_frames
    }

    /// Get the bit depth.
    pub fn bit_depth(&self) -> Option<u16> {
        self.metadata.bit_depth
    }

    /// Read all samples from the WAV file.
    ///
    /// Returns a vector of channel buffers, where each channel buffer contains
    /// all samples for that channel as f32 values in the range [-1.0, 1.0].
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or if the file contains invalid data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::wav::WavReader;
    ///
    /// let mut reader = WavReader::open("audio.wav").unwrap();
    /// let samples = reader.read_all().unwrap();
    /// println!("Channel 0 has {} samples", samples[0].len());
    /// ```
    pub fn read_all(&mut self) -> Result<Vec<Vec<f32>>> {
        let spec = self.reader.spec();
        let num_channels = spec.channels as usize;

        // Read all samples based on format
        let interleaved = match (spec.sample_format, spec.bits_per_sample) {
            (SampleFormat::Int, 8) => {
                // 8-bit samples are stored as i8 in hound
                self.reader
                    .samples::<i8>()
                    .map(|s| s.map(|sample| {
                        // Convert i8 [-128, 127] to f32 [-1.0, 1.0]
                        if sample >= 0 {
                            sample as f32 / 127.0
                        } else {
                            sample as f32 / 128.0
                        }
                    }))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| AudioFormatError::InvalidData(format!("Failed to read samples: {}", e)))?
            }
            (SampleFormat::Int, 16) => {
                self.reader
                    .samples::<i16>()
                    .map(|s| s.map(i16_to_f32))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| AudioFormatError::InvalidData(format!("Failed to read samples: {}", e)))?
            }
            (SampleFormat::Int, 24) => {
                self.reader
                    .samples::<i32>()
                    .map(|s| s.map(i24_to_f32))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| AudioFormatError::InvalidData(format!("Failed to read samples: {}", e)))?
            }
            (SampleFormat::Int, 32) => {
                self.reader
                    .samples::<i32>()
                    .map(|s| s.map(i32_to_f32))
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| AudioFormatError::InvalidData(format!("Failed to read samples: {}", e)))?
            }
            (SampleFormat::Float, 32) => {
                self.reader
                    .samples::<f32>()
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| AudioFormatError::InvalidData(format!("Failed to read samples: {}", e)))?
            }
            _ => {
                return Err(AudioFormatError::UnsupportedFormat(format!(
                    "Unsupported WAV format: {:?} with {} bits",
                    spec.sample_format, spec.bits_per_sample
                )));
            }
        };

        // Deinterleave into separate channel buffers
        Ok(deinterleave(&interleaved, num_channels))
    }

    /// Read a specific number of frames into the provided channel buffers.
    ///
    /// # Arguments
    ///
    /// * `buffers` - Mutable slice of channel buffers to fill
    /// * `num_frames` - Number of frames to read
    ///
    /// # Returns
    ///
    /// The actual number of frames read (may be less than requested if EOF is reached).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The number of buffers doesn't match the channel count
    /// - Reading fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::wav::WavReader;
    ///
    /// let mut reader = WavReader::open("audio.wav").unwrap();
    /// let mut left = vec![0.0; 1024];
    /// let mut right = vec![0.0; 1024];
    /// let mut buffers = vec![left.as_mut_slice(), right.as_mut_slice()];
    /// let frames_read = reader.read_frames(&mut buffers, 1024).unwrap();
    /// ```
    pub fn read_frames(&mut self, buffers: &mut [&mut [f32]], num_frames: usize) -> Result<usize> {
        if buffers.len() != self.num_channels() {
            return Err(AudioFormatError::ChannelCountMismatch {
                expected: self.num_channels(),
                actual: buffers.len(),
            });
        }

        let spec = self.reader.spec();
        let num_channels = spec.channels as usize;
        let mut frames_read = 0;

        // Read samples frame by frame
        for frame_idx in 0..num_frames {
            for ch in 0..num_channels {
                let sample = match (spec.sample_format, spec.bits_per_sample) {
                    (SampleFormat::Int, 8) => {
                        match self.reader.samples::<i8>().next() {
                            Some(Ok(s)) => {
                                if s >= 0 {
                                    s as f32 / 127.0
                                } else {
                                    s as f32 / 128.0
                                }
                            }
                            Some(Err(_)) => return Err(AudioFormatError::InvalidData("Failed to read sample".to_string())),
                            None => return Ok(frames_read),
                        }
                    }
                    (SampleFormat::Int, 16) => {
                        match self.reader.samples::<i16>().next() {
                            Some(Ok(s)) => i16_to_f32(s),
                            Some(Err(_)) => return Err(AudioFormatError::InvalidData("Failed to read sample".to_string())),
                            None => return Ok(frames_read),
                        }
                    }
                    (SampleFormat::Int, 24) => {
                        match self.reader.samples::<i32>().next() {
                            Some(Ok(s)) => i24_to_f32(s),
                            Some(Err(_)) => return Err(AudioFormatError::InvalidData("Failed to read sample".to_string())),
                            None => return Ok(frames_read),
                        }
                    }
                    (SampleFormat::Int, 32) => {
                        match self.reader.samples::<i32>().next() {
                            Some(Ok(s)) => i32_to_f32(s),
                            Some(Err(_)) => return Err(AudioFormatError::InvalidData("Failed to read sample".to_string())),
                            None => return Ok(frames_read),
                        }
                    }
                    (SampleFormat::Float, 32) => {
                        match self.reader.samples::<f32>().next() {
                            Some(Ok(s)) => s,
                            Some(Err(_)) => return Err(AudioFormatError::InvalidData("Failed to read sample".to_string())),
                            None => return Ok(frames_read),
                        }
                    }
                    _ => {
                        return Err(AudioFormatError::UnsupportedFormat(format!(
                            "Unsupported WAV format: {:?} with {} bits",
                            spec.sample_format, spec.bits_per_sample
                        )));
                    }
                };

                buffers[ch][frame_idx] = sample;
            }
            frames_read += 1;
        }

        Ok(frames_read)
    }
}

/// A WAV file writer.
///
/// This writer supports various bit depths and automatically converts f32 samples
/// in the range [-1.0, 1.0] to the appropriate format.
pub struct WavWriter {
    writer: HoundWriter<std::io::BufWriter<std::fs::File>>,
    metadata: AudioMetadata,
}

impl WavWriter {
    /// Create a new WAV file for writing.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the WAV file will be created
    /// * `sample_rate` - Sample rate in Hz
    /// * `num_channels` - Number of audio channels
    /// * `bit_depth` - Bit depth (8, 16, 24, or 32)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be created
    /// - Invalid parameters are provided
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::wav::WavWriter;
    ///
    /// let writer = WavWriter::create("output.wav", 44100.0, 2, 16).unwrap();
    /// ```
    pub fn create<P: AsRef<Path>>(
        path: P,
        sample_rate: f32,
        num_channels: usize,
        bit_depth: u16,
    ) -> Result<Self> {
        // Validate parameters
        if sample_rate <= 0.0 {
            return Err(AudioFormatError::InvalidSampleRate(sample_rate));
        }
        if num_channels == 0 {
            return Err(AudioFormatError::InvalidChannelCount(num_channels));
        }
        if !matches!(bit_depth, 8 | 16 | 24 | 32) {
            return Err(AudioFormatError::InvalidBitDepth(bit_depth));
        }

        let spec = WavSpec {
            channels: num_channels as u16,
            sample_rate: sample_rate as u32,
            bits_per_sample: bit_depth,
            sample_format: if bit_depth == 32 {
                SampleFormat::Float
            } else {
                SampleFormat::Int
            },
        };

        let writer = HoundWriter::create(path, spec)
            .map_err(|e| AudioFormatError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let metadata = AudioMetadata::with_bit_depth(sample_rate, num_channels, 0, bit_depth);

        Ok(Self { writer, metadata })
    }

    /// Get the metadata for this WAV file.
    pub fn metadata(&self) -> &AudioMetadata {
        &self.metadata
    }

    /// Get the sample rate in Hz.
    pub fn sample_rate(&self) -> f32 {
        self.metadata.sample_rate
    }

    /// Get the number of channels.
    pub fn num_channels(&self) -> usize {
        self.metadata.num_channels
    }

    /// Get the bit depth.
    pub fn bit_depth(&self) -> Option<u16> {
        self.metadata.bit_depth
    }

    /// Write samples to the WAV file.
    ///
    /// # Arguments
    ///
    /// * `samples` - A slice of channel buffers, where each buffer contains samples
    ///               as f32 values in the range [-1.0, 1.0]
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The number of channels doesn't match
    /// - Writing fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::wav::WavWriter;
    ///
    /// let left = vec![0.0, 0.5, 1.0];
    /// let right = vec![0.0, -0.5, -1.0];
    /// let samples = vec![left, right];
    ///
    /// let mut writer = WavWriter::create("output.wav", 44100.0, 2, 16).unwrap();
    /// writer.write_samples(&samples).unwrap();
    /// ```
    pub fn write_samples(&mut self, samples: &[Vec<f32>]) -> Result<()> {
        if samples.len() != self.num_channels() {
            return Err(AudioFormatError::ChannelCountMismatch {
                expected: self.num_channels(),
                actual: samples.len(),
            });
        }

        if samples.is_empty() {
            return Ok(());
        }

        // Interleave samples
        let interleaved = interleave(samples);

        // Write samples based on bit depth
        let bit_depth = self.metadata.bit_depth.unwrap_or(16);
        
        for &sample in &interleaved {
            match bit_depth {
                8 => {
                    // Convert f32 [-1.0, 1.0] to i8 [-128, 127]
                    let clamped = sample.clamp(-1.0, 1.0);
                    let i8_sample = if clamped >= 0.0 {
                        (clamped * 127.0) as i8
                    } else {
                        (clamped * 128.0) as i8
                    };
                    self.writer
                        .write_sample(i8_sample)
                        .map_err(|e| AudioFormatError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                }
                16 => {
                    let i16_sample = f32_to_i16(sample);
                    self.writer
                        .write_sample(i16_sample)
                        .map_err(|e| AudioFormatError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                }
                24 => {
                    let i24_sample = f32_to_i24(sample);
                    self.writer
                        .write_sample(i24_sample)
                        .map_err(|e| AudioFormatError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                }
                32 => {
                    // Write as float
                    self.writer
                        .write_sample(sample)
                        .map_err(|e| AudioFormatError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
                }
                _ => {
                    return Err(AudioFormatError::InvalidBitDepth(bit_depth));
                }
            }
        }

        Ok(())
    }

    /// Finalize the WAV file and flush all data to disk.
    ///
    /// This is called automatically when the writer is dropped, but calling it
    /// explicitly allows you to handle any errors that might occur.
    ///
    /// # Errors
    ///
    /// Returns an error if flushing fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::wav::WavWriter;
    ///
    /// let mut writer = WavWriter::create("output.wav", 44100.0, 2, 16).unwrap();
    /// // ... write samples ...
    /// writer.finalize().unwrap();
    /// ```
    pub fn finalize(self) -> Result<()> {
        self.writer
            .finalize()
            .map_err(|e| AudioFormatError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wav_write_read_16bit() {
        let temp_file = "test_wav_16bit.wav";
        
        // Create test data
        let left = vec![0.0, 0.5, 1.0, -0.5, -1.0];
        let right = vec![0.0, -0.5, -1.0, 0.5, 1.0];
        let samples = vec![left.clone(), right.clone()];

        // Write
        {
            let mut writer = WavWriter::create(temp_file, 44100.0, 2, 16).unwrap();
            writer.write_samples(&samples).unwrap();
            writer.finalize().unwrap();
        }

        // Read
        {
            let mut reader = WavReader::open(temp_file).unwrap();
            assert_eq!(reader.sample_rate(), 44100.0);
            assert_eq!(reader.num_channels(), 2);
            assert_eq!(reader.bit_depth(), Some(16));

            let read_samples = reader.read_all().unwrap();
            assert_eq!(read_samples.len(), 2);
            assert_eq!(read_samples[0].len(), 5);
            assert_eq!(read_samples[1].len(), 5);

            // Check samples (with tolerance for 16-bit quantization)
            for i in 0..5 {
                assert!((read_samples[0][i] - left[i]).abs() < 0.001);
                assert!((read_samples[1][i] - right[i]).abs() < 0.001);
            }
        }

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_wav_write_read_24bit() {
        let temp_file = "test_wav_24bit.wav";
        
        let samples = vec![
            vec![0.0, 0.25, 0.5, 0.75, 1.0],
            vec![0.0, -0.25, -0.5, -0.75, -1.0],
        ];

        // Write
        {
            let mut writer = WavWriter::create(temp_file, 48000.0, 2, 24).unwrap();
            writer.write_samples(&samples).unwrap();
        }

        // Read
        {
            let mut reader = WavReader::open(temp_file).unwrap();
            assert_eq!(reader.sample_rate(), 48000.0);
            assert_eq!(reader.bit_depth(), Some(24));

            let read_samples = reader.read_all().unwrap();
            
            // Check with tighter tolerance for 24-bit
            for ch in 0..2 {
                for i in 0..5 {
                    assert!((read_samples[ch][i] - samples[ch][i]).abs() < 0.0001);
                }
            }
        }

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_wav_write_read_32bit_float() {
        let temp_file = "test_wav_32bit_float.wav";
        
        let samples = vec![
            vec![0.0, 0.123456, 0.987654, -0.456789],
        ];

        // Write
        {
            let mut writer = WavWriter::create(temp_file, 96000.0, 1, 32).unwrap();
            writer.write_samples(&samples).unwrap();
        }

        // Read
        {
            let mut reader = WavReader::open(temp_file).unwrap();
            assert_eq!(reader.sample_rate(), 96000.0);
            assert_eq!(reader.num_channels(), 1);
            assert_eq!(reader.bit_depth(), Some(32));

            let read_samples = reader.read_all().unwrap();
            
            // Float should be exact
            for i in 0..samples[0].len() {
                assert!((read_samples[0][i] - samples[0][i]).abs() < 1e-6);
            }
        }

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_wav_invalid_parameters() {
        // Invalid sample rate
        assert!(WavWriter::create("test.wav", 0.0, 2, 16).is_err());
        assert!(WavWriter::create("test.wav", -44100.0, 2, 16).is_err());

        // Invalid channel count
        assert!(WavWriter::create("test.wav", 44100.0, 0, 16).is_err());

        // Invalid bit depth
        assert!(WavWriter::create("test.wav", 44100.0, 2, 12).is_err());
        assert!(WavWriter::create("test.wav", 44100.0, 2, 64).is_err());
    }

    #[test]
    fn test_wav_file_not_found() {
        let result = WavReader::open("nonexistent_file.wav");
        assert!(result.is_err());
        match result {
            Err(AudioFormatError::FileNotFound(_)) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }
}
