//! Integration tests for LookAndFeel system.

use nih_plug_juce::{Component, Colour, LookAndFeel};

#[test]
fn test_lookandfeel_creation() {
    // Test that we can create a LookAndFeel_V4
    let laf = LookAndFeel::new_v4();
    assert!(laf.is_ok(), "LookAndFeel creation should succeed");
}

#[test]
fn test_lookandfeel_set_colour() {
    let mut laf = LookAndFeel::new_v4().expect("Failed to create LookAndFeel");
    
    // Set a custom color for a button
    let custom_color = Colour::from_rgb(123, 45, 67).expect("Failed to create colour");
    let colour_id = 0x1000100; // TextButton::buttonColourId
    
    laf.set_colour(colour_id, custom_color);
    
    // If we get here without crashing, the test passed
}

#[test]
fn test_lookandfeel_find_colour() {
    let laf = LookAndFeel::new_v4().expect("Failed to create LookAndFeel");
    
    // Find a default color
    let colour_id = 0x1000100; // TextButton::buttonColourId
    let found_color = laf.find_colour(colour_id);
    
    // If we get here without crashing, the test passed
    drop(found_color);
}

#[test]
fn test_component_set_look_and_feel() {
    let laf = LookAndFeel::new_v4().expect("Failed to create LookAndFeel");
    let mut component = Component::new().expect("Failed to create component");
    
    // Set the LookAndFeel on the component
    let result = component.set_look_and_feel(&laf);
    assert!(result.is_ok(), "Setting LookAndFeel should succeed");
}

#[test]
fn test_lookandfeel_with_custom_colors() {
    let mut laf = LookAndFeel::new_v4().expect("Failed to create LookAndFeel");
    
    // Set multiple custom colors
    let red = Colour::from_rgb(255, 0, 0).expect("Failed to create red");
    let green = Colour::from_rgb(0, 255, 0).expect("Failed to create green");
    let blue = Colour::from_rgb(0, 0, 255).expect("Failed to create blue");
    
    laf.set_colour(0x1000100, red);   // Button background
    laf.set_colour(0x1000101, green); // Button text
    laf.set_colour(0x1000102, blue);  // Button outline
    
    // Verify we can find the colors we just set
    let _ = laf.find_colour(0x1000100);
    let _ = laf.find_colour(0x1000101);
    let _ = laf.find_colour(0x1000102);
}

#[test]
fn test_multiple_components_same_lookandfeel() {
    let laf = LookAndFeel::new_v4().expect("Failed to create LookAndFeel");
    
    // Create multiple components
    let mut comp1 = Component::new().expect("Failed to create component 1");
    let mut comp2 = Component::new().expect("Failed to create component 2");
    let mut comp3 = Component::new().expect("Failed to create component 3");
    
    // Set the same LookAndFeel on all components
    assert!(comp1.set_look_and_feel(&laf).is_ok());
    assert!(comp2.set_look_and_feel(&laf).is_ok());
    assert!(comp3.set_look_and_feel(&laf).is_ok());
}
