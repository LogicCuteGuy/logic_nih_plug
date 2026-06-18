//! OpenGL framebuffer object (FBO) for offscreen rendering.

use glow::HasContext as _;

use super::texture::{TextureFormat, OpenGLTexture};

/// A managed OpenGL framebuffer object with color and optional depth/stencil
/// attachments.
///
/// Provides offscreen rendering: bind the FBO, draw into it, then read back
/// the color attachment or use it as a texture input for another pass.
pub struct OpenGLFrameBuffer {
    fbo: glow::NativeFramebuffer,
    color_attachment: OpenGLTexture,
    depth_stencil_attachment: Option<glow::NativeRenderbuffer>,
    width: u32,
    height: u32,
}

impl OpenGLFrameBuffer {
    /// Create a new FBO with the given dimensions and color format.
    ///
    /// `with_depth_stencil` adds a `DEPTH24_STENCIL8` renderbuffer attachment
    /// (needed for depth testing / stencil operations).
    pub fn new(
        gl: &glow::Context,
        width: u32,
        height: u32,
        color_format: TextureFormat,
        with_depth_stencil: bool,
    ) -> Result<Self, String> {
        let fbo = unsafe {
            gl.create_framebuffer()
                .map_err(|e| format!("Failed to create framebuffer: {e}"))?
        };

        // Create and attach the color texture
        let color_attachment = OpenGLTexture::new(gl, width, height, color_format)?;
        color_attachment.set_default_params(gl);

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(color_attachment.native_texture()),
                0,
            );
        }

        // Optionally create a depth/stencil renderbuffer
        let depth_stencil_attachment = if with_depth_stencil {
            let rbo = unsafe {
                gl.create_renderbuffer()
                    .map_err(|e| format!("Failed to create renderbuffer: {e}"))?
            };
            unsafe {
                gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
                gl.renderbuffer_storage(
                    glow::RENDERBUFFER,
                    glow::DEPTH24_STENCIL8,
                    width as i32,
                    height as i32,
                );
                gl.framebuffer_renderbuffer(
                    glow::FRAMEBUFFER,
                    glow::DEPTH_STENCIL_ATTACHMENT,
                    glow::RENDERBUFFER,
                    Some(rbo),
                );
            }
            Some(rbo)
        } else {
            None
        };

        // Check completeness
        let status = unsafe {
            let s = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            s
        };

        if status != glow::FRAMEBUFFER_COMPLETE {
            let reason = match status {
                glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "INCOMPLETE_ATTACHMENT",
                glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "INCOMPLETE_MISSING_ATTACHMENT",
                glow::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "INCOMPLETE_DRAW_BUFFER",
                glow::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "INCOMPLETE_READ_BUFFER",
                glow::FRAMEBUFFER_UNSUPPORTED => "UNSUPPORTED",
                glow::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "INCOMPLETE_MULTISAMPLE",
                _ => "UNKNOWN",
            };
            return Err(format!("Framebuffer incomplete: {reason} (0x{status:04X})"));
        }

        Ok(Self {
            fbo,
            color_attachment,
            depth_stencil_attachment,
            width,
            height,
        })
    }

    /// Bind this FBO as the current render target.
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.viewport(0, 0, self.width as i32, self.height as i32);
        }
    }

    /// Unbind this FBO, restoring the default (screen) framebuffer.
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    /// Bind the color attachment as a texture to the given texture unit.
    pub fn bind_as_texture(&self, gl: &glow::Context, unit: u32) {
        self.color_attachment.bind(gl, unit);
    }

    /// Read back the color attachment pixels into a byte buffer.
    ///
    /// Returns `RGBA` data with one byte per channel (the underlying texture
    /// format must be `Rgba8` for correct results).
    pub fn read_pixels(&self, gl: &glow::Context) -> Vec<u8> {
        let size = (self.width * self.height * 4) as usize;
        let mut data = vec![0u8; size];
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            gl.read_pixels(
                0,
                0,
                self.width as i32,
                self.height as i32,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut data)),
            );
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
        data
    }

    /// Resize the FBO (recreates color texture and optional renderbuffer).
    pub fn resize(
        &mut self,
        gl: &glow::Context,
        width: u32,
        height: u32,
    ) -> Result<(), String> {
        if width == self.width && height == self.height {
            return Ok(());
        }

        let format = self.color_attachment.format();
        let has_depth = self.depth_stencil_attachment.is_some();

        // Release old resources
        self.release_internal(gl);

        // Recreate
        let new = Self::new(gl, width, height, format, has_depth)?;
        *self = new;
        Ok(())
    }

    /// Release all GPU resources. The GL context must be current.
    pub fn release(&mut self, gl: &glow::Context) {
        self.release_internal(gl);
    }

    fn release_internal(&self, gl: &glow::Context) {
        unsafe {
            if let Some(rbo) = self.depth_stencil_attachment {
                gl.delete_renderbuffer(rbo);
            }
            gl.delete_texture(self.color_attachment.native_texture());
            gl.delete_framebuffer(self.fbo);
        }
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Reference to the color attachment texture.
    pub fn color_texture(&self) -> &OpenGLTexture {
        &self.color_attachment
    }

    /// Mutable reference to the color attachment texture.
    pub fn color_texture_mut(&mut self) -> &mut OpenGLTexture {
        &mut self.color_attachment
    }

    /// Raw FBO handle.
    pub fn native_framebuffer(&self) -> glow::NativeFramebuffer {
        self.fbo
    }
}

impl Drop for OpenGLFrameBuffer {
    fn drop(&mut self) {
        // Same caveat as OpenGLTexture: caller should call release() with
        // context current, or let context destruction handle cleanup.
    }
}

impl std::fmt::Debug for OpenGLFrameBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenGLFrameBuffer")
            .field("fbo", &self.fbo.0.get())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("has_depth_stencil", &self.depth_stencil_attachment.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framebuffer_debug_format() {
        // Verify Debug compiles and shows key info (no GL context needed).
        let fbo = OpenGLFrameBuffer {
            fbo: glow::NativeFramebuffer(std::num::NonZeroU32::new(77).unwrap()),
            color_attachment: unsafe {
                OpenGLTexture {
                    texture: glow::NativeTexture(std::num::NonZeroU32::new(88).unwrap()),
                    width: 512,
                    height: 256,
                    format: TextureFormat::Rgba8,
                }
            },
            depth_stencil_attachment: None,
            width: 512,
            height: 256,
        };
        let dbg = format!("{:?}", fbo);
        assert!(dbg.contains("512"));
        assert!(dbg.contains("256"));
        assert!(dbg.contains("false")); // has_depth_stencil
    }

    #[test]
    fn framebuffer_dimensions() {
        let fbo = OpenGLFrameBuffer {
            fbo: glow::NativeFramebuffer(std::num::NonZeroU32::new(1).unwrap()),
            color_attachment: unsafe {
                OpenGLTexture {
                    texture: glow::NativeTexture(std::num::NonZeroU32::new(2).unwrap()),
                    width: 1920,
                    height: 1080,
                    format: TextureFormat::Rgba8,
                }
            },
            depth_stencil_attachment: None,
            width: 1920,
            height: 1080,
        };
        assert_eq!(fbo.width(), 1920);
        assert_eq!(fbo.height(), 1080);
        assert!(fbo.depth_stencil_attachment.is_none());
    }
}
