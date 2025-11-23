//! OGG Vorbis file format support.
//!
//! This module provides reading of OGG Vorbis files.
//! OGG Vorbis is a lossy compression format that provides good quality
//! at lower bitrates.
//!
//! # Examples
//!
//! ## Reading an OGG file
//!
//! ```no_run
//! use nih_plug_audio_formats::ogg::OggReader;
//!
//! let mut reader = OggReader::open("audio.ogg").unwrap();
//! let samples = reader.read_all().unwrap();
//! println!("Read {} channels with {} frames", samples.len(), samples[0].len());
//! ```

use crate::error::{AudioFormatError, Result};
use crate::AudioMetadata;
use lewton::inside_ogg::OggStreamReader;
use std::fs::File;
use std::path::Path;

/// An OGG Vorbis file reader.
///
/// This reader automatically converts samples to f32 in the range [-1.0, 1.0].
pub struct OggReader {
    reader: OggStreamReader<File>,
    metadata: AudioMetadata,
}

impl OggReader {
    /// Open an OGG Vorbis file for reading.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the OGG file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file is not a valid OGG Vorbis file
    /// - The file format is unsupported
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_audio_formats::ogg::OggReader;
    ///
    /// let reader = OggReader::open("audio.ogg").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(AudioFormatError::FileNotFound(path_ref.display().to_string()));
        }
        
        let file = File::open(path_ref).map_err(|e| {
            AudioFormatError::IoError(e)
        })?;
        
        let reader = OggStreamReader::new(file).map_err(|e| {
            AudioFormatError::InvalidData(format!("Failed to open OGG file: {:?}", e))
        })?;

        // Get stream information
        let sample_rate = reader.ident_hdr.audio_sample_rate;
        let num_channels = reader.ident_hdr.audio_channels as usize;
        
        // Validate sample rate
        if sample_rate == 0 {
            return Err(AudioFormatError::InvalidSampleRate(0.0));
        }
        
        // Validate channel count
        if num_channels == 0 {
            return Err(AudioFormatError::InvalidChannelCount(0));
        }

        // OGG doesn't store total sample count in header, so we set it to 0
        // It will be determined when reading
        let metadata = AudioMetadata::new(
            sample_rate as f32,
            num_channels,
            0, // Will be updated after reading
        );

        Ok(Self { reader, metadata })
    }

    /// Get the metadata for this OGG file.
    ///
    /// Note: The `num_frames` field will be 0 until `read_all()` is called,
    /// as OGG doesn't store the total sample count in the header.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nih_plug_audio_formats::ogg::OggReader;
    ///
    /// let reader = OggReader::open("audio.ogg").unwrap();
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
    ///
    /// Note: This will be 0 until `read_all()` is called.
    pub fn num_frames(&self) -> usize {
        self.metadata.num_frames
    }

    /// Get the bit depth.
    ///
    /// OGG Vorbis is a lossy format and doesn't have a fixed bit depth,
    /// so this always returns None.
    pub fn bit_depth(&self) -> Option<u16> {
        None
    }

    /// Read all samples from the OGG file.
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
    /// use nih_plug_audio_formats::ogg::OggReader;
    ///
    /// let mut reader = OggReader::open("audio.ogg").unwrap();
    /// let samples = reader.read_all().unwrap();
    /// println!("Channel 0 has {} samples", samples[0].len());
    /// ```
    pub fn read_all(&mut self) -> Result<Vec<Vec<f32>>> {
        let num_channels = self.metadata.num_channels;
        
        // Initialize channel buffers
        let mut channels: Vec<Vec<f32>> = vec![Vec::new(); num_channels];
        
        // Read all packets
        loop {
            match self.reader.read_dec_packet_itl() {
                Ok(Some(packet)) => {
                    // lewton returns interleaved i16 samples
                    // Convert to f32 and deinterleave
                    for (i, &sample) in packet.iter().enumerate() {
                        let channel = i % num_channels;
                        // Convert i16 to f32 in range [-1.0, 1.0]
                        let f32_sample = if sample >= 0 {
                            sample as f32 / i16::MAX as f32
                        } else {
                            sample as f32 / -(i16::MIN as f32)
                        };
                        channels[channel].push(f32_sample);
                    }
                }
                Ok(None) => {
                    // End of stream
                    break;
                }
                Err(e) => {
                    return Err(AudioFormatError::InvalidData(
                        format!("Failed to read OGG packet: {:?}", e)
                    ));
                }
            }
        }
        
        // Update metadata with actual frame count
        if !channels.is_empty() {
            self.metadata.num_frames = channels[0].len();
        }
        
        Ok(channels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_ogg_file_not_found() {
        let result = OggReader::open("nonexistent_file.ogg");
        assert!(result.is_err());
        match result {
            Err(AudioFormatError::FileNotFound(_)) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_ogg_invalid_file() {
        // Create a temporary file with invalid OGG data
        let temp_file = "test_invalid.ogg";
        fs::write(temp_file, b"Not an OGG file").unwrap();
        
        let result = OggReader::open(temp_file);
        assert!(result.is_err());
        match result {
            Err(AudioFormatError::InvalidData(_)) => {}
            _ => panic!("Expected InvalidData error"),
        }
        
        fs::remove_file(temp_file).ok();
    }
}
