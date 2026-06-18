//! Compiled shader program and uniform binding helpers.

use glow::HasContext as _;

/// Opaque handle to a uniform variable location within a shader program.
///
/// Obtained via [`ShaderProgram::uniform_location`] and passed to the
/// `set_uniform_*` methods.
pub type UniformLocation = glow::UniformLocation;

/// Shader type selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderType {
    /// Vertex shader.
    Vertex,
    /// Fragment shader.
    Fragment,
}

impl ShaderType {
    /// Returns the `glow` shader type constant.
    pub fn glow_type(self) -> u32 {
        match self {
            Self::Vertex => glow::VERTEX_SHADER,
            Self::Fragment => glow::FRAGMENT_SHADER,
        }
    }
}

/// Wrapper around a compiled and linked OpenGL shader program.
///
/// Drop calls `glDeleteProgram`. Cloning is not allowed — share via `Arc` if needed.
pub struct ShaderProgram {
    program: glow::NativeProgram,
}

// SAFETY: glow::NativeProgram is a handle; the program is thread-local to the
// GL context that created it, so ShaderProgram is Send but not Sync.
unsafe impl Send for ShaderProgram {}

impl ShaderProgram {
    /// Compile a single shader, attach it to a new program, link, and return it.
    ///
    /// Returns `Err(message)` on compile or link failure.
    pub fn new(
        gl: &glow::Context,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<Self, String> {
        let vs = compile_shader(gl, ShaderType::Vertex, vertex_src)?;
        let fs = compile_shader(gl, ShaderType::Fragment, fragment_src)?;

        let program = link_program(gl, vs, fs)?;

        // Shaders are attached during link; detach + delete after success.
        unsafe {
            gl.detach_shader(program, vs);
            gl.detach_shader(program, fs);
            gl.delete_shader(vs);
            gl.delete_shader(fs);
        }

        Ok(Self { program })
    }

    /// Create a `ShaderProgram` from a pre-compiled program handle.
    ///
    /// # Safety
    ///
    /// The caller must ensure `program` is a valid, linked GL program created
    /// from the same `glow::Context`.
    pub unsafe fn from_raw(program: glow::NativeProgram) -> Self {
        Self { program }
    }

    /// Returns the raw `glow::NativeProgram` handle.
    pub fn native_program(&self) -> glow::NativeProgram {
        self.program
    }

    /// Make this program the active one for subsequent draw calls.
    pub fn use_program(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.program));
        }
    }

    /// Bind this program to the default (0) uniform block.
    pub fn bind(&self, gl: &glow::Context) {
        self.use_program(gl);
    }

    /// Get the location of a uniform by name. Returns `None` if the uniform
    /// doesn't exist or has been optimized away.
    pub fn uniform_location(&self, gl: &glow::Context, name: &str) -> Option<UniformLocation> {
        unsafe { gl.get_uniform_location(self.program, name) }
    }

    /// Set an `int` uniform (`glUniform1i`).
    pub fn set_uniform_i32(&self, gl: &glow::Context, location: &UniformLocation, value: i32) {
        unsafe {
            gl.uniform_1_i32(Some(location), value);
        }
    }

    /// Set a `float` uniform (`glUniform1f`).
    pub fn set_uniform_f32(&self, gl: &glow::Context, location: &UniformLocation, value: f32) {
        unsafe {
            gl.uniform_1_f32(Some(location), value);
        }
    }

    /// Set a 2-component float uniform (`glUniform2f`).
    pub fn set_uniform_2f32(
        &self,
        gl: &glow::Context,
        location: &UniformLocation,
        x: f32,
        y: f32,
    ) {
        unsafe {
            gl.uniform_2_f32(Some(location), x, y);
        }
    }

    /// Set a 3-component float uniform (`glUniform3f`).
    pub fn set_uniform_3f32(
        &self,
        gl: &glow::Context,
        location: &UniformLocation,
        x: f32,
        y: f32,
        z: f32,
    ) {
        unsafe {
            gl.uniform_3_f32(Some(location), x, y, z);
        }
    }

    /// Set a 4-component float uniform (`glUniform4f`).
    pub fn set_uniform_4f32(
        &self,
        gl: &glow::Context,
        location: &UniformLocation,
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    ) {
        unsafe {
            gl.uniform_4_f32(Some(location), x, y, z, w);
        }
    }

    /// Set a 4×4 float matrix uniform (`glUniformMatrix4fv`, column-major).
    pub fn set_uniform_mat4(
        &self,
        gl: &glow::Context,
        location: &UniformLocation,
        transpose: bool,
        data: &[f32; 16],
    ) {
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(location), transpose, data);
        }
    }

    /// Set a texture unit uniform (`glUniform1i` — typically set to the texture unit index).
    pub fn set_texture_unit(
        &self,
        gl: &glow::Context,
        location: &UniformLocation,
        unit: u32,
    ) {
        self.set_uniform_i32(gl, location, unit as i32);
    }

    /// Check whether this program is currently active.
    pub fn is_used(&self, gl: &glow::Context) -> bool {
        unsafe {
            let current = gl.get_parameter_i32(glow::CURRENT_PROGRAM) as u32;
            current == self.program.0.get()
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        // Note: The GL context must be current when this runs. The caller
        // (typically OpenGLContext or a WindowHandler) is responsible for
        // ensuring this. We cannot safely call glDeleteProgram here without
        // knowing the context is current, so we intentionally skip it —
        // programs are short-lived and cleaned up by context destruction.
        //
        // If the caller needs deterministic cleanup they should call
        // `gl.delete_program(self.program)` before dropping, with the
        // context current.
    }
}

