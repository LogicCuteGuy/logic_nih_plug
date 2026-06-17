use logic_nih_plug_graphics::{Color, Graphics, GraphicsError};

#[test]
fn test_graphics_creation() {
    let graphics = Graphics::new(800, 600).unwrap();
    assert_eq!(graphics.width(), 800);
    assert_eq!(graphics.height(), 600);
    assert_eq!(graphics.as_bytes().len(), 800 * 600 * 4);
}

#[test]
fn test_graphics_invalid_dimensions() {
    assert!(matches!(
        Graphics::new(0, 600),
        Err(GraphicsError::InvalidDimensions(0, 600))
    ));
    assert!(matches!(
        Graphics::new(800, 0),
        Err(GraphicsError::InvalidDimensions(800, 0))
    ));
}

#[test]
fn test_set_color() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    let red = Color::rgb(255, 0, 0);
    graphics.set_color(red);
    assert_eq!(graphics.current_color(), red);
}

#[test]
fn test_clear() {
    let mut graphics = Graphics::new(10, 10).unwrap();
    graphics.set_color(Color::rgb(255, 0, 0));
    graphics.clear();

    // Check that all pixels are red
    for chunk in graphics.as_bytes().chunks_exact(4) {
        assert_eq!(chunk[0], 255); // R
        assert_eq!(chunk[1], 0);   // G
        assert_eq!(chunk[2], 0);   // B
        assert_eq!(chunk[3], 255); // A
    }
}

#[test]
fn test_set_pixel() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 0, 0));
    graphics.set_pixel(50, 50);

    let pixel = graphics.get_pixel(50, 50).unwrap();
    assert_eq!(pixel, Color::rgb(255, 0, 0));
}

#[test]
fn test_set_pixel_out_of_bounds() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 0, 0));
    
    // Should not panic
    graphics.set_pixel(-1, 50);
    graphics.set_pixel(50, -1);
    graphics.set_pixel(100, 50);
    graphics.set_pixel(50, 100);
}

#[test]
fn test_get_pixel_out_of_bounds() {
    let graphics = Graphics::new(100, 100).unwrap();
    
    assert!(graphics.get_pixel(-1, 50).is_none());
    assert!(graphics.get_pixel(50, -1).is_none());
    assert!(graphics.get_pixel(100, 50).is_none());
    assert!(graphics.get_pixel(50, 100).is_none());
}

#[test]
fn test_fill_rect() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 0, 0));
    graphics.fill_rect(10, 10, 20, 20);

    // Check pixels inside the rectangle
    for y in 10..30 {
        for x in 10..30 {
            let pixel = graphics.get_pixel(x, y).unwrap();
            assert_eq!(pixel, Color::rgb(255, 0, 0));
        }
    }

    // Check pixels outside the rectangle are still black
    let pixel = graphics.get_pixel(5, 5).unwrap();
    assert_eq!(pixel, Color::rgba(0, 0, 0, 0));
}

#[test]
fn test_fill_rect_clipping() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 0, 0));
    
    // Rectangle partially outside bounds
    graphics.fill_rect(-10, -10, 30, 30);
    
    // Should only fill the visible part
    let pixel = graphics.get_pixel(0, 0).unwrap();
    assert_eq!(pixel, Color::rgb(255, 0, 0));
    
    let pixel = graphics.get_pixel(19, 19).unwrap();
    assert_eq!(pixel, Color::rgb(255, 0, 0));
    
    let pixel = graphics.get_pixel(20, 20).unwrap();
    assert_eq!(pixel, Color::rgba(0, 0, 0, 0));
}

#[test]
fn test_draw_line_horizontal() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 0, 0));
    graphics.draw_line(10, 50, 90, 50);

    // Check that pixels along the line are red
    for x in 10..=90 {
        let pixel = graphics.get_pixel(x, 50).unwrap();
        assert_eq!(pixel, Color::rgb(255, 0, 0));
    }
}

