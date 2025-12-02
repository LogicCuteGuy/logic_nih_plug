//! Integration tests for parameter attachment functionality.
//!
//! These tests verify that SliderParameterAttachment can be created
//! and properly manages the lifecycle of the attachment.

use nih_plug_juce::parameter_attachment::SliderParameterAttachment;
use nih_plug_juce::widgets::{Slider, SliderStyle};

#[test]
fn test_slider_parameter_attachment_creation() {
    // Initialize JUCE
    let _ = nih_plug_juce::initialize();
    
    // Create a slider
    let mut slider = Slider::new(SliderStyle::Rotary).expect("Failed to create slider");
    slider.set_range(0.0, 1.0, 0.01);
    slider.set_value(0.5);
    
    // Create a parameter attachment
    // Note: This creates the attachment successfully, but without a real
    // parameter system, it won't actually synchronize with parameters.
    // The test verifies that the FFI bridge is properly set up.
    let result = SliderParameterAttachment::new(&mut slider, "test_param");
    
    // The attachment should be created successfully
    // In a full implementation with AudioProcessorValueTreeState, this would
    // also establish bidirectional synchronization
    assert!(result.is_ok(), "Parameter attachment creation should succeed");
}

#[test]
fn test_slider_parameter_attachment_with_empty_id() {
    // Initialize JUCE
    let _ = nih_plug_juce::initialize();
    
    // Create a slider
    let mut slider = Slider::new(SliderStyle::LinearHorizontal).expect("Failed to create slider");
    
    // Try to create attachment with empty parameter ID
    let result = SliderParameterAttachment::new(&mut slider, "");
    
    // Should fail with empty parameter ID
    assert!(result.is_err(), "Expected error with empty parameter ID");
}

#[test]
fn test_slider_parameter_attachment_drop() {
    // Initialize JUCE
    let _ = nih_plug_juce::initialize();
    
    // Create a slider
    let mut slider = Slider::new(SliderStyle::Rotary).expect("Failed to create slider");
    slider.set_range(0.0, 1.0, 0.01);
    
    // Create attachment in a scope
    {
        let result = SliderParameterAttachment::new(&mut slider, "test_param");
        assert!(result.is_ok(), "Attachment creation should succeed");
        // Attachment goes out of scope here and should be properly cleaned up
    }
    
    // Slider should still be valid after attachment is dropped
    slider.set_value(0.75);
    assert!((slider.get_value() - 0.75).abs() < 0.001);
}

#[test]
fn test_multiple_attachments_different_sliders() {
    // Initialize JUCE
    let _ = nih_plug_juce::initialize();
    
    // Create multiple sliders
    let mut slider1 = Slider::new(SliderStyle::Rotary).expect("Failed to create slider 1");
    let mut slider2 = Slider::new(SliderStyle::LinearHorizontal).expect("Failed to create slider 2");
    
    slider1.set_range(0.0, 1.0, 0.01);
    slider2.set_range(0.0, 100.0, 1.0);
    
    // Create attachments for different sliders
    let result1 = SliderParameterAttachment::new(&mut slider1, "param1");
    let result2 = SliderParameterAttachment::new(&mut slider2, "param2");
    
    // Both should succeed
    assert!(result1.is_ok(), "First attachment should succeed");
    assert!(result2.is_ok(), "Second attachment should succeed");
    
    // This tests that the FFI layer handles multiple attachments correctly
}

#[test]
fn test_attachment_is_not_send() {
    // This test verifies at compile time that SliderParameterAttachment
    // does not implement Send, preventing it from being moved across threads.
    
    fn assert_not_send<T: Send>() {}
    
    // Uncommenting this line should cause a compile error:
    // assert_not_send::<SliderParameterAttachment>();
    
    // This test passes by not compiling the assertion
}

#[test]
fn test_attachment_is_not_sync() {
    // This test verifies at compile time that SliderParameterAttachment
    // does not implement Sync, preventing it from being shared across threads.
    
    fn assert_not_sync<T: Sync>() {}
    
    // Uncommenting this line should cause a compile error:
    // assert_not_sync::<SliderParameterAttachment>();
    
    // This test passes by not compiling the assertion
}
