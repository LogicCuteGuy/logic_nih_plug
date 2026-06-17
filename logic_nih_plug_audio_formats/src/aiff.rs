//! AIFF file format support.
//!
//! This module provides reading and writing of AIFF (Audio Interchange File Format) files.
//! AIFF is a format developed by Apple and uses big-endian byte order.
//!
//! # Examples
//!
//! ## Reading an AIFF file
//!
//! ```no_run
//! use logic_nih_plug_audio_formats::aiff::AiffReader;
//!
//! let mut reader = AiffReader::open("audio.aiff").unwrap();
//! let samples = reader.read_all().unwrap();
//! println!("Read {} channels with {} frames", samples.len(), samples[0].len());
//! ```
//!
//! ## Writing an AIFF file
//!
//! ```no_run
//! use logic_nih_plug_audio_formats::aiff::AiffWriter;
//!
//! let left = vec![0.0, 0.5, 1.0, 0.5, 0.0];
//! let right = vec![0.0, -0.5, -1.0, -0.5, 0.0];
//! let samples = vec![left, right];
//!
//! let mut writer = AiffWriter::create("output.aiff", 44100.0, 2, 16).unwrap();
//! writer.write_samples(&samples).unwrap();
//! ```

use crate::error::{AudioFormatError, Result};
use crate::util::{deinterleave, f32_to_i16, f32_to_i24, f32_to_i32, i16_to_f32, i24_to_f32, i32_to_f32, interleave};
use crate::AudioMetadata;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

// AIFF chunk identifiers (4-byte codes)
const FORM_ID: &[u8; 4] = b"FORM";
const AIFF_ID: &[u8; 4] = b"AIFF";
const COMM_ID: &[u8; 4] = b"COMM";
const SSND_ID: &[u8; 4] = b"SSND";

/// A helper function to read a big-endian u32
fn read_be_u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

/// A helper function to read a big-endian i16
fn read_be_i16<R: Read>(reader: &mut R) -> Result<i16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(i16::from_be_bytes(buf))
}

/// A helper function to read a big-endian i32
fn read_be_i32<R: Read>(reader: &mut R) -> Result<i32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(i32::from_be_bytes(buf))
}

/// A helper function to write a big-endian u32
fn write_be_u32<W: Write>(writer: &mut W, value: u32) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// A helper function to write a big-endian i16
fn write_be_i16<W: Write>(writer: &mut W, value: i16) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// A helper function to write a big-endian i32
fn write_be_i32<W: Write>(writer: &mut W, value: i32) -> Result<()> {
    writer.write_all(&value.to_be_bytes())?;
    Ok(())
}

/// Convert an 80-bit extended precision float to f64 (for sample rate)
fn read_extended<R: Read>(reader: &mut R) -> Result<f64> {
    let mut buf = [0u8; 10];
    reader.read_exact(&mut buf)?;
    
    // Parse 80-bit IEEE 754 extended precision
    let sign = (buf[0] & 0x80) != 0;
    let exponent = (((buf[0] & 0x7F) as u16) << 8) | (buf[1] as u16);
    let mantissa = u64::from_be_bytes([
        buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
    ]);
    
    if exponent == 0 && mantissa == 0 {
        return Ok(0.0);
    }
    
    // Convert to f64
    let f = mantissa as f64 / (1u64 << 63) as f64;
    let result = f * 2.0f64.powi((exponent as i32) - 16383);
    
    Ok(if sign { -result } else { result })
}

/// Convert f64 to 80-bit extended precision float
fn write_extended<W: Write>(writer: &mut W, value: f64) -> Result<()> {
    if value == 0.0 {
        writer.write_all(&[0u8; 10])?;
        return Ok(());
    }
    
    let sign = if value < 0.0 { 0x80u8 } else { 0x00u8 };
    let abs_value = value.abs();
    
    // Calculate exponent and mantissa
    let exponent = abs_value.log2().floor() as i32 + 16383;
    let mantissa = (abs_value / 2.0f64.powi(exponent - 16383)) * (1u64 << 63) as f64;
    
    let mut buf = [0u8; 10];
    buf[0] = sign | ((exponent >> 8) as u8 & 0x7F);
    buf[1] = (exponent & 0xFF) as u8;
    
    let mantissa_bytes = (mantissa as u64).to_be_bytes();
    buf[2..10].copy_from_slice(&mantissa_bytes);
    
    writer.write_all(&buf)?;
    Ok(())
}

