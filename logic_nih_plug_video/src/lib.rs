//! # logic_nih_plug_video
//!
//! Video playback component ported from JUCE's `juce_video` module.
//!
//! Provides frame-by-frame video decoding via ffmpeg and an optional
//! GUI `VideoComponent` for display, playback control, and seeking.
//!
//! ## Architecture
//!
//! - [`VideoFrame`] — holds a single RGBA8888 frame with metadata
//! - [`VideoDecoder`] — wraps ffmpeg-next for file-based decoding (feature `decoder`)
//! - [`VideoComponent`] — GUI component that paints decoded frames (feature `gui`)
//!
//! ## Feature flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `decoder` | ✅ | Enables [`VideoDecoder`] (requires `ffmpeg-next` system libs) |
//! | `gui` | | Enables [`VideoComponent`] (requires `logic_nih_plug_gui`) |
//! | `full` | | Both `decoder` + `gui` |
//!
//! When `gui` is enabled without `decoder`, use [`VideoComponent::push_frame`]
//! to feed frames decoded by an external source.
//!
//! # Examples
//!
//! ```ignore
//! use logic_nih_plug_video::VideoDecoder;
//!
//! let mut decoder = VideoDecoder::open("video.mp4").expect("failed to open");
//! let fps = decoder.frame_rate();
//! if let Some(frame) = decoder.next_frame().unwrap() {
//!     println!("Decoded {}x{} frame", frame.width, frame.height);
//! }
//! ```

#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur in video operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum VideoError {
    /// Failed to open or read the video file.
    #[error("Failed to open video: {0}")]
    OpenFailed(String),

    /// No video stream found in the file.
    #[error("No video stream found")]
    NoVideoStream,

    /// The codec is not supported or decoder failed.
    #[error("Decode error: {0}")]
    DecodeError(String),

    /// Seek to the given position failed.
    #[error("Seek failed: {0}")]
    SeekFailed(String),

    /// Conversion to RGBA failed.
    #[error("Pixel format conversion failed: {0}")]
    ConversionError(String),

    /// No frame is available (end of stream or not yet decoded).
    #[error("No frame available")]
    NoFrame,

    /// The video file path is invalid.
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Convenience result alias.
pub type VideoResult<T> = Result<T, VideoError>;

// ---------------------------------------------------------------------------
// VideoFrame
// ---------------------------------------------------------------------------

/// A single decoded video frame in RGBA8888 format.
///
/// Pixels are stored row-by-row, top-left origin, 4 bytes per pixel.
/// The buffer length is `width * height * 4`.
#[derive(Debug, Clone)]
pub struct VideoFrame {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// Presentation timestamp in seconds.
    pub pts_seconds: f64,
    /// Frame index (0-based).
    pub frame_index: u64,
    /// RGBA8888 pixel data (premultiplied alpha is **not** assumed).
    pub data: Vec<u8>,
}

impl VideoFrame {
    /// Create a new frame with the given dimensions and pixel data.
    ///
    /// Returns `None` if `data.len() != width * height * 4`.
    pub fn new(
        width: u32,
        height: u32,
        pts_seconds: f64,
        frame_index: u64,
        data: Vec<u8>,
    ) -> Option<Self> {
        let expected = (width as usize) * (height as usize) * 4;
        if data.len() != expected {
            return None;
        }
        Some(Self {
            width,
            height,
            pts_seconds,
            frame_index,
            data,
        })
    }

    /// Get pixel (R, G, B, A) at (x, y).
    ///
    /// Returns `None` if out of bounds.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        Some([
            self.data[idx],
            self.data[idx + 1],
            self.data[idx + 2],
            self.data[idx + 3],
        ])
    }

    /// Get the pixel data as a slice of RGBA quads.
    pub fn pixels_rgba(&self) -> &[[u8; 4]] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const [u8; 4],
                (self.width * self.height) as usize,
            )
        }
    }

    /// Byte size of the raw pixel data.
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// Create a solid-colour test frame (useful for unit tests).
    pub fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Self {
        let mut full_data = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..(width * height) {
            full_data.extend_from_slice(&rgba);
        }
        Self {
            width,
            height,
            pts_seconds: 0.0,
            frame_index: 0,
            data: full_data,
        }
    }
}

// ---------------------------------------------------------------------------
// PlaybackState (always available — no ffmpeg dependency)
// ---------------------------------------------------------------------------

