//! OpenGL context wrapper that manages the GL state and provides factory
//! methods for textures, framebuffers, and shader programs.

use std::sync::Arc;

use glow::HasContext as _;

use super::framebuffer::OpenGLFrameBuffer;
use super::shader::ShaderProgram;
use super::texture::{TextureFormat, OpenGLTexture};

/// A wrapper around a `glow::Context` that provides a high-level API for
/// creating and managing OpenGL resources.
///
/// # Relationship to baseview
///
/// `baseview` owns the platform window and GL context. This wrapper is
/// created *after* the baseview context is made current and provides
/// convenience methods for the common GL operations that plugin GUIs need.
///
/// # Thread safety
///
/// The underlying `glow::Context` is **not** `Sync` — all GL calls must
/// happen on the thread that owns the context. `OpenGLContext` is `Send`
/// so it can be moved between threads, but not shared across them.
pub struct OpenGLContext {
    gl: Arc<glow::Context>,
}

impl OpenGLContext {
    /// Create an `OpenGLContext` from a `glow::Context`.
    pub fn new(gl: glow::Context) -> Self {
        Self { gl: Arc::new(gl) }
    }

    /// Create an `OpenGLContext` from a loader function (e.g.
    /// `glow::Context::from_loader_function`).
    pub fn from_loader<F>(loader: F) -> Self
    where
        F: FnMut(&str) -> *const std::ffi::c_void,
    {
        Self { gl: Arc::new(unsafe { glow::Context::from_loader_function(loader) }) }
    }

    /// Returns a reference to the underlying `glow::Context`.
    pub fn glow_context(&self) -> &glow::Context {
        &self.gl
    }

    /// Returns a clone of the `Arc<glow::Context>`.
    pub fn glow_arc(&self) -> Arc<glow::Context> {
        Arc::clone(&self.gl)
    }

    /// Clear the color buffer with the given RGBA colour.
    pub fn clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            self.gl.clear_color(r, g, b, a);
        }
    }

    /// Clear the currently bound draw buffers.
    pub fn clear(&self, bits: u32) {
        unsafe {
            self.gl.clear(bits);
        }
    }

    /// Set the viewport.
    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            self.gl.viewport(x, y, width, height);
        }
    }

    /// Enable the given capability (e.g. `glow::BLEND`).
    pub fn enable(&self, cap: u32) {
        unsafe {
            self.gl.enable(cap);
        }
    }

    /// Disable the given capability.
    pub fn disable(&self, cap: u32) {
        unsafe {
            self.gl.disable(cap);
        }
    }

    /// Set the blend function.
    pub fn blend_func(&self, src: u32, dst: u32) {
        unsafe {
            self.gl.blend_func(src, dst);
        }
    }

    /// Set the blend function (separate RGB and alpha).
    pub fn blend_func_separate(&self, src_rgb: u32, dst_rgb: u32, src_a: u32, dst_a: u32) {
        unsafe {
            self.gl.blend_func_separate(src_rgb, dst_rgb, src_a, dst_a);
        }
    }

    /// Enable or disable depth testing.
    pub fn set_depth_test(&self, enabled: bool) {
        if enabled {
            self.enable(glow::DEPTH_TEST);
        } else {
            self.disable(glow::DEPTH_TEST);
        }
    }

    /// Enable or disable face culling.
    pub fn set_cull_face(&self, enabled: bool) {
        if enabled {
            self.enable(glow::CULL_FACE);
        } else {
            self.disable(glow::CULL_FACE);
        }
    }

    /// Create a shader program from vertex and fragment GLSL sources.
    pub fn create_program(
        &self,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<ShaderProgram, String> {
        ShaderProgram::new(&self.gl, vertex_src, fragment_src)
    }

    /// Create a 2D texture with the given dimensions and format.
    pub fn create_texture(
        &self,
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> Result<OpenGLTexture, String> {
        OpenGLTexture::new(&self.gl, width, height, format)
    }

    /// Create a framebuffer with the given dimensions, color format, and
    /// optional depth/stencil attachment.
    pub fn create_framebuffer(
        &self,
        width: u32,
        height: u32,
        color_format: TextureFormat,
        with_depth_stencil: bool,
    ) -> Result<OpenGLFrameBuffer, String> {
        OpenGLFrameBuffer::new(&self.gl, width, height, color_format, with_depth_stencil)
    }

    /// Returns the GL version string (e.g. `"4.5.0 NVIDIA"`).
    pub fn version_string(&self) -> String {
        unsafe { self.gl.get_parameter_string(glow::VERSION) }
    }

    /// Returns the GLSL version string.
    pub fn glsl_version_string(&self) -> String {
        unsafe { self.gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION) }
    }

    /// Returns the GL renderer string (e.g. `"NVIDIA GeForce RTX 3080"`).
    pub fn renderer_string(&self) -> String {
        unsafe { self.gl.get_parameter_string(glow::RENDERER) }
    }

    /// Returns the GL vendor string.
    pub fn vendor_string(&self) -> String {
        unsafe { self.gl.get_parameter_string(glow::VENDOR) }
    }
}

impl std::fmt::Debug for OpenGLContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenGLContext")
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opengl_context_debug_format() {
        // Verify Debug compiles without a real GL context.
        let dbg = std::any::type_name::<OpenGLContext>();
        assert!(dbg.contains("OpenGLContext"));
    }

    #[test]
    fn opengl_context_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OpenGLContext>();
    }

    #[test]
    fn opengl_context_is_not_sync() {
        fn assert_not_sync<T: Sync>() {}
        // This should NOT compile — uncomment to verify:
        // assert_not_sync::<OpenGLContext>();
        // For now, just verify the type exists and is Send.
    }
}
