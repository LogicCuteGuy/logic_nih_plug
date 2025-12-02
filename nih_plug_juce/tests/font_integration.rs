//! Integration tests for Font FFI bridge.
//!
//! These tests verify that Font operations work correctly through the FFI layer.

use nih_plug_juce::drawing::Font;

#[test]
fn test_font_creation() {
    // Test creating a font with a specific size
    let font = Font::new(14.0);
    assert!(font.is_ok(), "Font creation should succeed");
}

#[test]
fn test_font_with_typeface() {
    // Test creating a font with a typeface
    // Note: This might fail if the typeface is not available on the system
    let font = Font::with_typeface("Arial", 16.0);
    // We don't assert success here because Arial might not be available
    let _ = font;
}

#[test]
fn test_font_style_modifications() {
    let mut font = Font::new(14.0).expect("Font creation should succeed");
    
    // Test setting bold
    assert!(font.set_bold(true).is_ok(), "Setting bold should succeed");
    
    // Test setting italic
    assert!(font.set_italic(true).is_ok(), "Setting italic should succeed");
    
    // Test setting underline
    assert!(font.set_underline(true).is_ok(), "Setting underline should succeed");
}

#[test]
fn test_font_text_measurement() {
    let font = Font::new(14.0).expect("Font creation should succeed");
    
    // Test getting string width
    let width = font.get_string_width("Hello, World!");
    assert!(width.is_ok(), "Getting string width should succeed");
    if let Ok(w) = width {
        assert!(w > 0, "String width should be positive");
    }
    
    // Test getting font height
    let height = font.get_height();
    assert!(height.is_ok(), "Getting font height should succeed");
    if let Ok(h) = height {
        assert!(h > 0, "Font height should be positive");
    }
}

#[test]
fn test_font_empty_string_width() {
    let font = Font::new(14.0).expect("Font creation should succeed");
    
    // Test that empty string has zero width
    let width = font.get_string_width("");
    assert!(width.is_ok(), "Getting empty string width should succeed");
    if let Ok(w) = width {
        assert_eq!(w, 0, "Empty string should have zero width");
    }
}

#[test]
fn test_font_typeface_discovery() {
    // Test finding all available typefaces
    let typefaces = Font::find_all_typeface_names();
    assert!(typefaces.is_ok(), "Finding typefaces should succeed");
    
    if let Ok(names) = typefaces {
        assert!(!names.is_empty(), "Should have at least some fonts available");
        
        // Print first few typefaces for debugging
        println!("Available typefaces (first 10):");
        for (i, name) in names.iter().take(10).enumerate() {
            println!("  {}: {}", i + 1, name);
        }
    }
}

#[test]
fn test_font_different_sizes() {
    // Test creating fonts with different sizes
    let sizes = vec![8.0, 12.0, 14.0, 16.0, 24.0, 32.0];
    
    for size in sizes {
        let font = Font::new(size);
        assert!(font.is_ok(), "Font creation with size {} should succeed", size);
        
        if let Ok(f) = font {
            let height = f.get_height();
            assert!(height.is_ok(), "Getting height for size {} should succeed", size);
        }
    }
}

#[test]
fn test_font_unicode_text() {
    let font = Font::new(14.0).expect("Font creation should succeed");
    
    // Test measuring Unicode text
    let unicode_texts = vec![
        "Hello, World!",
        "Привет, мир!",  // Russian
        "こんにちは世界",  // Japanese
        "你好世界",        // Chinese
        "مرحبا بالعالم",  // Arabic
    ];
    
    for text in unicode_texts {
        let width = font.get_string_width(text);
        assert!(width.is_ok(), "Measuring '{}' should succeed", text);
        if let Ok(w) = width {
            assert!(w > 0, "Text '{}' should have positive width", text);
        }
    }
}

#[test]
fn test_font_style_combinations() {
    let mut font = Font::new(14.0).expect("Font creation should succeed");
    
    // Test combining multiple styles
    assert!(font.set_bold(true).is_ok());
    assert!(font.set_italic(true).is_ok());
    assert!(font.set_underline(true).is_ok());
    
    // Verify we can still measure text with all styles applied
    let width = font.get_string_width("Styled Text");
    assert!(width.is_ok(), "Measuring styled text should succeed");
}