#[test]
fn test_draw_line_vertical() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(0, 255, 0));
    graphics.draw_line(50, 10, 50, 90);

    // Check that pixels along the line are green
    for y in 10..=90 {
        let pixel = graphics.get_pixel(50, y).unwrap();
        assert_eq!(pixel, Color::rgb(0, 255, 0));
    }
}

#[test]
fn test_draw_line_diagonal() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(0, 0, 255));
    graphics.draw_line(0, 0, 50, 50);

    // Check start and end points
    let pixel = graphics.get_pixel(0, 0).unwrap();
    assert_eq!(pixel, Color::rgb(0, 0, 255));
    
    let pixel = graphics.get_pixel(50, 50).unwrap();
    assert_eq!(pixel, Color::rgb(0, 0, 255));
}

#[test]
fn test_draw_circle() {
    let mut graphics = Graphics::new(200, 200).unwrap();
    graphics.set_color(Color::rgb(255, 255, 0));
    graphics.draw_circle(100, 100, 50);

    // Check that some points on the circle are yellow
    // Top point
    let pixel = graphics.get_pixel(100, 50).unwrap();
    assert_eq!(pixel, Color::rgb(255, 255, 0));
    
    // Right point
    let pixel = graphics.get_pixel(150, 100).unwrap();
    assert_eq!(pixel, Color::rgb(255, 255, 0));
    
    // Bottom point
    let pixel = graphics.get_pixel(100, 150).unwrap();
    assert_eq!(pixel, Color::rgb(255, 255, 0));
    
    // Left point
    let pixel = graphics.get_pixel(50, 100).unwrap();
    assert_eq!(pixel, Color::rgb(255, 255, 0));
}

#[test]
fn test_color_with_alpha() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    let semi_transparent = Color::rgba(255, 0, 0, 128);
    graphics.set_color(semi_transparent);
    graphics.set_pixel(50, 50);

    let pixel = graphics.get_pixel(50, 50).unwrap();
    assert_eq!(pixel, semi_transparent);
}

#[cfg(feature = "images")]
mod image_tests {
    use logic_nih_plug_graphics::{Image, GraphicsError};

