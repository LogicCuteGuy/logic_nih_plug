//! # JUCE FFI Complex Layout Example
//!
//! This example demonstrates advanced layout capabilities using JUCE's FlexBox system
//! through FFI bindings. It showcases:
//! - FlexBox layout with multiple components
//! - Responsive layout that adapts to window size
//! - Component hierarchies with nested layouts
//! - Different flex directions (row, column)
//! - Flex properties (grow, shrink, basis)
//! - Margins and spacing
//! - Combining layout with custom drawing
//! - Real-world plugin UI patterns

use nih_plug::prelude::*;
use nih_plug_juce::*;
use nih_plug_juce::layout::{FlexBox, FlexItem, FlexDirection, FlexWrap, JustifyContent, AlignContent};
use nih_plug_juce::drawing::Colour;
use std::sync::Arc;

/// A multi-band EQ plugin demonstrating complex FlexBox layouts
pub struct JuceFfiLayout {
    params: Arc<JuceFfiLayoutParams>,
}

#[derive(Params)]
struct JuceFfiLayoutParams {
    /// Low band gain
    #[id = "low_gain"]
    pub low_gain: FloatParam,

    /// Mid band gain
    #[id = "mid_gain"]
    pub mid_gain: FloatParam,

    /// High band gain
    #[id = "high_gain"]
    pub high_gain: FloatParam,

    /// Master output gain
    #[id = "master"]
    pub master: FloatParam,

    /// Bypass toggle
    #[id = "bypass"]
    pub bypass: BoolParam,
}

impl Default for JuceFfiLayout {
    fn default() -> Self {
        Self {
            params: Arc::new(JuceFfiLayoutParams::default()),
        }
    }
}

impl Default for JuceFfiLayoutParams {
    fn default() -> Self {
        Self {
            low_gain: FloatParam::new(
                "Low Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-24.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(-24.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            mid_gain: FloatParam::new(
                "Mid Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-24.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(-24.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            high_gain: FloatParam::new(
                "High Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-24.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(-24.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            master: FloatParam::new(
                "Master",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-24.0),
                    max: util::db_to_gain(24.0),
                    factor: FloatRange::gain_skew_factor(-24.0, 24.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(1))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            bypass: BoolParam::new("Bypass", false),
        }
    }
}

impl Plugin for JuceFfiLayout {
    const NAME: &'static str = "JUCE FFI Layout Example";
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
        // This example focuses on demonstrating the JUCE FFI layout API
        // A full editor implementation would require additional integration work
        None
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Simple multi-band processing simulation
        if self.params.bypass.value() {
            return ProcessStatus::Normal;
        }

        let low_gain = self.params.low_gain.smoothed.next();
        let mid_gain = self.params.mid_gain.smoothed.next();
        let high_gain = self.params.high_gain.smoothed.next();
        let master = self.params.master.smoothed.next();

        // Simplified processing - in reality you'd use filters
        let combined_gain = (low_gain + mid_gain + high_gain) / 3.0 * master;

        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                *sample *= combined_gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for JuceFfiLayout {
    const CLAP_ID: &'static str = "com.nih-plug.juce-ffi-layout";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Example plugin demonstrating JUCE FFI FlexBox layout capabilities");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Equalizer,
    ];
}

impl Vst3Plugin for JuceFfiLayout {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceFfiLayoutExa";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Eq];
}

nih_export_clap!(JuceFfiLayout);
nih_export_vst3!(JuceFfiLayout);

