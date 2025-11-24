//! # JUCE GUI Demo
//!
//! This example demonstrates using the ported JUCE GUI components.
//! It shows how to create a simple plugin interface with buttons, sliders, and labels.

use nih_plug::prelude::*;
use nih_plug_gui::components::{Bounds, Component, ComponentId};
use nih_plug_gui::controls::{Button, Label, Slider, SliderOrientation, TextAlignment};
use nih_plug_gui::lookandfeel::{DefaultLookAndFeel, Theme};
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use glow::HasContext as _;
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
        // We'll keep the component instances persistent inside the window handler below

        // Open a custom parented baseview window with a persistent GL-backed handler
        // that keeps the JUCE-like component tree in the window thread. This is closer to
        // how a real JUCE editor would keep components around and route events to them.
        struct ParentWindowHandleAdapter(nih_plug::editor::ParentWindowHandle);

        unsafe impl raw_window_handle::HasRawWindowHandle for ParentWindowHandleAdapter {
            fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
                match self.0 {
                    ParentWindowHandle::X11Window(window) => {
                        let mut handle = raw_window_handle::XcbWindowHandle::empty();
                        handle.window = window;
                        raw_window_handle::RawWindowHandle::Xcb(handle)
                    }
                    ParentWindowHandle::AppKitNsView(ns_view) => {
                        let mut handle = raw_window_handle::AppKitWindowHandle::empty();
                        handle.ns_view = ns_view;
                        raw_window_handle::RawWindowHandle::AppKit(handle)
                    }
                    ParentWindowHandle::Win32Hwnd(hwnd) => {
                        let mut handle = raw_window_handle::Win32WindowHandle::empty();
                        handle.hwnd = hwnd;
                        raw_window_handle::RawWindowHandle::Win32(handle)
                    }
                }
            }
        }

        // A persistent window handler which stores component instances and param state.
        struct JuceGlHandler {
            // GL state
            gl: Arc<glow::Context>,
            logical_width: u32,
            logical_height: u32,
            tex: Option<glow::NativeTexture>,
            program: Option<glow::NativeProgram>,
            vao: Option<glow::NativeVertexArray>,
            vbo: Option<glow::NativeBuffer>,

            // Components (persistent in window thread)
            title_label: Rc<RefCell<Label>>,
            main_slider: Rc<RefCell<Slider>>,
            bypass_button: Rc<RefCell<Button>>,
            // Widgets demo container
            widgets: WidgetsDemo,
            // other demos
            hello_demo: HelloWorldDemo,
            component_demo: ComponentDemo,
            // scene control buttons
            prev_button: Rc<RefCell<Button>>,
            next_button: Rc<RefCell<Button>>,

            // Root component for dispatch + mapping
            root_component: Component,
            // removed old comp_map mapping; handlers are registered on components

            // Drag state: dragging main slider or a widget slider index
            dragging_main: bool,
            dragging_demo_slider: bool,
            dragging_widget: Option<usize>,
            last_mouse: (i32, i32),

            // Bound references to plugin params and host context
            params: Arc<GuiDemoParams>,
            gui_context: Arc<dyn nih_plug::prelude::GuiContext>,
            // active scene selector
            active_scene: DemoScene,
        }

        // ControlKind removed: handlers are attached directly to components

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum DemoScene { Widgets, HelloWorld, Component }

        impl JuceGlHandler {
            fn new(gl: Arc<glow::Context>, width: u32, height: u32, params: Arc<GuiDemoParams>, gui_context: Arc<dyn nih_plug::prelude::GuiContext>) -> Self {
                // Initialize controls with bounds and values
                let mut slider = Slider::new(SliderOrientation::Horizontal);
                let _ = slider.set_range(0.0, 1.0);
                let _ = slider.set_bounds(Bounds::new(50, 80, 300, 40));
                slider.set_value(params.gain.modulated_normalized_value() as f64);

                let mut btn = Button::new("Bypass");
                let _ = btn.set_bounds(Bounds::new(150, 150, 100, 40));
                btn.set_button_state(if params.bypass.value() { nih_plug_gui::controls::ButtonState::Pressed } else { nih_plug_gui::controls::ButtonState::Normal });

                let mut title = Label::new("JUCE GUI Demo");
                let _ = title.set_bounds(Bounds::new(10, 10, 380, 30));
                title.set_alignment(TextAlignment::Center);
                title.set_font_size(24);

                // Build root component and add children for hit-testing
                let mut root = Component::new("root");
                let _ = root.set_bounds(Bounds::new(0, 0, width as u32, height as u32));

                let title_rc = Rc::new(RefCell::new(title));
                let main_slider_rc = Rc::new(RefCell::new(slider));
                let bypass_rc = Rc::new(RefCell::new(btn));
                // create additional demos and scene buttons
                let hello = HelloWorldDemo::new();
                let comp_demo = ComponentDemo::new();

                let mut prev = Button::new("<");
                let _ = prev.set_bounds(Bounds::new(10, 45, 40, 24));
                let mut next = Button::new(">");
                let _ = next.set_bounds(Bounds::new(350, 45, 40, 24));

                let prev_rc = Rc::new(RefCell::new(prev));
                let next_rc = Rc::new(RefCell::new(next));

                // Add components to root (clone underlying Component references)
                let _ = root.add_child(title_rc.borrow().component().clone());
                let _ = root.add_child(main_slider_rc.borrow().component().clone());
                let _ = root.add_child(bypass_rc.borrow().component().clone());

                // Add widget components
                    let widgets = WidgetsDemo::new();
                let mut widget_container = Component::new("widgets_container");
                let _ = widget_container.set_bounds(Bounds::new(0, 0, width as u32, height as u32));
                for s in &widgets.sliders {
                    let _ = root.add_child(s.component().clone());
                }

                // Add demo-specific components and scene buttons
                let _ = root.add_child(hello.hello_btn.borrow().component().clone());
                let _ = root.add_child(comp_demo.toggle.borrow().component().clone());
                let _ = root.add_child(comp_demo.slider.borrow().component().clone());
                let _ = root.add_child(prev_rc.borrow().component().clone());
                let _ = root.add_child(next_rc.borrow().component().clone());

                // register handlers by attaching closures directly to components
                // main slider: updates UI value and notifies host via ParamSetter
                {
                    let params2 = Arc::clone(&params);
                    let ctx2 = gui_context.clone();
                    let main_for_closure = main_slider_rc.clone();
                    {
                        let mut slider_mut = main_slider_rc.borrow_mut();
                        slider_mut.component_mut().set_mouse_handler(move |event| {
                        use nih_plug_gui::input::{MouseEvent, MouseButton, EventResult};
                        match event {
                            MouseEvent::ButtonDown { x, y, button, .. } if *button == MouseButton::Left => {
                                let bounds = main_for_closure.borrow().bounds();
                                let rel = (*x - bounds.x) as f64 / (bounds.width as f64);
                                let norm = rel.clamp(0.0, 1.0);
                                main_for_closure.borrow_mut().set_normalized_value(norm);
                                let setter = nih_plug::prelude::ParamSetter::new(ctx2.as_ref());
                                setter.begin_set_parameter(&params2.gain);
                                setter.set_parameter_normalized(&params2.gain, norm as f32);
                                EventResult::Handled
                            }
                            MouseEvent::Drag { x, y, .. } | MouseEvent::Move { x, y, .. } => {
                                let bounds = main_for_closure.borrow().bounds();
                                let rel = (*x - bounds.x) as f64 / (bounds.width as f64);
                                let norm = rel.clamp(0.0, 1.0);
                                main_for_closure.borrow_mut().set_normalized_value(norm);
                                let setter = nih_plug::prelude::ParamSetter::new(ctx2.as_ref());
                                setter.set_parameter_normalized(&params2.gain, norm as f32);
                                EventResult::Handled
                            }
                            MouseEvent::ButtonUp { .. } => {
                                let setter = nih_plug::prelude::ParamSetter::new(ctx2.as_ref());
                                setter.end_set_parameter(&params2.gain);
                                EventResult::Handled
                            }
                            _ => EventResult::NotHandled,
                        }
                        });
                    }
                }

                // bypass button: toggle parameter and visual state
                {
                    let params2 = Arc::clone(&params);
                    let ctx2 = gui_context.clone();
                    let btn_for_closure = bypass_rc.clone();
                    {
                        let mut bmut = bypass_rc.borrow_mut();
                        bmut.component_mut().set_mouse_handler(move |event| {
                        use nih_plug_gui::input::{MouseEvent, MouseButton, EventResult};
                        match event {
                            MouseEvent::ButtonDown { button, .. } if *button == MouseButton::Left => {
                                let new = !params2.bypass.value();
                                let setter = nih_plug::prelude::ParamSetter::new(ctx2.as_ref());
                                setter.set_parameter(&params2.bypass, new);
                                btn_for_closure.borrow_mut().set_button_state(if new { nih_plug_gui::controls::ButtonState::Pressed } else { nih_plug_gui::controls::ButtonState::Normal });
                                EventResult::Handled
                            }
                            _ => EventResult::NotHandled,
                        }
                        });
                    }
                }

                // Hello demo button: toggle label text (state stored in HelloWorldDemo struct in handler owner)
                {
                    let hello_btn_clone = hello.hello_btn.clone();
                    let label_ref = hello.label.clone();
                    {
                        let mut hb = hello_btn_clone.borrow_mut();
                        hb.component_mut().set_mouse_handler(move |event| {
                        use nih_plug_gui::input::{MouseEvent, MouseButton, EventResult};
                        match event {
                            MouseEvent::ButtonDown { button, .. } if *button == MouseButton::Left => {
                                // toggle label text directly here
                                let current = label_ref.borrow().text().to_string();
                                if current.starts_with("Hello!") {
                                    label_ref.borrow_mut().set_text("Hello JUCE-Style World");
                                } else {
                                    label_ref.borrow_mut().set_text("Hello! Click again");
                                }
                                EventResult::Handled
                            }
                            _ => EventResult::NotHandled,
                        }
                        });
                    }
                }

                // ComponentDemo controls: toggle and slider
                {
                    let toggle_clone = comp_demo.toggle.clone();
                    let toggle_state = comp_demo.rect_color_toggle.clone();
                    {
                        let mut tmut = toggle_clone.borrow_mut();
                        tmut.component_mut().set_mouse_handler(move |event| {
                        use nih_plug_gui::input::{MouseEvent, MouseButton, EventResult};
                        match event {
                            MouseEvent::ButtonDown { button, .. } if *button == MouseButton::Left => {
                                // flip color toggle and update button visual
                                let new = !*toggle_state.borrow();
                                *toggle_state.borrow_mut() = new;
                                toggle_clone.borrow_mut().set_button_state(if new { nih_plug_gui::controls::ButtonState::Pressed } else { nih_plug_gui::controls::ButtonState::Normal });
                                EventResult::Handled
                            }
                            _ => EventResult::NotHandled,
                        }
                        });
                    }
                }

                {
                    let slider_clone = comp_demo.slider.clone();
                    {
                        let mut smut = slider_clone.borrow_mut();
                        smut.component_mut().set_mouse_handler(move |event| {
                        use nih_plug_gui::input::{MouseEvent, MouseButton, EventResult};
                        match event {
                            MouseEvent::ButtonDown { button, .. } if *button == MouseButton::Left => EventResult::Handled,
                            MouseEvent::Drag { x, .. } | MouseEvent::Move { x, .. } => {
                                let bounds = slider_clone.borrow().bounds();
                                let rel = (*x - bounds.x) as f64 / (bounds.width as f64);
                                let norm = rel.clamp(0.0, 1.0);
                                slider_clone.borrow_mut().set_normalized_value(norm);
                                EventResult::Handled
                            }
                            MouseEvent::ButtonUp { .. } => EventResult::Handled,
                            _ => EventResult::NotHandled,
                        }
                    });
                }

                Self { gl, logical_width: width, logical_height: height, tex: None, program: None, vao: None, vbo: None, title_label: title_rc, main_slider: main_slider_rc, bypass_button: bypass_rc, widgets, hello_demo: hello, component_demo: comp_demo, prev_button: prev_rc, next_button: next_rc, root_component: root, dragging_main: false, dragging_demo_slider: false, dragging_widget: None, last_mouse: (0,0), params, gui_context, active_scene: DemoScene::Widgets }
            }

            fn find_component_at(&self, comp: &Component, x: i32, y: i32) -> Option<ComponentId> {
                // Check children first (top-most last)
                let count = comp.child_count();
                for i in (0..count).rev() {
                    if let Some(child) = comp.child(i) {
                        if child.contains_point(x, y) {
                            // Descend into child
                            if let Some(found) = self.find_component_at(&child, x, y) {
                                return Some(found);
                            } else {
                                return Some(child.id());
                            }
                        }
                    }
                }

                // If no child matches but this component contains point, return its id
                if comp.contains_point(x, y) {
                    return Some(comp.id());
                }

                None
            }
        }

        /// Small widgets demo containing many sliders arranged horizontally.
        struct WidgetsDemo {
            sliders: Vec<Slider>,
            labels: Vec<Label>,
        }

        /// Very small Hello World demo with a toggle button
        struct HelloWorldDemo {
            label: Rc<RefCell<Label>>,
            hello_btn: Rc<RefCell<Button>>,
            greeting_on: bool,
        }

        impl HelloWorldDemo {
            fn new() -> Self {
                let mut label = Label::new("Hello JUCE-Style World");
                let _ = label.set_bounds(Bounds::new(10, 70, 380, 30));
                label.set_alignment(TextAlignment::Center);

                let mut btn = Button::new("Say Hello");
                let _ = btn.set_bounds(Bounds::new(160, 110, 80, 28));

                Self { label: Rc::new(RefCell::new(label)), hello_btn: Rc::new(RefCell::new(btn)), greeting_on: false }
            }

            fn render(&self, graphics: &mut nih_plug_graphics::Graphics, font: Option<&nih_plug_graphics::Font>) {
                if let Some(f) = font {
                    let _ = self.label.borrow().render(graphics, f);
                }
                let _ = self.hello_btn.borrow().render(graphics);
            }
        }

        /// Small component demo showing a toggle and a slider controlling a rectangle.
        struct ComponentDemo {
            title: Rc<RefCell<Label>>,
            toggle: Rc<RefCell<Button>>,
            slider: Rc<RefCell<Slider>>,
            rect_color_toggle: Rc<RefCell<bool>>,
        }

        impl ComponentDemo {
            fn new() -> Self {
                let mut title = Label::new("Component Demo");
                let _ = title.set_bounds(Bounds::new(10, 10, 380, 30));
                title.set_alignment(TextAlignment::Center);
                title.set_font_size(18);

                let mut toggle = Button::new("Toggle Color");
                let _ = toggle.set_bounds(Bounds::new(20, 60, 120, 32));

                let mut slider = Slider::new(SliderOrientation::Horizontal);
                let _ = slider.set_bounds(Bounds::new(160, 60, 220, 32));
                let _ = slider.set_range(10.0, 200.0);
                slider.set_value(80.0);

                Self { title: Rc::new(RefCell::new(title)), toggle: Rc::new(RefCell::new(toggle)), slider: Rc::new(RefCell::new(slider)), rect_color_toggle: Rc::new(RefCell::new(false)) }
            }

            fn render(&self, graphics: &mut nih_plug_graphics::Graphics, font: Option<&nih_plug_graphics::Font>) {
                if let Some(f) = font {
                    let _ = self.title.borrow().render(graphics, f);
                }

                // Render a rectangle whose width is controlled by slider and color toggled
                let w = self.slider.borrow().value() as i32;
                let h = 80i32;
                let x = 20i32;
                let y = 110i32;

                graphics.set_color(if *self.rect_color_toggle.borrow() { nih_plug_graphics::Color::rgb(180, 60, 60) } else { nih_plug_graphics::Color::rgb(60, 120, 180) });
                graphics.fill_rect(x, y, w.abs() as u32, h as u32);

                let _ = self.toggle.borrow().render(graphics);
                let _ = self.slider.borrow().render(graphics);
            }
        }

        impl WidgetsDemo {
            fn new() -> Self {
                let mut sliders = Vec::new();
                let mut labels = Vec::new();

                // Create 8 vertical sliders laid out in a row at y=200
                let x_start = 20i32;
                let y = 200i32;
                let w = 40u32;
                let h = 120u32;
                let gap = 8i32;

                for i in 0..8 {
                    let mut s = Slider::new(SliderOrientation::Vertical);
                    let x = x_start + i as i32 * (w as i32 + gap);
                    let _ = s.set_bounds(Bounds::new(x, y, w, h));
                    s.set_range(0.0, 100.0);
                    s.set_value((i as f64 + 1.0) * 10.0);
                    sliders.push(s);

                    let mut l = Label::new(&format!("S{}", i+1));
                    let _ = l.set_bounds(Bounds::new(x, y - 24, w, 20));
                    labels.push(l);
                }

                Self { sliders, labels }
            }

            fn render(&self, graphics: &mut nih_plug_graphics::Graphics, font: Option<&nih_plug_graphics::Font>) {
                for s in &self.sliders {
                    let _ = s.render(graphics);
                }
                for l in &self.labels {
                    if let Some(f) = font {
                        let _ = l.render(graphics, f);
                    }
                }
            }

            fn hit_test_slider(&self, x: i32, y: i32) -> Option<usize> {
                for (i, s) in self.sliders.iter().enumerate() {
                    if s.bounds().contains(x, y) {
                        return Some(i);
                    }
                }
                None
            }

            fn set_slider_value(&mut self, idx: usize, v: f64) {
                if let Some(s) = self.sliders.get_mut(idx) {
                    s.set_value(v);
                }
            }
        }

        impl baseview::WindowHandler for JuceGlHandler {
            fn on_frame(&mut self, window: &mut baseview::Window) {
                // Create graphics buffer and ask each retained component to render into it
                use nih_plug_graphics::Graphics;

                let mut graphics = match Graphics::new(self.logical_width, self.logical_height) {
                    Ok(g) => g,
                    Err(_) => return,
                };

                // Clear background
                graphics.set_color(nih_plug_graphics::Color::rgb(24, 24, 24));
                graphics.clear();

                // Title (try to load font once and reuse for labels)
                let maybe_font = match nih_plug_graphics::text::Font::from_bytes(include_bytes!("../../../../nih_plug_graphics/tests/test_font.ttf"), nih_plug_graphics::text::FontSettings::default()) {
                    Ok(f) => Some(f),
                    Err(_) => None,
                };

                if let Some(f) = maybe_font.as_ref() {
                    graphics.set_color(nih_plug_graphics::Color::rgb(220, 220, 220));
                    graphics.draw_text("JUCE GUI Demo", (self.logical_width as i32) / 2, 30, f, 24.0);
                }

                // Render common controls
                let _ = self.main_slider.borrow().render(&mut graphics);
                let _ = self.bypass_button.borrow().render(&mut graphics);
                let _ = self.prev_button.borrow().render(&mut graphics);
                let _ = self.next_button.borrow().render(&mut graphics);

                // Render the currently active demo scene
                match self.active_scene {
                    DemoScene::Widgets => { let _ = self.widgets.render(&mut graphics, maybe_font.as_ref()); }
                    DemoScene::HelloWorld => { self.hello_demo.render(&mut graphics, maybe_font.as_ref()); }
                    DemoScene::Component => { self.component_demo.render(&mut graphics, maybe_font.as_ref()); }
                }

                // Upload to GL and render full-screen quad (same as GlHandler)
                let gl_ctx = window.gl_context().expect("no gl ctx");
                unsafe {
                    gl_ctx.make_current();
                    let bytes = graphics.as_bytes();

                    // create/upload texture and render textured quad (lazy init)
                    let tex = match self.tex {
                        Some(t) => t,
                        None => {
                            let t = self.gl.create_texture().expect("create tex");
                            self.gl.bind_texture(glow::TEXTURE_2D, Some(t));
                            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
                            self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
                            self.tex = Some(t);
                            t
                        }
                    };

                    self.gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                    self.gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
                    self.gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, self.logical_width as i32, self.logical_height as i32, 0, glow::RGBA, glow::UNSIGNED_BYTE, glow::PixelUnpackData::Slice(Some(bytes)));

                    if self.program.is_none() {
                        let vs = "#version 130\n in vec2 aPos; in vec2 aUV; out vec2 vUV; void main(){ vUV = aUV; gl_Position = vec4(aPos, 0.0, 1.0);} ";
                        let fs = "#version 130\n in vec2 vUV; out vec4 o; uniform sampler2D s; void main(){ o = texture(s, vUV); }";

                        let vert = self.gl.create_shader(glow::VERTEX_SHADER).unwrap();
                        self.gl.shader_source(vert, vs);
                        self.gl.compile_shader(vert);

                        let frag = self.gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
                        self.gl.shader_source(frag, fs);
                        self.gl.compile_shader(frag);

                        let prog = self.gl.create_program().unwrap();
                        self.gl.attach_shader(prog, vert);
                        self.gl.attach_shader(prog, frag);
                        self.gl.link_program(prog);
                        self.gl.delete_shader(vert);
                        self.gl.delete_shader(frag);
                        self.program = Some(prog);

                        let verts: [f32; 16] = [
                            -1.0, -1.0, 0.0, 1.0,
                            1.0, -1.0, 1.0, 1.0,
                            1.0, 1.0, 1.0, 0.0,
                            -1.0, 1.0, 0.0, 0.0,
                        ];

                        let vbo = self.gl.create_buffer().unwrap();
                        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                        let vtx_bytes = std::slice::from_raw_parts(verts.as_ptr() as *const u8, verts.len()*4);
                        self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vtx_bytes, glow::STATIC_DRAW);

                        let vao = self.gl.create_vertex_array().unwrap();
                        self.gl.bind_vertex_array(Some(vao));
                        let stride = 4 * std::mem::size_of::<f32>() as i32;
                        self.gl.enable_vertex_attrib_array(0);
                        self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
                        self.gl.enable_vertex_attrib_array(1);
                        self.gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 8);

                        self.vao = Some(vao);
                        self.vbo = Some(vbo);
                    }

                    self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
                    self.gl.clear(glow::COLOR_BUFFER_BIT);

                    if let Some(prog) = self.program {
                        self.gl.use_program(Some(prog));
                        self.gl.active_texture(glow::TEXTURE0);
                        self.gl.bind_texture(glow::TEXTURE_2D, self.tex);
                        if let Some(loc) = self.gl.get_uniform_location(prog, "s") { self.gl.uniform_1_i32(Some(&loc), 0); }
                        if let Some(vao) = self.vao { self.gl.bind_vertex_array(Some(vao)); self.gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4); }
                    }

                    gl_ctx.swap_buffers();
                    gl_ctx.make_not_current();
                }
            }

            

            fn on_event(&mut self, _window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
                // Map events to component actions. We use ParamSetter to talk to the host.
                match event {
                    baseview::Event::Mouse(mouse_event) => match mouse_event {
                        baseview::MouseEvent::CursorMoved { position, .. } => {
                            let (x, y) = (position.x as i32, position.y as i32);
                            self.last_mouse = (x, y);
                            if self.dragging_main {
                                // update slider value based on x and send normalized value to host
                                let bounds = self.main_slider.borrow().bounds();
                                let rel = (x - bounds.x) as f64 / (bounds.width as f64);
                                let norm = rel.clamp(0.0, 1.0);
                                self.main_slider.borrow_mut().set_normalized_value(norm);
                                let setter = nih_plug::prelude::ParamSetter::new(self.gui_context.as_ref());
                                setter.set_parameter_normalized(&self.params.gain, norm as f32);
                            }
                            if self.dragging_demo_slider {
                                // Update demo slider while dragging
                                let bounds = self.component_demo.slider.borrow().bounds();
                                let rel = (x - bounds.x) as f64 / (bounds.width as f64);
                                let norm = rel.clamp(0.0, 1.0);
                                self.component_demo.slider.borrow_mut().set_normalized_value(norm);
                            }
                            // If not currently dragging anything, dispatch a Move event to components (for hover/preview)
                            if !self.dragging_main && !self.dragging_demo_slider && self.dragging_widget.is_none() {
                                use nih_plug_gui::input::MouseEvent;
                                let ev = MouseEvent::Move { x, y, modifiers: nih_plug_gui::input::Modifiers::none() };
                                let _ = self.root_component.dispatch_mouse_event(&ev);
                            }
                        }
                        baseview::MouseEvent::ButtonPressed { button, .. } => {
                            if button == baseview::MouseButton::Left {
                                // Hit test based on last known cursor position
                                let (mx, my) = self.last_mouse;
                                // Use component tree hit-test to find what was clicked
                                if let Some(comp_id) = self.find_component_at(&self.root_component, mx, my) {
                                    use nih_plug_gui::input::{MouseEvent, MouseButton, EventResult};
                                    // dispatch ButtonDown to the component tree -> handlers attached to components will run
                                    let ev = MouseEvent::ButtonDown { x: mx, y: my, button: nih_plug_gui::input::MouseButton::Left, modifiers: nih_plug_gui::input::Modifiers::none() };
                                    let result = self.root_component.dispatch_mouse_event(&ev);

                                    // if handled, set dragging state appropriately (we still need to track drags)
                                    if result == EventResult::Handled {
                                        // main slider
                                        if comp_id == self.main_slider.borrow().component().id() {
                                            self.dragging_main = true;
                                            let bounds = self.main_slider.borrow().bounds();
                                            let rel = (mx - bounds.x) as f64 / (bounds.width as f64);
                                            let norm = rel.clamp(0.0, 1.0);
                                            self.main_slider.borrow_mut().set_normalized_value(norm);
                                            let setter = nih_plug::prelude::ParamSetter::new(self.gui_context.as_ref());
                                            setter.begin_set_parameter(&self.params.gain);
                                            setter.set_parameter_normalized(&self.params.gain, norm as f32);
                                            return baseview::EventStatus::Captured;
                                        }

                                        // widget slider? find index
                                        for (i, s) in self.widgets.sliders.iter().enumerate() {
                                            if comp_id == s.component().id() {
                                                self.dragging_widget = Some(i);
                                                return baseview::EventStatus::Captured;
                                            }
                                        }

                                        // component demo slider
                                        if comp_id == self.component_demo.slider.borrow().component().id() {
                                            self.dragging_demo_slider = true;
                                            let bounds = self.component_demo.slider.borrow().bounds();
                                            let rel = (mx - bounds.x) as f64 / (bounds.width as f64);
                                            let norm = rel.clamp(0.0, 1.0);
                                            self.component_demo.slider.borrow_mut().set_normalized_value(norm);
                                            return baseview::EventStatus::Captured;
                                        }

                                        // prev / next scene buttons
                                        if comp_id == self.prev_button.borrow().component().id() {
                                            self.active_scene = match self.active_scene { DemoScene::Widgets => DemoScene::Component, DemoScene::HelloWorld => DemoScene::Widgets, DemoScene::Component => DemoScene::HelloWorld };
                                            return baseview::EventStatus::Captured;
                                        }
                                        if comp_id == self.next_button.borrow().component().id() {
                                            self.active_scene = match self.active_scene { DemoScene::Widgets => DemoScene::HelloWorld, DemoScene::HelloWorld => DemoScene::Component, DemoScene::Component => DemoScene::Widgets };
                                            return baseview::EventStatus::Captured;
                                        }

                                        // bypass and other control handlers will be executed by dispatch already
                                        return baseview::EventStatus::Captured;
                                    }
                                }
                            }
                        }
                        baseview::MouseEvent::ButtonReleased { button, .. } => {
                            if button == baseview::MouseButton::Left {
                                // dispatch ButtonUp to the component tree first
                                use nih_plug_gui::input::{MouseEvent};
                                let ev_up = MouseEvent::ButtonUp { x: self.last_mouse.0, y: self.last_mouse.1, button: nih_plug_gui::input::MouseButton::Left, modifiers: nih_plug_gui::input::Modifiers::none() };
                                let _ = self.root_component.dispatch_mouse_event(&ev_up);

                                if let Some(idx) = self.dragging_widget {
                                    // update the widget slider value
                                    let mx = self.last_mouse.0;
                                    let bounds = self.widgets.sliders[idx].bounds();
                                    let rel = (mx - bounds.x) as f64 / (bounds.width as f64);
                                    let norm = rel.clamp(0.0, 1.0);
                                    self.widgets.set_slider_value(idx, norm * 100.0);
                                    return baseview::EventStatus::Captured;
                                } else if self.dragging_demo_slider {
                                    // finished dragging the demo slider
                                    self.dragging_demo_slider = false;
                                    return baseview::EventStatus::Captured;
                                } else if self.dragging_main {
                                    // main slider handler will receive ButtonUp via dispatch and call end_set_parameter
                                }
                                self.dragging_main = false;
                                self.dragging_widget = None;
                                return baseview::EventStatus::Captured;
                            }
                        }
                        _ => {}
                    },
                    baseview::Event::Window(w) => if let baseview::WindowEvent::Resized(info) = w {
                        self.logical_width = info.logical_size().width.round() as u32;
                        self.logical_height = info.logical_size().height.round() as u32;
                    },
                    _ => {}
                }

                baseview::EventStatus::Captured
            }
        }

        // Create and open the parented GL window with our handler
        let window = baseview::Window::open_parented(
            &ParentWindowHandleAdapter(_parent),
            baseview::WindowOpenOptions {
                title: String::from("JUCE GUI Demo"),
                size: baseview::Size::new(400.0, 300.0),
                scale: baseview::WindowScalePolicy::SystemScaleFactor,
                gl_config: Some(baseview::gl::GlConfig::default()),
            },
            move |window: &mut baseview::Window<'_>| {
                // Create GL context & glow
                let gl_ctx = window.gl_context().expect("failed to get baseview gl context");
                let gl = unsafe {
                    gl_ctx.make_current();
                    let ctx = glow::Context::from_loader_function(|s| gl_ctx.get_proc_address(s));
                    gl_ctx.make_not_current();
                    ctx
                };

                JuceGlHandler::new(Arc::new(gl), 400, 300, Arc::clone(&params), context.clone())
            }
        );

        // Keep window handle alive in returned handle object. We'll return an object that's
        // Send so the host wrapper can store it.
        struct EditorHandle { window: baseview::WindowHandle }
        unsafe impl Send for EditorHandle {}

        // Use the parent provided by the host so this editor gets embedded. If the host does not
        // support parented windows then baseview will typically fail; falling back to a standalone
        // window via `spawn` could be implemented here if desired.
        let _window_handle = window;

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
