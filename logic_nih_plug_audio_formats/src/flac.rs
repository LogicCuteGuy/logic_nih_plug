//! FLAC file format support.
//!
//! This module provides reading of FLAC (Free Lossless Audio Codec) files.
//! FLAC is a lossless compression format that preserves audio quality while
//! reducing file size.
//!
//! # Examples
//!
//! ## Reading a FLAC file
//!
//! ```no_run
//! use logic_nih_plug_audio_formats::flac::FlacReader;
//!
//! let mut reader = FlacReader::open("audio.flac").unwrap();
//! let samples = reader.read_all().unwrap();
//! println!("Read {} channels with {} frames", samples.len(), samples[0].len());
//! ```

use crate::error::{AudioFormatError, Result};
use crate::util::{i16_to_f32, i24_to_f32, i32_to_f32};
use crate::AudioMetadata;
use claxon::FlacReader as ClaxonReader;
use std::path::Path;

/// A FLAC file reader.
///
/// This reader supports various bit depths and automatically converts samples to f32
/// in the range [-1.0, 1.0].
pub struct FlacReader {
    reader: ClaxonReader<std::fs::File>,
    metadata: AudioMetadata,
}

impl FlacReader {
    /// Open a FLAC file for reading.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the FLAC file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file is not a valid FLAC file
    /// - The file format is unsupported
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::flac::FlacReader;
    ///
    /// let reader = FlacReader::open("audio.flac").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(AudioFormatError::FileNotFound(path_ref.display().to_string()));
        }
        
        let reader = ClaxonReader::open(path_ref).map_err(|e| {
            AudioFormatError::InvalidData(format!("Failed to open FLAC file: {}", e))
        })?;

        let streaminfo = reader.streaminfo();
        
        // Validate sample rate
        if streaminfo.sample_rate == 0 {
            return Err(AudioFormatError::InvalidSampleRate(0.0));
        }
        
        // Validate channel count
        if streaminfo.channels == 0 {
            return Err(AudioFormatError::InvalidChannelCount(0));
        }

        // Validate bit depth
        if streaminfo.bits_per_sample == 0 || streaminfo.bits_per_sample > 32 {
            return Err(AudioFormatError::InvalidBitDepth(streaminfo.bits_per_sample as u16));
        }

        let num_frames = streaminfo.samples.unwrap_or(0) as usize;
        
        let metadata = AudioMetadata::with_bit_depth(
            streaminfo.sample_rate as f32,
            streaminfo.channels as usize,
            num_frames,
            streaminfo.bits_per_sample as u16,
        );

        Ok(Self { reader, metadata })
    }

    /// Get the metadata for this FLAC file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::flac::FlacReader;
    ///
    /// let reader = FlacReader::open("audio.flac").unwrap();
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

    /// Read all samples from the FLAC file.
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
    /// use logic_nih_plug_audio_formats::flac::FlacReader;
    ///
    /// let mut reader = FlacReader::open("audio.flac").unwrap();
    /// let samples = reader.read_all().unwrap();
    /// println!("Channel 0 has {} samples", samples[0].len());
    /// ```
    pub fn read_all(&mut self) -> Result<Vec<Vec<f32>>> {
        let num_channels = self.metadata.num_channels;
        let bit_depth = self.metadata.bit_depth.unwrap_or(16);
        
        // Initialize channel buffers
        let mut channels: Vec<Vec<f32>> = vec![Vec::new(); num_channels];
        
        // Create a buffer for reading blocks
        let mut buffer = Vec::with_capacity(num_channels * 4096);
        
        // Read all blocks
        let mut block_reader = self.reader.blocks();
        
        loop {
            match block_reader.read_next_or_eof(buffer) {
                Ok(Some(block)) => {
                    // Process the block
                    let block_size = block.len() / num_channels as u32;
                    
                    for frame_idx in 0..block_size {
                        for ch in 0..num_channels {
                            let sample = block.sample(ch as u32, frame_idx);
                            
                            // Convert sample to f32 based on bit depth
                            let f32_sample = match bit_depth {
                                1..=8 => {
                                    // 8-bit samples
                                    let i8_val = sample as i8;
                                    if i8_val >= 0 {
                                        i8_val as f32 / 127.0
                                    } else {
                                        i8_val as f32 / 128.0
                                    }
                                }
                                9..=16 => {
                                    // 16-bit samples
                                    i16_to_f32(sample as i16)
                                }
                                17..=24 => {
                                    // 24-bit samples
                                    i24_to_f32(sample)
                                }
                                25..=32 => {
                                    // 32-bit samples
                                    i32_to_f32(sample)
                                }
                                _ => {
                                    return Err(AudioFormatError::UnsupportedFormat(
                                        format!("Unsupported bit depth: {}", bit_depth)
                                    ));
                                }
                            };
                            
                            channels[ch].push(f32_sample);
                        }
                    }
                    
                    // Reuse the buffer for the next block
                    buffer = block.into_buffer();
                }
                Ok(None) => {
                    // End of file
                    break;
                }
                Err(e) => {
                    return Err(AudioFormatError::InvalidData(
                        format!("Failed to read FLAC block: {}", e)
                    ));
                }
            }
        }
        
        Ok(channels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Note: These tests require actual FLAC files to test reading.
    // For now, we'll test the basic API and error handling.

    #[test]
    fn test_flac_file_not_found() {
        let result = FlacReader::open("nonexistent_file.flac");
        assert!(result.is_err());
        match result {
            Err(AudioFormatError::FileNotFound(_)) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_flac_invalid_file() {
        // Create a temporary file with invalid FLAC data
        let temp_file = "test_invalid.flac";
        fs::write(temp_file, b"Not a FLAC file").unwrap();
        
        let result = FlacReader::open(temp_file);
        assert!(result.is_err());
        match result {
            Err(AudioFormatError::InvalidData(_)) => {}
            _ => panic!("Expected InvalidData error"),
        }
        
        fs::remove_file(temp_file).ok();
    }
}