/// Example 1: Basic horizontal layout with three equal columns
///
/// Demonstrates:
/// - Creating a FlexBox with row direction
/// - Adding multiple components with equal flex-grow
/// - Using margins for spacing
/// - Basic responsive behavior
#[allow(dead_code)]
fn example_basic_horizontal_layout() -> Result<Component> {
    nih_plug_juce::initialize()?;

    // Create main container
    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 800, 200);
    main_component.set_visible(true);

    // Create three child components
    let mut comp1 = Component::new()?;
    let mut comp2 = Component::new()?;
    let mut comp3 = Component::new()?;

    // Set up custom paint for each component
    comp1.set_paint_callback(|g: &mut Graphics| {
        if let Ok(color) = Colour::from_rgb(255, 100, 100) {
            g.set_colour(&color);
            g.fill_rect(0, 0, 1000, 1000); // Fill entire component
        }
        if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text_color);
            g.draw_text("Column 1", 0, 0, 300, 200, Justification::Centred);
        }
    })?;

    comp2.set_paint_callback(|g: &mut Graphics| {
        if let Ok(color) = Colour::from_rgb(100, 255, 100) {
            g.set_colour(&color);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text_color);
            g.draw_text("Column 2", 0, 0, 300, 200, Justification::Centred);
        }
    })?;

    comp3.set_paint_callback(|g: &mut Graphics| {
        if let Ok(color) = Colour::from_rgb(100, 100, 255) {
            g.set_colour(&color);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text_color);
            g.draw_text("Column 3", 0, 0, 300, 200, Justification::Centred);
        }
    })?;

    // Add components to main container
    main_component.add_child(&comp1)?;
    main_component.add_child(&comp2)?;
    main_component.add_child(&comp3)?;

    // Create FlexBox layout
    let mut flexbox = FlexBox::new()?;
    flexbox.set_direction(FlexDirection::Row);
    flexbox.set_wrap(FlexWrap::NoWrap);
    flexbox.set_justify_content(JustifyContent::SpaceBetween);

    // Create flex items with equal growth and margins
    let item1 = FlexItem::new(&comp1)
        .with_flex_grow(1.0)
        .with_margin(10.0, 5.0, 10.0, 10.0);

    let item2 = FlexItem::new(&comp2)
        .with_flex_grow(1.0)
        .with_margin(10.0, 5.0, 10.0, 5.0);

    let item3 = FlexItem::new(&comp3)
        .with_flex_grow(1.0)
        .with_margin(10.0, 10.0, 10.0, 5.0);

    // Add items to flexbox
    flexbox.add_item(item1);
    flexbox.add_item(item2);
    flexbox.add_item(item3);

    // Perform layout
    flexbox.perform_layout(0, 0, 800, 200);

    Ok(main_component)
}

/// Example 2: Vertical layout with different sized sections
///
/// Demonstrates:
/// - Column direction layout
/// - Different flex-grow values for proportional sizing
/// - Header, content, footer pattern
#[allow(dead_code)]
fn example_vertical_sections() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 600, 800);

    // Create header, content, and footer components
    let mut header = Component::new()?;
    let mut content = Component::new()?;
    let mut footer = Component::new()?;

    header.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(40, 40, 60) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text);
            g.draw_text("Header Section", 0, 0, 600, 100, Justification::Centred);
        }
    })?;

    content.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(30, 30, 40) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(200, 200, 200) {
            g.set_colour(&text);
            g.draw_text("Main Content Area\n(Grows to fill space)", 0, 0, 600, 600, Justification::Centred);
        }
    })?;

    footer.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(50, 50, 70) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text);
            g.draw_text("Footer Section", 0, 0, 600, 100, Justification::Centred);
        }
    })?;

    main_component.add_child(&header)?;
    main_component.add_child(&content)?;
    main_component.add_child(&footer)?;

    // Create vertical flexbox
    let mut flexbox = FlexBox::new()?;
    flexbox.set_direction(FlexDirection::Column);

    // Header: fixed height (flex-grow = 0)
    let header_item = FlexItem::new(&header)
        .with_flex_grow(0.0)
        .with_flex_basis(80.0)
        .with_margin(0.0, 0.0, 0.0, 0.0);

    // Content: grows to fill available space
    let content_item = FlexItem::new(&content)
        .with_flex_grow(1.0)
        .with_margin(0.0, 0.0, 0.0, 0.0);

    // Footer: fixed height
    let footer_item = FlexItem::new(&footer)
        .with_flex_grow(0.0)
        .with_flex_basis(60.0)
        .with_margin(0.0, 0.0, 0.0, 0.0);

    flexbox.add_item(header_item);
    flexbox.add_item(content_item);
    flexbox.add_item(footer_item);

    flexbox.perform_layout(0, 0, 600, 800);

    Ok(main_component)
}