    #[test]
    fn test_image_from_rgba8() {
        // Create a 2x2 red image
        let data = vec![
            255, 0, 0, 255, // Red pixel
            255, 0, 0, 255, // Red pixel
            255, 0, 0, 255, // Red pixel
            255, 0, 0, 255, // Red pixel
        ];
        
        let image = Image::from_rgba8(2, 2, data).unwrap();
        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);
        assert_eq!(image.dimensions(), (2, 2));
    }

    #[test]
    fn test_image_from_rgba8_invalid_size() {
        // Data too short for 2x2 image
        let data = vec![255, 0, 0, 255];
        
        let result = Image::from_rgba8(2, 2, data);
        assert!(matches!(result, Err(GraphicsError::InvalidImageData { .. })));
    }

    #[test]
    fn test_image_get_pixel() {
        let data = vec![
            255, 0, 0, 255,   // Red pixel at (0, 0)
            0, 255, 0, 255,   // Green pixel at (1, 0)
            0, 0, 255, 255,   // Blue pixel at (0, 1)
            255, 255, 0, 255, // Yellow pixel at (1, 1)
        ];
        
        let image = Image::from_rgba8(2, 2, data).unwrap();
        
        assert_eq!(image.get_pixel(0, 0), Some((255, 0, 0, 255)));
        assert_eq!(image.get_pixel(1, 0), Some((0, 255, 0, 255)));
        assert_eq!(image.get_pixel(0, 1), Some((0, 0, 255, 255)));
        assert_eq!(image.get_pixel(1, 1), Some((255, 255, 0, 255)));
    }

    #[test]
    fn test_image_get_pixel_out_of_bounds() {
        let data = vec![255, 0, 0, 255];
        let image = Image::from_rgba8(1, 1, data).unwrap();
        
        assert_eq!(image.get_pixel(1, 0), None);
        assert_eq!(image.get_pixel(0, 1), None);
        assert_eq!(image.get_pixel(1, 1), None);
    }

    #[test]
    fn test_image_set_pixel() {
        let data = vec![0, 0, 0, 255];
        let mut image = Image::from_rgba8(1, 1, data).unwrap();
        
        assert!(image.set_pixel(0, 0, 255, 0, 0, 255));
        assert_eq!(image.get_pixel(0, 0), Some((255, 0, 0, 255)));
    }

    #[test]
    fn test_image_set_pixel_out_of_bounds() {
        let data = vec![0, 0, 0, 255];
        let mut image = Image::from_rgba8(1, 1, data).unwrap();
        
        assert!(!image.set_pixel(1, 0, 255, 0, 0, 255));
        assert!(!image.set_pixel(0, 1, 255, 0, 0, 255));
    }

    #[test]
    fn test_image_as_rgba8() {
        let data = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let image = Image::from_rgba8(2, 1, data.clone()).unwrap();
        
        assert_eq!(image.as_rgba8(), &data[..]);
    }

    #[test]
    fn test_image_from_bytes_invalid() {
        // Try to load invalid data
        let invalid_data = b"not a valid image";
        
        let result = Image::from_bytes(invalid_data);
        assert!(matches!(result, Err(GraphicsError::ImageLoadError(_))));
    }

    #[test]
    fn test_image_save_and_load() {
        use std::fs;
        
        // Create a simple 2x2 image
        let data = vec![
            255, 0, 0, 255,   // Red
            0, 255, 0, 255,   // Green
            0, 0, 255, 255,   // Blue
            255, 255, 0, 255, // Yellow
        ];
        
        let image = Image::from_rgba8(2, 2, data).unwrap();
        
        // Save to a temporary file
        let temp_path = "test_output.png";
        image.save(temp_path).unwrap();
        
        // Load it back
        let loaded = Image::load(temp_path).unwrap();
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        
        // Clean up
        fs::remove_file(temp_path).ok();
    }

    #[test]
    fn test_image_format_support() {
        use std::fs;
        
        // Create a simple 2x2 image
        let data = vec![
            255, 0, 0, 255,   // Red
            0, 255, 0, 255,   // Green
            0, 0, 255, 255,   // Blue
            255, 255, 0, 255, // Yellow
        ];
        
        let image = Image::from_rgba8(2, 2, data).unwrap();
        
        // Test PNG format
        let png_path = "test_format.png";
        image.save(png_path).unwrap();
        let loaded_png = Image::load(png_path).unwrap();
        assert_eq!(loaded_png.width(), 2);
        assert_eq!(loaded_png.height(), 2);
        fs::remove_file(png_path).ok();
        
        // Test JPEG format
        let jpg_path = "test_format.jpg";
        image.save(jpg_path).unwrap();
        let loaded_jpg = Image::load(jpg_path).unwrap();
        assert_eq!(loaded_jpg.width(), 2);
        assert_eq!(loaded_jpg.height(), 2);
        fs::remove_file(jpg_path).ok();
        
        // Test GIF format
        let gif_path = "test_format.gif";
        image.save(gif_path).unwrap();
        let loaded_gif = Image::load(gif_path).unwrap();
        assert_eq!(loaded_gif.width(), 2);
        assert_eq!(loaded_gif.height(), 2);
        fs::remove_file(gif_path).ok();
    }
}

// Transformation tests

#[test]
fn test_transform_identity() {
    let graphics = Graphics::new(100, 100).unwrap();
    let transform = graphics.get_transform();
    assert!(transform.is_identity());
}