/// An AIFF file reader.
///
/// This reader supports various bit depths and automatically converts samples to f32
/// in the range [-1.0, 1.0].
pub struct AiffReader {
    reader: BufReader<File>,
    metadata: AudioMetadata,
    ssnd_offset: u64,
}

impl AiffReader {
    /// Open an AIFF file for reading.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the AIFF file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file is not a valid AIFF file
    /// - The file format is unsupported
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::aiff::AiffReader;
    ///
    /// let reader = AiffReader::open("audio.aiff").unwrap();
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(AudioFormatError::FileNotFound(path_ref.display().to_string()));
        }
        
        let file = File::open(path_ref)?;
        let mut reader = BufReader::new(file);

        // Read FORM chunk
        let mut form_id = [0u8; 4];
        reader.read_exact(&mut form_id)?;
        if &form_id != FORM_ID {
            return Err(AudioFormatError::InvalidData("Not an AIFF file (missing FORM)".to_string()));
        }
        
        let _form_size = read_be_u32(&mut reader)?;
        
        let mut aiff_id = [0u8; 4];
        reader.read_exact(&mut aiff_id)?;
        if &aiff_id != AIFF_ID {
            return Err(AudioFormatError::InvalidData("Not an AIFF file (missing AIFF)".to_string()));
        }
        
        // Parse chunks
        let mut num_channels = 0u16;
        let mut num_frames = 0u32;
        let mut bit_depth = 0u16;
        let mut sample_rate = 0.0f64;
        let mut ssnd_offset = 0u64;
        
        loop {
            let mut chunk_id = [0u8; 4];
            if reader.read_exact(&mut chunk_id).is_err() {
                break; // End of file
            }
            
            let chunk_size = read_be_u32(&mut reader)?;
            
            match &chunk_id {
                COMM_ID => {
                    // Common chunk
                    num_channels = read_be_i16(&mut reader)? as u16;
                    num_frames = read_be_u32(&mut reader)?;
                    bit_depth = read_be_i16(&mut reader)? as u16;
                    sample_rate = read_extended(&mut reader)?;
                }
                SSND_ID => {
                    // Sound data chunk
                    let offset = read_be_u32(&mut reader)?;
                    let _block_size = read_be_u32(&mut reader)?;
                    
                    ssnd_offset = reader.stream_position()? + offset as u64;
                }
                _ => {
                    // Skip unknown chunks
                    reader.seek(SeekFrom::Current(chunk_size as i64))?;
                }
            }
            
            // AIFF chunks are word-aligned
            if chunk_size % 2 != 0 {
                reader.seek(SeekFrom::Current(1))?;
            }
        }
        
        // Validate
        if num_channels == 0 {
            return Err(AudioFormatError::InvalidChannelCount(0));
        }
        if sample_rate <= 0.0 {
            return Err(AudioFormatError::InvalidSampleRate(sample_rate as f32));
        }
        if !matches!(bit_depth, 8 | 16 | 24 | 32) {
            return Err(AudioFormatError::InvalidBitDepth(bit_depth));
        }

        let metadata = AudioMetadata::with_bit_depth(
            sample_rate as f32,
            num_channels as usize,
            num_frames as usize,
            bit_depth,
        );
        
        Ok(Self {
            reader,
            metadata,
            ssnd_offset,
        })
    }
    
    /// Get the metadata for this AIFF file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::aiff::AiffReader;
    ///
    /// let reader = AiffReader::open("audio.aiff").unwrap();
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

    /// Read all samples from the AIFF file.
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
    /// use logic_nih_plug_audio_formats::aiff::AiffReader;
    ///
    /// let mut reader = AiffReader::open("audio.aiff").unwrap();
    /// let samples = reader.read_all().unwrap();
    /// println!("Channel 0 has {} samples", samples[0].len());
    /// ```
    pub fn read_all(&mut self) -> Result<Vec<Vec<f32>>> {
        // Seek to sound data
        self.reader.seek(SeekFrom::Start(self.ssnd_offset))?;
        
        let num_channels = self.metadata.num_channels;
        let num_frames = self.metadata.num_frames;
        let bit_depth = self.metadata.bit_depth.unwrap_or(16);
        
        let total_samples = num_channels * num_frames;
        let mut interleaved = Vec::with_capacity(total_samples);
        
        // Read samples based on bit depth
        for _ in 0..total_samples {
            let sample = match bit_depth {
                8 => {
                    let mut buf = [0u8; 1];
                    self.reader.read_exact(&mut buf)?;
                    let i8_val = buf[0] as i8;
                    if i8_val >= 0 {
                        i8_val as f32 / 127.0
                    } else {
                        i8_val as f32 / 128.0
                    }
                }
                16 => {
                    let i16_val = read_be_i16(&mut self.reader)?;
                    i16_to_f32(i16_val)
                }
                24 => {
                    // Read 3 bytes and convert to i32
                    let mut buf = [0u8; 3];
                    self.reader.read_exact(&mut buf)?;
                    let i24_val = if buf[0] & 0x80 != 0 {
                        // Negative number - sign extend
                        i32::from_be_bytes([0xFF, buf[0], buf[1], buf[2]])
                    } else {
                        i32::from_be_bytes([0x00, buf[0], buf[1], buf[2]])
                    };
                    i24_to_f32(i24_val)
                }
                32 => {
                    let i32_val = read_be_i32(&mut self.reader)?;
                    i32_to_f32(i32_val)
                }
                _ => {
                    return Err(AudioFormatError::UnsupportedFormat(format!(
                        "Unsupported bit depth: {}",
                        bit_depth
                    )));
                }
            };
            
            interleaved.push(sample);
        }
        
        // Deinterleave into separate channel buffers
        Ok(deinterleave(&interleaved, num_channels))
    }
}

