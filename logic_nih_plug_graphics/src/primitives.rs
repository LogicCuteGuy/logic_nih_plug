//! Drawing primitives implementation.

use crate::{Color, GraphicsError, Transform};

/// A graphics context for 2D drawing operations.
///
/// This struct manages a pixel buffer and provides methods for drawing
/// shapes, lines, and other primitives. The pixel buffer is stored in
/// RGBA format with 4 bytes per pixel.
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance
/// for drawing operations.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_graphics::{Graphics, Color};
///
/// let mut graphics = Graphics::new(800, 600).unwrap();
/// graphics.set_color(Color::rgb(255, 0, 0));
/// graphics.fill_rect(10, 10, 100, 100);
/// ```
#[derive(Debug, Clone)]
pub struct Graphics {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
    current_color: Color,
    transform_stack: Vec<Transform>,
    current_transform: Transform,
}

impl Graphics {
    /// Creates a new graphics context with the specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the graphics context in pixels
    /// * `height` - The height of the graphics context in pixels
    ///
    /// # Errors
    ///
    /// Returns `GraphicsError::InvalidDimensions` if width or height is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::Graphics;
    ///
    /// let graphics = Graphics::new(800, 600).unwrap();
    /// assert_eq!(graphics.width(), 800);
    /// assert_eq!(graphics.height(), 600);
    /// ```
    pub fn new(width: u32, height: u32) -> Result<Self, GraphicsError> {
        if width == 0 || height == 0 {
            return Err(GraphicsError::InvalidDimensions(width, height));
        }

        let pixel_count = (width as usize)
            .checked_mul(height as usize)
            .ok_or(GraphicsError::InvalidDimensions(width, height))?;
        
        let buffer_size = pixel_count
            .checked_mul(4)
            .ok_or(GraphicsError::InvalidDimensions(width, height))?;

        Ok(Self {
            width,
            height,
            pixels: vec![0; buffer_size],
            current_color: Color::rgb(0, 0, 0),
            transform_stack: Vec::new(),
            current_transform: Transform::identity(),
        })
    }