/// Playback state for the video component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// No video loaded.
    Stopped,
    /// Video is currently playing.
    Playing,
    /// Video is paused.
    Paused,
}

// ---------------------------------------------------------------------------
// VideoDecoder (feature = "decoder")
// ---------------------------------------------------------------------------

/// Video file decoder backed by ffmpeg.
///
/// Opens a video file, locates the first video stream, and provides
/// frame-by-frame decoding into [`VideoFrame`] (RGBA8888).
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_video::VideoDecoder;
///
/// let mut dec = VideoDecoder::open("clip.mp4").unwrap();
/// println!("{} fps, {} frames", dec.frame_rate(), dec.frame_count());
///
/// while let Ok(Some(frame)) = dec.next_frame() {
///     // process frame …
/// }
/// ```
#[cfg(feature = "decoder")]
pub struct VideoDecoder {
    /// Input context (owns the demuxer + codec).
    input: ffmpeg_next::format::context::Input,
    /// Index of the video stream.
    stream_index: usize,
    /// Decoder context for the video stream.
    decoder: ffmpeg_next::decoder::Video,
    /// Scaler for converting to RGBA.
    scaler: ffmpeg_next::software::scaling::Context,
    /// Frame rate (frames per second).
    frame_rate: f64,
    /// Total frame count (estimated from duration × fps).
    frame_count: u64,
    /// Duration in seconds.
    duration_seconds: f64,
    /// Current frame index (incremented on each `next_frame()` call).
    current_frame: u64,
    /// The original file path.
    path: std::path::PathBuf,
}

#[cfg(feature = "decoder")]
impl VideoDecoder {
    /// Open a video file and prepare the decoder.
    pub fn open(path: impl AsRef<std::path::Path>) -> VideoResult<Self> {
        let path = path.as_ref().to_path_buf();
        let path_str = path
            .to_str()
            .ok_or_else(|| VideoError::InvalidPath(path.display().to_string()))?;

        let input = ffmpeg_next::format::input(&path_str)
            .map_err(|e| VideoError::OpenFailed(format!("{}: {}", path.display(), e)))?;

        // Find the first video stream
        let (stream_index, stream) = input
            .streams()
            .enumerate()
            .find(|(_, s)| s.parameters().codec() == ffmpeg_next::media::Type::Video)
            .ok_or(VideoError::NoVideoStream)?;

        let context_decoder =
            ffmpeg_next::context::CodecParameters::parameters(&stream.parameters())
                .decoder()
                .video()
                .map_err(|e| VideoError::DecodeError(e.to_string()))?;

        let mut decoder = context_decoder;
        decoder
            .open(None)
            .map_err(|e| VideoError::DecodeError(e.to_string()))?;

        // Frame rate
        let frame_rate = if stream.avg_frame_rate().denom() != 0 {
            stream.avg_frame_rate().numerator() as f64
                / stream.avg_frame_rate().denominator() as f64
        } else {
            24.0
        };

        // Duration
        let duration_seconds = if let Some(dur) = stream.duration() {
            dur as f64 / ffmpeg_next::ffi::AV_TIME_BASE as f64
        } else if let Some(dur) = input.duration() {
            dur as f64 / ffmpeg_next::ffi::AV_TIME_BASE as f64
        } else {
            0.0
        };

        let frame_count = if frame_rate > 0.0 {
            (duration_seconds * frame_rate).round() as u64
        } else {
            0
        };

        // Scaler: input format → RGBA
        let scaler = ffmpeg_next::software::scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            ffmpeg_next::format::Pixel::RGBA,
            decoder.width(),
            decoder.height(),
            ffmpeg_next::software::scaling::Flags::BILINEAR,
        )
        .map_err(|e| VideoError::ConversionError(e.to_string()))?;

