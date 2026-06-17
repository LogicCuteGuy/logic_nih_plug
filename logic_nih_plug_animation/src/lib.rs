//! # logic_nih_plug_animation
//!
//! Animation utilities ported from JUCE.
//!
//! This crate provides:
//!
//! - **Easing**: Various easing functions for smooth animations
//! - **Animation**: Core animation types for value interpolation
//! - **Chaining**: Sequence multiple animations
//!
//! ## Examples
//!
//! ```
//! use logic_nih_plug_animation::{Animation, AnimationState};
//! use logic_nih_plug_animation::easing::ease_in_out_cubic;
//!
//! // Create an animation from 0.0 to 100.0 over 1 second
//! let mut anim = Animation::new(0.0, 100.0, 1.0, ease_in_out_cubic);
//!
//! // Update the animation (e.g., in a render loop)
//! let delta_time = 0.016; // 16ms frame time
//! anim.update(delta_time);
//!
//! // Get the current value
//! let current_value = anim.current_value();
//!
//! // Check if animation is complete
//! if anim.is_complete() {
//!     println!("Animation finished!");
//! }
//! ```

#![warn(missing_docs)]

pub mod error;

#[cfg(feature = "easing")]
pub mod easing;

#[cfg(feature = "chaining")]
pub mod chaining;

pub use error::AnimationError;

#[cfg(feature = "easing")]
pub use easing::EasingFunction;

/// The state of an animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimationState {
    /// Animation has not started yet.
    NotStarted,
    /// Animation is currently running.
    Running,
    /// Animation has completed.
    Complete,
    /// Animation was cancelled.
    Cancelled,
}

/// A value animation that interpolates between a start and end value over time.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_animation::{Animation, AnimationState};
/// use logic_nih_plug_animation::easing::linear;
///
/// let mut anim = Animation::new(0.0, 100.0, 2.0, linear);
/// assert_eq!(anim.state(), AnimationState::NotStarted);
///
/// anim.start();
/// assert_eq!(anim.state(), AnimationState::Running);
///
/// anim.update(1.0); // Update by 1 second
/// assert!((anim.current_value() - 50.0).abs() < 0.01);
/// ```
#[derive(Debug, Clone)]
pub struct Animation {
    start_value: f32,
    end_value: f32,
    duration: f32,
    elapsed: f32,
    state: AnimationState,
    easing: EasingFunction,
}

impl Animation {
    /// Creates a new animation.
    ///
    /// # Arguments
    ///
    /// * `start_value` - The starting value
    /// * `end_value` - The ending value
    /// * `duration` - The duration in seconds
    /// * `easing` - The easing function to use
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_animation::Animation;
    /// use logic_nih_plug_animation::easing::ease_in_out_cubic;
    ///
    /// let anim = Animation::new(0.0, 100.0, 1.0, ease_in_out_cubic);
    /// ```
    pub fn new(start_value: f32, end_value: f32, duration: f32, easing: EasingFunction) -> Self {
        Self {
            start_value,
            end_value,
            duration: duration.max(0.0),
            elapsed: 0.0,
            state: AnimationState::NotStarted,
            easing,
        }
    }

    /// Starts the animation.
    ///
    /// If the animation is already running, this has no effect.
    pub fn start(&mut self) {
        if self.state == AnimationState::NotStarted {
            self.state = AnimationState::Running;
        }
    }

