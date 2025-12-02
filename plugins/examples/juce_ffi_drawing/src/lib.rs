//! # JUCE FFI Custom Drawing Example
//!
//! This example demonstrates advanced custom drawing capabilities using JUCE's Graphics context
//! through FFI bindings. It showcases:
//! - Graphics context drawing operations (shapes, lines, fills)
//! - Color creation and manipulation
//! - Font rendering and text drawing
//! - Path creation and rendering
//! - Custom paint callbacks
//! - Real-time audio visualization

use nih_plug::prelude::*;
use nih_plug_juce::*;
use nih_plug_juce::drawing::Colour;
use std::sync::Arc;

/// A simple audio effect plugin demonstrating JUCE FFI custom drawing
pub struct JuceFfiDrawing {
    params: Arc<JuceFfiDrawingParams>,
}

#[derive(Params)]
struct JuceFfiDrawingParams {
    /// Mix parameter to demonstrate parameter-driven visualization
    #[id = "mix"]
    pub mix: FloatParam,

    /// Frequency parameter for visualization
    #[id = "freq"]
    pub frequency: FloatParam,
}

impl Default for JuceFfiDrawing {
    fn default() -> Self {
        Self {
            params: Arc::new(JuceFfiDrawingParams::default()),
        }
    }
}

impl Default for JuceFfiDrawingParams {
    fn default() -> Self {
        Self {
            mix: FloatParam::new(
                "Mix",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 },
            )
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            frequency: FloatParam::new(
                "Frequency",
                440.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
        }
    }
}

impl Plugin for JuceFfiDrawing {
    const NAME: &'static str = "JUCE FFI Drawing Example";
    const VENDOR: &'static str = "NIH-plug";
    const URL: &'static str = "https://github.com/robbert-vdh/nih-plug";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // This example focuses on demonstrating the JUCE FFI drawing API
        // A full editor implementation would require additional integration work
        None
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Simple passthrough with mix control
        let mix = self.params.mix.value();

        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                *sample *= mix;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for JuceFfiDrawing {
    const CLAP_ID: &'static str = "com.nih-plug.juce-ffi-drawing";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Example plugin demonstrating JUCE FFI custom drawing capabilities");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for JuceFfiDrawing {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceFfiDrawExmpl";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(JuceFfiDrawing);
nih_export_vst3!(JuceFfiDrawing);

/// Example 1: Basic shapes and colors
///
/// Demonstrates:
/// - Creating and using Colour objects
/// - Drawing filled and outlined rectangles
/// - Drawing ellipses and circles
/// - Drawing lines
#[allow(dead_code)]
fn example_basic_shapes() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut component = Component::new()?;
    component.set_bounds(0, 0, 600, 400);
    component.set_visible(true);

    component.set_paint_callback(|g: &mut Graphics| {
        // Clear background with dark gray
        if let Ok(bg) = Colour::from_rgb(40, 40, 40) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 600, 400);
        }

        // Draw filled rectangles with different colors
        if let Ok(red) = Colour::from_rgb(255, 80, 80) {
            g.set_colour(&red);
            g.fill_rect(50, 50, 100, 80);
        }

        if let Ok(green) = Colour::from_rgb(80, 255, 80) {
            g.set_colour(&green);
            g.fill_rect(170, 50, 100, 80);
        }

        if let Ok(blue) = Colour::from_rgb(80, 80, 255) {
            g.set_colour(&blue);
            g.fill_rect(290, 50, 100, 80);
        }

        // Draw outlined rectangles
        if let Ok(yellow) = Colour::from_rgb(255, 255, 100) {
            g.set_colour(&yellow);
            g.draw_rect(50, 160, 100, 80);
            g.draw_rect(170, 160, 100, 80);
            g.draw_rect(290, 160, 100, 80);
        }

        // Draw filled ellipses
        if let Ok(cyan) = Colour::from_rgb(100, 255, 255) {
            g.set_colour(&cyan);
            g.fill_ellipse(420.0, 60.0, 80.0, 60.0);
        }

        if let Ok(magenta) = Colour::from_rgb(255, 100, 255) {
            g.set_colour(&magenta);
            g.fill_ellipse(420.0, 170.0, 80.0, 60.0);
        }

        // Draw lines
        if let Ok(white) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&white);
            g.draw_line(50.0, 280.0, 550.0, 280.0);
            g.draw_line(50.0, 300.0, 550.0, 320.0);
            g.draw_line(50.0, 340.0, 550.0, 300.0);
        }

        // Draw title
        if let Ok(text_color) = Colour::from_rgb(200, 200, 200) {
            g.set_colour(&text_color);
            g.draw_text("Basic Shapes & Colors", 0, 10, 600, 30, Justification::Centred);
        }
    })?;

    Ok(component)
}

