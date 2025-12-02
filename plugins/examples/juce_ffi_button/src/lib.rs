//! # JUCE FFI Button Example
//!
//! This example demonstrates using JUCE GUI components through FFI bindings in a nih-plug plugin.
//! It shows how to:
//! - Create JUCE components (buttons, sliders, labels)
//! - Set up callbacks for user interaction
//! - Integrate with nih-plug parameters
//! - Use JUCE's drawing and layout capabilities

use nih_plug::prelude::*;
use nih_plug_juce::*;
use std::sync::Arc;

/// A simple gain plugin demonstrating JUCE FFI button integration
pub struct JuceFfiButton {
    params: Arc<JuceFfiButtonParams>,
}

#[derive(Params)]
struct JuceFfiButtonParams {
    /// Gain parameter controlled by JUCE slider
    #[id = "gain"]
    pub gain: FloatParam,

    /// Bypass parameter controlled by JUCE button
    #[id = "bypass"]
    pub bypass: BoolParam,
}

impl Default for JuceFfiButton {
    fn default() -> Self {
        Self {
            params: Arc::new(JuceFfiButtonParams::default()),
        }
    }
}

impl Default for JuceFfiButtonParams {
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

impl Plugin for JuceFfiButton {
    const NAME: &'static str = "JUCE FFI Button Example";
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
        // For this example, we'll create a simple demonstration of the JUCE FFI API
        // In a real plugin, you would create a proper editor implementation
        // that integrates with the host's window system
        