#[test]
fn test_transform_translation() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 0, 0));
    
    graphics.translate(10.0, 20.0);
    graphics.set_pixel(0, 0);
    
    // Pixel should be drawn at (10, 20) due to translation
    let pixel = graphics.get_pixel(10, 20).unwrap();
    assert_eq!(pixel, Color::rgb(255, 0, 0));
    
    // Original position should be empty
    let pixel = graphics.get_pixel(0, 0).unwrap();
    assert_eq!(pixel, Color::rgba(0, 0, 0, 0));
}

#[test]
fn test_transform_scaling() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(0, 255, 0));
    
    graphics.scale(2.0, 2.0);
    graphics.set_pixel(5, 5);
    
    // Pixel should be drawn at (10, 10) due to 2x scaling
    let pixel = graphics.get_pixel(10, 10).unwrap();
    assert_eq!(pixel, Color::rgb(0, 255, 0));
}

#[test]
fn test_transform_rotation() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(0, 0, 255));
    
    // Rotate 90 degrees counter-clockwise
    graphics.rotate(std::f32::consts::PI / 2.0);
    graphics.set_pixel(10, 0);
    
    // After 90 degree rotation, (10, 0) should map to approximately (0, 10)
    let pixel = graphics.get_pixel(0, 10).unwrap();
    assert_eq!(pixel, Color::rgb(0, 0, 255));
}

#[test]
fn test_transform_combined() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 255, 0));
    
    // Apply multiple transformations
    graphics.translate(20.0, 20.0);
    graphics.scale(2.0, 2.0);
    graphics.set_pixel(5, 5);
    
    // (5, 5) -> translate -> (25, 25) -> scale -> (50, 50)
    let pixel = graphics.get_pixel(50, 50).unwrap();
    assert_eq!(pixel, Color::rgb(255, 255, 0));
}

#[test]
fn test_transform_save_restore() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(255, 0, 255));
    
    // Save initial transform
    graphics.save_transform();
    
    // Apply transformation
    graphics.translate(10.0, 10.0);
    graphics.set_pixel(0, 0);
    
    // Pixel should be at (10, 10)
    let pixel = graphics.get_pixel(10, 10).unwrap();
    assert_eq!(pixel, Color::rgb(255, 0, 255));
    
    // Restore transform
    graphics.restore_transform();
    assert!(graphics.get_transform().is_identity());
    
    // Now pixel should be at (0, 0)
    graphics.set_pixel(0, 0);
    let pixel = graphics.get_pixel(0, 0).unwrap();
    assert_eq!(pixel, Color::rgb(255, 0, 255));
}

#[test]
fn test_transform_reset() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    
    graphics.translate(10.0, 20.0);
    graphics.rotate(1.0);
    graphics.scale(2.0, 3.0);
    
    assert!(!graphics.get_transform().is_identity());
    
    graphics.reset_transform();
    assert!(graphics.get_transform().is_identity());
}

#[test]
fn test_transform_point() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.translate(10.0, 20.0);
    
    let (x, y) = graphics.transform_point(5, 5);
    assert_eq!(x, 15);
    assert_eq!(y, 25);
}

#[test]
fn test_transform_fill_rect() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(128, 128, 128));
    
    graphics.translate(10.0, 10.0);
    graphics.fill_rect(0, 0, 10, 10);
    
    // Rectangle should be drawn at (10, 10) to (20, 20)
    let pixel = graphics.get_pixel(15, 15).unwrap();
    assert_eq!(pixel, Color::rgb(128, 128, 128));
    
    // Original position should be empty
    let pixel = graphics.get_pixel(5, 5).unwrap();
    assert_eq!(pixel, Color::rgba(0, 0, 0, 0));
}

#[test]
fn test_transform_draw_line() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.set_color(Color::rgb(200, 100, 50));
    
    graphics.translate(20.0, 20.0);
    graphics.draw_line(0, 0, 10, 0);
    
    // Line should be drawn from (20, 20) to (30, 20)
    let pixel = graphics.get_pixel(25, 20).unwrap();
    assert_eq!(pixel, Color::rgb(200, 100, 50));
}

