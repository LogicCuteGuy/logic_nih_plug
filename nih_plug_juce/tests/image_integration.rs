//! Integration tests for JUCE Image FFI bindings.
//!
//! These tests verify that the Image wrapper correctly interfaces with
//! JUCE's C++ Image class through the FFI boundary.

use nih_plug_juce::drawing::{Image, ImageFormat};
use std::path::Path;

#[test]
fn test_image_creation() {
    // Test creating images with different formats
    let rgb_image = Image::new(ImageFormat::RGB, 100, 100);
    assert!(rgb_image.is_ok());

    let argb_image = Image::new(ImageFormat::ARGB, 200, 150);
    assert!(argb_image.is_ok());

    let single_channel = Image::new(ImageFormat::SingleChannel, 50, 50);
    assert!(single_channel.is_ok());
}

#[test]
fn test_image_dimensions() {
    let image = Image::new(ImageFormat::ARGB, 320, 240).unwrap();
    
    assert_eq!(image.get_width().unwrap(), 320);
    assert_eq!(image.get_height().unwrap(), 240);
}

#[test]
fn test_image_invalid_dimensions() {
    // Negative width
    let result = Image::new(ImageFormat::ARGB, -100, 100);
    assert!(result.is_err());

    // Zero height
    let result = Image::new(ImageFormat::ARGB, 100, 0);
    assert!(result.is_err());

    // Both negative
    let result = Image::new(ImageFormat::ARGB, -50, -50);
    assert!(result.is_err());
}

#[test]
fn test_image_blur() {
    let mut image = Image::new(ImageFormat::ARGB, 100, 100).unwrap();
    
    // Apply blur with valid radius
    let result = image.apply_blur(3.0);
    assert!(result.is_ok());

    // Apply blur with zero radius (should succeed)
    let result = image.apply_blur(0.0);
    assert!(result.is_ok());
}

#[test]
fn test_image_blur_invalid_radius() {
    let mut image = Image::new(ImageFormat::ARGB, 100, 100).unwrap();
    
    // Negative radius should fail
    let result = image.apply_blur(-1.0);
    assert!(result.is_err());
}

#[test]
fn test_image_graphics_context() {
    let mut image = Image::new(ImageFormat::ARGB, 200, 200).unwrap();
    
    // Get graphics context for drawing
    let result = image.get_graphics_context();
    assert!(result.is_ok());
    
    // We can get the graphics context and use it for drawing
    let mut g = result.unwrap();
    g.fill_rect(10, 10, 50, 50);
}

#[test]
fn test_image_load_nonexistent_file() {
    // Try to load a file that doesn't exist
    let result = Image::load_from_file(Path::new("/nonexistent/path/image.png"));
    assert!(result.is_err());
}

#[test]
fn test_image_save_invalid_path() {
    let image = Image::new(ImageFormat::ARGB, 100, 100).unwrap();
    
    // Try to save to an invalid path
    let result = image.save_to_file(Path::new("/invalid/path/that/does/not/exist/image.png"));
    assert!(result.is_err());
}

#[test]
fn test_image_multiple_formats() {
    // Test that we can create images with all supported formats
    let formats = vec![
        ImageFormat::RGB,
        ImageFormat::ARGB,
        ImageFormat::SingleChannel,
    ];

    for format in formats {
        let image = Image::new(format, 64, 64);
        assert!(image.is_ok(), "Failed to create image with format {:?}", format);
        
        let image = image.unwrap();
        assert_eq!(image.get_width().unwrap(), 64);
        assert_eq!(image.get_height().unwrap(), 64);
    }
}

#[test]
fn test_image_large_dimensions() {
    // Test creating a large image
    let image = Image::new(ImageFormat::ARGB, 2048, 2048);
    assert!(image.is_ok());
    
    let image = image.unwrap();
    assert_eq!(image.get_width().unwrap(), 2048);
    assert_eq!(image.get_height().unwrap(), 2048);
}

#[test]
fn test_image_small_dimensions() {
    // Test creating a very small image
    let image = Image::new(ImageFormat::ARGB, 1, 1);
    assert!(image.is_ok());
    
    let image = image.unwrap();
    assert_eq!(image.get_width().unwrap(), 1);
    assert_eq!(image.get_height().unwrap(), 1);
}