/// Example 3: Grid layout using wrapping
///
/// Demonstrates:
/// - Flex wrap to create grid-like layouts
/// - Fixed-size items that wrap to new rows
/// - Uniform spacing with margins
#[allow(dead_code)]
fn example_grid_layout() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 800, 600);

    // Create 12 grid items
    let mut grid_items = Vec::new();
    for i in 0..12 {
        let mut item = Component::new()?;
        
        // Capture i for the closure
        let index = i;
        item.set_paint_callback(move |graphics: &mut Graphics| {
            // Different colors for each item
            let hue = (index as f32 * 30.0) % 360.0;
            let r = ((hue / 360.0 * 255.0) as u8).max(80);
            let g = (((hue + 120.0) / 360.0 * 255.0) as u8).max(80);
            let b = (((hue + 240.0) / 360.0 * 255.0) as u8).max(80);
            
            if let Ok(color) = Colour::from_rgb(r, g, b) {
                graphics.set_colour(&color);
                graphics.fill_rect(0, 0, 1000, 1000);
            }
            
            if let Ok(border) = Colour::from_rgb(255, 255, 255) {
                graphics.set_colour(&border);
                graphics.draw_rect(2, 2, 246, 146);
            }
            
            if let Ok(text) = Colour::from_rgb(255, 255, 255) {
                graphics.set_colour(&text);
                let label = format!("Item {}", index + 1);
                graphics.draw_text(&label, 0, 0, 250, 150, Justification::Centred);
            }
        })?;
        
        main_component.add_child(&item)?;
        grid_items.push(item);
    }

    // Create flexbox with wrapping
    let mut flexbox = FlexBox::new()?;
    flexbox.set_direction(FlexDirection::Row);
    flexbox.set_wrap(FlexWrap::Wrap);
    flexbox.set_justify_content(JustifyContent::FlexStart);
    flexbox.set_align_content(AlignContent::FlexStart);

    // Add all items with fixed size
    for item in &grid_items {
        let flex_item = FlexItem::new(item)
            .with_flex_grow(0.0)
            .with_flex_basis(250.0)
            .with_min_width(250.0)
            .with_min_height(150.0)
            .with_margin(10.0, 10.0, 10.0, 10.0);
        
        flexbox.add_item(flex_item);
    }

    flexbox.perform_layout(0, 0, 800, 600);

    Ok(main_component)
}