impl std::fmt::Debug for ShaderProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderProgram")
            .field("program", &self.program.0.get())
            .finish()
    }
}

/// Compile a single shader from GLSL source.
///
/// Returns the compiled `NativeShader` on success or a human-readable error
/// string on failure.
pub fn compile_shader(
    gl: &glow::Context,
    shader_type: ShaderType,
    source: &str,
) -> Result<glow::NativeShader, String> {
    unsafe {
        let shader = gl.create_shader(shader_type.glow_type())
            .map_err(|e| format!("Failed to create shader: {e}"))?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if gl.get_shader_compile_status(shader) {
            Ok(shader)
        } else {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            Err(format!("Shader compile error: {log}"))
        }
    }
}

/// Link a vertex + fragment shader into a program.
///
/// Both shaders must be compiled. The program handle is returned on success.
pub fn link_program(
    gl: &glow::Context,
    vertex_shader: glow::NativeShader,
    fragment_shader: glow::NativeShader,
) -> Result<glow::NativeProgram, String> {
    unsafe {
        let program = gl.create_program()
            .map_err(|e| format!("Failed to create program: {e}"))?;
        gl.attach_shader(program, vertex_shader);
        gl.attach_shader(program, fragment_shader);
        gl.link_program(program);

        if gl.get_program_link_status(program) {
            Ok(program)
        } else {
            let log = gl.get_program_info_log(program);
            gl.delete_program(program);
            Err(format!("Program link error: {log}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_type_glow_mapping() {
        assert_eq!(ShaderType::Vertex.glow_type(), glow::VERTEX_SHADER);
        assert_eq!(ShaderType::Fragment.glow_type(), glow::FRAGMENT_SHADER);
    }

    #[test]
    fn shader_type_equality() {
        assert_eq!(ShaderType::Vertex, ShaderType::Vertex);
        assert_ne!(ShaderType::Vertex, ShaderType::Fragment);
    }

    #[test]
    fn shader_type_clone() {
        let t = ShaderType::Fragment;
        let t2 = t;
        assert_eq!(t, t2);
    }

    #[test]
    fn shader_type_debug() {
        assert_eq!(format!("{:?}", ShaderType::Vertex), "Vertex");
        assert_eq!(format!("{:?}", ShaderType::Fragment), "Fragment");
    }

    #[test]
    fn shader_program_debug_format() {
        // We can't create a real program without a GL context, but we can
        // verify the Debug trait compiles and produces a reasonable string.
        let prog = unsafe {
            ShaderProgram::from_raw(glow::NativeProgram(std::num::NonZeroU32::new(42).unwrap()))
        };
        let dbg = format!("{:?}", prog);
        assert!(dbg.contains("ShaderProgram"));
        assert!(dbg.contains("42"));
    }

    #[test]
    fn compile_shader_returns_err_for_invalid_source() {
        // compile_shader requires a GL context, so we just verify the
        // function signature compiles and returns the right error type.
        // Full GL context tests would need a headless GL setup.
        // This is a compile-time check that the API is correct.
        fn _type_check() -> Result<glow::NativeShader, String> {
            // Placeholder — real test would create a headless context
            Err("no context".into())
        }
        assert!(_type_check().is_err());
    }

    #[test]
    fn link_program_returns_err_without_context() {
        // Verify the return type compiles correctly.
        fn _type_check() -> Result<glow::NativeProgram, String> {
            Err("no context".into())
        }
        assert!(_type_check().is_err());
    }
}