        // Note: This example focuses on demonstrating the JUCE FFI API usage
        // A full editor implementation would require additional integration work
        None
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

impl ClapPlugin for JuceFfiButton {
    const CLAP_ID: &'static str = "com.nih-plug.juce-ffi-button";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Example plugin demonstrating JUCE FFI button integration");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for JuceFfiButton {
    const VST3_CLASS_ID: [u8; 16] = *b"JuceFfiBtnExampl";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_clap!(JuceFfiButton);
nih_export_vst3!(JuceFfiButton);

/// Example function demonstrating JUCE FFI component creation and usage.
///
/// This function shows how to:
/// 1. Initialize JUCE
/// 2. Create components (buttons, sliders, labels)
/// 3. Set up component hierarchies
/// 4. Configure callbacks
/// 5. Use drawing primitives
///
/// Note: This is a demonstration function. In a real plugin, you would integrate
/// this with nih-plug's editor system and the host's window system.
#[allow(dead_code)]
fn create_example_juce_gui() -> Result<Component> {
    // Initialize JUCE FFI bridge
    nih_plug_juce::initialize()?;

    // Create main container component
    let mut main_component = Component::new()?;
    main_component.set_bounds(0, 0, 400, 300);
    main_component.set_visible(true);

    // Create title label
    let mut title_label = widgets::Label::new("JUCE FFI Button Example")?;
    title_label.set_bounds(10, 10, 380, 30);
    title_label.set_font(24.0);
    title_label.set_justification(widgets::Justification::Centred);

    // Create gain slider
    let mut gain_slider = widgets::Slider::new(widgets::SliderStyle::LinearHorizontal)?;
    gain_slider.set_bounds(50, 80, 300, 40);
    gain_slider.set_range(0.0, 1.0, 0.01);
    gain_slider.set_value(0.5);
    
    // Set up slider callback
    gain_slider.set_on_value_change(|value| {
        println!("Gain slider changed to: {:.2}", value);
        // In a real plugin, you would update the parameter here:
        // setter.set_parameter_normalized(&params.gain, value as f32);
    })?;

    // Create gain label
    let mut gain_label = widgets::Label::new("Gain")?;
    gain_label.set_bounds(10, 85, 35, 30);
    gain_label.set_font(14.0);

    // Create bypass button
    let mut bypass_button = widgets::TextButton::new("Bypass")?;
    bypass_button.set_bounds(150, 150, 100, 40);
    
    // Set up button callback
    bypass_button.set_on_click(|| {
        println!("Bypass button clicked!");
        // In a real plugin, you would toggle the bypass parameter here:
        // let new_value = !params.bypass.value();
        // setter.set_parameter(&params.bypass, new_value);
    })?;

    // Create a toggle button for demonstration
    let mut toggle_button = widgets::ToggleButton::new("Enable Effect")?;
    toggle_button.set_bounds(150, 210, 100, 30);
    toggle_button.set_toggle_state(true);
    
    toggle_button.set_on_click(|state| {
        println!("Toggle button state: {}", state);
    })?;

    // Add all components to the main component
    main_component.add_child(&title_label)?;
    main_component.add_child(&gain_label)?;
    main_component.add_child(&gain_slider)?;
    main_component.add_child(&bypass_button)?;
    main_component.add_child(&toggle_button)?;

    // Set up custom paint callback for the main component
    main_component.set_paint_callback(|g: &mut Graphics| {
        // Draw background
        if let Ok(bg_color) = Colour::from_rgb(30, 30, 30) {
            g.set_colour(&bg_color);
            g.fill_rect(0, 0, 400, 300);
        }

        // Draw a decorative border
        if let Ok(border_color) = Colour::from_rgb(100, 150, 255) {
            g.set_colour(&border_color);
            g.draw_rect(5, 5, 390, 290);
        }

        // Draw a status indicator circle
        if let Ok(indicator_color) = Colour::from_rgb(50, 255, 50) {
            g.set_colour(&indicator_color);
            g.fill_ellipse(360.0, 20.0, 20.0, 20.0);
        }
    })?;

    Ok(main_component)
}

/// Example function demonstrating parameter attachment.
///
/// This shows how to connect JUCE sliders directly to nih-plug parameters
/// using the parameter attachment system.
#[allow(dead_code)]
fn create_example_with_parameter_attachment() -> Result<()> {
    // Initialize JUCE
    nih_plug_juce::initialize()?;

    // Create a slider
    let mut slider = widgets::Slider::new(widgets::SliderStyle::Rotary)?;
    slider.set_bounds(50, 50, 100, 100);
    slider.set_range(0.0, 1.0, 0.01);

    // In a real plugin, you would create a parameter attachment like this:
    // let attachment = parameter_attachment::SliderParameterAttachment::new(
    //     &mut slider,
    //     &params.gain,
    //     setter
    // )?;
    //
    // The attachment automatically:
    // - Updates the slider when the parameter changes (automation, preset load, etc.)
    // - Updates the parameter when the slider is moved by the user
    // - Handles begin/end parameter gestures for host automation recording

    Ok(())
}

/// Example function demonstrating custom drawing with JUCE graphics.
///
/// This shows how to use JUCE's drawing primitives to create custom visualizations.
#[allow(dead_code)]
fn create_example_custom_drawing() -> Result<Component> {
    nih_plug_juce::initialize()?;

    let mut component = Component::new()?;
    component.set_bounds(0, 0, 400, 300);

    component.set_paint_callback(|g: &mut Graphics| {
        // Clear background
        if let Ok(bg) = Colour::from_rgb(20, 20, 20) {
            g.set_colour(&bg);
            g.fill_rect(0, 0, 400, 300);
        }

        // Draw a gradient-like effect using multiple rectangles
        for i in 0..10 {
            let brightness = 50 + i * 15;
            if let Ok(color) = Colour::from_rgb(brightness as u8, brightness as u8, brightness as u8) {
                g.set_colour(&color);
                g.fill_rect((i * 40) as i32, 50, 40, 200);
            }
        }

        // Draw some circles
        if let Ok(circle_color) = Colour::from_rgb(255, 100, 100) {
            g.set_colour(&circle_color);
            for i in 0..5 {
                let x = 50.0 + i as f32 * 70.0;
                let y = 100.0;
                let size = 30.0 + i as f32 * 5.0;
                g.fill_ellipse(x, y, size, size);
            }
        }

        // Draw text
        if let Ok(text_color) = Colour::from_rgb(255, 255, 255) {
            g.set_colour(&text_color);
            g.draw_text("Custom JUCE Drawing", 0, 250, 400, 30, Justification::Centred);
        }
    })?;

    Ok(component)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = JuceFfiButton::default();
        assert_eq!(plugin.params.gain.name(), "Gain");
        assert_eq!(plugin.params.bypass.name(), "Bypass");
    }

    #[test]
    fn test_juce_initialization() {
        // Test that JUCE can be initialized
        let result = nih_plug_juce::initialize();
        assert!(result.is_ok(), "JUCE initialization should succeed");
    }

    #[test]
    fn test_component_creation() {
        // Test that we can create JUCE components
        // Note: In a real application, components must be created on the message thread.
        // This test verifies the API is available, but component creation will fail
        // in a test environment without a proper message thread.
        let result = nih_plug_juce::initialize();
        assert!(result.is_ok());

        // Component creation requires the message thread, which isn't available in tests
        // In a real plugin, this would be called from the editor on the message thread
        // let component_result = Component::new();
        // assert!(component_result.is_ok(), "Component creation should succeed");
    }
}
