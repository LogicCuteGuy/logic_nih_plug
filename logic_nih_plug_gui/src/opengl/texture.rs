//! Managed OpenGL texture with upload, bind, and lifecycle helpers.

use glow::HasContext as _;

/// Pixel data type for texture uploads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureDataType {
    /// Unsigned 8-bit per channel (most common for RGBA images).
    UnsignedByte,
    /// 32-bit floating point per channel (HDR / float textures).
    Float32,
}

impl TextureDataType {
    /// Returns the `glow` pixel data type constant.
    pub fn glow_type(self) -> u32 {
        match self {
            Self::UnsignedByte => glow::UNSIGNED_BYTE,
            Self::Float32 => glow::FLOAT,
        }
    }
}

/// Internal texture format (how OpenGL stores texels on the GPU).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    /// 8-bit RGBA (most common).
    Rgba8,
    /// 8-bit RGB (no alpha).
    Rgb8,
    /// 32-bit float per channel, 4 channels.
    Rgba32F,
    /// 32-bit float per channel, 1 channel (e.g. depth or single-channel data).
    R32F,
    /// 16-bit depth + 8-bit stencil.
    Depth24Stencil8,
}

impl TextureFormat {
    /// Returns the `glow` internal format constant.
    pub fn glow_internal(self) -> i32 {
        match self {
            Self::Rgba8 => glow::RGBA as i32,
            Self::Rgb8 => glow::RGB as i32,
            Self::Rgba32F => glow::RGBA32F as i32,
            Self::R32F => glow::R32F as i32,
            Self::Depth24Stencil8 => glow::DEPTH24_STENCIL8 as i32,
        }
    }

    /// Returns the `glow` pixel format constant (the channel layout).
    pub fn glow_format(self) -> u32 {
        match self {
            Self::Rgba8 | Self::Rgba32F => glow::RGBA,
            Self::Rgb8 => glow::RGB,
            Self::R32F => glow::RED,
            Self::Depth24Stencil8 => glow::DEPTH_STENCIL,
        }
    }

    /// Returns the default `TextureDataType` for this format.
    pub fn default_data_type(self) -> TextureDataType {
        match self {
            Self::Rgba8 | Self::Rgb8 | Self::Depth24Stencil8 => TextureDataType::UnsignedByte,
            Self::Rgba32F | Self::R32F => TextureDataType::Float32,
        }
    }

    /// Bytes per pixel for this format (used for stride calculations).
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            Self::Rgba8 => 4,
            Self::Rgb8 => 3,
            Self::Rgba32F => 16,
            Self::R32F => 4,
            Self::Depth24Stencil8 => 4,
        }
    }
}

/// Minification / magnification filter for textures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMinFilter {
    /// Nearest-neighbor (pixelated).
    Nearest,
    /// Bilinear interpolation.
    Linear,
    /// Nearest + mipmap nearest.
    NearestMipmapNearest,
    /// Bilinear + mipmap nearest.
    LinearMipmapNearest,
    /// Nearest + mipmap linear (trilinear).
    NearestMipmapLinear,
    /// Bilinear + mipmap linear (best quality trilinear).
    LinearMipmapLinear,
}

impl TextureMinFilter {
    /// Returns the `glow` filter constant.
    pub fn glow_filter(self) -> i32 {
        match self {
            Self::Nearest => glow::NEAREST as i32,
            Self::Linear => glow::LINEAR as i32,
            Self::NearestMipmapNearest => glow::NEAREST_MIPMAP_NEAREST as i32,
            Self::LinearMipmapNearest => glow::LINEAR_MIPMAP_NEAREST as i32,
            Self::NearestMipmapLinear => glow::NEAREST_MIPMAP_LINEAR as i32,
            Self::LinearMipmapLinear => glow::LINEAR_MIPMAP_LINEAR as i32,
        }
    }
}

