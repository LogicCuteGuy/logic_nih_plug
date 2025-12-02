//! Integration tests for DocumentWindow component.
//!
//! These tests verify that the DocumentWindow FFI bindings work correctly
//! and that the component can be created, configured, and used.

use nih_plug_juce::containers::DocumentWindow;
use nih_plug_juce::Component;

#[test]
fn test_document_window_creation() {
    // Test that we can create a DocumentWindow
    let result = DocumentWindow::new("Test Window");
    assert!(result.is_ok(), "DocumentWindow creation should succeed");
}

#[test]
fn test_document_window_set_name() {
    // Test that we can set the window name
    let mut window = DocumentWindow::new("Initial Title").unwrap();
    window.set_name("Updated Title");
    // If we get here without crashing, the test passed
}

#[test]
fn test_document_window_set_content() {
    // Test that we can set content for the window
    let mut window = DocumentWindow::new("Test Window").unwrap();
    let content = Component::new().unwrap();
    
    let result = window.set_content_owned(content);
    assert!(result.is_ok(), "Setting window content should succeed");
}

#[test]
fn test_document_window_visibility() {
    // Test that we can set window visibility (inherited from Component)
    let mut window = DocumentWindow::new("Test Window").unwrap();
    window.set_visible(true);
    window.set_visible(false);
    // If we get here without crashing, the test passed
}

#[test]
fn test_document_window_bounds() {
    // Test that we can set window bounds (inherited from Component)
    let mut window = DocumentWindow::new("Test Window").unwrap();
    window.set_bounds(100, 100, 400, 300);
    // If we get here without crashing, the test passed
}

#[test]
fn test_document_window_close_callback() {
    // Test that we can set a close callback
    let mut window = DocumentWindow::new("Test Window").unwrap();
    
    let result = window.set_on_close(|| {
        // Return true to allow closing
        true
    });
    
    assert!(result.is_ok(), "Setting close callback should succeed");
}

#[test]
fn test_document_window_close_callback_prevent_close() {
    // Test that we can set a close callback that prevents closing
    let mut window = DocumentWindow::new("Test Window").unwrap();
    
    let result = window.set_on_close(|| {
        // Return false to prevent closing
        false
    });
    
    assert!(result.is_ok(), "Setting close callback should succeed");
}

#[test]
fn test_document_window_with_content_and_callback() {
    // Test a complete setup with content and callback
    let mut window = DocumentWindow::new("Complete Test").unwrap();
    
    // Set content
    let mut content = Component::new().unwrap();
    content.set_bounds(0, 0, 400, 300);
    window.set_content_owned(content).unwrap();
    
    // Set close callback
    window.set_on_close(|| {
        println!("Window closing");
        true
    }).unwrap();
    
    // Set window properties
    window.set_bounds(100, 100, 400, 300);
    window.set_name("Updated Complete Test");
    
    // If we get here without crashing, the test passed
}