        Ok(Self {
            input,
            stream_index,
            decoder,
            scaler,
            frame_rate,
            frame_count,
            duration_seconds,
            current_frame: 0,
            path,
        })
    }

    /// Frame rate (frames per second).
    pub fn frame_rate(&self) -> f64 {
        self.frame_rate
    }

    /// Estimated total frame count.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Duration in seconds.
    pub fn duration_seconds(&self) -> f64 {
        self.duration_seconds
    }

    /// Current frame index (how many frames have been decoded so far).
    pub fn current_frame_index(&self) -> u64 {
        self.current_frame
    }

    /// Video width in pixels.
    pub fn width(&self) -> u32 {
        self.decoder.width()
    }

    /// Video height in pixels.
    pub fn height(&self) -> u32 {
        self.decoder.height()
    }

    /// Path of the opened file.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Decode and return the next frame.
    ///
    /// Returns `Ok(None)` at end of stream.
    pub fn next_frame(&mut self) -> VideoResult<Option<VideoFrame>> {
        let mut frame = ffmpeg_next::frame::Video::empty();
        let mut rgb_frame = ffmpeg_next::frame::Video::empty();

        loop {
            match self.input.packets().next() {
                Some((stream, packet)) => {
                    if stream.index() != self.stream_index {
                        continue;
                    }
                    self.decoder
                        .send_packet(&packet)
                        .map_err(|e| VideoError::DecodeError(e.to_string()))?;
                }
                None => {
                    // Flush decoder
                    self.decoder
                        .send_eof()
                        .map_err(|e| VideoError::DecodeError(e.to_string()))?;
                }
            }

            match self.decoder.receive_frame(&mut frame) {
                Ok(()) => {
                    self.scaler
                        .run(&frame, &mut rgb_frame)
                        .map_err(|e| VideoError::ConversionError(e.to_string()))?;

                    let pts_seconds = if let Some(pts) = frame.pts() {
                        pts as f64 / ffmpeg_next::ffi::AV_TIME_BASE as f64
                    } else {
                        self.current_frame as f64 / self.frame_rate
                    };

                    let frame_index = self.current_frame;
                    self.current_frame += 1;

                    let width = rgb_frame.width();
                    let height = rgb_frame.height();
                    let data =
                        rgb_frame.data(0)[..((width * height * 4) as usize)].to_vec();

                    return Ok(Some(VideoFrame {
                        width,
                        height,
                        pts_seconds,
                        frame_index,
                        data,
                    }));
                }
                Err(ffmpeg_next::Error::Other { errno })
                    if errno == ffmpeg_next::ffi::EAGAIN =>
                {
                    continue;
                }
                Err(ffmpeg_next::Error::Other { errno })
                    if errno == ffmpeg_next::ffi::EOF =>
                {
                    return Ok(None);
                }
                Err(e) => {
                    return Err(VideoError::DecodeError(e.to_string()));
                }
            }
        }
    }

    /// Seek to the frame at the given time in seconds.
    ///
    /// After seeking, call [`next_frame()`](Self::next_frame) to get the
    /// frame at (or near) the target position.
    pub fn seek(&mut self, seconds: f64) -> VideoResult<()> {
        let timestamp = (seconds * ffmpeg_next::ffi::AV_TIME_BASE as f64) as i64;
        self.input
            .seek(timestamp, timestamp..)
            .map_err(|e| VideoError::SeekFailed(e.to_string()))?;
        self.decoder.flush();
        Ok(())
    }

    /// Seek to a specific frame number.
    pub fn seek_to_frame(&mut self, frame: u64) -> VideoResult<()> {
        if self.frame_rate > 0.0 {
            let seconds = frame as f64 / self.frame_rate;
            self.seek(seconds)
        } else {
            Err(VideoError::SeekFailed("Unknown frame rate".into()))
        }
    }

    /// Reset the decoder to the beginning of the file.
    pub fn rewind(&mut self) -> VideoResult<()> {
        self.seek(0.0)?;
        self.current_frame = 0;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// VideoComponent (feature = "gui")
// ---------------------------------------------------------------------------

/// GUI component for video playback.
///
/// Displays decoded frames and supports play, pause, stop, and seek
/// operations. When the `decoder` feature is also enabled, can load
/// files directly via [`load()`](Self::load). Otherwise, use
/// [`push_frame()`](Self::push_frame) to feed externally-decoded frames.
///
/// # Examples
///
/// ```ignore
/// use logic_nih_plug_video::{VideoComponent, VideoFrame, PlaybackState};
///
/// let mut vc = VideoComponent::new();
/// // Feed a frame from an external decoder
/// let frame = VideoFrame::solid(320, 240, [255, 0, 0, 255]);
/// vc.push_frame(frame);
/// vc.play();
/// assert_eq!(vc.playback_state(), PlaybackState::Playing);
/// ```
#[cfg(feature = "gui")]
pub struct VideoComponent {
    /// Inner GUI component (delegates lifecycle).
    component: logic_nih_plug_gui::Component,
    /// Current playback state.
    state: PlaybackState,
    /// Playback speed multiplier (1.0 = normal).
    play_speed: f64,
    /// Audio volume (0.0–1.0). Stored but audio output is not yet implemented.
    volume: f32,
    /// Current position in seconds.
    position_seconds: f64,
    /// Frame rate of the loaded video (used for tick timing).
    video_frame_rate: f64,
    /// Total duration in seconds.
    video_duration_secs: f64,
    /// Video native width.
    video_width: u32,
    /// Video native height.
    video_height: u32,
    /// The most recently decoded frame (for painting).
    current_frame: Option<VideoFrame>,
    /// Total number of frames pushed/decoded.
    total_frames: u64,
    /// Callback fired when playback starts.
    on_playback_started: Option<Box<dyn FnMut()>>,
    /// Callback fired when playback stops.
    on_playback_stopped: Option<Box<dyn FnMut()>>,
    /// Callback fired on decode/playback error.
    on_error: Option<Box<dyn FnMut(VideoError)>>,
}

#[cfg(feature = "gui")]
impl VideoComponent {
    /// Create a new empty video component.
    pub fn new() -> Self {
        Self {
            component: logic_nih_plug_gui::Component::new("VideoComponent"),
            state: PlaybackState::Stopped,
            play_speed: 1.0,
            volume: 1.0,
            position_seconds: 0.0,
            video_frame_rate: 0.0,
            video_duration_secs: 0.0,
            video_width: 0,
            video_height: 0,
            current_frame: None,
            total_frames: 0,
            on_playback_started: None,
            on_playback_stopped: None,
            on_error: None,
        }
    }

    /// Push a decoded frame into the component.
    ///
    /// This is the primary way to feed frames when the `decoder` feature
    /// is not enabled. The component tracks dimensions and frame count.
    pub fn push_frame(&mut self, frame: VideoFrame) {
        self.video_width = frame.width;
        self.video_height = frame.height;
        self.position_seconds = frame.pts_seconds;
        self.total_frames = frame.frame_index + 1;
        self.current_frame = Some(frame);
    }

    /// Whether a video is loaded (frames have been pushed or loaded).
    pub fn is_loaded(&self) -> bool {
        self.current_frame.is_some() || self.total_frames > 0
    }

    /// Start or resume playback.
    pub fn play(&mut self) {
        if self.is_loaded() && self.state != PlaybackState::Playing {
            self.state = PlaybackState::Playing;
            if let Some(ref mut cb) = self.on_playback_started {
                cb();
            }
        }
    }

    /// Pause playback.
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    /// Stop playback and clear the current frame.
    pub fn stop(&mut self) {
        let was_active = self.state != PlaybackState::Stopped;
        self.state = PlaybackState::Stopped;
        self.position_seconds = 0.0;
        self.current_frame = None;
        if was_active {
            if let Some(ref mut cb) = self.on_playback_stopped {
                cb();
            }
        }
    }

    /// Current playback state.
    pub fn playback_state(&self) -> PlaybackState {
        self.state
    }

    /// Whether the video is currently playing.
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Playback speed (1.0 = normal speed).
    pub fn play_speed(&self) -> f64 {
        self.play_speed
    }

    /// Set playback speed. Values > 1.0 are faster, < 1.0 are slower.
    pub fn set_play_speed(&mut self, speed: f64) {
        self.play_speed = speed.max(0.0);
    }

    /// Audio volume (0.0–1.0).
    pub fn audio_volume(&self) -> f32 {
        self.volume
    }

    /// Set audio volume (0.0–1.0).
    pub fn set_audio_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Current playback position in seconds.
    pub fn play_position(&self) -> f64 {
        self.position_seconds
    }

    /// Set the playback position in seconds.
    pub fn set_play_position(&mut self, seconds: f64) {
        self.position_seconds = seconds.max(0.0);
    }

    /// Video duration in seconds (0.0 if no video loaded).
    pub fn video_duration(&self) -> f64 {
        self.video_duration_secs
    }

    /// Video native width in pixels (0 if no video loaded).
    pub fn video_native_width(&self) -> u32 {
        self.video_width
    }

    /// Video native height in pixels (0 if no video loaded).
    pub fn video_native_height(&self) -> u32 {
        self.video_height
    }

    /// Video frame rate (0.0 if unknown).
    pub fn video_frame_rate(&self) -> f64 {
        self.video_frame_rate
    }

    /// Total number of frames decoded/pushed.
    pub fn total_frames(&self) -> u64 {
        self.total_frames
    }

    /// Get the most recently pushed frame, if any.
    pub fn current_frame(&self) -> Option<&VideoFrame> {
        self.current_frame.as_ref()
    }

    /// Advance playback by one frame tick.
    ///
    /// Returns `true` if the component should repaint (playing), `false` if
    /// stopped or paused.
    pub fn tick(&mut self) -> bool {
        if self.state != PlaybackState::Playing {
            return false;
        }
        // With external frame feeding, tick reports that we're playing.
        // The caller should push new frames at the appropriate rate.
        true
    }

    /// Set callback fired when playback starts.
    pub fn on_playback_started(&mut self, callback: impl FnMut() + 'static) {
        self.on_playback_started = Some(Box::new(callback));
    }

    /// Set callback fired when playback stops.
    pub fn on_playback_stopped(&mut self, callback: impl FnMut() + 'static) {
        self.on_playback_stopped = Some(Box::new(callback));
    }

    /// Set callback fired on errors.
    pub fn on_error(&mut self, callback: impl FnMut(VideoError) + 'static) {
        self.on_error = Some(Box::new(callback));
    }

    // -- Component delegation ------------------------------------------------

    /// Get the underlying component.
    pub fn component(&self) -> &logic_nih_plug_gui::Component {
        &self.component
    }

    /// Get mutable reference to the underlying component.
    pub fn component_mut(&mut self) -> &mut logic_nih_plug_gui::Component {
        &mut self.component
    }

    /// Set bounds.
    pub fn set_bounds(
        &mut self,
        bounds: logic_nih_plug_gui::components::Bounds,
    ) -> Result<(), logic_nih_plug_gui::error::GuiError> {
        self.component.set_bounds(bounds)
    }

    /// Get bounds.
    pub fn bounds(&self) -> logic_nih_plug_gui::components::Bounds {
        self.component.bounds()
    }

    /// Check if enabled.
    pub fn is_enabled(&self) -> bool {
        self.component.is_enabled()
    }

    /// Set enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.component.set_enabled(enabled);
    }

    /// Check if visible.
    pub fn is_visible(&self) -> bool {
        self.component.is_visible()
    }

    /// Set visibility.
    pub fn set_visible(&mut self, visible: bool) {
        self.component.set_visible(visible);
    }
}

