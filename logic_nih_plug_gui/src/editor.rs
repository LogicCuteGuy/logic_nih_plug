//! Lightweight softbuffer-backed window helper for prototyping UIs
//!
//! This module is feature gated behind `softbuffer-editor` and intentionally provides a
//! minimal API so examples and small plugins can spawn a simple window and render
//! using `logic_nih_plug_graphics::Graphics` without duplicating baseview/softbuffer plumbing.
#![cfg(feature = "softbuffer-editor")]

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::ptr::NonNull;
use std::num::{NonZeroU32, NonZeroIsize};

use baseview::WindowOpenOptions;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use raw_window_handle_06 as raw_window_handle_06;
use softbuffer::Surface;

use crate::components::Bounds;
use logic_nih_plug_graphics::Graphics;

/// A handle to a softbuffer-backed window. The provided draw closure is called on every
/// frame with a mutable `Graphics` context. The closure receives the logical size (width, height)
/// in pixels.
pub struct SoftbufferWindow {
    pub(crate) join_handle: Option<std::thread::JoinHandle<()>>,
    should_close: Arc<AtomicBool>,
}

/// A builder for a softbuffer window. The draw closure must be 'static + Send so it can be
/// executed from the window thread.
pub struct SoftbufferWindowBuilder {
    title: String,
    width: u32,
    height: u32,
    draw: Option<Arc<Mutex<dyn FnMut(&mut Graphics, (u32, u32)) + Send + 'static>>>,
}

impl SoftbufferWindowBuilder {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self { title: title.into(), width, height, draw: None }
    }

    /// Set the draw callback; this will be called every frame with a fresh `Graphics` buffer.
    pub fn draw<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut Graphics, (u32, u32)) + Send + 'static,
    {
        self.draw = Some(Arc::new(Mutex::new(f)));
        self
    }

    /// Open the window. Returns a `SoftbufferWindow` handle that keeps the window alive until
    /// dropped.
    pub fn open(self) -> SoftbufferWindow {
        let title = self.title.clone();
        let width = self.width;
        let height = self.height;
        let draw = self.draw.expect("Missing draw callback");

        // baseview requires a closure that returns the window handler. We'll create a
        // small handler that stores the draw closure and surface and calls it on each frame.
        let options = WindowOpenOptions {
            title: title.clone(),
            size: baseview::Size::new(width as f64, height as f64),
            scale: baseview::WindowScalePolicy::SystemScaleFactor,
            gl_config: None,
        };

            let should_close = Arc::new(AtomicBool::new(false));
            let thread_flag = Arc::clone(&should_close);

            let join_handle = thread::spawn(move || {
                baseview::Window::open_blocking(options, move |window: &mut baseview::Window<'_>| -> SoftbufferHandler {
                    // Create softbuffer surface for this window
                    let target = baseview_window_to_surface_target(window);
                    let sb_ctx = softbuffer::Context::new(target.clone()).expect("softbuffer ctx");
                    let mut sb_surface = Surface::new(&sb_ctx, target).expect("softbuffer surface");

                    // Resize surface to match initial size
                    let (logical_w, logical_h) = (width, height);
                    sb_surface
                        .resize(
                            std::num::NonZeroU32::new(width).unwrap(),
                            std::num::NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap_or(());

                    SoftbufferHandler {
                        draw: draw.clone(),
                        sb_ctx: Some(sb_ctx),
                        sb_surface: Some(sb_surface),
                        logical_width: logical_w,
                        logical_height: logical_h,
                        should_close: thread_flag.clone(),
                    }
                });
            });

            SoftbufferWindow { join_handle: Some(join_handle), should_close }
    }
}

impl SoftbufferWindow {
    /// Convenience to spawn and keep a window open for the lifetime of the returned handle.
    pub fn spawn<F>(title: impl Into<String>, width: u32, height: u32, f: F) -> Self
    where
        F: FnMut(&mut Graphics, (u32, u32)) + Send + 'static,
    {
        SoftbufferWindowBuilder::new(title, width, height).draw(f).open()
    }
}

// The WindowHandler implementation used by the helper.
// Softbuffer uses raw_window_handle v6, but baseview uses raw_window_handle v5, so we need to
// adapt it ourselves. We create a small adapter type used as the generic parameter for the
// softbuffer Context/Surface types.
#[derive(Clone)]
struct SoftbufferWindowHandleAdapter {
    raw_display_handle: raw_window_handle_06::RawDisplayHandle,
    raw_window_handle: raw_window_handle_06::RawWindowHandle,
}

impl raw_window_handle_06::HasDisplayHandle for SoftbufferWindowHandleAdapter {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle_06::DisplayHandle<'_>, raw_window_handle_06::HandleError> {
        unsafe {
            Ok(raw_window_handle_06::DisplayHandle::borrow_raw(self.raw_display_handle))
        }
    }
}

impl raw_window_handle_06::HasWindowHandle for SoftbufferWindowHandleAdapter {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle_06::WindowHandle<'_>, raw_window_handle_06::HandleError> {
        unsafe { Ok(raw_window_handle_06::WindowHandle::borrow_raw(self.raw_window_handle)) }
    }
}

struct SoftbufferHandler {
    draw: Arc<Mutex<dyn FnMut(&mut Graphics, (u32, u32)) + Send + 'static>>,
    sb_ctx: Option<softbuffer::Context<SoftbufferWindowHandleAdapter>>,
    sb_surface: Option<Surface<SoftbufferWindowHandleAdapter, SoftbufferWindowHandleAdapter>>,
    logical_width: u32,
    logical_height: u32,
    should_close: Arc<AtomicBool>,
}