/// An AIFF file writer.
///
/// This writer supports various bit depths and automatically converts f32 samples
/// in the range [-1.0, 1.0] to the appropriate format.
pub struct AiffWriter {
    writer: BufWriter<File>,
    metadata: AudioMetadata,
    samples_written: usize,
}

impl AiffWriter {
    /// Create a new AIFF file for writing.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the AIFF file will be created
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
    /// use logic_nih_plug_audio_formats::aiff::AiffWriter;
    ///
    /// let writer = AiffWriter::create("output.aiff", 44100.0, 2, 16).unwrap();
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
        
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        
        // Write FORM chunk header (we'll update the size later)
        writer.write_all(FORM_ID)?;
        write_be_u32(&mut writer, 0)?; // Placeholder for size
        writer.write_all(AIFF_ID)?;
        
        // Write COMM chunk
        writer.write_all(COMM_ID)?;
        write_be_u32(&mut writer, 18)?; // COMM chunk size
        write_be_i16(&mut writer, num_channels as i16)?;
        write_be_u32(&mut writer, 0)?; // Placeholder for num_frames
        write_be_i16(&mut writer, bit_depth as i16)?;
        write_extended(&mut writer, sample_rate as f64)?;
        
        // Write SSND chunk header
        writer.write_all(SSND_ID)?;
        write_be_u32(&mut writer, 0)?; // Placeholder for size
        write_be_u32(&mut writer, 0)?; // Offset
        write_be_u32(&mut writer, 0)?; // Block size
        
        let metadata = AudioMetadata::with_bit_depth(sample_rate, num_channels, 0, bit_depth);
        