/// A managed OpenGL texture.
///
/// Wraps a `glow::NativeTexture` and provides RAII cleanup (via manual `release()`)
/// plus upload / bind helpers. The texture is **not** automatically deleted on drop
/// because the GL context must be current for deletion; call [`release()`](OpenGLTexture::release)
/// with the context current, or let the context handle cleanup on destruction.
pub struct OpenGLTexture {
    pub(crate) texture: glow::NativeTexture,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: TextureFormat,
}

impl OpenGLTexture {
    /// Create a new texture with the given dimensions and format.
    ///
    /// Allocates GPU storage but does not fill it with data. Use [`upload()`](Self::upload)
    /// to fill with pixel data.
    pub fn new(
        gl: &glow::Context,
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> Result<Self, String> {
        let texture = unsafe {
            gl.create_texture()
                .map_err(|e| format!("Failed to create texture: {e}"))?
        };

        let tex = Self { texture, width, height, format };

        // Allocate storage
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format.glow_internal(),
                width as i32,
                height as i32,
                0,
                format.glow_format(),
                format.default_data_type().glow_type(),
                glow::PixelUnpackData::Slice(None),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Ok(tex)
    }

    /// Upload pixel data to this texture.
    ///
    /// `data` is a slice of raw bytes in the format matching this texture's
    /// internal format. The byte length should be at least
    /// `width * height * format.bytes_per_pixel()`.
    pub fn upload(
        &mut self,
        gl: &glow::Context,
        data: &[u8],
        data_type: TextureDataType,
    ) -> Result<(), String> {
        let expected = (self.width * self.height) as usize * self.format.bytes_per_pixel();
        if data.len() < expected {
            return Err(format!(
                "Upload data too small: got {} bytes, need {} ({w}×{h}×{bpp})",
                data.len(),
                expected,
                w = self.width,
                h = self.height,
                bpp = self.format.bytes_per_pixel(),
            ));
        }

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                0,
                0,
                self.width as i32,
                self.height as i32,
                self.format.glow_format(),
                data_type.glow_type(),
                glow::PixelUnpackData::Slice(Some(data)),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Ok(())
    }

    /// Upload pixel data using the format's default data type.
    pub fn upload_default(
        &mut self,
        gl: &glow::Context,
        data: &[u8],
    ) -> Result<(), String> {
        self.upload(gl, data, self.format.default_data_type())
    }

    /// Bind this texture to the given texture unit (0 = `GL_TEXTURE0`).
    pub fn bind(&self, gl: &glow::Context, unit: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
        }
    }

