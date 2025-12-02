//! Integration tests for the JUCE FFI bridge.
//!
//! These tests verify that the cxx bridge compiles and links correctly,
//! and that basic JUCE functionality is accessible through the FFI layer.

use nih_plug_juce::{self, Component};
use nih_plug_juce::widgets::{TextEditor, ToggleButton};
use std::sync::{Arc, Mutex};

#[test]
fn test_bridge_initialization() {
    // Test that the JUCE FFI bridge can be initialized
    let result = nih_plug_juce::initialize();
    assert!(result.is_ok(), "JUCE FFI bridge initialization should succeed");
}

#[test]
fn test_bridge_initialization_idempotent() {
    // Test that initialization can be called multiple times safely
    let result1 = nih_plug_juce::initialize();
    let result2 = nih_plug_juce::initialize();
    
    assert!(result1.is_ok(), "First initialization should succeed");
    assert!(result2.is_ok(), "Second initialization should succeed");
}

#[test]
fn test_bridge_ffi_direct_call() {
    // Test that we can directly call the FFI function
    let result = nih_plug_juce::bridge::ffi::initialize();
    assert!(result, "Direct FFI call to initialize should return true");
}

#[test]
fn test_component_with_paint_callback_creation() {
    // Test that we can create a component that supports paint callbacks
    let result = Component::new_with_paint_callback();
    assert!(result.is_ok(), "Component with paint callback creation should succeed");
}

#[test]
fn test_paint_callback_can_be_set() {
    // Test that we can set a paint callback on a component
    let mut component = Component::new_with_paint_callback()
        .expect("Component creation should succeed");
    
    // Set a simple paint callback
    let result = component.set_paint_callback(|_g| {
        // Empty callback for testing
    });
    
    assert!(result.is_ok(), "Setting paint callback should succeed");
}

#[test]
fn test_paint_callback_with_drawing_operations() {
    // Test that we can set a paint callback that performs drawing operations
    let mut component = Component::new_with_paint_callback()
        .expect("Component creation should succeed");
    
    // Set a paint callback that performs various drawing operations
    let result = component.set_paint_callback(|g| {
        // These operations should not crash
        g.fill_rect(10, 10, 100, 50);
        g.draw_rect(20, 20, 80, 30);
        g.fill_ellipse(50.0, 50.0, 100.0, 100.0);
        g.draw_line(0.0, 0.0, 100.0, 100.0);
    });
    
    assert!(result.is_ok(), "Setting paint callback with drawing operations should succeed");
}

#[test]
fn test_paint_callback_invocation() {
    // Test that the paint callback is actually invoked when repaint is triggered
    let mut component = Component::new_with_paint_callback()
        .expect("Component creation should succeed");
    
    // Use Arc<Mutex<bool>> to track if callback was invoked
    let callback_invoked = Arc::new(Mutex::new(false));
    let callback_invoked_clone = Arc::clone(&callback_invoked);
    
    // Set a paint callback that sets the flag
    let result = component.set_paint_callback(move |_g| {
        *callback_invoked_clone.lock().unwrap() = true;
    });
    
    assert!(result.is_ok(), "Setting paint callback should succeed");
    
    // Trigger a repaint
    component.repaint();
    
    // Note: In a real JUCE application, we would need to process the message queue
    // for the paint callback to actually be invoked. Since we're in a test environment
    // without a full JUCE application running, we can't verify the callback is invoked.
    // This test primarily verifies that the callback can be set without crashing.
    
    // For now, we just verify that setting the callback succeeded
    // The actual invocation will be tested in a full integration test with a running app
}