    /// Updates the animation by the given delta time.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time elapsed since the last update, in seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use logic_nih_plug_animation::Animation;
    /// use logic_nih_plug_animation::easing::linear;
    ///
    /// let mut anim = Animation::new(0.0, 100.0, 1.0, linear);
    /// anim.start();
    /// anim.update(0.5); // Update by 0.5 seconds
    /// ```
    pub fn update(&mut self, delta_time: f32) {
        if self.state != AnimationState::Running {
            return;
        }

        self.elapsed += delta_time;

        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.state = AnimationState::Complete;
        }
    }

    /// Returns the current value of the animation.
    ///
    /// The value is calculated by applying the easing function to the progress
    /// and interpolating between the start and end values.
    pub fn current_value(&self) -> f32 {
        if self.duration == 0.0 {
            return self.end_value;
        }

        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let eased = (self.easing)(t);
        self.start_value + (self.end_value - self.start_value) * eased
    }

    /// Returns the current progress of the animation as a value between 0.0 and 1.0.
    ///
    /// This is the raw progress without easing applied.
    pub fn progress(&self) -> f32 {
        if self.duration == 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Returns the current state of the animation.
    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Returns whether the animation is complete.
    pub fn is_complete(&self) -> bool {
        self.state == AnimationState::Complete
    }

    /// Returns whether the animation is running.
    pub fn is_running(&self) -> bool {
        self.state == AnimationState::Running
    }

    /// Cancels the animation.
    ///
    /// The animation will stop updating and its state will be set to `Cancelled`.
    pub fn cancel(&mut self) {
        self.state = AnimationState::Cancelled;
    }

    /// Resets the animation to its initial state.
    ///
    /// The elapsed time is reset to 0 and the state is set to `NotStarted`.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.state = AnimationState::NotStarted;
    }

    /// Jumps to the end of the animation immediately.
    ///
    /// The state is set to `Complete` and the elapsed time is set to the duration.
    pub fn jump_to_end(&mut self) {
        self.elapsed = self.duration;
        self.state = AnimationState::Complete;
    }

    /// Sets a new target value for the animation.
    ///
    /// The animation will continue from its current value to the new target.
    /// The duration is reset and the animation restarts.
    ///
    /// # Arguments
    ///
    /// * `new_end_value` - The new target value
    pub fn set_target(&mut self, new_end_value: f32) {
        self.start_value = self.current_value();
        self.end_value = new_end_value;
        self.elapsed = 0.0;
        self.state = AnimationState::Running;
    }

    /// Returns the start value of the animation.
    pub fn start_value(&self) -> f32 {
        self.start_value
    }

    /// Returns the end value of the animation.
    pub fn end_value(&self) -> f32 {
        self.end_value
    }

    /// Returns the duration of the animation in seconds.
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Returns the elapsed time in seconds.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easing::linear;

    #[test]
    fn test_animation_creation() {
        let anim = Animation::new(0.0, 100.0, 1.0, linear);
        assert_eq!(anim.state(), AnimationState::NotStarted);
        assert_eq!(anim.start_value(), 0.0);
        assert_eq!(anim.end_value(), 100.0);
        assert_eq!(anim.duration(), 1.0);
    }

    #[test]
    fn test_animation_start() {
        let mut anim = Animation::new(0.0, 100.0, 1.0, linear);
        anim.start();
        assert_eq!(anim.state(), AnimationState::Running);
    }

    #[test]
    fn test_animation_update() {
        let mut anim = Animation::new(0.0, 100.0, 1.0, linear);
        anim.start();
        
        anim.update(0.5);
        assert!((anim.current_value() - 50.0).abs() < 0.01);
        assert_eq!(anim.progress(), 0.5);
        assert!(anim.is_running());
        
        anim.update(0.5);
        assert!((anim.current_value() - 100.0).abs() < 0.01);
        assert_eq!(anim.progress(), 1.0);
        assert!(anim.is_complete());
    }

    #[test]
    fn test_animation_cancel() {
        let mut anim = Animation::new(0.0, 100.0, 1.0, linear);
        anim.start();
        anim.update(0.5);
        anim.cancel();
        
        assert_eq!(anim.state(), AnimationState::Cancelled);
        assert!(!anim.is_running());
    }

    #[test]
    fn test_animation_reset() {
        let mut anim = Animation::new(0.0, 100.0, 1.0, linear);
        anim.start();
        anim.update(0.5);
        anim.reset();
        
        assert_eq!(anim.state(), AnimationState::NotStarted);
        assert_eq!(anim.elapsed(), 0.0);
        assert_eq!(anim.current_value(), 0.0);
    }

    #[test]
    fn test_animation_jump_to_end() {
        let mut anim = Animation::new(0.0, 100.0, 1.0, linear);
        anim.start();
        anim.jump_to_end();
        
        assert!(anim.is_complete());
        assert_eq!(anim.current_value(), 100.0);
    }

    #[test]
    fn test_animation_set_target() {
        let mut anim = Animation::new(0.0, 100.0, 1.0, linear);
        anim.start();
        anim.update(0.5);
        
        let current = anim.current_value();
        anim.set_target(200.0);
        
        assert_eq!(anim.start_value(), current);
        assert_eq!(anim.end_value(), 200.0);
        assert_eq!(anim.elapsed(), 0.0);
        assert!(anim.is_running());
    }

    #[test]
    fn test_zero_duration() {
        let mut anim = Animation::new(0.0, 100.0, 0.0, linear);
        anim.start();
        assert_eq!(anim.current_value(), 100.0);
        assert_eq!(anim.progress(), 1.0);
    }
}