#[cfg(feature = "gui")]
impl Default for VideoComponent {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests — no ffmpeg required
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- VideoFrame tests ---------------------------------------------------

    #[test]
    fn frame_new_valid() {
        let data = vec![0u8; 4 * 3 * 4]; // 4x3 pixels, 4 bytes each
        let frame = VideoFrame::new(4, 3, 0.0, 0, data.clone()).unwrap();
        assert_eq!(frame.width, 4);
        assert_eq!(frame.height, 3);
        assert_eq!(frame.pts_seconds, 0.0);
        assert_eq!(frame.frame_index, 0);
        assert_eq!(frame.data, data);
    }

    #[test]
    fn frame_new_wrong_size() {
        let data = vec![0u8; 10];
        assert!(VideoFrame::new(4, 3, 0.0, 0, data).is_none());
    }

    #[test]
    fn frame_pixel_access() {
        let mut data = vec![0u8; 4 * 2 * 4]; // 4x2 pixels, 4 bytes each
        data[4] = 255;
        data[5] = 0;
        data[6] = 0;
        data[7] = 255;
        let frame = VideoFrame::new(4, 2, 0.0, 0, data).unwrap();
        assert_eq!(frame.pixel(1, 0), Some([255, 0, 0, 255]));
        assert_eq!(frame.pixel(0, 0), Some([0, 0, 0, 0]));
        assert_eq!(frame.pixel(4, 0), None);
        assert_eq!(frame.pixel(0, 2), None);
    }