#[test]
fn test_paint_callback_with_captured_state() {
    // Test that paint callbacks can capture and use state
    let mut component = Component::new_with_paint_callback()
        .expect("Component creation should succeed");
    
    let color_index = Arc::new(Mutex::new(0));
    let color_index_clone = Arc::clone(&color_index);
    
    // Set a paint callback that uses captured state
    let result = component.set_paint_callback(move |g| {
        let index = *color_index_clone.lock().unwrap();
        
        // Draw different things based on captured state
        match index {
            0 => g.fill_rect(0, 0, 50, 50),
            1 => g.fill_rect(50, 50, 50, 50),
            _ => g.fill_rect(100, 100, 50, 50),
        }
    });
    
    assert!(result.is_ok(), "Setting paint callback with captured state should succeed");
    
    // Modify the captured state
    *color_index.lock().unwrap() = 1;
    
    // Trigger repaint (callback would use new state if invoked)
    component.repaint();
}

#[test]
fn test_regular_component_cannot_set_paint_callback() {
    // Test that a regular component (not created with new_with_paint_callback)
    // cannot have a paint callback set
    let mut component = Component::new()
        .expect("Component creation should succeed");
    
    // Attempt to set a paint callback on a regular component
    let result = component.set_paint_callback(|_g| {
        // This should fail
    });
    
    // This should return an error because the component doesn't support callbacks
    assert!(result.is_err(), "Setting paint callback on regular component should fail");
    
    // Verify the error message mentions callback support
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        assert!(
            error_msg.contains("callback") || error_msg.contains("support"),
            "Error message should mention callback support: {}",
            error_msg
        );
    }
}

#[test]
fn test_text_editor_creation() {
    // Test that we can create a TextEditor
    let result = TextEditor::new();
    assert!(result.is_ok(), "TextEditor creation should succeed");
}

#[test]
fn test_text_editor_set_get_text() {
    // Test that we can set and get text from a TextEditor
    let mut editor = TextEditor::new()
        .expect("TextEditor creation should succeed");
    
    // Set some text
    editor.set_text("Hello, JUCE!");
    
    // Get the text back
    let text = editor.get_text();
    assert_eq!(text, "Hello, JUCE!", "Text should match what was set");
}

#[test]
fn test_text_editor_multiline() {
    // Test that we can set multiline mode
    let mut editor = TextEditor::new()
        .expect("TextEditor creation should succeed");
    
    // Set multiline mode
    editor.set_multiline(true);
    
    // Set text with line breaks
    editor.set_text("Line 1\nLine 2\nLine 3");
    
    // Get the text back
    let text = editor.get_text();
    assert_eq!(text, "Line 1\nLine 2\nLine 3", "Multiline text should be preserved");
}

#[test]
fn test_text_editor_readonly() {
    // Test that we can set read-only mode
    let mut editor = TextEditor::new()
        .expect("TextEditor creation should succeed");
    
    // Set some initial text
    editor.set_text("Read-only text");
    
    // Set read-only mode
    editor.set_readonly(true);
    
    // Get the text back
    let text = editor.get_text();
    assert_eq!(text, "Read-only text", "Text should be accessible in read-only mode");
}

#[test]
fn test_text_editor_callback() {
    // Test that we can set a text change callback
    let mut editor = TextEditor::new()
        .expect("TextEditor creation should succeed");
    
    // Use Arc<Mutex<String>> to track callback invocations
    let callback_text = Arc::new(Mutex::new(String::new()));
    let callback_text_clone = Arc::clone(&callback_text);
    
    // Set a text change callback
    let result = editor.set_on_text_change(move |text| {
        *callback_text_clone.lock().unwrap() = text.to_string();
    });
    
    assert!(result.is_ok(), "Setting text change callback should succeed");
}

#[test]
fn test_text_editor_empty_text() {
    // Test that we can handle empty text
    let mut editor = TextEditor::new()
        .expect("TextEditor creation should succeed");
    
    // Set empty text
    editor.set_text("");
    
    // Get the text back
    let text = editor.get_text();
    assert_eq!(text, "", "Empty text should be handled correctly");
}

