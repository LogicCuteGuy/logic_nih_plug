//! Integration tests for Colour FFI bridge.
//!
//! These tests verify that the Colour wrapper correctly interfaces with
//! JUCE's Colour class through the FFI boundary.

use nih_plug_juce::drawing::Colour;

#[test]
fn test_colour_creation_and_conversion() {
    // Test RGBA creation
    let red = Colour::from_rgba(255, 0, 0, 255).expect("Failed to create red colour");
    let hex = red.to_hex();
    assert!(hex.starts_with("FF0000"), "Expected red hex to start with FF0000, got {}", hex);
    
    // Test RGB creation (should default to full opacity)
    let green = Colour::from_rgb(0, 255, 0).expect("Failed to create green colour");
    let hex = green.to_hex();
    assert!(hex.starts_with("00FF00"), "Expected green hex to start with 00FF00, got {}", hex);
    
    // Test hex creation
    let blue = Colour::from_hex("#0000FF").expect("Failed to create blue from hex");
    let hex = blue.to_hex();
    assert!(hex.starts_with("0000FF"), "Expected blue hex to start with 0000FF, got {}", hex);
}

#[test]
fn test_colour_hex_formats() {
    // Test various hex formats
    let c1 = Colour::from_hex("#FF0000").expect("Failed with # prefix");
    let c2 = Colour::from_hex("FF0000").expect("Failed without # prefix");
    let c3 = Colour::from_hex("#F00").expect("Failed with short form");
    
    // All should produce red
    assert!(c1.to_hex().starts_with("FF0000"));
    assert!(c2.to_hex().starts_with("FF0000"));
    assert!(c3.to_hex().starts_with("FF0000"));
}

#[test]
fn test_colour_alpha_manipulation() {
    let opaque_red = Colour::from_rgb(255, 0, 0).expect("Failed to create red");
    
    // Create semi-transparent version
    let semi_transparent = opaque_red.with_alpha(0.5).expect("Failed to set alpha");
    let hex = semi_transparent.to_hex();
    
    // Should have red color but different alpha
    assert!(hex.starts_with("FF0000"), "Color should still be red");
    
    // Create fully transparent version
    let transparent = opaque_red.with_alpha(0.0).expect("Failed to set alpha to 0");
    let hex = transparent.to_hex();
    assert!(hex.ends_with("00"), "Alpha should be 00 for fully transparent");
}

#[test]
fn test_colour_brightness_manipulation() {
    let dark_red = Colour::from_rgb(128, 0, 0).expect("Failed to create dark red");
    
    // Make it brighter
    let brighter = dark_red.brighter(0.5).expect("Failed to brighten");
    // Brighter color should exist (we can't easily test the exact value due to JUCE's algorithm)
    let _ = brighter.to_hex();
    
    // Make it darker
    let darker = dark_red.darker(0.5).expect("Failed to darken");
    let _ = darker.to_hex();
}

#[test]
fn test_colour_interpolation() {
    let red = Colour::from_rgb(255, 0, 0).expect("Failed to create red");
    let blue = Colour::from_rgb(0, 0, 255).expect("Failed to create blue");
    
    // Interpolate at 0.0 should give red
    let at_zero = red.interpolated_with(&blue, 0.0).expect("Failed to interpolate at 0.0");
    let hex = at_zero.to_hex();
    assert!(hex.starts_with("FF0000"), "At 0.0 should be red, got {}", hex);
    
    // Interpolate at 1.0 should give blue
    let at_one = red.interpolated_with(&blue, 1.0).expect("Failed to interpolate at 1.0");
    let hex = at_one.to_hex();
    assert!(hex.starts_with("0000FF"), "At 1.0 should be blue, got {}", hex);
    
    // Interpolate at 0.5 should give purple-ish
    let at_half = red.interpolated_with(&blue, 0.5).expect("Failed to interpolate at 0.5");
    let _ = at_half.to_hex(); // Just verify it doesn't crash
}

#[test]
fn test_colour_invalid_hex() {
    // Test invalid hex strings
    let result = Colour::from_hex("invalid");
    assert!(result.is_err(), "Should fail with invalid hex string");
    
    let result = Colour::from_hex("#GG0000");
    assert!(result.is_err(), "Should fail with invalid hex characters");
    
    let result = Colour::from_hex("");
    assert!(result.is_err(), "Should fail with empty string");
}

#[test]
fn test_colour_thread_safety() {
    // Colour should be Send + Sync, so we can use it across threads
    let colour = Colour::from_rgb(255, 0, 0).expect("Failed to create colour");
    
    // This should compile because Colour implements Send
    std::thread::spawn(move || {
        let _ = colour.to_hex();
    }).join().expect("Thread panicked");
}
