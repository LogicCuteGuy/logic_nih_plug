//! Throttled animation frame rate controller.
//!
//! `AnimationFrameRate` controls how often a component should request
//! repaints. It tracks elapsed time since the last allowed frame and
//! decides whether enough time has passed to schedule the next one.
//!
//! This avoids burning CPU by redrawing at the display's refresh rate
//! while still providing smooth animation at up to the target FPS.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_gui::layout::animation_frame_rate::AnimationFrameRate;
//!
//! let mut afr = AnimationFrameRate::new(60);
//!
//! // First frame is always allowed
//! assert!(afr.should_frame(0));
//!
//! // Immediately after, not enough time has elapsed
//! assert!(!afr.should_frame(1_000_000)); // 1 ms later
//!
//! // ~16.67 ms later (at 60 fps), another frame is allowed
//! assert!(afr.should_frame(16_700_000)); // 16.7 ms
//! ```

use std::time::Duration;

/// A throttled frame rate controller.
///
/// Computes the minimum interval between frames from the target FPS,
/// then gates frame requests accordingly.
#[derive(Debug, Clone)]
pub struct AnimationFrameRate {
    /// Target frames per second.
    fps: u32,
    /// Minimum interval between frames, derived from `fps`.
    min_interval: Duration,
    /// Timestamp of the last allowed frame (monotonic).
    last_frame_time: Option<Duration>,
    /// Whether the animation is currently enabled.
    enabled: bool,
}

impl AnimationFrameRate {
    /// Create a new frame rate controller targeting the given FPS.
    ///
    /// The animation starts **enabled**. Call `set_enabled(false)` to pause.
    pub fn new(fps: u32) -> Self {
        let fps = fps.max(1); // avoid division by zero
        let interval_ns = 1_000_000_000u64 / fps as u64;
        Self {
            fps,
            min_interval: Duration::from_nanos(interval_ns),
            last_frame_time: None,
            enabled: true,
        }
    }

    /// Create a controller with a custom interval (overrides FPS).
    pub fn with_interval(interval: Duration) -> Self {
        let fps = if interval.as_nanos() > 0 {
            (1_000_000_000u64 / interval.as_nanos() as u64) as u32
        } else {
            60
        };
        Self {
            fps,
            min_interval: interval,
            last_frame_time: None,
            enabled: true,
        }
    }

    /// Target FPS.
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Minimum interval between frames.
    pub fn min_interval(&self) -> Duration {
        self.min_interval
    }

    /// Whether animation is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Enable or disable animation.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Update the target FPS at runtime.
    pub fn set_fps(&mut self, fps: u32) {
        let fps = fps.max(1);
        let interval_ns = 1_000_000_000u64 / fps as u64;
        self.fps = fps;
        self.min_interval = Duration::from_nanos(interval_ns);
    }