#[test]
fn test_try_restore_transform_empty_stack() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    assert!(!graphics.try_restore_transform());
}

#[test]
fn test_try_restore_transform_with_saved() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.save_transform();
    graphics.translate(10.0, 10.0);
    assert!(graphics.try_restore_transform());
    assert!(graphics.get_transform().is_identity());
}

#[test]
#[should_panic(expected = "No saved transformation to restore")]
fn test_restore_transform_panic() {
    let mut graphics = Graphics::new(100, 100).unwrap();
    graphics.restore_transform(); // Should panic
}

// Text rendering tests (only when text feature is enabled)
#[cfg(feature = "text")]
mod text_tests {
    use super::*;
    use logic_nih_plug_graphics::{Font, FontSettings};

    fn load_test_font() -> Font {
        let font_data = include_bytes!("test_font.ttf");
        Font::from_bytes(font_data, FontSettings::default()).unwrap()
    }

    #[test]
    fn test_draw_text_basic() {
        let mut graphics = Graphics::new(400, 200).unwrap();
        graphics.set_color(Color::rgb(255, 255, 255));
        
        let font = load_test_font();
        graphics.draw_text("Hello", 10, 50, &font, 24.0);
        
        // Check that some pixels were modified (text was drawn)
        let pixels = graphics.as_bytes();
        let mut has_white_pixels = false;
        
        for chunk in pixels.chunks_exact(4) {
            if chunk[0] > 0 || chunk[1] > 0 || chunk[2] > 0 {
                has_white_pixels = true;
                break;
            }
        }
        
        assert!(has_white_pixels, "Text should have been drawn");
    }

    #[test]
    fn test_draw_text_multiple_characters() {
        let mut graphics = Graphics::new(400, 200).unwrap();
        graphics.set_color(Color::rgb(0, 255, 0));
        
        let font = load_test_font();
        graphics.draw_text("ABC", 10, 50, &font, 32.0);
        
        // Verify text was drawn
        let pixels = graphics.as_bytes();
        let green_pixel_count = pixels.chunks_exact(4)
            .filter(|chunk| chunk[1] > 0)
            .count();
        
        assert!(green_pixel_count > 0, "Text should have been drawn in green");
    }

    #[test]
    fn test_draw_text_different_sizes() {
        let mut graphics = Graphics::new(400, 200).unwrap();
        graphics.set_color(Color::rgb(255, 0, 0));
        
        let font = load_test_font();
        
        // Draw small text
        graphics.draw_text("A", 10, 50, &font, 12.0);
        
        // Draw large text
        graphics.draw_text("A", 100, 100, &font, 48.0);
        
        // Both should have drawn something
        let pixels = graphics.as_bytes();
        let red_pixel_count = pixels.chunks_exact(4)
            .filter(|chunk| chunk[0] > 0)
            .count();
        
        assert!(red_pixel_count > 0, "Text should have been drawn");
    }

    #[test]
    fn test_draw_text_empty_string() {
        let mut graphics = Graphics::new(400, 200).unwrap();
        graphics.set_color(Color::rgb(255, 255, 255));
        
        let font = load_test_font();
        
        // Should not panic
        graphics.draw_text("", 10, 50, &font, 24.0);
    }

    #[test]
    fn test_draw_text_with_color() {
        let mut graphics = Graphics::new(400, 200).unwrap();
        
        let font = load_test_font();
        
        // Draw in blue
        graphics.set_color(Color::rgb(0, 0, 255));
        graphics.draw_text("Blue", 10, 50, &font, 24.0);
        
        // Check for blue pixels
        let pixels = graphics.as_bytes();
        let has_blue = pixels.chunks_exact(4)
            .any(|chunk| chunk[2] > 0);
        
        assert!(has_blue, "Should have blue pixels from text");
    }
}