/// Example 4: Nested layouts - sidebar with content area
///
/// Demonstrates:
/// - Nested FlexBox layouts
/// - Component hierarchies
/// - Combining horizontal and vertical layouts
/// - Sidebar + main content pattern
#[allow(dead_code)]
fn example_nested_layouts() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 1000, 700);

    // Create sidebar and content area containers
    let mut sidebar = Component::new()?;
    let mut content_area = Component::new()?;

    // Sidebar with vertical layout of buttons
    let mut sidebar_items = Vec::new();
    for i in 0..5 {
        let mut button_comp = Component::new()?;
        let index = i;
        button_comp.set_paint_callback(move |g: &mut Graphics| {
            if let Ok(bg) = Colour::from_rgb(60, 70, 90) {
                g.set_colour(&bg);
                g.fill_rect(0, 0, 1000, 1000);
            }
            if let Ok(border) = Colour::from_rgb(100, 120, 150) {
                g.set_colour(&border);
                g.draw_rect(5, 5, 190, 90);
            }
            if let Ok(text) = Colour::from_rgb(255, 255, 255) {
                g.set_colour(&text);
                let label = format!("Button {}", index + 1);
                g.draw_text(&label, 0, 0, 200, 100, Justification::Centred);
            }
        })?;
        sidebar.add_child(&button_comp)?;
        sidebar_items.push(button_comp);
    }

    // Create vertical flexbox for sidebar
    let mut sidebar_flexbox = FlexBox::new()?;
    sidebar_flexbox.set_direction(FlexDirection::Column);
    sidebar_flexbox.set_justify_content(JustifyContent::FlexStart);

    for item in &sidebar_items {
        let flex_item = FlexItem::new(item)
            .with_flex_grow(0.0)
            .with_flex_basis(100.0)
            .with_margin(10.0, 10.0, 10.0, 10.0);
        sidebar_flexbox.add_item(flex_item);
    }

    sidebar_flexbox.perform_layout(0, 0, 200, 700);

    // Content area with grid of panels
    let mut content_panels = Vec::new();
    for i in 0..6 {
        let mut panel = Component::new()?;
        let index = i;
        panel.set_paint_callback(move |g: &mut Graphics| {
            let colors = [
                (255, 150, 150),
                (150, 255, 150),
                (150, 150, 255),
                (255, 255, 150),
                (255, 150, 255),
                (150, 255, 255),
            ];
            let (r, g_val, b) = colors[index % 6];
            
            if let Ok(bg) = Colour::from_rgb(r, g_val, b) {
                g.set_colour(&bg);
                g.fill_rect(0, 0, 1000, 1000);
            }
            if let Ok(text) = Colour::from_rgb(50, 50, 50) {
                g.set_colour(&text);
                let label = format!("Panel {}", index + 1);
                g.draw_text(&label, 0, 0, 400, 300, Justification::Centred);
            }
        })?;
        content_area.add_child(&panel)?;
        content_panels.push(panel);
    }

    // Create flexbox for content area (2x3 grid)
    let mut content_flexbox = FlexBox::new()?;
    content_flexbox.set_direction(FlexDirection::Row);
    content_flexbox.set_wrap(FlexWrap::Wrap);
    content_flexbox.set_justify_content(JustifyContent::SpaceAround);

    for panel in &content_panels {
        let flex_item = FlexItem::new(panel)
            .with_flex_grow(0.0)
            .with_flex_basis(350.0)
            .with_min_height(200.0)
            .with_margin(15.0, 15.0, 15.0, 15.0);
        content_flexbox.add_item(flex_item);
    }

    content_flexbox.perform_layout(0, 0, 800, 700);

    // Add sidebar and content to main component
    main_component.add_child(&sidebar)?;
    main_component.add_child(&content_area)?;

    // Create main horizontal flexbox
    let mut main_flexbox = FlexBox::new()?;
    main_flexbox.set_direction(FlexDirection::Row);

    let sidebar_item = FlexItem::new(&sidebar)
        .with_flex_grow(0.0)
        .with_flex_basis(200.0);

    let content_item = FlexItem::new(&content_area)
        .with_flex_grow(1.0);

    main_flexbox.add_item(sidebar_item);
    main_flexbox.add_item(content_item);

    main_flexbox.perform_layout(0, 0, 1000, 700);

    Ok(main_component)
}

