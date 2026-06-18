//! OpenGL render-loop trait.

/// Trait for objects that perform continuous OpenGL rendering.
///
/// This mirrors JUCE's `OpenGLRenderer` abstract class. Implementors receive
/// lifecycle callbacks (`new_context_created`, `render`, `context_closing`)
/// that are driven by the rendering loop.
///
/// # Example
///
/// ```
/// use logic_nih_plug_gui::opengl::renderer::OpenGLRenderer;
/// use glow::HasContext as _;
///
/// struct MyRenderer {
///     frame_count: u32,
/// }
///
/// impl OpenGLRenderer for MyRenderer {
///     fn new_context_created(&mut self, gl: &glow::Context) {
///         // Compile shaders, create VAOs, etc.
///         unsafe { gl.clear_color(0.1, 0.1, 0.1, 1.0); }
///     }
///
///     fn render(&mut self, gl: &glow::Context, width: u32, height: u32) {
///         unsafe {
///             gl.viewport(0, 0, width as i32, height as i32);
///             gl.clear(glow::COLOR_BUFFER_BIT);
///         }
///         self.frame_count += 1;
///     }
///
///     fn context_closing(&mut self, gl: &glow::Context) {
///         // Clean up GL resources.
///     }
/// }
/// ```
pub trait OpenGLRenderer {
    /// Called once when a new OpenGL context is made current. Compile shaders,
    /// create textures, and set up any persistent GL state here.
    fn new_context_created(&mut self, gl: &glow::Context);

    /// Called every frame. Perform your GL drawing here.
    ///
    /// `width` and `height` are the current viewport dimensions in pixels.
    fn render(&mut self, gl: &glow::Context, width: u32, height: u32);

    /// Called when the OpenGL context is about to be destroyed. Release all
    /// GL resources here.
    fn context_closing(&mut self, gl: &glow::Context);

    /// Called when the viewport is resized. Override to update projection
    /// matrices or FBOs. Default implementation does nothing.
    fn viewport_resized(&mut self, _gl: &glow::Context, _width: u32, _height: u32) {}

    /// Whether this renderer should continue rendering. Return `false` to
    /// stop the render loop. Default returns `true`.
    fn should_continue(&self) -> bool {
        true
    }
}

/// A simple render-loop driver that calls `OpenGLRenderer` methods in order.
///
/// This is useful when you own the window and control the frame loop yourself
/// (e.g. in a `baseview::WindowHandler`).
pub struct RenderLoopDriver<R: OpenGLRenderer> {
    renderer: R,
    initialized: bool,
}

impl<R: OpenGLRenderer> RenderLoopDriver<R> {
    /// Create a new driver wrapping the given renderer.
    pub fn new(renderer: R) -> Self {
        Self { renderer, initialized: false }
    }

    /// Access the inner renderer.
    pub fn renderer(&self) -> &R {
        &self.renderer
    }

    /// Mutable access to the inner renderer.
    pub fn renderer_mut(&mut self) -> &mut R {
        &mut self.renderer
    }

    /// Consume the driver and return the inner renderer.
    ///
    /// Calls `context_closing` if the context was initialized (pass `None`
    /// for `gl` to skip the GL cleanup — only use when the context is already
    /// torn down).
    pub fn into_renderer(mut self, gl: Option<&glow::Context>) -> R {
        if self.initialized {
            if let Some(gl) = gl {
                self.renderer.context_closing(gl);
            }
            self.initialized = false;
        }
        let renderer = unsafe {
            // SAFETY: We take ownership of `renderer` out of self. The Drop
            // impl only accesses `self.initialized` (a bool), not `self.renderer`.
            std::ptr::read(&self.renderer)
        };
        // Prevent Drop from running — we've manually extracted the renderer
        // and already cleaned up the initialized flag.
        std::mem::forget(self);
        renderer
    }

    /// Called from your frame handler. Ensures `new_context_created` runs
    /// exactly once, then calls `render` each frame.
    pub fn on_frame(&mut self, gl: &glow::Context, width: u32, height: u32) {
        if !self.initialized {
            self.renderer.new_context_created(gl);
            self.initialized = true;
        }

        if self.renderer.should_continue() {
            self.renderer.render(gl, width, height);
        }
    }

