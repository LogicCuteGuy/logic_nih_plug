//! # JUCE GUI Demo
//!
//! This example demonstrates using the ported JUCE GUI components.
//! It shows how to create a simple plugin interface with buttons, sliders, and labels.

use nih_plug::prelude::*;
use nih_plug_gui::components::{Bounds, Component};
use nih_plug_gui::controls::{Button, Label, Slider, SliderOrientation, TextAlignment};
use nih_plug_gui::lookandfeel::{DefaultLookAndFeel, Theme};
use std::sync::Arc;
use crossbeam::atomic::AtomicCell;

/// A simple plugin demonstrating the ported JUCE GUI components
struct JuceGuiDemo {
    params: Arc<GuiDemoParams>,
}

#[derive(Params)]
struct GuiDemoParams {
    /// Gain parameter controlled by slider
    #[id = "gain"]
    pub gain: FloatParam,

    /// Bypass parameter controlled by button
    #[id = "bypass"]
    pub bypass: BoolParam,
}

impl Default for JuceGuiDemo {
    fn default() -> Self {
        Self {
            params: Arc::new(GuiDemoParams::default()),
        }
    }
}

impl Default for GuiDemoParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            bypass: BoolParam::new("Bypass", false),
        }
    }
}

impl Plugin for JuceGuiDemo {
    const NAME: &'static str = "JUCE GUI Demo";
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

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Simple gain processing with bypass
        if self.params.bypass.value() {
            return ProcessStatus::Normal;
        }

        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl JuceGuiDemo {
    // Make a simple editor available via `editor()` below.
    fn make_editor(&self) -> JuceGuiEditor {
        JuceGuiEditor { params: Arc::clone(&self.params), scaling_factor: AtomicCell::new(None) }
    }
}

struct JuceGuiEditor {
    params: Arc<GuiDemoParams>,
    scaling_factor: AtomicCell<Option<f32>>,
}

impl nih_plug::prelude::Editor for JuceGuiEditor {
    fn spawn(&self, _parent: nih_plug::editor::ParentWindowHandle, _context: Arc<dyn nih_plug::prelude::GuiContext>) -> Box<dyn std::any::Any + Send> {
        // Spawn a simple softbuffer window and render a placeholder UI using nih_plug_graphics
        let params = Arc::clone(&self.params);

        // Try GL window first: draw into a Graphics buffer which is uploaded to GL every frame.
        let window = nih_plug_gui::GlWindowBuilder::spawn("JUCE GUI Demo", 400, 300, move |graphics: &mut nih_plug_graphics::Graphics, (w, h)| {
            use nih_plug_graphics::{Color, text::FontSettings, Font};

            // Clear background
            graphics.set_color(Color::rgb(24, 24, 24));
            graphics.clear();

            // Title text (try to load the bundled test font)
            if let Ok(font) = Font::from_bytes(
                include_bytes!("../../../../nih_plug_graphics/tests/test_font.ttf"),
                FontSettings::default(),
            ) {
                graphics.set_color(Color::rgb(220, 220, 220));
                graphics.draw_text("JUCE GUI Demo", (w as i32) / 2, 30, &font, 24.0);
            }

            // Slider track
            graphics.set_color(Color::rgb(150, 150, 150));
            let track_x = 50i32;
            let track_y = 80i32;
            let track_w = (w.saturating_sub(100)) as u32;
            let track_h = 40u32;
            graphics.fill_rect(track_x, track_y, track_w, track_h);

            // Slider thumb (centered demo position)
            graphics.set_color(Color::rgb(100, 100, 200));
            let thumb_x = track_x + (track_w as i32 / 2) - 5;
            let thumb_y = track_y + (track_h as i32 / 2) - 5;
            graphics.fill_rect(thumb_x, thumb_y, 10u32, 10u32);

            // Bypass button
            graphics.set_color(Color::rgb(200, 200, 200));
            let btn_x = (w as i32 / 2) - 50;
            let btn_y = 150i32;
            graphics.fill_rect(btn_x, btn_y, 100u32, 40u32);
            // Button border
            graphics.set_color(Color::rgb(0, 0, 0));
            graphics.draw_line(btn_x, btn_y, btn_x + 100, btn_y);
            graphics.draw_line(btn_x + 100, btn_y, btn_x + 100, btn_y + 40);
            graphics.draw_line(btn_x + 100, btn_y + 40, btn_x, btn_y + 40);
            graphics.draw_line(btn_x, btn_y + 40, btn_x, btn_y);
        });

        // Keep window handle alive in returned handle object. We'll return an object that's
        // Send so the host wrapper can store it.
        struct EditorHandle { window: nih_plug_gui::GlWindow }
        unsafe impl Send for EditorHandle {}

        Box::new(EditorHandle { window })
    }

    fn size(&self) -> (u32, u32) { (400, 300) }

    fn set_scale_factor(&self, factor: f32) -> bool {
        if self.scaling_factor.load().is_some() { return false; }
        self.scaling_factor.store(Some(factor));
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {}
    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {}
    fn param_values_changed(&self) {}
}

impl ClapPlugin for JuceGuiDemo {
    const CLAP_ID: &'static str = "com.nih-plug.juce-gui-demo";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Example plugin demonstrating JUCE GUI components");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for JuceGuiDemo {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceGuiDemoPlugn";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(JuceGuiDemo);
nih_export_vst3!(JuceGuiDemo);

/// Example of how to create GUI components using the ported JUCE modules
///
/// Note: This is a demonstration of the component API. Full GUI integration
/// with nih-plug would require additional editor implementation.
#[allow(dead_code)]
fn create_example_gui() -> Result<Component, nih_plug_gui::GuiError> {
    // Create main container
    let mut main_component = Component::new("main");
    main_component.set_bounds(Bounds::new(0, 0, 400, 300))?;
    main_component.initialize();

    // Create title label
    let mut title_label = Label::new("JUCE GUI Demo");
    title_label.set_bounds(Bounds::new(10, 10, 380, 30))?;
    title_label.set_alignment(TextAlignment::Center);
    title_label.set_font_size(24);

    // Create gain slider
    let mut gain_slider = Slider::new(SliderOrientation::Horizontal);
    gain_slider.set_bounds(Bounds::new(50, 80, 300, 40))?;
    let _ = gain_slider.set_range(0.0, 1.0);
    gain_slider.set_value(0.5);

    // Create gain label
    let mut gain_label = Label::new("Gain");
    gain_label.set_bounds(Bounds::new(10, 85, 35, 30))?;
    gain_label.set_alignment(TextAlignment::Left);

    // Create bypass button
    let mut bypass_button = Button::new("Bypass");
    bypass_button.set_bounds(Bounds::new(150, 150, 100, 40))?;

    // Apply dark theme
    let _look_and_feel = DefaultLookAndFeel::with_theme(Theme::Dark);

    // In a real implementation, you would:
    // 1. Add components to the main component
    // 2. Set up event handlers for user interaction
    // 3. Connect to plugin parameters
    // 4. Implement rendering using nih_plug_graphics

    Ok(main_component)
}
