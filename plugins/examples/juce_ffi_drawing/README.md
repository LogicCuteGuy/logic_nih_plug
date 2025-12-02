# JUCE FFI Custom Drawing Example

This example demonstrates advanced custom drawing capabilities using JUCE's Graphics context through FFI bindings. It showcases the full range of drawing primitives, color manipulation, font rendering, and path-based graphics available through the `nih_plug_juce` crate.

## Overview

This plugin provides comprehensive examples of:

- **Graphics Context Operations**: Drawing shapes, lines, fills, and strokes
- **Color System**: Creating, manipulating, and interpolating colors
- **Font Rendering**: Text drawing with different sizes, styles, and justifications
- **Path Graphics**: Creating complex shapes using lines, curves, and geometric primitives
- **Real-time Visualization**: Techniques for audio visualization and dynamic graphics
- **Advanced Techniques**: Gradients, transparency, shadows, and composite effects

## Features Demonstrated

### 1. Basic Shapes and Colors

The `example_basic_shapes()` function demonstrates fundamental drawing operations:

```rust
component.set_paint_callback(|g: &mut Graphics| {
    // Draw filled rectangles
    if let Ok(red) = Colour::from_rgb(255, 80, 80) {
        g.set_colour(&red);
        g.fill_rect(50, 50, 100, 80);
    }
    
    // Draw outlined rectangles
    if let Ok(yellow) = Colour::from_rgb(255, 255, 100) {
        g.set_colour(&yellow);
        g.draw_rect(50, 160, 100, 80);
    }
    
    // Draw ellipses
    if let Ok(cyan) = Colour::from_rgb(100, 255, 255) {
        g.set_colour(&cyan);
        g.fill_ellipse(420.0, 60.0, 80.0, 60.0);
    }
    
    // Draw lines
    if let Ok(white) = Colour::from_rgb(255, 255, 255) {
        g.set_colour(&white);
        g.draw_line(50.0, 280.0, 550.0, 280.0);
    }
})?;
```

**Key Operations:**
- `fill_rect()` - Draw filled rectangles
- `draw_rect()` - Draw outlined rectangles
- `fill_ellipse()` - Draw filled ellipses and circles
- `draw_line()` - Draw straight lines
- `set_colour()` - Set the current drawing color

### 2. Color Manipulation and Gradients

The `example_color_manipulation()` function shows advanced color techniques:

```rust
// Create colors from RGB
let color = Colour::from_rgb(255, 100, 50)?;

// Create colors from hex strings
let hex_color = Colour::from_hex("#FF6B35")?;

// Interpolate between colors for gradients
let start = Colour::from_rgb(255, 50, 50)?;
let end = Colour::from_rgb(50, 50, 255)?;
for i in 0..50 {
    let proportion = i as f32 / 49.0;
    let interpolated = start.interpolated_with(&end, proportion)?;
    g.set_colour(&interpolated);
    g.fill_rect(50 + i * 10, 50, 10, 60);
}

// Adjust brightness
let brighter = color.brighter(1.5)?;
let darker = color.darker(0.5)?;

// Add transparency
let transparent = color.with_alpha(0.5)?;
```

**Color Operations:**
- `from_rgb()` / `from_rgba()` - Create colors from RGB(A) values
- `from_hex()` - Create colors from hex strings
- `to_hex()` - Convert colors to hex strings
- `interpolated_with()` - Blend between two colors
- `brighter()` / `darker()` - Adjust brightness
- `with_alpha()` - Set transparency

### 3. Text and Font Rendering

The `example_text_and_fonts()` function demonstrates text drawing:

```rust
component.set_paint_callback(|g: &mut Graphics| {
    if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
        g.set_colour(&text_color);
        
        // Draw text with different justifications
        g.draw_text("Left Justified", 50, 220, 500, 30, Justification::Left);
        g.draw_text("Centered Text", 50, 260, 500, 30, Justification::Centred);
        g.draw_text("Right Justified", 50, 300, 500, 30, Justification::Right);
        
        // Multiline text
        let multiline = "Line 1\nLine 2\nLine 3";
        g.draw_text(multiline, 50, 350, 500, 100, Justification::Centred);
    }
})?;
```

**Text Operations:**
- `draw_text()` - Draw text with specified bounds and justification
- Support for multiline text with `\n`
- Three justification modes: Left, Centred, Right

**Font System:**
```rust
// Create fonts with different sizes
let small_font = Font::new(12.0)?;
let large_font = Font::new(24.0)?;

// Create fonts with specific typefaces
let custom_font = Font::with_typeface("Arial", 16.0)?;

// Set font styles
let mut font = Font::new(14.0)?;
font.set_bold(true);
font.set_italic(true);
font.set_underline(true);

// Measure text
let width = font.get_string_width("Hello")?;
let height = font.get_height()?;
```

### 4. Paths and Complex Shapes

The `example_paths_and_shapes()` function shows path-based drawing:

```rust
// Create a simple polygon
let mut path = Path::new()?;
path.start_new_sub_path(50.0, 80.0);
path.line_to(150.0, 80.0);
path.line_to(150.0, 150.0);
path.line_to(50.0, 150.0);
path.close_sub_path();

g.fill_path(&path);

// Create curves with quadratic bezier
let mut curved = Path::new()?;
curved.start_new_sub_path(200.0, 80.0);
curved.quadratic_to(250.0, 50.0, 300.0, 80.0);  // control point, end point

// Create curves with cubic bezier
let mut smooth = Path::new()?;
smooth.start_new_sub_path(350.0, 80.0);
smooth.cubic_to(380.0, 50.0, 420.0, 50.0, 450.0, 80.0);  // cp1, cp2, end

// Add geometric shapes to paths
let mut shapes = Path::new()?;
shapes.add_rectangle(100.0, 100.0, 50.0, 50.0);
shapes.add_ellipse(200.0, 200.0, 60.0, 60.0);
shapes.add_arc(300.0, 300.0, 80.0, 80.0, 0.0, std::f32::consts::PI);

// Stroke or fill paths
g.stroke_path(&path);  // Draw outline
g.fill_path(&path);    // Fill interior
```

**Path Operations:**
- `start_new_sub_path()` - Begin a new path segment
- `line_to()` - Add straight line
- `quadratic_to()` - Add quadratic bezier curve
- `cubic_to()` - Add cubic bezier curve
- `add_rectangle()` - Add rectangle to path
- `add_ellipse()` - Add ellipse to path
- `add_arc()` - Add arc to path
- `close_sub_path()` - Close current path segment
- `stroke_path()` - Draw path outline
- `fill_path()` - Fill path interior

**Creating Complex Shapes:**

Star shape example:
```rust
let mut star = Path::new()?;
let center_x = 100.0;
let center_y = 280.0;
let outer_radius = 50.0;
let inner_radius = 20.0;

for i in 0..10 {
    let angle = (i as f32 * std::f32::consts::PI / 5.0) - std::f32::consts::PI / 2.0;
    let radius = if i % 2 == 0 { outer_radius } else { inner_radius };
    let x = center_x + radius * angle.cos();
    let y = center_y + radius * angle.sin();
    
    if i == 0 {
        star.start_new_sub_path(x, y);
    } else {
        star.line_to(x, y);
    }
}
star.close_sub_path();
g.fill_path(&star);
```

### 5. Audio Visualization

The `example_audio_visualization()` function demonstrates real-time graphics:

```rust
component.set_paint_callback(|g: &mut Graphics| {
    // Draw waveform using path
    let mut waveform = Path::new()?;
    waveform.start_new_sub_path(50.0, 120.0);
    for i in 0..120 {
        let x = 50.0 + i as f32 * 5.0;
        let phase = i as f32 * 0.1;
        let y = 120.0 + (phase.sin() * 40.0);
        waveform.line_to(x, y);
    }
    g.stroke_path(&waveform);
    
    // Draw level meters
    let levels = [0.8, 0.6, 0.9, 0.4, 0.7];
    for (i, &level) in levels.iter().enumerate() {
        let height = (level * 120.0) as i32;
        let color = if level > 0.8 {
            Colour::from_rgb(255, 80, 80)  // Red for high levels
        } else if level > 0.6 {
            Colour::from_rgb(255, 200, 80)  // Yellow for medium
        } else {
            Colour::from_rgb(80, 255, 80)  // Green for low
        };
        g.set_colour(&color?);
        g.fill_rect(x, y, 40, height);
    }
})?;
```

**Visualization Techniques:**
- Waveform display using paths
- Level meters with color-coded thresholds
- Grid overlays for reference
- Real-time data representation

### 6. Advanced Techniques

The `example_combined_demo()` function shows composite effects:

```rust
// Gradient backgrounds
for i in 0..60 {
    let proportion = i as f32 / 59.0;
    let color = top_color.interpolated_with(&bottom_color, proportion)?;
    g.set_colour(&color);
    g.fill_rect(0, i * 10, 800, 10);
}

// Text with shadow effect
let shadow = Colour::from_rgba(0, 0, 0, 150)?;
g.set_colour(&shadow);
g.draw_text("Text", 202, 452, 400, 80, Justification::Centred);

let text_color = Colour::from_rgb(255, 255, 255)?;
g.set_colour(&text_color);
g.draw_text("Text", 200, 450, 400, 80, Justification::Centred);

// Transparent overlays
let overlay = Colour::from_rgba(255, 200, 100, 200)?;
g.set_colour(&overlay);
g.fill_path(&shape);
```

## Building

### Prerequisites

- Rust toolchain (1.70 or later)
- CMake (3.15 or later)
- Platform-specific requirements:
  - **Windows**: Visual Studio 2019 or later with C++ tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: GCC/Clang, X11 development libraries

### Build Commands

Build the plugin in release mode:

```bash
cargo xtask bundle juce_ffi_drawing --release
```

Build in debug mode for development:

```bash
cargo xtask bundle juce_ffi_drawing
```

Run tests:

```bash
cargo test -p juce_ffi_drawing
```