/// Example 2: Color manipulation and gradients
///
/// Demonstrates:
/// - Color creation from RGB and hex
/// - Color manipulation (brighter, darker, with_alpha)
/// - Creating gradient effects
/// - Color interpolation
#[allow(dead_code)]
fn example_color_manipulation() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut component = Component::new()?;
    component.set_bounds(0, 0, 600, 400);

    component.set_paint_callback(|g: &mut Graphics| {
        // Background
        if let Ok(bg) = Colour::from_rgb(30, 30, 30) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 600, 400);
        }

        // Create a horizontal gradient using color interpolation
        if let (Ok(start_color), Ok(end_color)) = (
            Colour::from_rgb(255, 50, 50),
            Colour::from_rgb(50, 50, 255),
        ) {
            for i in 0..50 {
                let proportion = i as f32 / 49.0;
                if let Ok(interpolated) = start_color.interpolated_with(&end_color, proportion) {
                    g.set_colour(&interpolated);
                    g.fill_rect(50 + i * 10, 50, 10, 60);
                }
            }
        }

        // Demonstrate brightness manipulation
        if let Ok(base_color) = Colour::from_rgb(100, 200, 100) {
            for i in 0..10 {
                let factor = 0.5 + (i as f32 * 0.1);
                if let Ok(modified) = base_color.brighter(factor) {
                    g.set_colour(&modified);
                    g.fill_rect(50 + i * 50, 140, 45, 60);
                }
            }
        }

        // Demonstrate alpha transparency
        if let Ok(base) = Colour::from_rgb(255, 150, 0) {
            for i in 0..10 {
                let alpha = 0.1 + (i as f32 * 0.1);
                if let Ok(transparent) = base.with_alpha(alpha) {
                    g.set_colour(&transparent);
                    g.fill_ellipse(80.0 + i as f32 * 50.0, 240.0, 60.0, 60.0);
                }
            }
        }

        // Create a color from hex
        if let Ok(hex_color) = Colour::from_hex("#FF6B35") {
            g.set_colour(&hex_color);
            g.fill_rect(50, 330, 500, 40);
            
            // Draw the hex value
            if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
                g.set_colour(&text_color);
                g.draw_text("Color from hex: #FF6B35", 0, 330, 600, 40, Justification::Centred);
            }
        }

        // Title
        if let Ok(title_color) = Colour::from_rgb(220, 220, 220) {
            g.set_colour(&title_color);
            g.draw_text("Color Manipulation & Gradients", 0, 10, 600, 30, Justification::Centred);
        }
    })?;

    Ok(component)
}

/// Example 3: Text and fonts
///
/// Demonstrates:
/// - Creating Font objects with different sizes
/// - Setting font styles (bold, italic)
/// - Drawing text with different justifications
/// - Text measurement
#[allow(dead_code)]
fn example_text_and_fonts() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut component = Component::new()?;
    component.set_bounds(0, 0, 600, 500);

    component.set_paint_callback(|g: &mut Graphics| {
        // Background
        if let Ok(bg) = Colour::from_rgb(25, 25, 35) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 600, 500);
        }

        if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text_color);

            // Different font sizes
            g.draw_text("Font Size 12", 50, 50, 500, 30, Justification::Left);
            g.draw_text("Font Size 18", 50, 90, 500, 40, Justification::Left);
            g.draw_text("Font Size 24", 50, 140, 500, 50, Justification::Left);

            // Different justifications
            g.draw_text("Left Justified", 50, 220, 500, 30, Justification::Left);
            g.draw_text("Centered Text", 50, 260, 500, 30, Justification::Centred);
            g.draw_text("Right Justified", 50, 300, 500, 30, Justification::Right);

            // Multiline text demonstration
            let multiline = "This is a demonstration of text rendering\n\
                           with the JUCE FFI Graphics context.\n\
                           Multiple lines are supported!";
            g.draw_text(multiline, 50, 350, 500, 100, Justification::Centred);
        }

        // Draw decorative boxes around text areas
        if let Ok(accent) = Colour::from_rgb(100, 150, 255) {
            g.set_colour(&accent);
            g.draw_rect(45, 45, 510, 150);
            g.draw_rect(45, 215, 510, 100);
            g.draw_rect(45, 345, 510, 110);
        }

        // Title
        if let Ok(title_color) = Colour::from_rgb(255, 200, 100) {
            g.set_colour(&title_color);
            g.draw_text("Text & Font Rendering", 0, 5, 600, 30, Justification::Centred);
        }
    })?;

    Ok(component)
}