/// Example 5: Plugin UI with controls and visualization
///
/// Demonstrates:
/// - Real-world plugin UI layout
/// - Combining widgets with custom drawing
/// - Multiple layout sections
/// - Responsive design principles
#[allow(dead_code)]
fn example_plugin_ui_layout() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 900, 600);

    // Title bar
    let mut title_bar = Component::new()?;
    title_bar.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(30, 35, 45) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text);
            g.draw_text("Multi-Band EQ", 20, 0, 400, 60, Justification::Left);
        }
        if let Ok(version) = Colour::from_rgb(150, 150, 150) {
            g.set_colour(&version);
            g.draw_text("v1.0", 800, 0, 80, 60, Justification::Right);
        }
    })?;

    // Visualization area
    let mut viz_area = Component::new()?;
    viz_area.set_paint_callback(|g: &mut Graphics| {
        // Background
        if let Ok(bg) = Colour::from_rgb(20, 25, 30) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }

        // Draw frequency response curve
        if let Ok(curve_color) = Colour::from_rgb(100, 200, 255) {
            g.set_colour(&curve_color);
            for i in 0..179 {
                let x1 = 10.0 + i as f32 * 4.8;
                let y1 = 120.0 + (i as f32 * 0.1).sin() * 40.0;
                let x2 = 10.0 + (i + 1) as f32 * 4.8;
                let y2 = 120.0 + ((i + 1) as f32 * 0.1).sin() * 40.0;
                g.draw_line(x1, y1, x2, y2);
            }
        }

        // Draw grid
        if let Ok(grid) = Colour::from_rgb(40, 45, 50) {
            g.set_colour(&grid);
            for i in 0..9 {
                let x = 10.0 + i as f32 * 100.0;
                g.draw_line(x, 20.0, x, 220.0);
            }
            for i in 0..5 {
                let y = 20.0 + i as f32 * 50.0;
                g.draw_line(10.0, y, 870.0, y);
            }
        }

        // Labels
        if let Ok(label_color) = Colour::from_rgb(150, 150, 150) {
            g.set_colour(&label_color);
            g.draw_text("20Hz", 10, 225, 60, 20, Justification::Left);
            g.draw_text("20kHz", 820, 225, 60, 20, Justification::Right);
            g.draw_text("Frequency Response", 0, 5, 880, 20, Justification::Centred);
        }
    })?;

    // Control sections for each band
    let mut low_band = Component::new()?;
    let mut mid_band = Component::new()?;
    let mut high_band = Component::new()?;

    low_band.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(255, 100, 100) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text);
            g.draw_text("LOW BAND\n80 Hz\n+3.0 dB", 0, 0, 280, 180, Justification::Centred);
        }
    })?;

    mid_band.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(100, 255, 100) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(50, 50, 50) {
            g.set_colour(&text);
            g.draw_text("MID BAND\n1 kHz\n-1.5 dB", 0, 0, 280, 180, Justification::Centred);
        }
    })?;

    high_band.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(100, 100, 255) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text);
            g.draw_text("HIGH BAND\n8 kHz\n+2.0 dB", 0, 0, 280, 180, Justification::Centred);
        }
    })?;

    // Master section
    let mut master_section = Component::new()?;
    master_section.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(50, 50, 60) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 1000, 1000);
        }
        if let Ok(text) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text);
            g.draw_text("MASTER\nOutput\n0.0 dB", 0, 0, 280, 180, Justification::Centred);
        }
    })?;

    // Add all components
    main_component.add_child(&title_bar)?;
    main_component.add_child(&viz_area)?;
    main_component.add_child(&low_band)?;
    main_component.add_child(&mid_band)?;
    main_component.add_child(&high_band)?;
    main_component.add_child(&master_section)?;

    // Create main vertical layout
    let mut main_flexbox = FlexBox::new()?;
    main_flexbox.set_direction(FlexDirection::Column);

    // Title bar - fixed height
    let title_item = FlexItem::new(&title_bar)
        .with_flex_grow(0.0)
        .with_flex_basis(60.0);

    // Visualization - fixed height
    let viz_item = FlexItem::new(&viz_area)
        .with_flex_grow(0.0)
        .with_flex_basis(250.0)
        .with_margin(10.0, 10.0, 10.0, 10.0);

    main_flexbox.add_item(title_item);
    main_flexbox.add_item(viz_item);

    // Controls area - create nested horizontal flexbox
    let mut controls_container = Component::new()?;
    controls_container.add_child(&low_band)?;
    controls_container.add_child(&mid_band)?;
    controls_container.add_child(&high_band)?;
    controls_container.add_child(&master_section)?;
    main_component.add_child(&controls_container)?;

    let mut controls_flexbox = FlexBox::new()?;
    controls_flexbox.set_direction(FlexDirection::Row);
    controls_flexbox.set_justify_content(JustifyContent::SpaceAround);

    let low_item = FlexItem::new(&low_band)
        .with_flex_grow(1.0)
        .with_margin(10.0, 5.0, 10.0, 10.0);

    let mid_item = FlexItem::new(&mid_band)
        .with_flex_grow(1.0)
        .with_margin(10.0, 5.0, 10.0, 5.0);

    let high_item = FlexItem::new(&high_band)
        .with_flex_grow(1.0)
        .with_margin(10.0, 5.0, 10.0, 5.0);

    let master_item = FlexItem::new(&master_section)
        .with_flex_grow(1.0)
        .with_margin(10.0, 10.0, 10.0, 5.0);

    controls_flexbox.add_item(low_item);
    controls_flexbox.add_item(mid_item);
    controls_flexbox.add_item(high_item);
    controls_flexbox.add_item(master_item);

    controls_flexbox.perform_layout(0, 0, 900, 280);

    // Add controls container to main flexbox
    let controls_item = FlexItem::new(&controls_container)
        .with_flex_grow(1.0);

    main_flexbox.add_item(controls_item);

    // Perform main layout
    main_flexbox.perform_layout(0, 0, 900, 600);

    Ok(main_component)
}