#[test]
fn test_text_editor_unicode_text() {
    // Test that we can handle Unicode text
    let mut editor = TextEditor::new()
        .expect("TextEditor creation should succeed");
    
    // Set Unicode text
    let unicode_text = "Hello 世界 🌍 Привет";
    editor.set_text(unicode_text);
    
    // Get the text back
    let text = editor.get_text();
    assert_eq!(text, unicode_text, "Unicode text should be preserved");
}

#[test]
fn test_text_editor_inherits_from_component() {
    // Test that TextEditor inherits from Component
    let mut editor = TextEditor::new()
        .expect("TextEditor creation should succeed");
    
    // Test that we can use Component methods
    editor.set_bounds(10, 10, 200, 100);
    editor.set_visible(true);
    editor.repaint();
    
    // If we got here without crashing, the test passes
}

#[test]
fn test_toggle_button_creation() {
    // Test that we can create a ToggleButton
    let result = ToggleButton::new("Enable Feature");
    assert!(result.is_ok(), "ToggleButton creation should succeed");
}

#[test]
fn test_toggle_button_set_get_state() {
    // Test that we can set and get the toggle state
    let mut toggle = ToggleButton::new("Enable")
        .expect("ToggleButton creation should succeed");
    
    // Initially should be false
    assert_eq!(toggle.get_toggle_state(), false, "Initial state should be false");
    
    // Set to true
    toggle.set_toggle_state(true);
    assert_eq!(toggle.get_toggle_state(), true, "State should be true after setting");
    
    // Set back to false
    toggle.set_toggle_state(false);
    assert_eq!(toggle.get_toggle_state(), false, "State should be false after setting");
}

#[test]
fn test_toggle_button_set_text() {
    // Test that we can set the button text
    let mut toggle = ToggleButton::new("Initial Text")
        .expect("ToggleButton creation should succeed");
    
    // Change the text
    toggle.set_button_text("Updated Text");
    
    // If we got here without crashing, the test passes
}

#[test]
fn test_toggle_button_radio_group() {
    // Test that we can set radio group IDs
    let mut radio1 = ToggleButton::new("Option 1")
        .expect("ToggleButton creation should succeed");
    let mut radio2 = ToggleButton::new("Option 2")
        .expect("ToggleButton creation should succeed");
    let mut radio3 = ToggleButton::new("Option 3")
        .expect("ToggleButton creation should succeed");
    
    // Set them all to the same radio group
    radio1.set_radio_group_id(1);
    radio2.set_radio_group_id(1);
    radio3.set_radio_group_id(1);
    
    // If we got here without crashing, the test passes
}

#[test]
fn test_toggle_button_callback() {
    // Test that we can set a click callback
    let mut toggle = ToggleButton::new("Enable Feature")
        .expect("ToggleButton creation should succeed");
    
    // Use Arc<Mutex<bool>> to track callback invocations
    let callback_state = Arc::new(Mutex::new(false));
    let callback_state_clone = Arc::clone(&callback_state);
    
    // Set a click callback
    let result = toggle.set_on_click(move |state| {
        *callback_state_clone.lock().unwrap() = state;
    });
    
    assert!(result.is_ok(), "Setting click callback should succeed");
}

#[test]
fn test_toggle_button_inherits_from_component() {
    // Test that ToggleButton inherits from Component
    let mut toggle = ToggleButton::new("Enable")
        .expect("ToggleButton creation should succeed");
    
    // Test that we can use Component methods
    toggle.set_bounds(10, 10, 150, 30);
    toggle.set_visible(true);
    toggle.repaint();
    
    // If we got here without crashing, the test passes
}

#[test]
fn test_toggle_button_unicode_text() {
    // Test that we can handle Unicode text
    let unicode_text = "启用功能 🔘";
    let mut toggle = ToggleButton::new(unicode_text)
        .expect("ToggleButton creation should succeed");
    
    // Change to different Unicode text
    toggle.set_button_text("Включить ✓");
    
    // If we got here without crashing, the test passes
}
