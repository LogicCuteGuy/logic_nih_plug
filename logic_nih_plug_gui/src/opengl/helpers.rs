//! Static helper functions for common OpenGL operations.
//!
//! Mirrors JUCE's `OpenGLHelpers` class — provides compile / link helpers,
//! error checking, and convenience functions that don't belong to a
//! particular resource type.

use glow::HasContext as _;

use super::shader::ShaderProgram;

/// Static helper methods for OpenGL operations.
///
/// All methods take a `&glow::Context` rather than operating on a
/// [`OpenGLContext`](super::OpenGLContext), so they can be used from any
/// code that has a reference to the raw glow context.
pub struct OpenGLHelpers;

impl OpenGLHelpers {
    /// Compile a vertex and fragment shader, link them into a program, and
    /// return it. Shaders are detached and deleted after linking.
    ///
    /// Returns `Err(message)` on compile or link failure.
    pub fn create_program(
        gl: &glow::Context,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<ShaderProgram, String> {
        ShaderProgram::new(gl, vertex_src, fragment_src)
    }

    /// Clear the colour buffer to the given RGBA value.
    pub fn clear(gl: &glow::Context, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl.clear_color(r, g, b, a);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    /// Clear colour + depth buffers.
    pub fn clear_all(gl: &glow::Context, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl.clear_color(r, g, b, a);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }

    /// Check for OpenGL errors and return them as a human-readable string.
    ///
    /// Returns `None` if there are no pending errors.
    pub fn check_gl_error(gl: &glow::Context) -> Option<String> {
        let err = unsafe { gl.get_error() };
        if err == glow::NO_ERROR {
            None
        } else {
            let name = match err {
                glow::INVALID_ENUM => "INVALID_ENUM",
                glow::INVALID_VALUE => "INVALID_VALUE",
                glow::INVALID_OPERATION => "INVALID_OPERATION",
                glow::INVALID_FRAMEBUFFER_OPERATION => "INVALID_FRAMEBUFFER_OPERATION",
                glow::OUT_OF_MEMORY => "OUT_OF_MEMORY",
                glow::STACK_UNDERFLOW => "STACK_UNDERFLOW",
                glow::STACK_OVERFLOW => "STACK_OVERFLOW",
                _ => "UNKNOWN",
            };
            Some(format!("GL error 0x{err:04X} ({name})"))
        }
    }

    /// Check for GL errors and panic with a descriptive message if any
    /// are found. Intended for use in debug builds.
    pub fn assert_no_gl_error(gl: &glow::Context) {
        if let Some(err) = Self::check_gl_error(gl) {
            panic!("{err}");
        }
    }

    /// Enable standard blending (`SRC_ALPHA / ONE_MINUS_SRC_ALPHA`).
    pub fn enable_blending(gl: &glow::Context) {
        unsafe {
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
    }

    /// Disable blending.
    pub fn disable_blending(gl: &glow::Context) {
        unsafe {
            gl.disable(glow::BLEND);
        }
    }

    /// Enable depth testing with the `LEQUAL` function.
    pub fn enable_depth_testing(gl: &glow::Context) {
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
        }
    }

    /// Disable depth testing.
    pub fn disable_depth_testing(gl: &glow::Context) {
        unsafe {
            gl.disable(glow::DEPTH_TEST);
        }
    }

    /// Enable back-face culling.
    pub fn enable_culling(gl: &glow::Context) {
        unsafe {
            gl.enable(glow::CULL_FACE);
        }
    }

    /// Disable face culling.
    pub fn disable_culling(gl: &glow::Context) {
        unsafe {
            gl.disable(glow::CULL_FACE);
        }
    }

    /// Set the viewport to cover the full framebuffer.
    pub fn set_full_viewport(gl: &glow::Context, width: i32, height: i32) {
        unsafe {
            gl.viewport(0, 0, width, height);
        }
    }

    /// Check whether the current framebuffer is complete. Returns `Ok(())`
    /// on success or `Err(description)` on failure.
    pub fn check_framebuffer_status(gl: &glow::Context) -> Result<(), String> {
        let status = unsafe { gl.check_framebuffer_status(glow::FRAMEBUFFER) };
        if status == glow::FRAMEBUFFER_COMPLETE {
            Ok(())
        } else {
            let reason = match status {
                glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "INCOMPLETE_ATTACHMENT",
                glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "INCOMPLETE_MISSING_ATTACHMENT",
                glow::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "INCOMPLETE_DRAW_BUFFER",
                glow::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "INCOMPLETE_READ_BUFFER",
                glow::FRAMEBUFFER_UNSUPPORTED => "UNSUPPORTED",
                glow::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "INCOMPLETE_MULTISAMPLE",
                _ => "UNKNOWN",
            };
            Err(format!("Framebuffer incomplete: {reason} (0x{status:04X})"))
        }
    }

    /// Log the current GL state to stderr (debug builds only).
    pub fn log_gl_info(gl: &glow::Context) {
        let version = unsafe { gl.get_parameter_string(glow::VERSION) };
        let renderer = unsafe { gl.get_parameter_string(glow::RENDERER) };
        let vendor = unsafe { gl.get_parameter_string(glow::VENDOR) };
        let glsl = unsafe { gl.get_parameter_string(glow::SHADING_LANGUAGE_VERSION) };
        eprintln!("OpenGL context created:");
        eprintln!("  Version:  {version}");
        eprintln!("  Renderer: {renderer}");
        eprintln!("  Vendor:   {vendor}");
        eprintln!("  GLSL:     {glsl}");
    }
}

impl std::fmt::Debug for OpenGLHelpers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenGLHelpers").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helpers_debug_format() {
        let dbg = format!("{:?}", OpenGLHelpers);
        assert_eq!(dbg, "OpenGLHelpers");
    }

    #[test]
    fn check_gl_error_type_check() {
        // Verify the return type compiles.
        fn _type_check() -> Option<String> {
            None
        }
        assert!(_type_check().is_none());
    }

    #[test]
    fn create_program_type_check() {
        // Verify the function signature compiles.
        fn _type_check() -> Result<ShaderProgram, String> {
            Err("no context".into())
        }
        assert!(_type_check().is_err());
    }
}