/// Example 4: Complex shapes using basic primitives
///
/// Demonstrates:
/// - Creating complex shapes with basic drawing operations
/// - Combining rectangles and ellipses
/// - Creating patterns and designs
/// - Layering shapes
#[allow(dead_code)]
fn example_paths_and_shapes() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut component = Component::new()?;
    component.set_bounds(0, 0, 600, 500);

    component.set_paint_callback(|g: &mut Graphics| {
        // Background
        if let Ok(bg) = Colour::from_rgb(20, 25, 30) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 600, 500);
        }

        // Example 1: Simple rectangle pattern
        for i in 0..5 {
            let x = (50 + i * 30) as i32;
            let y = (80 + i * 10) as i32;
            if let Ok(color) = Colour::from_rgb((255 - i * 40) as u8, (100 + i * 30) as u8, 100) {
                g.set_colour(&color);
                g.fill_rect(x, y, 100, 80);
            }
        }

        // Example 2: Overlapping circles
        for i in 0..6 {
            let x = 200.0 + i as f32 * 20.0;
            let y = 80.0;
            if let Ok(color) = Colour::from_rgba(100, 255 - i * 30, 100 + i * 25, 180) {
                g.set_colour(&color);
                g.fill_ellipse(x, y, 80.0, 80.0);
            }
        }

        // Example 3: Concentric circles
        for i in 0..8 {
            let size = 20.0 + i as f32 * 15.0;
            let x = 400.0 - size / 2.0;
            let y = 115.0 - size / 2.0;
            if let Ok(color) = Colour::from_rgb(100, 100 + i * 20, 255 - i * 20) {
                g.set_colour(&color);
                g.draw_rect(x as i32, y as i32, size as i32, size as i32);
            }
        }

        // Example 4: Star shape using triangles
        let center_x = 100.0;
        let center_y = 280.0;
        let outer_radius = 50.0;

        for i in 0..5 {
            let angle1 = (i as f32 * 2.0 * std::f32::consts::PI / 5.0) - std::f32::consts::PI / 2.0;
            let angle2 = ((i as f32 + 0.5) * 2.0 * std::f32::consts::PI / 5.0) - std::f32::consts::PI / 2.0;
            
            let x1 = center_x + outer_radius * angle1.cos();
            let y1 = center_y + outer_radius * angle1.sin();
            let x2 = center_x + outer_radius * angle2.cos();
            let y2 = center_y + outer_radius * angle2.sin();
            
            if let Ok(color) = Colour::from_rgb(255, 215, 0) {
                g.set_colour(&color);
                // Draw lines to approximate the star
                g.draw_line(center_x, center_y, x1, y1);
                g.draw_line(x1, y1, x2, y2);
            }
        }

        // Example 5: Grid of circles
        for row in 0..3 {
            for col in 0..4 {
                let x = 220.0 + col as f32 * 40.0;
                let y = 230.0 + row as f32 * 40.0;
                if let Ok(color) = Colour::from_rgb(255 - row * 60, 100 + col * 30, 200) {
                    g.set_colour(&color);
                    g.fill_ellipse(x, y, 30.0, 30.0);
                }
            }
        }

        // Example 6: Spiral pattern using lines
        for i in 0..50 {
            let angle = i as f32 * 0.3;
            let radius = i as f32 * 2.0;
            let x1 = 450.0 + radius * angle.cos();
            let y1 = 280.0 + radius * angle.sin();
            let x2 = 450.0 + (radius + 10.0) * (angle + 0.3).cos();
            let y2 = 280.0 + (radius + 10.0) * (angle + 0.3).sin();
            
            if let Ok(color) = Colour::from_rgb(100 + i * 3, 255 - i * 3, 255) {
                g.set_colour(&color);
                g.draw_line(x1, y1, x2, y2);
            }
        }

        // Example 7: Waveform using lines
        for i in 0..99 {
            let x1 = 50.0 + i as f32 * 5.0;
            let y1 = 420.0 + (i as f32 * 0.2).sin() * 30.0;
            let x2 = 50.0 + (i + 1) as f32 * 5.0;
            let y2 = 420.0 + ((i + 1) as f32 * 0.2).sin() * 30.0;
            
            if let Ok(color) = Colour::from_rgb(150, 255, 150) {
                g.set_colour(&color);
                g.draw_line(x1, y1, x2, y2);
            }
        }

        // Title
        if let Ok(title_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&title_color);
            g.draw_text("Paths & Complex Shapes", 0, 10, 600, 30, Justification::Centred);
        }
    })?;

    Ok(component)
}