    /// Called when the viewport is resized.
    pub fn on_resize(&mut self, gl: &glow::Context, width: u32, height: u32) {
        self.renderer.viewport_resized(gl, width, height);
    }

    /// Called when the context is being torn down.
    pub fn on_shutdown(&mut self, gl: &glow::Context) {
        if self.initialized {
            self.renderer.context_closing(gl);
            self.initialized = false;
        }
    }
}

impl<R: OpenGLRenderer> Drop for RenderLoopDriver<R> {
    fn drop(&mut self) {
        // Note: context_closing() should be called explicitly before dropping
        // while the GL context is current. This Drop impl just resets state.
        self.initialized = false;
    }
}

impl<R: OpenGLRenderer> std::fmt::Debug for RenderLoopDriver<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderLoopDriver")
            .field("initialized", &self.initialized)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRenderer {
        context_created_count: u32,
        render_count: u32,
        closing_count: u32,
    }

    impl TestRenderer {
        fn new() -> Self {
            Self { context_created_count: 0, render_count: 0, closing_count: 0 }
        }
    }

    impl OpenGLRenderer for TestRenderer {
        fn new_context_created(&mut self, _gl: &glow::Context) {
            self.context_created_count += 1;
        }

        fn render(&mut self, _gl: &glow::Context, _w: u32, _h: u32) {
            self.render_count += 1;
        }

        fn context_closing(&mut self, _gl: &glow::Context) {
            self.closing_count += 1;
        }
    }

    struct CountingRenderer(u32);

    impl OpenGLRenderer for CountingRenderer {
        fn new_context_created(&mut self, _gl: &glow::Context) {}
        fn render(&mut self, _gl: &glow::Context, _w: u32, _h: u32) {
            self.0 += 1;
        }
        fn context_closing(&mut self, _gl: &glow::Context) {}
        fn should_continue(&self) -> bool {
            self.0 < 5
        }
    }

    // Note: These tests verify trait logic and driver state management.
    // Full GL context tests would need a headless GL setup.

    #[test]
    fn renderer_trait_object_safety() {
        // Verify OpenGLRenderer can be used as a trait object.
        let _dyn_check: &dyn OpenGLRenderer = &TestRenderer::new();
    }

    #[test]
    fn render_loop_driver_debug() {
        let driver = RenderLoopDriver::new(TestRenderer::new());
        let dbg = format!("{:?}", driver);
        assert!(dbg.contains("RenderLoopDriver"));
        assert!(dbg.contains("false")); // initialized
    }

    #[test]
    fn render_loop_driver_into_renderer() {
        let mut r = TestRenderer::new();
        r.context_created_count = 42;
        let driver = RenderLoopDriver::new(r);
        let r2 = driver.into_renderer(None);
        assert_eq!(r2.context_created_count, 42);
    }

    #[test]
    fn render_loop_driver_accessors() {
        let driver = RenderLoopDriver::new(TestRenderer::new());
        assert_eq!(driver.renderer().render_count, 0);
    }

    #[test]
    fn render_loop_driver_mutable_access() {
        let mut driver = RenderLoopDriver::new(TestRenderer::new());
        driver.renderer_mut().render_count = 99;
        assert_eq!(driver.renderer().render_count, 99);
    }

    #[test]
    fn should_continue_default_true() {
        struct DefaultRenderer;
        impl OpenGLRenderer for DefaultRenderer {
            fn new_context_created(&mut self, _gl: &glow::Context) {}
            fn render(&mut self, _gl: &glow::Context, _w: u32, _h: u32) {}
            fn context_closing(&mut self, _gl: &glow::Context) {}
        }
        let r = DefaultRenderer;
        assert!(r.should_continue());
    }

    #[test]
    fn viewport_resized_default_noop() {
        struct DefaultRenderer;
        impl OpenGLRenderer for DefaultRenderer {
            fn new_context_created(&mut self, _gl: &glow::Context) {}
            fn render(&mut self, _gl: &glow::Context, _w: u32, _h: u32) {}
            fn context_closing(&mut self, _gl: &glow::Context) {}
        }
        let mut r = DefaultRenderer;
        // Should not panic — default implementation is a no-op.
        // We can't call viewport_resized without a real glow::Context, but
        // the default impl doesn't use it.
    }
}
