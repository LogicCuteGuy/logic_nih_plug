//! Property-based tests for audio format round-trip preservation.
//!
//! **Feature: juce-modules-integration, Property 4: Audio file round-trip preserves data**
//! **Validates: Requirements 6.1, 6.2**

use logic_nih_plug_audio_formats::wav::{WavReader, WavWriter};
use logic_nih_plug_audio_formats::aiff::{AiffReader, AiffWriter};
use proptest::prelude::*;
use std::fs;

/// A wrapper type that ensures all channels have the same length
#[derive(Debug, Clone)]
struct AudioSamples {
    channels: Vec<Vec<f32>>,
}

impl AudioSamples {
    fn new(channels: Vec<Vec<f32>>) -> Self {
        // Verify all channels have the same length
        if !channels.is_empty() {
            let expected_len = channels[0].len();
            for ch in &channels {
                assert_eq!(ch.len(), expected_len, "All channels must have the same length");
            }
        }
        Self { channels }
    }
    
    fn into_inner(self) -> Vec<Vec<f32>> {
        self.channels
    }
}

/// Generate valid audio samples in the range [-1.0, 1.0]
/// All channels will have the same number of frames
fn audio_samples_strategy() -> impl Strategy<Value = AudioSamples> {
    (1..8usize, 100..1000usize).prop_flat_map(|(num_channels, num_frames)| {
        // Generate a single flat vector of all samples
        // Use no_shrink() to prevent proptest from breaking the channel length invariant
        prop::collection::vec(-1.0f32..=1.0f32, num_channels * num_frames)
            .no_shrink()
            .prop_map(move |flat_samples| {
                // Split into channels
                let mut channels = vec![Vec::with_capacity(num_frames); num_channels];
                for (i, &sample) in flat_samples.iter().enumerate() {
                    let channel = i % num_channels;
                    channels[channel].push(sample);
                }
                AudioSamples::new(channels)
            })
    })
}

/// Generate valid sample rates
fn sample_rate_strategy() -> impl Strategy<Value = f32> {
    proptest::sample::select(vec![22050.0, 44100.0, 48000.0, 88200.0, 96000.0])
}

/// Generate valid bit depths for WAV
fn bit_depth_strategy() -> impl Strategy<Value = u16> {
    proptest::sample::select(vec![16, 24, 32])
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,
        ..ProptestConfig::default()
    })]

    /// **Feature: juce-modules-integration, Property 4: Audio file round-trip preserves data**
    /// **Validates: Requirements 6.1, 6.2**
    ///
    /// For any audio buffer, writing to a WAV file and reading it back should produce
    /// equivalent sample values within the precision of the file format.
    #[test]
    fn prop_wav_roundtrip_preserves_data(
        audio_samples in audio_samples_strategy(),
        sample_rate in sample_rate_strategy(),
        bit_depth in bit_depth_strategy(),
    ) {
        let temp_file = format!("test_prop_wav_{}_{}.wav", 
            std::process::id(), 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        );
        
        let samples = audio_samples.into_inner();
        let num_channels = samples.len();
        
        // Write
        {
            let mut writer = WavWriter::create(&temp_file, sample_rate, num_channels, bit_depth)
                .expect("Failed to create WAV writer");
            writer.write_samples(&samples)
                .expect("Failed to write samples");
            writer.finalize()
                .expect("Failed to finalize WAV file");
        }
        
        // Read
        let read_samples = {
            let mut reader = WavReader::open(&temp_file)
                .expect("Failed to open WAV file");
            
            // Verify metadata
            prop_assert_eq!(reader.sample_rate(), sample_rate);
            prop_assert_eq!(reader.num_channels(), num_channels);
            prop_assert_eq!(reader.bit_depth(), Some(bit_depth));
            
            reader.read_all()
                .expect("Failed to read samples")
        };
        
        // Compare
        prop_assert_eq!(samples.len(), read_samples.len(), "Channel count mismatch");
        
        for (ch_idx, (orig_ch, read_ch)) in samples.iter().zip(read_samples.iter()).enumerate() {
            prop_assert_eq!(orig_ch.len(), read_ch.len(), 
                "Frame count mismatch in channel {}", ch_idx);
            
            // Determine tolerance based on bit depth
            // Add a small epsilon to account for rounding in both directions
            let tolerance = match bit_depth {
                16 => 2.0 / 32768.0,  // 16-bit precision + epsilon
                24 => 2.0 / 8388608.0, // 24-bit precision + epsilon
                32 => 1e-5,            // 32-bit float precision
                _ => unreachable!(),
            };
            
            for (frame_idx, (orig, read)) in orig_ch.iter().zip(read_ch.iter()).enumerate() {
                prop_assert!(
                    (orig - read).abs() <= tolerance,
                    "Sample mismatch at channel {} frame {}: {} vs {} (diff: {})",
                    ch_idx, frame_idx, orig, read, (orig - read).abs()
                );
            }
        }
        
        // Cleanup
        fs::remove_file(&temp_file).ok();
    }

    /// **Feature: juce-modules-integration, Property 4: Audio file round-trip preserves data**
    /// **Validates: Requirements 6.1, 6.2**
    ///
    /// For any audio buffer, writing to an AIFF file and reading it back should produce
    /// equivalent sample values within the precision of the file format.
    #[test]
    fn prop_aiff_roundtrip_preserves_data(
        audio_samples in audio_samples_strategy(),
        sample_rate in sample_rate_strategy(),
        bit_depth in bit_depth_strategy(),
    ) {
        let temp_file = format!("test_prop_aiff_{}_{}.aiff", 
            std::process::id(), 
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        );
        
        let samples = audio_samples.into_inner();
        let num_channels = samples.len();
        
        // Write
        {
            let mut writer = AiffWriter::create(&temp_file, sample_rate, num_channels, bit_depth)
                .expect("Failed to create AIFF writer");
            writer.write_samples(&samples)
                .expect("Failed to write samples");
            writer.finalize()
                .expect("Failed to finalize AIFF file");
        }
        
        // Read
        let read_samples = {
            let mut reader = AiffReader::open(&temp_file)
                .expect("Failed to open AIFF file");
            
            // Verify metadata
            prop_assert_eq!(reader.sample_rate(), sample_rate);
            prop_assert_eq!(reader.num_channels(), num_channels);
            prop_assert_eq!(reader.bit_depth(), Some(bit_depth));
            
            reader.read_all()
                .expect("Failed to read samples")
        };
        
        // Compare
        prop_assert_eq!(samples.len(), read_samples.len(), "Channel count mismatch");
        
        for (ch_idx, (orig_ch, read_ch)) in samples.iter().zip(read_samples.iter()).enumerate() {
            prop_assert_eq!(orig_ch.len(), read_ch.len(), 
                "Frame count mismatch in channel {}", ch_idx);
            
            // Determine tolerance based on bit depth
            // Add a small epsilon to account for rounding in both directions
            let tolerance = match bit_depth {
                16 => 2.0 / 32768.0,  // 16-bit precision + epsilon
                24 => 2.0 / 8388608.0, // 24-bit precision + epsilon
                32 => 1e-5,            // 32-bit float precision
                _ => unreachable!(),
            };
            
            for (frame_idx, (orig, read)) in orig_ch.iter().zip(read_ch.iter()).enumerate() {
                prop_assert!(
                    (orig - read).abs() <= tolerance,
                    "Sample mismatch at channel {} frame {}: {} vs {} (diff: {})",
                    ch_idx, frame_idx, orig, read, (orig - read).abs()
                );
            }
        }
        
        // Cleanup
        fs::remove_file(&temp_file).ok();
    }
}
