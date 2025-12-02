//! Integration tests for AlertWindow functionality.
//!
//! These tests verify that the AlertWindow FFI bridge works correctly
//! for showing message boxes and confirmation dialogs.

use nih_plug_juce::dialogs::AlertWindow;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_alert_window_show_message_box() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Note: This test can't actually verify the dialog was shown since it's
    // a GUI operation, but we can verify it doesn't crash
    // In a real GUI environment, this would block until the user clicks OK
    
    // For now, we just verify the function can be called without panicking
    // In a headless environment, JUCE may not actually show the dialog
    AlertWindow::show_message_box("Test Title", "Test Message");
    
    // If we get here without panicking, the test passes
}

#[test]
fn test_alert_window_show_message_box_async() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a flag to track if the callback was invoked
    let callback_invoked = Arc::new(Mutex::new(false));
    let callback_invoked_clone = callback_invoked.clone();
    
    // Show an async message box
    let result = AlertWindow::show_message_box_async(
        "Async Test",
        "This is an async message",
        move || {
            *callback_invoked_clone.lock().unwrap() = true;
        }
    );
    
    // Verify the operation succeeded
    assert!(result.is_ok(), "show_message_box_async should succeed");
    
    // Note: In a headless environment, the callback may not actually be invoked
    // since there's no GUI event loop running. This test mainly verifies that
    // the FFI bridge doesn't crash.
}

#[test]
fn test_alert_window_show_ok_cancel_box() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Create a flag to track the user's choice
    let user_choice = Arc::new(Mutex::new(None));
    let user_choice_clone = user_choice.clone();
    
    // Show an OK/Cancel dialog
    let result = AlertWindow::show_ok_cancel_box(
        "Confirmation",
        "Do you want to proceed?",
        move |confirmed| {
            *user_choice_clone.lock().unwrap() = Some(confirmed);
        }
    );
    
    // Verify the operation succeeded
    assert!(result.is_ok(), "show_ok_cancel_box should succeed");
    
    // Note: In a headless environment, the callback may not actually be invoked
    // since there's no GUI event loop running. This test mainly verifies that
    // the FFI bridge doesn't crash.
}

#[test]
fn test_alert_window_with_empty_strings() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Test with empty strings - should not crash
    AlertWindow::show_message_box("", "");
    
    let result = AlertWindow::show_message_box_async("", "", || {});
    assert!(result.is_ok(), "Empty strings should be handled gracefully");
    
    let result = AlertWindow::show_ok_cancel_box("", "", |_| {});
    assert!(result.is_ok(), "Empty strings should be handled gracefully");
}

#[test]
fn test_alert_window_with_unicode() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Test with Unicode characters
    AlertWindow::show_message_box(
        "Unicode Test 🎵",
        "This message contains Unicode: 你好世界 🌍"
    );
    
    let result = AlertWindow::show_message_box_async(
        "Emoji Test 😀",
        "Testing emojis: 🎸 🎹 🎤",
        || {}
    );
    assert!(result.is_ok(), "Unicode should be handled correctly");
}

#[test]
fn test_alert_window_with_long_text() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Test with long text
    let long_message = "This is a very long message. ".repeat(100);
    
    AlertWindow::show_message_box("Long Message Test", &long_message);
    
    let result = AlertWindow::show_message_box_async(
        "Long Message Test",
        &long_message,
        || {}
    );
    assert!(result.is_ok(), "Long messages should be handled correctly");
}

#[test]
fn test_alert_window_multiple_callbacks() {
    // Initialize JUCE
    nih_plug_juce::initialize().expect("Failed to initialize JUCE");
    
    // Test showing multiple dialogs in sequence
    let result1 = AlertWindow::show_message_box_async("Test 1", "Message 1", || {});
    let result2 = AlertWindow::show_message_box_async("Test 2", "Message 2", || {});
    let result3 = AlertWindow::show_ok_cancel_box("Test 3", "Message 3", |_| {});
    
    assert!(result1.is_ok(), "First dialog should succeed");
    assert!(result2.is_ok(), "Second dialog should succeed");
    assert!(result3.is_ok(), "Third dialog should succeed");
}