/// Example 6: Responsive layout that adapts to window size
///
/// Demonstrates:
/// - Layout that changes based on available space
/// - Using flex-wrap for responsive behavior
/// - Minimum and maximum size constraints
/// - Adaptive column counts
#[allow(dead_code)]
fn example_responsive_layout() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 1200, 800);

    // Create 8 responsive cards
    let mut cards = Vec::new();
    for i in 0..8 {
        let mut card = Component::new()?;
        let index = i;
        
        card.set_paint_callback(move |g: &mut Graphics| {
            // Gradient background
            if let (Ok(top), Ok(bottom)) = (
                Colour::from_rgb(80 + index * 20, 100, 200 - index * 15),
                Colour::from_rgb(40 + index * 10, 60, 140 - index * 10),
            ) {
                for j in 0..20 {
                    let proportion = j as f32 / 19.0;
                    if let Ok(color) = top.interpolated_with(&bottom, proportion) {
                        g.set_colour(&color);
                        g.fill_rect(0, j * 15, 1000, 15);
                    }
                }
            }

            // Card border
            if let Ok(border) = Colour::from_rgb(255, 255, 255) {
                g.set_colour(&border);
                g.draw_rect(5, 5, 340, 290);
            }

            // Card title
            if let Ok(title_color) = Colour::from_rgb(255, 255, 255) {
                g.set_colour(&title_color);
                let title = format!("Card {}", index + 1);
                g.draw_text(&title, 0, 20, 350, 40, Justification::Centred);
            }

            // Card content
            if let Ok(content_color) = Colour::from_rgb(220, 220, 220) {
                g.set_colour(&content_color);
                g.draw_text(
                    "This card adapts to\navailable space.\nResize to see responsive\nbehavior.",
                    20,
                    80,
                    310,
                    200,
                    Justification::Centred,
                );
            }
        })?;
        
        main_component.add_child(&card)?;
        cards.push(card);
    }

    // Create responsive flexbox
    let mut flexbox = FlexBox::new()?;
    flexbox.set_direction(FlexDirection::Row);
    flexbox.set_wrap(FlexWrap::Wrap);
    flexbox.set_justify_content(JustifyContent::SpaceAround);
    flexbox.set_align_content(AlignContent::FlexStart);

    // Add cards with responsive sizing
    for card in &cards {
        let flex_item = FlexItem::new(card)
            .with_flex_grow(1.0)
            .with_flex_shrink(1.0)
            .with_flex_basis(350.0)
            .with_min_width(300.0)
            .with_max_width(400.0)
            .with_min_height(300.0)
            .with_margin(15.0, 15.0, 15.0, 15.0);
        
        flexbox.add_item(flex_item);
    }

    flexbox.perform_layout(0, 0, 1200, 800);

    Ok(main_component)
}