    /// Returns `true` if enough time has elapsed since the last frame
    /// and a new frame should be drawn.
    ///
    /// `current_time` should be a monotonically increasing timestamp
    /// (e.g. `std::time::Instant::elapsed()` or a platform high-res timer).
    /// The first call (when no previous frame exists) always returns `true`.
    pub fn should_frame(&mut self, current_time_nanos: u64) -> bool {
        if !self.enabled {
            return false;
        }

        let now = Duration::from_nanos(current_time_nanos);

        match self.last_frame_time {
            None => {
                self.last_frame_time = Some(now);
                true
            }
            Some(last) => {
                if now >= last && now - last >= self.min_interval {
                    self.last_frame_time = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Returns `true` if enough time has elapsed since the last frame,
    /// using a `Duration` timestamp.
    pub fn should_frame_duration(&mut self, current_time: Duration) -> bool {
        if !self.enabled {
            return false;
        }

        match self.last_frame_time {
            None => {
                self.last_frame_time = Some(current_time);
                true
            }
            Some(last) => {
                if current_time >= last && current_time - last >= self.min_interval {
                    self.last_frame_time = Some(current_time);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Reset the internal state so the next `should_frame` call succeeds
    /// regardless of timing.
    pub fn reset(&mut self) {
        self.last_frame_time = None;
    }

    /// Compute the remaining time (in nanoseconds) until the next frame
    /// is allowed. Returns `0` if a frame is ready now.
    pub fn time_until_next_frame(&self, current_time_nanos: u64) -> u64 {
        let now = Duration::from_nanos(current_time_nanos);
        match self.last_frame_time {
            None => 0,
            Some(last) => {
                let elapsed = if now >= last { now - last } else { Duration::ZERO };
                if elapsed >= self.min_interval {
                    0
                } else {
                    (self.min_interval - elapsed).as_nanos() as u64
                }
            }
        }
    }

    /// Duration until the next frame is allowed.
    pub fn duration_until_next_frame(&self, current_time: Duration) -> Duration {
        match self.last_frame_time {
            None => Duration::ZERO,
            Some(last) => {
                let elapsed = if current_time >= last {
                    current_time - last
                } else {
                    Duration::ZERO
                };
                if elapsed >= self.min_interval {
                    Duration::ZERO
                } else {
                    self.min_interval - elapsed
                }
            }
        }
    }
}

impl Default for AnimationFrameRate {
    fn default() -> Self {
        Self::new(60)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_fps_is_60() {
        let afr = AnimationFrameRate::default();
        assert_eq!(afr.fps(), 60);
        assert!(afr.is_enabled());
    }

    #[test]
    fn first_frame_always_allowed() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
    }

    #[test]
    fn frames_within_interval_are_throttled() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
        // 1 ms later — not enough for 60 fps (need ~16.67 ms)
        assert!(!afr.should_frame(1_000_000));
        assert!(!afr.should_frame(5_000_000));
        assert!(!afr.should_frame(10_000_000));
    }

    #[test]
    fn frames_after_interval_are_allowed() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
        // 16.7 ms later — should be enough for 60 fps
        assert!(afr.should_frame(16_700_000));
    }

    #[test]
    fn exact_interval_boundary() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
        // Exactly 16.666... ms = 16_666_666 ns (for 60 fps)
        assert!(afr.should_frame(16_666_667));
    }

    #[test]
    fn just_below_interval() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
        // 16.66 ms — just below 16.667 ms
        assert!(!afr.should_frame(16_660_000));
    }

    #[test]
    fn disabled_never_frames() {
        let mut afr = AnimationFrameRate::new(60);
        afr.set_enabled(false);
        assert!(!afr.should_frame(0));
        assert!(!afr.should_frame(100_000_000));
    }

    #[test]
    fn reenable_after_disable() {
        let mut afr = AnimationFrameRate::new(60);
        afr.set_enabled(false);
        assert!(!afr.should_frame(0));
        afr.set_enabled(true);
        // First frame after re-enable should succeed (reset logic)
        // But last_frame_time is still None from never having recorded one
        assert!(afr.should_frame(0));
    }

    #[test]
    fn custom_fps() {
        let mut afr = AnimationFrameRate::new(30);
        assert_eq!(afr.fps(), 30);
        assert!(afr.should_frame(0));
        // 33.33 ms for 30 fps
        assert!(!afr.should_frame(16_000_000));
        assert!(afr.should_frame(33_400_000));
    }

    #[test]
    fn set_fps_updates_interval() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
        afr.set_fps(30);
        assert_eq!(afr.fps(), 30);
        // With new 30fps, 16ms is not enough
        assert!(!afr.should_frame(16_000_000));
    }

    #[test]
    fn fps_zero_clamps_to_one() {
        let afr = AnimationFrameRate::new(0);
        assert_eq!(afr.fps(), 1);
    }

    #[test]
    fn reset_allows_immediate_frame() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
        assert!(!afr.should_frame(1_000_000));
        afr.reset();
        assert!(afr.should_frame(1_000_000));
    }

    #[test]
    fn time_until_next_frame_initial() {
        let afr = AnimationFrameRate::new(60);
        assert_eq!(afr.time_until_next_frame(0), 0);
    }

    #[test]
    fn time_until_next_frame_after_frame() {
        let mut afr = AnimationFrameRate::new(60);
        afr.should_frame(0);
        // 5 ms after first frame, ~11.67 ms remaining
        let remaining = afr.time_until_next_frame(5_000_000);
        // Interval is 16_666_667 ns, elapsed is 5_000_000, remaining ~11_666_667
        assert!(remaining > 11_000_000);
        assert!(remaining <= 12_000_000);
    }

    #[test]
    fn time_until_next_frame_ready() {
        let mut afr = AnimationFrameRate::new(60);
        afr.should_frame(0);
        assert_eq!(afr.time_until_next_frame(20_000_000), 0);
    }

    #[test]
    fn duration_until_next_frame() {
        let mut afr = AnimationFrameRate::new(60);
        afr.should_frame_duration(Duration::ZERO);
        let remaining = afr.duration_until_next_frame(Duration::from_millis(5));
        assert!(remaining > Duration::from_millis(11));
        assert!(remaining <= Duration::from_millis(12));
    }

    #[test]
    fn with_interval_constructor() {
        let mut afr = AnimationFrameRate::with_interval(Duration::from_millis(33));
        // Interval is 33ms, so 20ms should not pass
        assert!(afr.should_frame_duration(Duration::ZERO));
        assert!(!afr.should_frame_duration(Duration::from_millis(20)));
        assert!(afr.should_frame_duration(Duration::from_millis(34)));
    }

    #[test]
    fn consecutive_frames_track_time() {
        let mut afr = AnimationFrameRate::new(60);
        assert!(afr.should_frame(0));
        assert!(afr.should_frame(17_000_000)); // 17ms — OK
        assert!(!afr.should_frame(20_000_000)); // only 3ms after last — not OK
        assert!(afr.should_frame(37_000_000)); // 17ms after last — OK
    }
}