## Usage

### Loading the Plugin

After building, the plugin will be available in the `target/bundled` directory:

- **VST3**: `target/bundled/juce_ffi_drawing.vst3`
- **CLAP**: `target/bundled/juce_ffi_drawing.clap`

Load the plugin in your DAW to see the custom drawing examples.

### Standalone Mode

Run the plugin as a standalone application:

```bash
cargo run --release -p juce_ffi_drawing --features nih_plug/standalone
```

## Code Structure

The example includes six demonstration functions:

### `example_basic_shapes()`
Basic drawing operations with rectangles, ellipses, and lines.

### `example_color_manipulation()`
Color creation, manipulation, gradients, and transparency.

### `example_text_and_fonts()`
Text rendering with different sizes, styles, and justifications.

### `example_paths_and_shapes()`
Path-based drawing with lines, curves, and complex shapes.

### `example_audio_visualization()`
Real-time graphics for waveforms and level meters.

### `example_combined_demo()`
Comprehensive demonstration combining all techniques.

## Performance Considerations

### Drawing Performance

- **Paint callbacks**: Called on repaint, keep operations efficient
- **Path complexity**: Complex paths with many points may impact performance
- **Color operations**: Color creation and manipulation are lightweight
- **Text rendering**: Font operations are cached by JUCE

### Optimization Tips

1. **Cache paths**: Create paths once, reuse them
2. **Minimize allocations**: Reuse color objects when possible
3. **Batch operations**: Group similar drawing operations together
4. **Use appropriate precision**: Integer coordinates for pixel-perfect rendering

Example of efficient drawing:

```rust
// Good: Create path once, reuse
let path = create_complex_path()?;
component.set_paint_callback(move |g| {
    g.fill_path(&path);
});

// Less efficient: Recreate path every frame
component.set_paint_callback(|g| {
    let path = create_complex_path()?;  // Recreated each paint
    g.fill_path(&path);
});
```

## Thread Safety

All JUCE GUI operations, including drawing, must occur on the message thread. The `nih_plug_juce` crate enforces this through:

1. **Type System**: Graphics types don't implement `Send` or `Sync`
2. **Paint Callbacks**: Always called on the message thread
3. **Safe Updates**: Use `MessageManager::call_async()` for cross-thread updates

Example of safe cross-thread drawing update:

```rust
// From audio processing thread
let audio_level = compute_level();
MessageManager::call_async(move || {
    // This runs on message thread, safe to update UI
    component.repaint();
});
```

## Graphics Context Lifetime

The Graphics context is only valid during the paint callback:

```rust
// Correct: Use Graphics within callback
component.set_paint_callback(|g: &mut Graphics| {
    g.fill_rect(0, 0, 100, 100);  // Valid
})?;

// Incorrect: Cannot store Graphics reference
let mut stored_graphics = None;
component.set_paint_callback(|g: &mut Graphics| {
    stored_graphics = Some(g);  // Compile error: lifetime violation
})?;
```

The lifetime parameter on `Graphics<'a>` prevents misuse at compile time.

## Error Handling

All drawing operations that can fail return `Result<T>`:

```rust
match Colour::from_hex("#INVALID") {
    Ok(color) => {
        g.set_colour(&color);
    }
    Err(JuceError::ColourParseError(msg)) => {
        eprintln!("Invalid color: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

Most basic drawing operations (fill_rect, draw_line, etc.) don't return errors as they operate on valid Graphics contexts.

## Integration with Plugin Parameters

Drawing can be driven by plugin parameters:

```rust
component.set_paint_callback(move || {
    let mix = params.mix.value();
    
    // Use parameter value to control visualization
    let color_intensity = (mix * 255.0) as u8;
    if let Ok(color) = Colour::from_rgb(color_intensity, 100, 255 - color_intensity) {
        g.set_colour(&color);
        g.fill_rect(0, 0, 400, 300);
    }
    
    // Draw parameter value as text
    let text = format!("Mix: {:.0}%", mix * 100.0);
    g.draw_text(&text, 0, 0, 400, 30, Justification::Centred);
})?;
```

## Requirements Validated

This example validates the following requirements from the JUCE FFI Integration spec:

- **Requirement 2**: Graphics context usage for custom drawing
- **Requirement 13**: Colour system (creation, manipulation, interpolation)
- **Requirement 14**: Font system (creation, styles, text measurement)
- **Requirement 15**: Image class usage
- **Requirement 28.2**: Complete plugin examples with documentation
- **Requirement 31**: Path class for custom shapes
- **Requirement 32**: AffineTransform for transformations

## Further Reading

- [nih_plug_juce Documentation](../../../nih_plug_juce/DOCUMENTATION.md)
- [JUCE Graphics Tutorial](https://docs.juce.com/master/tutorial_graphics_class.html)
- [JUCE Path Tutorial](https://docs.juce.com/master/tutorial_paths.html)
- [nih-plug Guide](https://github.com/robbert-vdh/nih-plug)

## License

ISC License - See LICENSE file for details.