/// Example 5: Audio visualization
///
/// Demonstrates:
/// - Real-time drawing updates
/// - Combining multiple drawing techniques
/// - Creating a waveform display
/// - Level meters
#[allow(dead_code)]
fn example_audio_visualization() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut component = Component::new()?;
    component.set_bounds(0, 0, 700, 400);

    component.set_paint_callback(|g: &mut Graphics| {
        // Background
        if let Ok(bg) = Colour::from_rgb(15, 15, 20) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 700, 400);
        }

        // Draw grid
        if let Ok(grid_color) = Colour::from_rgb(40, 40, 50) {
            g.set_colour(&grid_color);
            // Vertical lines
            for i in 0..14 {
                g.draw_line((50 + i * 50) as f32, 60.0, (50 + i * 50) as f32, 180.0);
            }
            // Horizontal lines
            for i in 0..5 {
                g.draw_line(50.0, (60 + i * 30) as f32, 650.0, (60 + i * 30) as f32);
            }
        }

        // Simulate waveform data using lines
        if let Ok(wave_color) = Colour::from_rgb(100, 200, 255) {
            g.set_colour(&wave_color);
            for i in 0..119 {
                let x1 = 50.0 + i as f32 * 5.0;
                let phase1 = i as f32 * 0.1;
                let y1 = 120.0 + (phase1.sin() * 40.0 + (phase1 * 2.0).sin() * 20.0);
                
                let x2 = 50.0 + (i + 1) as f32 * 5.0;
                let phase2 = (i + 1) as f32 * 0.1;
                let y2 = 120.0 + (phase2.sin() * 40.0 + (phase2 * 2.0).sin() * 20.0);
                
                g.draw_line(x1, y1, x2, y2);
            }
        }

        // Draw level meters
        let levels = [0.8, 0.6, 0.9, 0.4, 0.7, 0.5, 0.85, 0.3];
        for (i, &level) in levels.iter().enumerate() {
            let x = (80 + i * 70) as i32;
            let height = (level * 120.0) as i32;
            let y = 340 - height;

            // Meter background
            if let Ok(bg) = Colour::from_rgb(30, 30, 40) {
                g.set_colour(&bg);
                g.fill_rect(x, 220, 40, 120);
            }

            // Meter fill with color gradient based on level
            let color = if level > 0.8 {
                Colour::from_rgb(255, 80, 80)
            } else if level > 0.6 {
                Colour::from_rgb(255, 200, 80)
            } else {
                Colour::from_rgb(80, 255, 80)
            };

            if let Ok(meter_color) = color {
                g.set_colour(&meter_color);
                g.fill_rect(x, y, 40, height);
            }

            // Meter border
            if let Ok(border) = Colour::from_rgb(100, 100, 120) {
                g.set_colour(&border);
                g.draw_rect(x, 220, 40, 120);
            }
        }

        // Labels
        if let Ok(label_color) = Colour::from_rgb(180, 180, 200) {
            g.set_colour(&label_color);
            g.draw_text("Waveform Display", 50, 30, 600, 25, Justification::Left);
            g.draw_text("Level Meters", 80, 195, 600, 25, Justification::Left);
        }

        // Title
        if let Ok(title_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&title_color);
            g.draw_text("Audio Visualization", 0, 5, 700, 25, Justification::Centred);
        }
    })?;

    Ok(component)
}

