//! # OpenGL Utilities
//!
//! Reusable OpenGL abstractions ported from JUCE's `juce_opengl` module, built
//! on top of [`glow`](https://docs.rs/glow). These types wrap raw GL resources
//! (programs, textures, framebuffers) in safe Rust RAII handles and provide
//! helpers for shader compilation, uniform binding, and matrix math.
//!
//! ## Feature gate
//!
//! Everything in this module requires the `gl-editor` feature (which pulls in
//! `glow` + `baseview` with `opengl`).
//!
//! ## Quick start
//!
//! ```rust,no_run
//! # #[cfg(feature = "gl-editor")]
//! # {
//! use logic_nih_plug_gui::opengl::{ShaderProgram, OpenGLTexture, OpenGLHelpers};
//!
//! # unsafe fn example(gl: &glow::Context) {
//! // Compile & link a shader program
//! let program = OpenGLHelpers::create_program(gl, VERTEX_SRC, FRAGMENT_SRC)
//!     .expect("shader compile failed");
//!
//! // Upload a texture from RGBA pixel data
//! let mut tex = OpenGLTexture::new(gl, 256, 256).unwrap();
//! tex.upload(gl, &pixels, TextureFormat::Rgba8);
//! tex.bind(gl, 0);
//! # }
//! # }
//! ```

mod context;
mod framebuffer;
mod helpers;
mod matrix;
mod renderer;
mod shader;
mod texture;

pub use context::OpenGLContext;
pub use framebuffer::OpenGLFrameBuffer;
pub use helpers::OpenGLHelpers;
pub use matrix::{Matrix3D, Matrix4x4};
pub use renderer::OpenGLRenderer;
pub use shader::{ShaderProgram, ShaderType, UniformLocation};
pub use texture::{TextureDataType, TextureFormat, TextureMinFilter, OpenGLTexture};