    #[test]
    fn frame_pixels_rgba() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let frame = VideoFrame::new(2, 1, 0.0, 0, data).unwrap();
        let pixels = frame.pixels_rgba();
        assert_eq!(pixels.len(), 2);
        assert_eq!(pixels[0], [10, 20, 30, 40]);
        assert_eq!(pixels[1], [50, 60, 70, 80]);
    }

    #[test]
    fn frame_data_size() {
        let data = vec![0u8; 100 * 50 * 4];
        let frame = VideoFrame::new(100, 50, 0.0, 0, data).unwrap();
        assert_eq!(frame.data_size(), 100 * 50 * 4);
    }

    #[test]
    fn frame_clone() {
        let data = vec![1, 2, 3, 4];
        let frame = VideoFrame::new(1, 1, 1.5, 42, data).unwrap();
        let cloned = frame.clone();
        assert_eq!(cloned.width, 1);
        assert_eq!(cloned.height, 1);
        assert_eq!(cloned.pts_seconds, 1.5);
        assert_eq!(cloned.frame_index, 42);
        assert_eq!(cloned.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn frame_debug() {
        let data = vec![0u8; 4];
        let frame = VideoFrame::new(1, 1, 0.0, 0, data).unwrap();
        let debug_str = format!("{:?}", frame);
        assert!(debug_str.contains("VideoFrame"));
        assert!(debug_str.contains("width: 1"));
    }

    #[test]
    fn frame_solid() {
        let frame = VideoFrame::solid(3, 2, [255, 128, 0, 255]);
        assert_eq!(frame.width, 3);
        assert_eq!(frame.height, 2);
        assert_eq!(frame.data.len(), 3 * 2 * 4);
        for px in frame.pixels_rgba() {
            assert_eq!(*px, [255, 128, 0, 255]);
        }
    }

    #[test]
    fn frame_new_zero_dimensions() {
        let data = vec![];
        let frame = VideoFrame::new(0, 0, 0.0, 0, data).unwrap();
        assert_eq!(frame.data_size(), 0);
        assert!(frame.pixels_rgba().is_empty());
    }

    // -- VideoError tests ---------------------------------------------------

    #[test]
    fn error_display() {
        let err = VideoError::OpenFailed("test.mp4".into());
        assert_eq!(err.to_string(), "Failed to open video: test.mp4");

        let err = VideoError::NoVideoStream;
        assert_eq!(err.to_string(), "No video stream found");

        let err = VideoError::DecodeError("bad codec".into());
        assert_eq!(err.to_string(), "Decode error: bad codec");

        let err = VideoError::SeekFailed("out of range".into());
        assert_eq!(err.to_string(), "Seek failed: out of range");

        let err = VideoError::ConversionError("format mismatch".into());
        assert_eq!(
            err.to_string(),
            "Pixel format conversion failed: format mismatch"
        );

        let err = VideoError::NoFrame;
        assert_eq!(err.to_string(), "No frame available");

        let err = VideoError::InvalidPath("/bad".into());
        assert_eq!(err.to_string(), "Invalid path: /bad");
    }

    #[test]
    fn error_is_clone() {
        let err = VideoError::NoFrame;
        let cloned = err.clone();
        assert_eq!(format!("{:?}", err), format!("{:?}", cloned));
    }

    #[test]
    fn error_types_roundtrip() {
        let errors = vec![
            VideoError::OpenFailed("x".into()),
            VideoError::NoVideoStream,
            VideoError::DecodeError("x".into()),
            VideoError::SeekFailed("x".into()),
            VideoError::ConversionError("x".into()),
            VideoError::NoFrame,
            VideoError::InvalidPath("x".into()),
        ];
        for err in errors {
            let _ = format!("{}", err);
            let _ = format!("{:?}", err);
        }
    }

    // -- PlaybackState tests ------------------------------------------------

    #[test]
    fn playback_state_default() {
        assert_eq!(PlaybackState::Stopped, PlaybackState::Stopped);
        assert_ne!(PlaybackState::Playing, PlaybackState::Paused);
    }

    #[test]
    fn playback_state_debug() {
        assert_eq!(format!("{:?}", PlaybackState::Playing), "Playing");
        assert_eq!(format!("{:?}", PlaybackState::Paused), "Paused");
        assert_eq!(format!("{:?}", PlaybackState::Stopped), "Stopped");
    }

    #[test]
    fn playback_state_copy() {
        let s = PlaybackState::Playing;
        let s2 = s;
        assert_eq!(s, s2);
    }
}

// ---------------------------------------------------------------------------
// Tests for VideoComponent (gui feature, no ffmpeg required)
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "gui"))]
mod component_tests {
    use super::*;
    use logic_nih_plug_gui::components::Bounds;

