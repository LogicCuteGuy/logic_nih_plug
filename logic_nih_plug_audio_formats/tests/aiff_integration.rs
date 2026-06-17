//! Integration tests for AIFF file format support

use nih_plug_audio_formats::aiff::{AiffReader, AiffWriter};
use std::fs;

#[test]
fn test_aiff_round_trip() {
    let temp_file = "test_aiff_integration.aiff";
    
    // Create test data with multiple channels
    let left = vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
    let right = vec![0.0, -0.1, -0.2, -0.3, -0.4, -0.5, -0.6, -0.7, -0.8, -0.9, -1.0];
    let samples = vec![left.clone(), right.clone()];
    
    // Write AIFF file
    {
        let mut writer = AiffWriter::create(temp_file, 44100.0, 2, 16).unwrap();
        writer.write_samples(&samples).unwrap();
        writer.finalize().unwrap();
    }
    
    // Read AIFF file
    {
        let mut reader = AiffReader::open(temp_file).unwrap();
        
        // Verify metadata
        assert_eq!(reader.sample_rate(), 44100.0);
        assert_eq!(reader.num_channels(), 2);
        assert_eq!(reader.num_frames(), 11);
        assert_eq!(reader.bit_depth(), Some(16));
        
        // Read samples
        let read_samples = reader.read_all().unwrap();
        assert_eq!(read_samples.len(), 2);
        assert_eq!(read_samples[0].len(), 11);
        assert_eq!(read_samples[1].len(), 11);
        
        // Verify samples (with tolerance for 16-bit quantization)
        for i in 0..11 {
            assert!(
                (read_samples[0][i] - left[i]).abs() < 0.001,
                "Left channel sample {} mismatch: {} vs {}",
                i,
                read_samples[0][i],
                left[i]
            );
            assert!(
                (read_samples[1][i] - right[i]).abs() < 0.001,
                "Right channel sample {} mismatch: {} vs {}",
                i,
                read_samples[1][i],
                right[i]
            );
        }
    }
    
    // Cleanup
    fs::remove_file(temp_file).ok();
}

#[test]
fn test_aiff_different_sample_rates() {
    let sample_rates = vec![22050.0, 44100.0, 48000.0, 96000.0];
    
    for sample_rate in sample_rates {
        let temp_file = format!("test_aiff_sr_{}.aiff", sample_rate as u32);
        
        let samples = vec![vec![0.5; 100]];
        
        // Write
        {
            let mut writer = AiffWriter::create(&temp_file, sample_rate, 1, 16).unwrap();
            writer.write_samples(&samples).unwrap();
            writer.finalize().unwrap();
        }
        
        // Read and verify
        {
            let mut reader = AiffReader::open(&temp_file).unwrap();
            assert_eq!(reader.sample_rate(), sample_rate);
            let read_samples = reader.read_all().unwrap();
            assert_eq!(read_samples[0].len(), 100);
        }
        
        fs::remove_file(&temp_file).ok();
    }
}
