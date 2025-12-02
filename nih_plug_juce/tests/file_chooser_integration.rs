//! Integration tests for FileChooser functionality.
//!
//! These tests verify that the FileChooser FFI bridge works correctly
//! for showing file open/save dialogs.

use nih_plug_juce::dialogs::FileChooser;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[test]
fn test_file_chooser_creation() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser
    let result = FileChooser::new(
        "Select File",
        &PathBuf::from("."),
        "*.txt;*.md"
    );
    
    assert!(result.is_ok(), "FileChooser creation should succeed");
}

#[test]
fn test_file_chooser_browse_for_file_to_open() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser
    let mut chooser = FileChooser::new(
        "Open File",
        &PathBuf::from("."),
        "*.txt"
    ).expect("Failed to create FileChooser");
    
    // Create a flag to track if the callback was invoked
    let callback_invoked = Arc::new(Mutex::new(false));
    let callback_invoked_clone = callback_invoked.clone();
    
    // Browse for a file to open
    let result = chooser.browse_for_file_to_open(move |path| {
        *callback_invoked_clone.lock().unwrap() = true;
        // In a headless environment, path will likely be None
        // In a real GUI environment, the user would select a file
    });
    
    // Verify the operation succeeded
    assert!(result.is_ok(), "browse_for_file_to_open should succeed");
    
    // Note: In a headless environment, the callback may not actually be invoked
    // since there's no GUI event loop running. This test mainly verifies that
    // the FFI bridge doesn't crash.
}

#[test]
fn test_file_chooser_browse_for_file_to_save() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser
    let mut chooser = FileChooser::new(
        "Save File",
        &PathBuf::from("."),
        "*.txt"
    ).expect("Failed to create FileChooser");
    
    // Create a flag to track if the callback was invoked
    let callback_invoked = Arc::new(Mutex::new(false));
    let callback_invoked_clone = callback_invoked.clone();
    
    // Browse for a file to save
    let result = chooser.browse_for_file_to_save(move |path| {
        *callback_invoked_clone.lock().unwrap() = true;
        // In a headless environment, path will likely be None
        // In a real GUI environment, the user would select a file
    });
    
    // Verify the operation succeeded
    assert!(result.is_ok(), "browse_for_file_to_save should succeed");
}

#[test]
fn test_file_chooser_with_empty_filters() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser with empty filters (should show all files)
    let result = FileChooser::new(
        "Select Any File",
        &PathBuf::from("."),
        ""
    );
    
    assert!(result.is_ok(), "FileChooser with empty filters should succeed");
}

#[test]
fn test_file_chooser_with_wildcard() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser with wildcard filter
    let result = FileChooser::new(
        "Select Any File",
        &PathBuf::from("."),
        "*.*"
    );
    
    assert!(result.is_ok(), "FileChooser with wildcard should succeed");
}

#[test]
fn test_file_chooser_with_multiple_filters() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser with multiple filters
    let result = FileChooser::new(
        "Select Audio File",
        &PathBuf::from("."),
        "*.wav;*.mp3;*.flac;*.ogg"
    );
    
    assert!(result.is_ok(), "FileChooser with multiple filters should succeed");
}

#[test]
fn test_file_chooser_with_unicode_title() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser with Unicode title
    let result = FileChooser::new(
        "选择文件 🎵",
        &PathBuf::from("."),
        "*.txt"
    );
    
    assert!(result.is_ok(), "FileChooser with Unicode title should succeed");
}

#[test]
fn test_file_chooser_with_nonexistent_directory() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser with a non-existent directory
    // JUCE should handle this gracefully (may default to a valid directory)
    let result = FileChooser::new(
        "Select File",
        &PathBuf::from("/nonexistent/directory/path"),
        "*.txt"
    );
    
    // This should still succeed - JUCE will handle the invalid path
    assert!(result.is_ok(), "FileChooser with non-existent directory should not crash");
}

#[test]
fn test_file_chooser_multiple_instances() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create multiple file choosers
    let chooser1 = FileChooser::new("Chooser 1", &PathBuf::from("."), "*.txt");
    let chooser2 = FileChooser::new("Chooser 2", &PathBuf::from("."), "*.md");
    let chooser3 = FileChooser::new("Chooser 3", &PathBuf::from("."), "*.rs");
    
    assert!(chooser1.is_ok(), "First FileChooser should succeed");
    assert!(chooser2.is_ok(), "Second FileChooser should succeed");
    assert!(chooser3.is_ok(), "Third FileChooser should succeed");
}

#[test]
fn test_file_chooser_callback_with_path_handling() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser
    let mut chooser = FileChooser::new(
        "Test Callback",
        &PathBuf::from("."),
        "*.txt"
    ).expect("Failed to create FileChooser");
    
    // Track the path received in the callback
    let received_path = Arc::new(Mutex::new(None));
    let received_path_clone = received_path.clone();
    
    // Browse for a file
    let result = chooser.browse_for_file_to_open(move |path| {
        *received_path_clone.lock().unwrap() = path;
    });
    
    assert!(result.is_ok(), "browse_for_file_to_open should succeed");
}

#[test]
fn test_file_chooser_drop_cleanup() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a file chooser in a scope
    {
        let _chooser = FileChooser::new(
            "Test Drop",
            &PathBuf::from("."),
            "*.txt"
        ).expect("Failed to create FileChooser");
        
        // FileChooser should be dropped here
    }
    
    // If we get here without crashing, the Drop implementation works correctly
}