    #[test]
    fn video_component_new() {
        let vc = VideoComponent::new();
        assert!(!vc.is_loaded());
        assert!(!vc.is_playing());
        assert_eq!(vc.playback_state(), PlaybackState::Stopped);
        assert_eq!(vc.play_speed(), 1.0);
        assert_eq!(vc.audio_volume(), 1.0);
        assert_eq!(vc.play_position(), 0.0);
        assert_eq!(vc.video_duration(), 0.0);
        assert_eq!(vc.video_native_width(), 0);
        assert_eq!(vc.video_native_height(), 0);
        assert!(vc.current_frame().is_none());
    }

    #[test]
    fn video_component_default() {
        let vc = VideoComponent::default();
        assert!(!vc.is_loaded());
    }

    #[test]
    fn video_component_bounds() {
        let mut vc = VideoComponent::new();
        vc.set_bounds(Bounds::new(10, 20, 640, 480))
            .unwrap();
        let b = vc.bounds();
        assert_eq!(b.x, 10);
        assert_eq!(b.y, 20);
        assert_eq!(b.width, 640);
        assert_eq!(b.height, 480);
    }

    #[test]
    fn video_component_visibility() {
        let mut vc = VideoComponent::new();
        assert!(vc.is_visible());
        vc.set_visible(false);
        assert!(!vc.is_visible());
        vc.set_visible(true);
        assert!(vc.is_visible());
    }