    /// Unbind this texture from its current texture unit.
    pub fn unbind(&self, gl: &glow::Context, unit: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Set the minification filter.
    pub fn set_min_filter(&self, gl: &glow::Context, filter: TextureMinFilter) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                filter.glow_filter(),
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Set the magnification filter (`NEAREST` or `LINEAR`).
    pub fn set_mag_filter(&self, gl: &glow::Context, linear: bool) {
        let filter = if linear { glow::LINEAR } else { glow::NEAREST };
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                filter as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Set the wrapping mode for both axes.
    pub fn set_wrap_clamp_to_edge(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Set the wrapping mode to repeat.
    pub fn set_wrap_repeat(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::REPEAT as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::REPEAT as i32);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Generate mipmaps for this texture.
    pub fn generate_mipmaps(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Set common "default" parameters: linear filtering, clamp-to-edge.
    pub fn set_default_params(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Release (delete) this texture. The GL context **must** be current.
    pub fn release(self, gl: &glow::Context) {
        unsafe {
            gl.delete_texture(self.texture);
        }
        // Forget so Drop doesn't try to double-delete
        std::mem::forget(self);
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The internal format.
    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// Raw handle (use with care).
    pub fn native_texture(&self) -> glow::NativeTexture {
        self.texture
    }
}

impl Drop for OpenGLTexture {
    fn drop(&mut self) {
        // If release() was not called, we can't safely delete here because
        // the GL context may not be current. This is a deliberate trade-off:
        // the caller must call `release()` with the context current, or
        // accept the leak until context destruction cleans up.
    }
}

impl std::fmt::Debug for OpenGLTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenGLTexture")
            .field("texture", &self.texture.0.get())
            .field("width", &self.width)
            .field("height", &self.height)
            .field("format", &self.format)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_data_type_mapping() {
        assert_eq!(TextureDataType::UnsignedByte.glow_type(), glow::UNSIGNED_BYTE);
        assert_eq!(TextureDataType::Float32.glow_type(), glow::FLOAT);
    }

    #[test]
    fn texture_format_internal() {
        assert_eq!(TextureFormat::Rgba8.glow_internal(), glow::RGBA as i32);
        assert_eq!(TextureFormat::Rgb8.glow_internal(), glow::RGB as i32);
        assert_eq!(TextureFormat::Rgba32F.glow_internal(), glow::RGBA32F as i32);
        assert_eq!(TextureFormat::R32F.glow_internal(), glow::R32F as i32);
    }

    #[test]
    fn texture_format_bytes_per_pixel() {
        assert_eq!(TextureFormat::Rgba8.bytes_per_pixel(), 4);
        assert_eq!(TextureFormat::Rgb8.bytes_per_pixel(), 3);
        assert_eq!(TextureFormat::Rgba32F.bytes_per_pixel(), 16);
        assert_eq!(TextureFormat::R32F.bytes_per_pixel(), 4);
    }

    #[test]
    fn texture_format_default_data_type() {
        assert_eq!(TextureFormat::Rgba8.default_data_type(), TextureDataType::UnsignedByte);
        assert_eq!(TextureFormat::Rgba32F.default_data_type(), TextureDataType::Float32);
    }

    #[test]
    fn texture_min_filter_mapping() {
        assert_eq!(TextureMinFilter::Nearest.glow_filter(), glow::NEAREST as i32);
        assert_eq!(TextureMinFilter::Linear.glow_filter(), glow::LINEAR as i32);
        assert_eq!(
            TextureMinFilter::LinearMipmapLinear.glow_filter(),
            glow::LINEAR_MIPMAP_LINEAR as i32
        );
    }

    #[test]
    fn texture_debug_format() {
        // Verify Debug compiles and shows key info.
        let tex = unsafe {
            OpenGLTexture {
                texture: glow::NativeTexture(std::num::NonZeroU32::new(99).unwrap()),
                width: 64,
                height: 32,
                format: TextureFormat::Rgba8,
            }
        };
        let dbg = format!("{:?}", tex);
        assert!(dbg.contains("64"));
        assert!(dbg.contains("32"));
        assert!(dbg.contains("Rgba8"));
    }

    #[test]
    fn texture_format_debug() {
        assert_eq!(format!("{:?}", TextureFormat::Rgba8), "Rgba8");
        assert_eq!(format!("{:?}", TextureFormat::Rgb8), "Rgb8");
        assert_eq!(format!("{:?}", TextureFormat::Rgba32F), "Rgba32F");
    }

    #[test]
    fn texture_data_type_debug() {
        assert_eq!(format!("{:?}", TextureDataType::UnsignedByte), "UnsignedByte");
        assert_eq!(format!("{:?}", TextureDataType::Float32), "Float32");
    }

    #[test]
    fn texture_min_filter_debug() {
        assert_eq!(format!("{:?}", TextureMinFilter::Nearest), "Nearest");
        assert_eq!(format!("{:?}", TextureMinFilter::Linear), "Linear");
    }

    #[test]
    fn texture_format_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        TextureFormat::Rgba8.hash(&mut h1);
        TextureFormat::Rgb8.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn texture_min_filter_clone() {
        let f = TextureMinFilter::LinearMipmapLinear;
        let f2 = f;
        assert_eq!(f, f2);
    }
}