        Ok(Self {
            writer,
            metadata,
            samples_written: 0,
        })
    }

    /// Get the metadata for this AIFF file.
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
    
    /// Write samples to the AIFF file.
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
    /// use logic_nih_plug_audio_formats::aiff::AiffWriter;
    ///
    /// let left = vec![0.0, 0.5, 1.0];
    /// let right = vec![0.0, -0.5, -1.0];
    /// let samples = vec![left, right];
    ///
    /// let mut writer = AiffWriter::create("output.aiff", 44100.0, 2, 16).unwrap();
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
        let bit_depth = self.metadata.bit_depth.unwrap_or(16);
        
        // Write samples based on bit depth
        for &sample in &interleaved {
            match bit_depth {
                8 => {
                    let clamped = sample.clamp(-1.0, 1.0);
                    let i8_sample = if clamped >= 0.0 {
                        (clamped * 127.0) as i8
                    } else {
                        (clamped * 128.0) as i8
                    };
                    self.writer.write_all(&[i8_sample as u8])?;
                }
                16 => {
                    let i16_sample = f32_to_i16(sample);
                    write_be_i16(&mut self.writer, i16_sample)?;
                }
                24 => {
                    let i24_sample = f32_to_i24(sample);
                    // Write 3 bytes (big-endian)
                    let bytes = i24_sample.to_be_bytes();
                    self.writer.write_all(&bytes[1..4])?;
                }
                32 => {
                    let i32_sample = f32_to_i32(sample);
                    write_be_i32(&mut self.writer, i32_sample)?;
                }
                _ => {
                    return Err(AudioFormatError::InvalidBitDepth(bit_depth));
                }
            }
        }
        
        self.samples_written += interleaved.len();
        
        Ok(())
    }

    /// Finalize the AIFF file and flush all data to disk.
    ///
    /// This updates the file headers with the correct sizes and flushes all data.
    /// You should call this explicitly to handle any errors that might occur.
    ///
    /// # Errors
    ///
    /// Returns an error if flushing or updating headers fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use logic_nih_plug_audio_formats::aiff::AiffWriter;
    ///
    /// let mut writer = AiffWriter::create("output.aiff", 44100.0, 2, 16).unwrap();
    /// // ... write samples ...
    /// writer.finalize().unwrap();
    /// ```
    pub fn finalize(&mut self) -> Result<()> {
        self.writer.flush()?;
        
        // Get the underlying file to update headers
        let file = self.writer.get_mut();
        
        // Calculate sizes
        let num_channels = self.metadata.num_channels;
        let num_frames = self.samples_written / num_channels;
        let bit_depth = self.metadata.bit_depth.unwrap_or(16);
        let bytes_per_sample = (bit_depth / 8) as u32;
        let ssnd_data_size = (self.samples_written as u32) * bytes_per_sample;
        let ssnd_chunk_size = ssnd_data_size + 8; // +8 for offset and block_size fields
        
        // FORM size = 4 (AIFF) + 8 (COMM header) + 18 (COMM data) + 8 (SSND header) + ssnd_chunk_size
        let form_size = 4 + 8 + 18 + 8 + ssnd_chunk_size;
        
        // Update FORM size
        file.seek(SeekFrom::Start(4))?;
        write_be_u32(file, form_size)?;
        
        // Update num_frames in COMM chunk
        file.seek(SeekFrom::Start(22))?; // 12 (FORM+size+AIFF) + 8 (COMM header) + 2 (num_channels)
        write_be_u32(file, num_frames as u32)?;
        
        // Update SSND chunk size
        file.seek(SeekFrom::Start(42))?; // 12 + 8 + 18 + 4 (SSND ID)
        write_be_u32(file, ssnd_chunk_size)?;
        
        file.flush()?;
        
        Ok(())
    }
}