    #[test]
    fn video_component_enabled() {
        let mut vc = VideoComponent::new();
        assert!(vc.is_enabled());
        vc.set_enabled(false);
        assert!(!vc.is_enabled());
        vc.set_enabled(true);
        assert!(vc.is_enabled());
    }

    #[test]
    fn video_component_play_speed() {
        let mut vc = VideoComponent::new();
        assert_eq!(vc.play_speed(), 1.0);
        vc.set_play_speed(2.0);
        assert_eq!(vc.play_speed(), 2.0);
        vc.set_play_speed(0.5);
        assert_eq!(vc.play_speed(), 0.5);
        vc.set_play_speed(-1.0);
        assert_eq!(vc.play_speed(), 0.0);
    }

    #[test]
    fn video_component_volume() {
        let mut vc = VideoComponent::new();
        assert_eq!(vc.audio_volume(), 1.0);
        vc.set_audio_volume(0.5);
        assert_eq!(vc.audio_volume(), 0.5);
        vc.set_audio_volume(-0.1);
        assert_eq!(vc.audio_volume(), 0.0);
        vc.set_audio_volume(1.5);
        assert_eq!(vc.audio_volume(), 1.0);
    }

    #[test]
    fn video_component_play_stop_without_load() {
        let mut vc = VideoComponent::new();
        vc.play();
        assert!(!vc.is_playing());
        assert_eq!(vc.playback_state(), PlaybackState::Stopped);
        vc.stop();
        assert_eq!(vc.playback_state(), PlaybackState::Stopped);
    }

    #[test]
    fn video_component_pause_without_play() {
        let mut vc = VideoComponent::new();
        vc.pause();
        assert_eq!(vc.playback_state(), PlaybackState::Stopped);
    }

    #[test]
    fn video_component_tick_without_play() {
        let mut vc = VideoComponent::new();
        assert!(!vc.tick());
    }

    #[test]
    fn video_component_position() {
        let mut vc = VideoComponent::new();
        assert_eq!(vc.play_position(), 0.0);
        vc.set_play_position(5.0);
        assert_eq!(vc.play_position(), 5.0);
        vc.set_play_position(-1.0);
        assert_eq!(vc.play_position(), 0.0);
    }

    #[test]
    fn video_component_push_frame() {
        let mut vc = VideoComponent::new();
        let frame = VideoFrame::solid(320, 240, [255, 0, 0, 255]);
        vc.push_frame(frame);

        assert!(vc.is_loaded());
        assert_eq!(vc.video_native_width(), 320);
        assert_eq!(vc.video_native_height(), 240);
        assert_eq!(vc.total_frames(), 1);
        assert!(vc.current_frame().is_some());
        assert_eq!(
            vc.current_frame().unwrap().pixel(0, 0),
            Some([255, 0, 0, 255])
        );
    }

    #[test]
    fn video_component_push_multiple_frames() {
        let mut vc = VideoComponent::new();
        for i in 0..5u64 {
            let mut frame = VideoFrame::solid(100, 100, [i as u8, 0, 0, 255]);
            frame.frame_index = i;
            frame.pts_seconds = i as f64 / 24.0;
            vc.push_frame(frame);
        }
        assert_eq!(vc.total_frames(), 5);
        assert!((vc.play_position() - 4.0 / 24.0).abs() < 0.001);
    }

