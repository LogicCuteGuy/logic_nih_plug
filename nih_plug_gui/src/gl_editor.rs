#![cfg(feature = "gl-editor")]
//! Lightweight GL-backed window helper for prototyping UI rendering using OpenGL.
//!
//! This helper is intentionally small: it creates a baseview window with an OpenGL
//! context and provides a frame-callback that receives a `glow::Context`. The
//! consumer is responsible for performing GL draws.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use baseview::{gl::GlConfig, WindowOpenOptions, WindowScalePolicy};
use glow::HasContext as _;

/// A handle to an OpenGL window.
pub struct GlWindow {
    pub(crate) join_handle: Option<std::thread::JoinHandle<()>>,
    should_close: Arc<AtomicBool>,
}

/// Builder for a GL window. The draw callback receives a reference to the glow::Context
/// and the logical size (width, height) in pixels.
pub struct GlWindowBuilder {
    title: String,
    width: u32,
    height: u32,
    draw: Option<Arc<Mutex<dyn FnMut(&mut nih_plug_graphics::Graphics, (u32,u32)) + Send + 'static>>>,
}

impl GlWindowBuilder {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self { title: title.into(), width, height, draw: None }
    }

    pub fn draw<F>(mut self, f: F) -> Self
    where F: FnMut(&mut nih_plug_graphics::Graphics, (u32,u32)) + Send + 'static
    {
        self.draw = Some(Arc::new(Mutex::new(f)));
        self
    }

    pub fn open(self) -> GlWindow {
        let title = self.title.clone();
        let width = self.width;
        let height = self.height;
        let draw = self.draw.expect("Missing draw callback");

        let options = WindowOpenOptions {
            title: title.clone(),
            size: baseview::Size::new(width as f64, height as f64),
            scale: WindowScalePolicy::SystemScaleFactor,
            gl_config: Some(GlConfig::default()),
        };

        let should_close = Arc::new(AtomicBool::new(false));
        let thread_flag = Arc::clone(&should_close);

        let join_handle = thread::spawn(move || {
            baseview::Window::open_blocking(options, move |window: &mut baseview::Window<'_>| {
                // Try to get GL context
                let gl_ctx = window.gl_context().expect("failed to get baseview gl context");

                // Create glow context from window's loader
                let gl = unsafe {
                    gl_ctx.make_current();
                    let ctx = glow::Context::from_loader_function(|s| gl_ctx.get_proc_address(s));
                    gl_ctx.make_not_current();
                    ctx
                };

                GlHandler { draw: draw.clone(), gl: Arc::new(gl), logical_width: width, logical_height: height, should_close: thread_flag.clone(), tex: None, program: None, vao: None, vbo: None }
            });
        });

        GlWindow { join_handle: Some(join_handle), should_close }
    }

    pub fn spawn<F>(title: impl Into<String>, width: u32, height: u32, f: F) -> GlWindow
    where F: FnMut(&mut nih_plug_graphics::Graphics, (u32,u32)) + Send + 'static
    {
        GlWindowBuilder::new(title, width, height).draw(f).open()
    }
}

struct GlHandler {
    draw: Arc<Mutex<dyn FnMut(&mut nih_plug_graphics::Graphics, (u32,u32)) + Send + 'static>>,
    gl: Arc<glow::Context>,
    logical_width: u32,
    logical_height: u32,
    should_close: Arc<AtomicBool>,
    // GL resources (lazily initialized)
    tex: Option<glow::NativeTexture>,
    program: Option<glow::NativeProgram>,
    vao: Option<glow::NativeVertexArray>,
    vbo: Option<glow::NativeBuffer>,
}

impl baseview::WindowHandler for GlHandler {
    fn on_frame(&mut self, window: &mut baseview::Window) {
        if self.should_close.load(Ordering::Acquire) {
            window.close();
            return;
        }
        use nih_plug_graphics::Graphics;

        // create a graphics buffer and ask the callback to paint it
        let mut graphics = match Graphics::new(self.logical_width, self.logical_height) {
            Ok(g) => g,
            Err(_) => return,
        };

        if let Ok(mut f) = self.draw.lock() {
            (f)(&mut graphics, (self.logical_width, self.logical_height));
        }

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

            // initialize resources on demand
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
                    -1.0, -1.0, 0.0, 0.0,
                    1.0, -1.0, 1.0, 0.0,
                    1.0, 1.0, 1.0, 1.0,
                    -1.0, 1.0, 0.0, 1.0,
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

            // draw
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
        if let baseview::Event::Window(w) = event {
            if let baseview::WindowEvent::Resized(info) = w {
                self.logical_width = info.logical_size().width.round() as u32;
                self.logical_height = info.logical_size().height.round() as u32;
            }
        }
        baseview::EventStatus::Captured
    }
}

impl Drop for GlWindow {
    fn drop(&mut self) {
        // Signal the background thread to close the window and join it.
        self.should_close.store(true, Ordering::Release);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}