impl Drop for AiffWriter {
    fn drop(&mut self) {
        // Try to finalize on drop, but ignore errors since we can't handle them in Drop
        let _ = self.finalize();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_aiff_write_read_16bit() {
        let temp_file = "test_aiff_16bit.aiff";
        
        // Create test data
        let left = vec![0.0, 0.5, 1.0, -0.5, -1.0];
        let right = vec![0.0, -0.5, -1.0, 0.5, 1.0];
        let samples = vec![left.clone(), right.clone()];

        // Write
        {
            let mut writer = AiffWriter::create(temp_file, 44100.0, 2, 16).unwrap();
            writer.write_samples(&samples).unwrap();
            writer.finalize().unwrap();
        }

        // Read
        {
            let mut reader = AiffReader::open(temp_file).unwrap();
            assert_eq!(reader.sample_rate(), 44100.0);
            assert_eq!(reader.num_channels(), 2);
            assert_eq!(reader.bit_depth(), Some(16));

            let read_samples = reader.read_all().unwrap();
            assert_eq!(read_samples.len(), 2);
            assert_eq!(read_samples[0].len(), 5);
            assert_eq!(read_samples[1].len(), 5);

            // Check samples (with tolerance for 16-bit quantization)
            for i in 0..5 {
                assert!((read_samples[0][i] - left[i]).abs() < 0.001, 
                    "Left channel sample {} mismatch: {} vs {}", i, read_samples[0][i], left[i]);
                assert!((read_samples[1][i] - right[i]).abs() < 0.001,
                    "Right channel sample {} mismatch: {} vs {}", i, read_samples[1][i], right[i]);
            }
        }

        // Cleanup
        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_aiff_write_read_24bit() {
        let temp_file = "test_aiff_24bit.aiff";
        
        let samples = vec![
            vec![0.0, 0.25, 0.5, 0.75, 1.0],
            vec![0.0, -0.25, -0.5, -0.75, -1.0],
        ];

        // Write
        {
            let mut writer = AiffWriter::create(temp_file, 48000.0, 2, 24).unwrap();
            writer.write_samples(&samples).unwrap();
            writer.finalize().unwrap();
        }

        // Read
        {
            let mut reader = AiffReader::open(temp_file).unwrap();
            assert_eq!(reader.sample_rate(), 48000.0);
            assert_eq!(reader.bit_depth(), Some(24));

            let read_samples = reader.read_all().unwrap();
            
            // Check with tighter tolerance for 24-bit
            for ch in 0..2 {
                for i in 0..5 {
                    assert!((read_samples[ch][i] - samples[ch][i]).abs() < 0.0001,
                        "Channel {} sample {} mismatch: {} vs {}", ch, i, read_samples[ch][i], samples[ch][i]);
                }
            }
        }

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_aiff_write_read_32bit() {
        let temp_file = "test_aiff_32bit.aiff";
        
        let samples = vec![
            vec![0.0, 0.123456, 0.987654, -0.456789],
        ];

        // Write
        {
            let mut writer = AiffWriter::create(temp_file, 96000.0, 1, 32).unwrap();
            writer.write_samples(&samples).unwrap();
            writer.finalize().unwrap();
        }

        // Read
        {
            let mut reader = AiffReader::open(temp_file).unwrap();
            assert_eq!(reader.sample_rate(), 96000.0);
            assert_eq!(reader.num_channels(), 1);
            assert_eq!(reader.bit_depth(), Some(32));

            let read_samples = reader.read_all().unwrap();
            
            // 32-bit integer should have good precision
            for i in 0..samples[0].len() {
                assert!((read_samples[0][i] - samples[0][i]).abs() < 1e-6,
                    "Sample {} mismatch: {} vs {}", i, read_samples[0][i], samples[0][i]);
            }
        }

        fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_aiff_invalid_parameters() {
        // Invalid sample rate
        assert!(AiffWriter::create("test.aiff", 0.0, 2, 16).is_err());
        assert!(AiffWriter::create("test.aiff", -44100.0, 2, 16).is_err());

        // Invalid channel count
        assert!(AiffWriter::create("test.aiff", 44100.0, 0, 16).is_err());

        // Invalid bit depth
        assert!(AiffWriter::create("test.aiff", 44100.0, 2, 12).is_err());
        assert!(AiffWriter::create("test.aiff", 44100.0, 2, 64).is_err());
    }

    #[test]
    fn test_aiff_file_not_found() {
        let result = AiffReader::open("nonexistent_file.aiff");
        assert!(result.is_err());
        match result {
            Err(AudioFormatError::FileNotFound(_)) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_aiff_mono() {
        let temp_file = "test_aiff_mono.aiff";
        
        let samples = vec![vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5]];

        // Write
        {
            let mut writer = AiffWriter::create(temp_file, 22050.0, 1, 16).unwrap();
            writer.write_samples(&samples).unwrap();
            writer.finalize().unwrap();
        }

        // Read
        {
            let mut reader = AiffReader::open(temp_file).unwrap();
            assert_eq!(reader.num_channels(), 1);
            assert_eq!(reader.sample_rate(), 22050.0);

            let read_samples = reader.read_all().unwrap();
            assert_eq!(read_samples.len(), 1);
            assert_eq!(read_samples[0].len(), 6);
        }

        fs::remove_file(temp_file).ok();
    }
}