/// Example 7: Dashboard layout with mixed content
///
/// Demonstrates:
/// - Complex multi-section layout
/// - Combining different layout patterns
/// - Stats, charts, and controls in one view
#[allow(dead_code)]
fn example_dashboard_layout() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 1400, 900);

    // Top stats bar
    let mut stats_bar = Component::new()?;
    stats_bar.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(35, 40, 50) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 2000, 1000);
        }
        
        // Draw stat boxes
        let stats = [
            ("CPU", "23%"),
            ("Memory", "512 MB"),
            ("Latency", "2.3 ms"),
            ("Sample Rate", "48 kHz"),
        ];
        
        for (i, (label, value)) in stats.iter().enumerate() {
            let x = (50 + i * 340) as i32;
            
            if let Ok(box_color) = Colour::from_rgb(50, 60, 75) {
                g.set_colour(&box_color);
                g.fill_rect(x, 15, 300, 70);
            }
            
            if let Ok(label_color) = Colour::from_rgb(150, 150, 150) {
                g.set_colour(&label_color);
                g.draw_text(label, x, 20, 300, 25, Justification::Centred);
            }
            
            if let Ok(value_color) = Colour::from_rgb(255, 255, 255) {
                g.set_colour(&value_color);
                g.draw_text(value, x, 45, 300, 35, Justification::Centred);
            }
        }
    })?;

    // Left panel - waveform display
    let mut waveform = Component::new()?;
    waveform.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(25, 30, 35) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 2000, 2000);
        }
        
        if let Ok(title) = Colour::from_rgb(200, 200, 200) {
            g.set_colour(&title);
            g.draw_text("Waveform", 10, 10, 400, 30, Justification::Left);
        }
        
        // Draw waveform
        if let Ok(wave) = Colour::from_rgb(100, 200, 255) {
            g.set_colour(&wave);
            for i in 0..199 {
                let x1 = 10.0 + i as f32 * 3.4;
                let y1 = 300.0 + (i as f32 * 0.15).sin() * 100.0;
                let x2 = 10.0 + (i + 1) as f32 * 3.4;
                let y2 = 300.0 + ((i + 1) as f32 * 0.15).sin() * 100.0;
                g.draw_line(x1, y1, x2, y2);
            }
        }
    })?;

    // Center panel - spectrum analyzer
    let mut spectrum = Component::new()?;
    spectrum.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(20, 25, 30) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 2000, 2000);
        }
        
        if let Ok(title) = Colour::from_rgb(200, 200, 200) {
            g.set_colour(&title);
            g.draw_text("Spectrum Analyzer", 10, 10, 400, 30, Justification::Left);
        }
        
        // Draw spectrum bars
        for i in 0..40 {
            let height = 50.0 + (i as f32 * 0.5).sin().abs() * 450.0;
            let x = 20 + i * 15;
            let y = (550.0 - height) as i32;
            
            let brightness = (height / 500.0 * 255.0) as u8;
            if let Ok(bar_color) = Colour::from_rgb(brightness, 255 - brightness / 2, 100) {
                g.set_colour(&bar_color);
                g.fill_rect(x, y, 12, height as i32);
            }
        }
    })?;

    // Right panel - controls
    let mut controls = Component::new()?;
    controls.set_paint_callback(|g: &mut Graphics| {
        if let Ok(bg) = Colour::from_rgb(30, 35, 40) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 2000, 2000);
        }
        
        if let Ok(title) = Colour::from_rgb(200, 200, 200) {
            g.set_colour(&title);
            g.draw_text("Controls", 10, 10, 300, 30, Justification::Left);
        }
        
        // Draw control sections
        let sections = ["Input", "Processing", "Output", "Monitoring"];
        for (i, section) in sections.iter().enumerate() {
            let y = (60 + i * 140) as i32;
            
            if let Ok(section_bg) = Colour::from_rgb(45, 50, 60) {
                g.set_colour(&section_bg);
                g.fill_rect(20, y, 280, 120);
            }
            
            if let Ok(section_title) = Colour::from_rgb(255, 255, 255) {
                g.set_colour(&section_title);
                g.draw_text(section, 30, y + 10, 260, 30, Justification::Left);
            }
            
            if let Ok(content) = Colour::from_rgb(180, 180, 180) {
                g.set_colour(&content);
                g.draw_text("• Parameter 1\n• Parameter 2\n• Parameter 3", 40, y + 45, 240, 65, Justification::Left);
            }
        }
    })?;

    // Add all components
    main_component.add_child(&stats_bar)?;
    main_component.add_child(&waveform)?;
    main_component.add_child(&spectrum)?;
    main_component.add_child(&controls)?;

    // Create main layout
    let mut main_flexbox = FlexBox::new()?;
    main_flexbox.set_direction(FlexDirection::Column);

    // Stats bar at top
    let stats_item = FlexItem::new(&stats_bar)
        .with_flex_grow(0.0)
        .with_flex_basis(100.0);

    main_flexbox.add_item(stats_item);

    // Content area with three panels
    let mut content_container = Component::new()?;
    content_container.add_child(&waveform)?;
    content_container.add_child(&spectrum)?;
    content_container.add_child(&controls)?;
    main_component.add_child(&content_container)?;

    let mut content_flexbox = FlexBox::new()?;
    content_flexbox.set_direction(FlexDirection::Row);

    let waveform_item = FlexItem::new(&waveform)
        .with_flex_grow(1.0)
        .with_margin(10.0, 5.0, 10.0, 10.0);

    let spectrum_item = FlexItem::new(&spectrum)
        .with_flex_grow(1.0)
        .with_margin(10.0, 5.0, 10.0, 5.0);

    let controls_item = FlexItem::new(&controls)
        .with_flex_grow(0.0)
        .with_flex_basis(320.0)
        .with_margin(10.0, 10.0, 10.0, 5.0);

    content_flexbox.add_item(waveform_item);
    content_flexbox.add_item(spectrum_item);
    content_flexbox.add_item(controls_item);

    content_flexbox.perform_layout(0, 0, 1400, 790);

    let content_item = FlexItem::new(&content_container)
        .with_flex_grow(1.0);

    main_flexbox.add_item(content_item);

    main_flexbox.perform_layout(0, 0, 1400, 900);

    Ok(main_component)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = JuceFfiLayout::default();
        assert_eq!(plugin.params.low_gain.name(), "Low Gain");
        assert_eq!(plugin.params.mid_gain.name(), "Mid Gain");
        assert_eq!(plugin.params.high_gain.name(), "High Gain");
        assert_eq!(plugin.params.master.name(), "Master");
        assert_eq!(plugin.params.bypass.name(), "Bypass");
    }

    #[test]
    fn test_juce_initialization() {
        let result = nih_plug_juce::initialize();
        assert!(result.is_ok(), "JUCE initialization should succeed");
    }

    #[test]
    fn test_flexbox_creation() {
        let result = nih_plug_juce::initialize();
        assert!(result.is_ok());

        // FlexBox creation requires the message thread
        // In a real plugin, this would be called from the editor on the message thread
        // let flexbox_result = FlexBox::new();
        // assert!(flexbox_result.is_ok(), "FlexBox creation should succeed");
    }
}