    /// Returns the width of the graphics context.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the graphics context.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Sets the current drawing color.
    ///
    /// All subsequent drawing operations will use this color until it is changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(255, 0, 0));
    /// ```
    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.current_color = color;
    }

    /// Returns the current drawing color.
    #[inline]
    pub fn current_color(&self) -> Color {
        self.current_color
    }

    /// Returns a reference to the pixel buffer.
    ///
    /// The buffer is in RGBA format with 4 bytes per pixel.
    /// Pixels are stored row by row from top to bottom.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::Graphics;
    ///
    /// let graphics = Graphics::new(800, 600).unwrap();
    /// let bytes = graphics.as_bytes();
    /// assert_eq!(bytes.len(), 800 * 600 * 4);
    /// ```
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.pixels
    }

    /// Returns a mutable reference to the pixel buffer.
    ///
    /// This allows direct manipulation of the pixel data if needed.
    /// The buffer is in RGBA format with 4 bytes per pixel.
    #[inline]
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Clears the entire graphics context to the current color.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(255, 255, 255));
    /// graphics.clear();
    /// ```
    pub fn clear(&mut self) {
        let color = self.current_color;
        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk[0] = color.r;
            chunk[1] = color.g;
            chunk[2] = color.b;
            chunk[3] = color.a;
        }
    }

    /// Sets a single pixel at the specified coordinates.
    ///
    /// If the coordinates are out of bounds, this method does nothing.
    /// The current transformation is applied to the coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the pixel
    /// * `y` - The y coordinate of the pixel
    #[inline]
    pub fn set_pixel(&mut self, x: i32, y: i32) {
        // Apply transformation
        let (tx, ty) = self.current_transform.apply_int(x, y);
        
        if tx < 0 || ty < 0 || tx >= self.width as i32 || ty >= self.height as i32 {
            return;
        }

        let index = ((ty as u32 * self.width + tx as u32) * 4) as usize;
        let color = self.current_color;
        
        if index + 3 < self.pixels.len() {
            self.pixels[index] = color.r;
            self.pixels[index + 1] = color.g;
            self.pixels[index + 2] = color.b;
            self.pixels[index + 3] = color.a;
        }
    }

    /// Gets the color of a pixel at the specified coordinates.
    ///
    /// Returns `None` if the coordinates are out of bounds.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the pixel
    /// * `y` - The y coordinate of the pixel
    #[inline]
    pub fn get_pixel(&self, x: i32, y: i32) -> Option<Color> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        let index = ((y as u32 * self.width + x as u32) * 4) as usize;
        
        if index + 3 < self.pixels.len() {
            Some(Color::rgba(
                self.pixels[index],
                self.pixels[index + 1],
                self.pixels[index + 2],
                self.pixels[index + 3],
            ))
        } else {
            None
        }
    }

    /// Fills a rectangle with the current color.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the top-left corner
    /// * `y` - The y coordinate of the top-left corner
    /// * `width` - The width of the rectangle
    /// * `height` - The height of the rectangle
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(255, 0, 0));
    /// graphics.fill_rect(10, 10, 100, 100);
    /// ```
    pub fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32) {
        let x_end = x.saturating_add(width as i32);
        let y_end = y.saturating_add(height as i32);

        let x_start = x.max(0).min(self.width as i32);
        let x_end = x_end.max(0).min(self.width as i32);
        let y_start = y.max(0).min(self.height as i32);
        let y_end = y_end.max(0).min(self.height as i32);

        for py in y_start..y_end {
            for px in x_start..x_end {
                self.set_pixel(px, py);
            }
        }
    }

    /// Draws a line from (x1, y1) to (x2, y2) using Bresenham's line algorithm.
    ///
    /// # Arguments
    ///
    /// * `x1` - The x coordinate of the start point
    /// * `y1` - The y coordinate of the start point
    /// * `x2` - The x coordinate of the end point
    /// * `y2` - The y coordinate of the end point
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(0, 255, 0));
    /// graphics.draw_line(0, 0, 100, 100);
    /// ```
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let dx = (x2 - x1).abs();
        let dy = -(y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x1;
        let mut y = y1;

        loop {
            self.set_pixel(x, y);

            if x == x2 && y == y2 {
                break;
            }

            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draws a circle using the midpoint circle algorithm.
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate of the center
    /// * `y` - The y coordinate of the center
    /// * `radius` - The radius of the circle
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(0, 0, 255));
    /// graphics.draw_circle(400, 300, 50);
    /// ```
    pub fn draw_circle(&mut self, x: i32, y: i32, radius: u32) {
        let r = radius as i32;
        let mut dx = r;
        let mut dy = 0;
        let mut err = 0;

        while dx >= dy {
            self.set_pixel(x + dx, y + dy);
            self.set_pixel(x + dy, y + dx);
            self.set_pixel(x - dy, y + dx);
            self.set_pixel(x - dx, y + dy);
            self.set_pixel(x - dx, y - dy);
            self.set_pixel(x - dy, y - dx);
            self.set_pixel(x + dy, y - dx);
            self.set_pixel(x + dx, y - dy);

            if err <= 0 {
                dy += 1;
                err += 2 * dy + 1;
            }

            if err > 0 {
                dx -= 1;
                err -= 2 * dx + 1;
            }
        }
    }

    /// Draws text at the specified position using the provided font.
    ///
    /// This method is only available when the `text` feature is enabled.
    ///
    /// # Arguments
    ///
    /// * `text` - The text string to draw
    /// * `x` - The x coordinate of the baseline start
    /// * `y` - The y coordinate of the baseline
    /// * `font` - The font to use for rendering
    /// * `size` - The font size in pixels
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    /// #[cfg(feature = "text")]
    /// use logic_nih_plug_graphics::{Font, FontSettings};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(255, 255, 255));
    ///
    /// #[cfg(feature = "text")]
    /// {
    ///     let font_data = include_bytes!("../tests/test_font.ttf");
    ///     let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
    ///     graphics.draw_text("Hello", 10, 50, &font, 24.0);
    /// }
    /// ```
    #[cfg(feature = "text")]
    pub fn draw_text(&mut self, text: &str, x: i32, y: i32, font: &crate::text::Font, size: f32) {
        let mut cursor_x = x as f32;
        
        for ch in text.chars() {
            let (metrics, bitmap) = font.rasterize(ch, size);
            
            // Calculate the position for this glyph
            let glyph_x = cursor_x + metrics.xmin as f32;
            let glyph_y = y as f32 - metrics.ymin as f32 - metrics.height as f32;
            
            // Draw the glyph bitmap
            for bitmap_y in 0..metrics.height {
                for bitmap_x in 0..metrics.width {
                    let alpha = bitmap[bitmap_y * metrics.width + bitmap_x];
                    
                    if alpha > 0 {
                        let pixel_x = (glyph_x + bitmap_x as f32) as i32;
                        let pixel_y = (glyph_y + bitmap_y as f32) as i32;
                        
                        // Blend the text color with the existing pixel based on alpha
                        if pixel_x >= 0 && pixel_x < self.width as i32 
                            && pixel_y >= 0 && pixel_y < self.height as i32 {
                            
                            let idx = ((pixel_y as u32 * self.width + pixel_x as u32) * 4) as usize;
                            
                            if idx + 3 < self.pixels.len() {
                                let alpha_f = alpha as f32 / 255.0;
                                let inv_alpha = 1.0 - alpha_f;
                                
                                // Alpha blend
                                self.pixels[idx] = (self.current_color.r as f32 * alpha_f 
                                    + self.pixels[idx] as f32 * inv_alpha) as u8;
                                self.pixels[idx + 1] = (self.current_color.g as f32 * alpha_f 
                                    + self.pixels[idx + 1] as f32 * inv_alpha) as u8;
                                self.pixels[idx + 2] = (self.current_color.b as f32 * alpha_f 
                                    + self.pixels[idx + 2] as f32 * inv_alpha) as u8;
                                // Keep existing alpha
                            }
                        }
                    }
                }
            }
            
            // Advance cursor
            cursor_x += metrics.advance_width;
        }
    }

    // Transformation methods

    /// Returns the current transformation matrix.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Transform};
    ///
    /// let graphics = Graphics::new(800, 600).unwrap();
    /// let transform = graphics.get_transform();
    /// assert!(transform.is_identity());
    /// ```
    #[inline]
    pub fn get_transform(&self) -> Transform {
        self.current_transform
    }

    /// Sets the current transformation matrix.
    ///
    /// This replaces the current transformation with the specified one.
    ///
    /// # Arguments
    ///
    /// * `transform` - The new transformation matrix
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Transform};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// let transform = Transform::translation(10.0, 20.0);
    /// graphics.set_transform(transform);
    /// ```
    pub fn set_transform(&mut self, transform: Transform) {
        self.current_transform = transform;
    }

    /// Resets the transformation to identity (no transformation).
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Transform};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.translate(10.0, 20.0);
    /// graphics.reset_transform();
    /// assert!(graphics.get_transform().is_identity());
    /// ```
    pub fn reset_transform(&mut self) {
        self.current_transform = Transform::identity();
    }

    /// Applies a translation to the current transformation.
    ///
    /// # Arguments
    ///
    /// * `tx` - Translation in x direction
    /// * `ty` - Translation in y direction
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(255, 0, 0));
    /// graphics.translate(10.0, 20.0);
    /// graphics.fill_rect(0, 0, 50, 50); // Will be drawn at (10, 20)
    /// ```
    pub fn translate(&mut self, tx: f32, ty: f32) {
        self.current_transform = self.current_transform.translate(tx, ty);
    }

    /// Applies a rotation to the current transformation.
    ///
    /// # Arguments
    ///
    /// * `angle` - Rotation angle in radians (positive = counter-clockwise)
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(0, 255, 0));
    /// graphics.rotate(std::f32::consts::PI / 4.0); // 45 degrees
    /// graphics.fill_rect(0, 0, 50, 50);
    /// ```
    pub fn rotate(&mut self, angle: f32) {
        self.current_transform = self.current_transform.rotate(angle);
    }

    /// Applies a scaling to the current transformation.
    ///
    /// # Arguments
    ///
    /// * `sx` - Scale factor in x direction
    /// * `sy` - Scale factor in y direction
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(0, 0, 255));
    /// graphics.scale(2.0, 2.0);
    /// graphics.fill_rect(0, 0, 50, 50); // Will be drawn as 100x100
    /// ```
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.current_transform = self.current_transform.scale_by(sx, sy);
    }

    /// Saves the current transformation state onto a stack.
    ///
    /// This allows you to temporarily modify the transformation and then
    /// restore it later with `restore_transform()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::{Graphics, Color};
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.set_color(Color::rgb(255, 0, 0));
    ///
    /// graphics.save_transform();
    /// graphics.translate(100.0, 100.0);
    /// graphics.fill_rect(0, 0, 50, 50); // Drawn at (100, 100)
    ///
    /// graphics.restore_transform();
    /// graphics.fill_rect(0, 0, 50, 50); // Drawn at (0, 0)
    /// ```
    pub fn save_transform(&mut self) {
        self.transform_stack.push(self.current_transform);
    }

    /// Restores the transformation state from the stack.
    ///
    /// This pops the most recently saved transformation from the stack
    /// and makes it the current transformation.
    ///
    /// # Panics
    ///
    /// Panics if there are no saved transformations on the stack.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::Graphics;
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.save_transform();
    /// graphics.translate(10.0, 20.0);
    /// graphics.restore_transform();
    /// assert!(graphics.get_transform().is_identity());
    /// ```
    pub fn restore_transform(&mut self) {
        if let Some(transform) = self.transform_stack.pop() {
            self.current_transform = transform;
        } else {
            panic!("No saved transformation to restore");
        }
    }

    /// Tries to restore the transformation state from the stack.
    ///
    /// Returns `true` if a transformation was restored, `false` if the stack was empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::Graphics;
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// assert!(!graphics.try_restore_transform());
    ///
    /// graphics.save_transform();
    /// graphics.translate(10.0, 20.0);
    /// assert!(graphics.try_restore_transform());
    /// ```
    pub fn try_restore_transform(&mut self) -> bool {
        if let Some(transform) = self.transform_stack.pop() {
            self.current_transform = transform;
            true
        } else {
            false
        }
    }

    /// Applies the current transformation to a point.
    ///
    /// This is useful for transforming coordinates before drawing.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate
    /// * `y` - Y coordinate
    ///
    /// # Returns
    ///
    /// A tuple `(x', y')` representing the transformed point as integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_graphics::Graphics;
    ///
    /// let mut graphics = Graphics::new(800, 600).unwrap();
    /// graphics.translate(10.0, 20.0);
    /// let (x, y) = graphics.transform_point(5, 5);
    /// assert_eq!(x, 15);
    /// assert_eq!(y, 25);
    /// ```
    #[inline]
    pub fn transform_point(&self, x: i32, y: i32) -> (i32, i32) {
        self.current_transform.apply_int(x, y)
    }
}
