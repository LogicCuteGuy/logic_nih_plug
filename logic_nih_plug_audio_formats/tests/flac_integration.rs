//! Integration tests for FLAC file reading.

#![cfg(feature = "flac")]

use logic_nih_plug_audio_formats::flac::FlacReader;
use logic_nih_plug_audio_formats::AudioFormatError;

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
    use std::fs;
    
    // Create a temporary file with invalid FLAC data
    let temp_file = "test_invalid_flac.flac";
    fs::write(temp_file, b"Not a FLAC file").unwrap();
    
    let result = FlacReader::open(temp_file);
    assert!(result.is_err());
    match result {
        Err(AudioFormatError::InvalidData(_)) => {}
        _ => panic!("Expected InvalidData error"),
    }
    
    fs::remove_file(temp_file).ok();
}

// Note: To test actual FLAC reading, we would need real FLAC files.
// The claxon library is well-tested, so our wrapper should work correctly
// if the basic API integration tests pass.
