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

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(self.make_editor()))
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
    fn spawn(&self, _parent: nih_plug::editor::ParentWindowHandle, context: Arc<dyn nih_plug::prelude::GuiContext>) -> Box<dyn std::any::Any + Send> {
        // Spawn a simple softbuffer window and render a placeholder UI using nih_plug_graphics
        let params = Arc::clone(&self.params);

        // Try GL window first: draw into a Graphics buffer which is uploaded to GL every frame.
        // Open a window parented inside the host's provided parent window. This will allow the
        // plugin GUI to be embedded into the host (DAW) instead of creating a separate OS window.
        // Create a small, sharable state object for the editor UI that's safe to send across
        // threads. The actual control objects will be created each frame from that state since
        // the control types in nih_plug_gui use non-Send Rc/RefCell internals.
        use std::sync::Mutex as StdMutex;
        use std::sync::Arc as StdArc;

        #[derive(Debug, Clone)]
        struct EditorState {
            slider_value: f64, // normalized 0..1
            // fixed layout bounds (in logical pixels)
            slider_x: i32,
            slider_y: i32,
            slider_w: u32,
            slider_h: u32,
            btn_x: i32,
            btn_y: i32,
            btn_w: u32,
            btn_h: u32,
            dragging: bool,
            last_mouse: (i32, i32),
        }

        let state = StdArc::new(StdMutex::new(EditorState {
            slider_value: params.gain.modulated_normalized_value() as f64,
            slider_x: 50,
            slider_y: 80,
            slider_w: 300,
            slider_h: 40,
            btn_x: (400 / 2) as i32 - 50,
            btn_y: 150,
            btn_w: 100,
            btn_h: 40,
            dragging: false,
            last_mouse: (0, 0),
        }));

        // Shared params for draw / event callbacks
        let params_for_draw = Arc::clone(&params);
        let params_for_event = Arc::clone(&params);

        let state_for_draw = state.clone();
        let state_for_event = state.clone();

        // Provide both draw and event callbacks, so the editor becomes interactive.
        let window = nih_plug_gui::GlWindowBuilder::new("JUCE GUI Demo", 400, 300).draw_with_event(
            move |graphics: &mut nih_plug_graphics::Graphics, (w, h)| {
                use nih_plug_graphics::{Color, text::FontSettings, Font};

                // Pull current state copy (short-lived lock)
                let snapshot = { state_for_draw.lock().unwrap().clone() };

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
                graphics.fill_rect(snapshot.slider_x, snapshot.slider_y, snapshot.slider_w, snapshot.slider_h);

                // Draw slider thumb by materializing a temporary Slider control
                let mut tmp_slider = Slider::new(SliderOrientation::Horizontal);
                let _ = tmp_slider.set_range(0.0, 1.0);
                let mut tmp_bounds = nih_plug_gui::components::Bounds::new(snapshot.slider_x, snapshot.slider_y, snapshot.slider_w, snapshot.slider_h);
                let _ = tmp_slider.set_bounds(tmp_bounds);
                tmp_slider.set_value(snapshot.slider_value);
                tmp_slider.render(graphics).ok();

                // Bypass button (rendered via a temporary Button)
                graphics.set_color(Color::rgb(200, 200, 200));
                let mut tmp_btn = Button::new("Bypass");
                let _ = tmp_btn.set_bounds(nih_plug_gui::components::Bounds::new(snapshot.btn_x, snapshot.btn_y, snapshot.btn_w, snapshot.btn_h));
                tmp_btn.set_button_state(if params_for_draw.bypass.value() { nih_plug_gui::controls::ButtonState::Pressed } else { nih_plug_gui::controls::ButtonState::Normal });
                tmp_btn.render(graphics).ok();
                // Button border will be drawn inside render

            },
            move |event| {
                    use baseview::Event;

                // Update internal state and convert events to parameter updates
                    // Baseview's events use a separate Mouse variant with specific sub-variants.
                    if let Event::Mouse(mouse_event) = event {
                        match mouse_event {
                            baseview::MouseEvent::CursorMoved { position, .. } => {
                                let (x, y) = (position.x as i32, position.y as i32);
                                let mut s = state_for_event.lock().unwrap();
                                s.last_mouse = (x, y);
                                if s.dragging {
                                    // Convert x into normalized slider value
                                    let rel = (x - s.slider_x) as f64 / (s.slider_w as f64);
                                    let norm = rel.clamp(0.0, 1.0);
                                    s.slider_value = norm;
                                    // Update the plugin parameter (use ParamSetter for host automation)
                                    let setter = nih_plug::prelude::ParamSetter::new(context.as_ref());
                                    setter.set_parameter_normalized(&params_for_event.gain, norm as f32);
                                }
                            }
                            baseview::MouseEvent::ButtonPressed { button, .. } => {
                                if button == baseview::MouseButton::Left {
                                    let mut s = state_for_event.lock().unwrap();
                                    let (mx, my) = s.last_mouse;
                                    // Check button hit
                                    if mx >= s.btn_x && mx < s.btn_x + s.btn_w as i32 && my >= s.btn_y && my < s.btn_y + s.btn_h as i32 {
                                        // toggle bypass
                                        let new = !params_for_event.bypass.value();
                                        let setter = nih_plug::prelude::ParamSetter::new(context.as_ref());
                                        setter.set_parameter(&params_for_event.bypass, new);
                                    }

                                    // check slider hit -> start dragging
                                    if mx >= s.slider_x && mx < s.slider_x + s.slider_w as i32 && my >= s.slider_y && my < s.slider_y + s.slider_h as i32 {
                                        s.dragging = true;
                                        // set immediate value
                                        let rel = (mx - s.slider_x) as f64 / (s.slider_w as f64);
                                        let norm = rel.clamp(0.0, 1.0);
                                        s.slider_value = norm;
                                        let setter = nih_plug::prelude::ParamSetter::new(context.as_ref());
                                        setter.begin_set_parameter(&params_for_event.gain);
                                        setter.set_parameter_normalized(&params_for_event.gain, norm as f32);
                                    }
                                }
                            }
                            baseview::MouseEvent::ButtonReleased { button, .. } => {
                                if button == baseview::MouseButton::Left {
                                    let mut s = state_for_event.lock().unwrap();
                                    if s.dragging {
                                        // finalize gesture
                                        let setter = nih_plug::prelude::ParamSetter::new(context.as_ref());
                                        setter.end_set_parameter(&params_for_event.gain);
                                    }
                                    s.dragging = false;
                                }
                            }
                            _ => {}
                        }
                    }

                // We return Ignored; the GL handler still handles the resize case locally.
                baseview::EventStatus::Ignored
            }
        );

        // Keep window handle alive in returned handle object. We'll return an object that's
        // Send so the host wrapper can store it.
        struct EditorHandle { window: baseview::WindowHandle }
        unsafe impl Send for EditorHandle {}

        // Use the parent provided by the host so this editor gets embedded. If the host does not
        // support parented windows then baseview will typically fail; falling back to a standalone
        // window via `spawn` could be implemented here if desired.
        let _window_handle = window.open_parented(&_parent);

        Box::new(EditorHandle { window: _window_handle })
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