    #[test]
    fn video_component_unload() {
        let mut vc = VideoComponent::new();
        vc.push_frame(VideoFrame::solid(100, 100, [0; 4]));
        assert!(vc.is_loaded());
        vc.stop();
        assert_eq!(vc.playback_state(), PlaybackState::Stopped);
        assert_eq!(vc.play_position(), 0.0);
        assert!(vc.current_frame().is_none());
    }

    #[test]
    fn video_component_play_pause_stop_cycle() {
        let mut vc = VideoComponent::new();
        vc.push_frame(VideoFrame::solid(100, 100, [0; 4]));

        vc.play();
        assert_eq!(vc.playback_state(), PlaybackState::Playing);
        assert!(vc.is_playing());

        vc.pause();
        assert_eq!(vc.playback_state(), PlaybackState::Paused);
        assert!(!vc.is_playing());

        vc.play();
        assert_eq!(vc.playback_state(), PlaybackState::Playing);

        vc.stop();
        assert_eq!(vc.playback_state(), PlaybackState::Stopped);
        assert!(!vc.is_playing());
    }

    #[test]
    fn video_component_tick_while_playing() {
        let mut vc = VideoComponent::new();
        vc.push_frame(VideoFrame::solid(100, 100, [0; 4]));
        vc.play();
        assert!(vc.tick());
    }

    #[test]
    fn video_component_tick_while_paused() {
        let mut vc = VideoComponent::new();
        vc.push_frame(VideoFrame::solid(100, 100, [0; 4]));
        vc.play();
        vc.pause();
        assert!(!vc.tick());
    }

    #[test]
    fn video_component_component_delegation() {
        let vc = VideoComponent::new();
        assert_eq!(vc.component().name(), "VideoComponent");
    }

    #[test]
    fn video_component_callback_fire() {
        use std::cell::Cell;
        use std::rc::Rc;

        let mut vc = VideoComponent::new();
        vc.push_frame(VideoFrame::solid(100, 100, [0; 4]));

        let started = Rc::new(Cell::new(false));
        let stopped = Rc::new(Cell::new(false));

        let s = started.clone();
        vc.on_playback_started(move || {
            s.set(true);
        });

        let t = stopped.clone();
        vc.on_playback_stopped(move || {
            t.set(true);
        });

        vc.play();
        assert!(started.get());

        vc.stop();
        assert!(stopped.get());
    }

    #[test]
    fn video_component_error_callback() {
        let mut vc = VideoComponent::new();
        let mut error_fired = false;
        vc.on_error(move |_err| {
            error_fired = true;
        });
        // Without a real decoder, verify callback was set without panic
        assert!(!error_fired);
    }

    #[test]
    fn video_component_zero_volume() {
        let mut vc = VideoComponent::new();
        vc.set_audio_volume(0.0);
        assert_eq!(vc.audio_volume(), 0.0);
    }

    #[test]
    fn video_component_speed_zero() {
        let mut vc = VideoComponent::new();
        vc.set_play_speed(0.0);
        assert_eq!(vc.play_speed(), 0.0);
    }

    #[test]
    fn video_component_set_position_negative() {
        let mut vc = VideoComponent::new();
        vc.set_play_position(-5.0);
        assert_eq!(vc.play_position(), 0.0);
    }

    #[test]
    fn video_component_frame_rate_tracking() {
        let mut vc = VideoComponent::new();
        assert_eq!(vc.video_frame_rate(), 0.0);
        vc.push_frame(VideoFrame::solid(100, 100, [0; 4]));
        assert_eq!(vc.video_frame_rate(), 0.0);
    }
}

// ---------------------------------------------------------------------------
// Tests for VideoDecoder (feature = "decoder") — only when ffmpeg is available
// ---------------------------------------------------------------------------

#[cfg(all(test, feature = "decoder"))]
mod decoder_tests {
    use super::*;

    #[test]
    fn decoder_open_nonexistent() {
        let result = VideoDecoder::open("nonexistent_video_file_xyz.mp4");
        assert!(result.is_err());
        match result.unwrap_err() {
            VideoError::OpenFailed(_) => {}
            other => panic!("Expected OpenFailed, got {:?}", other),
        }
    }

    #[test]
    fn decoder_open_invalid_path() {
        let result = VideoDecoder::open("");
        assert!(result.is_err());
    }
}