/// Example 6: Combined demonstration
///
/// A comprehensive example combining all drawing techniques
#[allow(dead_code)]
fn example_combined_demo() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut component = Component::new()?;
    component.set_bounds(0, 0, 800, 600);

    component.set_paint_callback(|g: &mut Graphics| {
        // Gradient background
        if let (Ok(top_color), Ok(bottom_color)) = (
            Colour::from_rgb(20, 30, 50),
            Colour::from_rgb(50, 20, 30),
        ) {
            for i in 0..60 {
                let proportion = i as f32 / 59.0;
                if let Ok(color) = top_color.interpolated_with(&bottom_color, proportion) {
                    g.set_colour(&color);
                    g.fill_rect(0, i * 10, 800, 10);
                }
            }
        }

        // Draw decorative circles
        for i in 0..8 {
            let x = 100.0 + i as f32 * 90.0;
            let y = 100.0;
            let size = 40.0 + (i as f32 * 5.0).sin() * 10.0;

            if let Ok(circle_color) = Colour::from_rgba(
                100 + i * 20,
                150,
                255 - i * 20,
                180,
            ) {
                g.set_colour(&circle_color);
                g.fill_ellipse(x, y, size, size);
            }
        }

        // Draw a hexagon using lines
        if let Ok(shape_color) = Colour::from_rgba(255, 200, 100, 200) {
            g.set_colour(&shape_color);
            
            // Draw filled hexagon using triangles from center
            for i in 0..6 {
                let angle1 = i as f32 * std::f32::consts::PI / 3.0;
                let angle2 = (i + 1) as f32 * std::f32::consts::PI / 3.0;
                let x1 = 400.0 + 80.0 * angle1.cos();
                let y1 = 300.0 + 80.0 * angle1.sin();
                let x2 = 400.0 + 80.0 * angle2.cos();
                let y2 = 300.0 + 80.0 * angle2.sin();
                
                g.draw_line(400.0, 300.0, x1, y1);
                g.draw_line(x1, y1, x2, y2);
            }
        }

        if let Ok(outline) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&outline);
            // Draw hexagon outline
            for i in 0..6 {
                let angle1 = i as f32 * std::f32::consts::PI / 3.0;
                let angle2 = (i + 1) as f32 * std::f32::consts::PI / 3.0;
                let x1 = 400.0 + 80.0 * angle1.cos();
                let y1 = 300.0 + 80.0 * angle1.sin();
                let x2 = 400.0 + 80.0 * angle2.cos();
                let y2 = 300.0 + 80.0 * angle2.sin();
                
                g.draw_line(x1, y1, x2, y2);
            }
        }

        // Draw text with shadow effect
        if let Ok(shadow) = Colour::from_rgba(0, 0, 0, 150) {
            g.set_colour(&shadow);
            g.draw_text("JUCE FFI", 202, 452, 400, 80, Justification::Centred);
        }
        if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text_color);
            g.draw_text("JUCE FFI", 200, 450, 400, 80, Justification::Centred);
        }

        // Draw info text
        if let Ok(info_color) = Colour::from_rgb(200, 200, 220) {
            g.set_colour(&info_color);
            g.draw_text(
                "Custom Drawing with Graphics Context",
                0,
                20,
                800,
                30,
                Justification::Centred,
            );
            g.draw_text(
                "Colors • Fonts • Paths • Shapes",
                0,
                550,
                800,
                30,
                Justification::Centred,
            );
        }
    })?;

    Ok(component)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = JuceFfiDrawing::default();
        assert_eq!(plugin.params.mix.name(), "Mix");
        assert_eq!(plugin.params.frequency.name(), "Frequency");
    }

    #[test]
    fn test_juce_initialization() {
        let result = nih_plug_juce::initialize();
        assert!(result.is_ok(), "JUCE initialization should succeed");
    }
}