impl baseview::WindowHandler for SoftbufferHandler {
    fn on_frame(&mut self, _window: &mut baseview::Window) {
        if self.should_close.load(Ordering::Acquire) {
            _window.close();
            return;
        }
        // Create a Graphics that matches the logical size and call the draw closure.
        if let Some(surface) = &mut self.sb_surface {
            let phys_w = self.logical_width;
            let phys_h = self.logical_height;

            let mut graphics = Graphics::new(phys_w, phys_h).expect("create graphics");
            if let Ok(mut draw) = self.draw.lock() {
                (draw)(&mut graphics, (phys_w, phys_h));
            }

            // Copy pixel bytes into softbuffer buffer
            if let Ok(mut buffer) = surface.buffer_mut() {
                let bytes = graphics.as_bytes();
                for (dst, src) in buffer.iter_mut().zip(bytes.chunks_exact(4)) {
                    *dst = src[0] as u32 | ((src[1] as u32) << 8) | ((src[2] as u32) << 16) | ((src[3] as u32) << 24);
                }
                buffer.present().unwrap_or(());
            }
        }
    }

    fn on_event(&mut self, _window: &mut baseview::Window, event: baseview::Event) -> baseview::EventStatus {
        use baseview::WindowEvent;
        if let baseview::Event::Window(w) = event {
            if let WindowEvent::Resized(info) = w {
                self.logical_width = info.logical_size().width.round() as u32;
                self.logical_height = info.logical_size().height.round() as u32;
                if let Some(surface) = &mut self.sb_surface {
                    surface
                        .resize(
                            std::num::NonZeroU32::new(info.physical_size().width).unwrap(),
                            std::num::NonZeroU32::new(info.physical_size().height).unwrap(),
                        )
                        .unwrap_or(());
                }
            }
        }

        baseview::EventStatus::Captured
    }
}

impl Drop for SoftbufferWindow {
    fn drop(&mut self) {
        self.should_close.store(true, Ordering::Release);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

// Convert baseview window to softbuffer target: reuse same helper the examples use.
fn baseview_window_to_surface_target(window: &baseview::Window<'_>) -> SoftbufferWindowHandleAdapter {
    let raw_display_handle = window.raw_display_handle();
    let raw_window_handle = window.raw_window_handle();

    SoftbufferWindowHandleAdapter {
        raw_display_handle: match raw_display_handle {
            raw_window_handle::RawDisplayHandle::AppKit(_) => {
                raw_window_handle_06::RawDisplayHandle::AppKit(raw_window_handle_06::AppKitDisplayHandle::new())
            }
            raw_window_handle::RawDisplayHandle::Xlib(handle) => {
                raw_window_handle_06::RawDisplayHandle::Xlib(raw_window_handle_06::XlibDisplayHandle::new(
                    NonNull::new(handle.display),
                    handle.screen,
                ))
            }
            raw_window_handle::RawDisplayHandle::Xcb(handle) => {
                raw_window_handle_06::RawDisplayHandle::Xcb(raw_window_handle_06::XcbDisplayHandle::new(
                    NonNull::new(handle.connection),
                    handle.screen,
                ))
            }
            raw_window_handle::RawDisplayHandle::Windows(_) => {
                raw_window_handle_06::RawDisplayHandle::Windows(raw_window_handle_06::WindowsDisplayHandle::new())
            }
            _ => todo!(),
        },
        raw_window_handle: match raw_window_handle {
            raw_window_handle::RawWindowHandle::AppKit(handle) => raw_window_handle_06::RawWindowHandle::AppKit(
                raw_window_handle_06::AppKitWindowHandle::new(NonNull::new(handle.ns_view).unwrap()),
            ),
            raw_window_handle::RawWindowHandle::Xlib(handle) => {
                raw_window_handle_06::RawWindowHandle::Xlib(raw_window_handle_06::XlibWindowHandle::new(handle.window))
            }
            raw_window_handle::RawWindowHandle::Xcb(handle) => raw_window_handle_06::RawWindowHandle::Xcb(
                raw_window_handle_06::XcbWindowHandle::new(NonZeroU32::new(handle.window).unwrap()),
            ),
            raw_window_handle::RawWindowHandle::Win32(handle) => {
                let mut raw_handle = raw_window_handle_06::Win32WindowHandle::new(NonZeroIsize::new(handle.hwnd as isize).unwrap());
                raw_handle.hinstance = NonZeroIsize::new(handle.hinstance as isize);
                raw_window_handle_06::RawWindowHandle::Win32(raw_handle)
            }
            _ => todo!(),
        },
    }
}

// Public helpers for rendering components into a Graphics buffer. These are intentionally
// simple: they iterate children and call render methods on known control types. This will not
// cover all possible component content, but it eases adopting logic_nih_plug_gui with the softbuffer
// helper.
use crate::controls::{Button, Label, Slider};

/// Renders a subset of known controls into `graphics`.
pub fn render_controls_sample(graphics: &mut Graphics, items: &[&dyn std::any::Any]) {
    // For now we support Button, Slider and Label. Consumers can call control.render_* directly
    // if they keep concrete types.
    for any in items {
        if let Some(b) = any.downcast_ref::<Button>() {
            let _ = b.render(graphics);
        } else if let Some(s) = any.downcast_ref::<Slider>() {
            let _ = s.render(graphics);
        } else if let Some(l) = any.downcast_ref::<Label>() {
            // Labels require a font for nice text when using `text` feature; use simple render
            #[cfg(not(feature = "text"))]
            let _ = l.render(graphics, &logic_nih_plug_graphics::Font::from_bytes(include_bytes!("../../logic_nih_plug_graphics/tests/test_font.ttf"), logic_nih_plug_graphics::text::FontSettings::default()).unwrap());
            #[cfg(not(feature = "text"))]
            {
                // fallback: do nothing
            }
        }
    }
}
