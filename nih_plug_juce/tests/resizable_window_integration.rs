//! Integration tests for ResizableWindow FFI bindings.
//!
//! These tests verify that the ResizableWindow wrapper correctly interfaces
//! with the JUCE C++ library through the FFI bridge.

use nih_plug_juce::containers::ResizableWindow;
use std::sync::{Arc, Mutex};

#[test]
fn test_resizable_window_creation() {
    // Test that we can create a ResizableWindow
    let window = ResizableWindow::new("Test Window");
    assert!(window.is_ok(), "Failed to create ResizableWindow");
}

#[test]
fn test_resizable_window_set_resizable() {
    // Test that we can enable/disable resizing
    let mut window = ResizableWindow::new("Test Window").unwrap();
    
    // Should not panic
    window.set_resizable(true);
    window.set_resizable(false);
}

#[test]
fn test_resizable_window_set_resize_limits() {
    // Test that we can set resize limits
    let mut window = ResizableWindow::new("Test Window").unwrap();
    
    // Should not panic
    window.set_resize_limits(400, 300, 1920, 1080);
    window.set_resize_limits(200, 150, 3840, 2160);
}

#[test]
fn test_resizable_window_set_on_resized_callback() {
    // Test that we can set a resize callback
    let mut window = ResizableWindow::new("Test Window").unwrap();
    
    let callback_invoked = Arc::new(Mutex::new(false));
    let callback_invoked_clone = callback_invoked.clone();
    
    let result = window.set_on_resized(move |width, height| {
        *callback_invoked_clone.lock().unwrap() = true;
        println!("Window resized to {}x{}", width, height);
    });
    
    assert!(result.is_ok(), "Failed to set resize callback");
}

#[test]
fn test_resizable_window_inherits_component() {
    // Test that ResizableWindow can be used as a Component through Deref
    let mut window = ResizableWindow::new("Test Window").unwrap();
    
    // These methods come from Component through Deref
    window.set_bounds(100, 100, 800, 600);
    window.set_visible(false);
    window.repaint();
}

#[test]
fn test_resizable_window_with_constraints() {
    // Test a complete workflow with resize constraints
    let mut window = ResizableWindow::new("Constrained Window").unwrap();
    
    // Set up the window
    window.set_resizable(true);
    window.set_resize_limits(400, 300, 1920, 1080);
    window.set_bounds(0, 0, 800, 600);
    
    // Set a callback
    let result = window.set_on_resized(|w, h| {
        println!("Resized to {}x{}", w, h);
    });
    
    assert!(result.is_ok());
}

#[test]
fn test_multiple_resizable_windows() {
    // Test that we can create multiple windows
    let window1 = ResizableWindow::new("Window 1");
    let window2 = ResizableWindow::new("Window 2");
    let window3 = ResizableWindow::new("Window 3");
    
    assert!(window1.is_ok());
    assert!(window2.is_ok());
    assert!(window3.is_ok());
}

#[test]
fn test_resizable_window_empty_title() {
    // Test that we can create a window with an empty title
    let window = ResizableWindow::new("");
    assert!(window.is_ok(), "Failed to create ResizableWindow with empty title");
}

#[test]
fn test_resizable_window_unicode_title() {
    // Test that we can create a window with Unicode characters in the title
    let window = ResizableWindow::new("测试窗口 🎵");
    assert!(window.is_ok(), "Failed to create ResizableWindow with Unicode title");
}

#[test]
fn test_resizable_window_extreme_limits() {
    // Test with extreme but valid resize limits
    let mut window = ResizableWindow::new("Test Window").unwrap();
    
    // Very small minimum
    window.set_resize_limits(1, 1, 100, 100);
    
    // Very large maximum
    window.set_resize_limits(100, 100, 10000, 10000);
}
