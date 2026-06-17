//! Animation chaining implementation.
//!
//! This module provides functionality to sequence multiple animations,
//! allowing them to play one after another or in parallel.
//!
//! # Examples
//!
//! ```
//! use logic_nih_plug_animation::{Animation, AnimationState};
//! use logic_nih_plug_animation::chaining::AnimationSequence;
//! use logic_nih_plug_animation::easing::linear;
//!
//! let anim1 = Animation::new(0.0, 50.0, 1.0, linear);
//! let anim2 = Animation::new(50.0, 100.0, 1.0, linear);
//!
//! let mut sequence = AnimationSequence::new();
//! sequence.add(anim1);
//! sequence.add(anim2);
//!
//! sequence.start();
//! ```

use crate::{Animation, AnimationState};

/// A sequence of animations that play one after another.
///
/// # Examples
///
/// ```
/// use logic_nih_plug_animation::{Animation, AnimationState};
/// use logic_nih_plug_animation::chaining::AnimationSequence;
/// use logic_nih_plug_animation::easing::linear;
///
/// let mut sequence = AnimationSequence::new();
/// sequence.add(Animation::new(0.0, 50.0, 1.0, linear));
/// sequence.add(Animation::new(50.0, 100.0, 1.0, linear));
///
/// sequence.start();
/// assert_eq!(sequence.state(), AnimationState::Running);
/// ```
#[derive(Debug, Clone)]
pub struct AnimationSequence {
    animations: Vec<Animation>,
    current_index: usize,
    state: AnimationState,
}

impl AnimationSequence {
    /// Creates a new empty animation sequence.
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            current_index: 0,
            state: AnimationState::NotStarted,
        }
    }

    /// Adds an animation to the end of the sequence.
    ///
    /// # Arguments
    ///
    /// * `animation` - The animation to add
    pub fn add(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    /// Starts the animation sequence.
    ///
    /// This will start the first animation in the sequence.
    pub fn start(&mut self) {
        if self.animations.is_empty() {
            self.state = AnimationState::Complete;
            return;
        }

        self.state = AnimationState::Running;
        self.current_index = 0;
        self.animations[0].start();
    }

    /// Updates the animation sequence by the given delta time.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time elapsed since the last update, in seconds
    pub fn update(&mut self, delta_time: f32) {
        if self.state != AnimationState::Running {
            return;
        }

        if self.current_index >= self.animations.len() {
            self.state = AnimationState::Complete;
            return;
        }

        let current_anim = &mut self.animations[self.current_index];
        current_anim.update(delta_time);

        if current_anim.is_complete() {
            self.current_index += 1;
            
            if self.current_index < self.animations.len() {
                self.animations[self.current_index].start();
            } else {
                self.state = AnimationState::Complete;
            }
        }
    }

    /// Returns the current value of the active animation.
    ///
    /// If the sequence is complete, returns the end value of the last animation.
    /// If the sequence hasn't started, returns the start value of the first animation.
    pub fn current_value(&self) -> f32 {
        if self.animations.is_empty() {
            return 0.0;
        }

        if self.current_index >= self.animations.len() {
            return self.animations.last().unwrap().end_value();
        }

        self.animations[self.current_index].current_value()
    }

    /// Returns the overall progress of the sequence as a value between 0.0 and 1.0.
    pub fn progress(&self) -> f32 {
        if self.animations.is_empty() {
            return 1.0;
        }

        let total_animations = self.animations.len() as f32;
        let completed_animations = self.current_index as f32;
        
        if self.current_index >= self.animations.len() {
            return 1.0;
        }

        let current_progress = self.animations[self.current_index].progress();
        (completed_animations + current_progress) / total_animations
    }

    /// Returns the current state of the sequence.
    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Returns whether the sequence is complete.
    pub fn is_complete(&self) -> bool {
        self.state == AnimationState::Complete
    }

    /// Returns whether the sequence is running.
    pub fn is_running(&self) -> bool {
        self.state == AnimationState::Running
    }

    /// Cancels the animation sequence.
    ///
    /// The current animation will be cancelled and the sequence will stop.
    pub fn cancel(&mut self) {
        if self.current_index < self.animations.len() {
            self.animations[self.current_index].cancel();
        }
        self.state = AnimationState::Cancelled;
    }

    /// Resets the animation sequence to its initial state.
    pub fn reset(&mut self) {
        for anim in &mut self.animations {
            anim.reset();
        }
        self.current_index = 0;
        self.state = AnimationState::NotStarted;
    }

    /// Returns the number of animations in the sequence.
    pub fn len(&self) -> usize {
        self.animations.len()
    }

    /// Returns whether the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }

    /// Returns the index of the currently playing animation.
    pub fn current_index(&self) -> usize {
        self.current_index
    }
}

impl Default for AnimationSequence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::easing::linear;

    #[test]
    fn test_sequence_creation() {
        let sequence = AnimationSequence::new();
        assert_eq!(sequence.len(), 0);
        assert!(sequence.is_empty());
        assert_eq!(sequence.state(), AnimationState::NotStarted);
    }

    #[test]
    fn test_sequence_add() {
        let mut sequence = AnimationSequence::new();
        sequence.add(Animation::new(0.0, 50.0, 1.0, linear));
        sequence.add(Animation::new(50.0, 100.0, 1.0, linear));
        
        assert_eq!(sequence.len(), 2);
        assert!(!sequence.is_empty());
    }

    #[test]
    fn test_sequence_start() {
        let mut sequence = AnimationSequence::new();
        sequence.add(Animation::new(0.0, 50.0, 1.0, linear));
        
        sequence.start();
        assert_eq!(sequence.state(), AnimationState::Running);
        assert_eq!(sequence.current_index(), 0);
    }

    #[test]
    fn test_empty_sequence() {
        let mut sequence = AnimationSequence::new();
        sequence.start();
        assert_eq!(sequence.state(), AnimationState::Complete);
    }

    #[test]
    fn test_sequence_update() {
        let mut sequence = AnimationSequence::new();
        sequence.add(Animation::new(0.0, 50.0, 1.0, linear));
        sequence.add(Animation::new(50.0, 100.0, 1.0, linear));
        
        sequence.start();
        
        // Update first animation halfway
        sequence.update(0.5);
        assert_eq!(sequence.current_index(), 0);
        assert!((sequence.current_value() - 25.0).abs() < 0.01);
        
        // Complete first animation
        sequence.update(0.5);
        assert_eq!(sequence.current_index(), 1);
        
        // Update second animation halfway
        sequence.update(0.5);
        assert!((sequence.current_value() - 75.0).abs() < 0.01);
        
        // Complete second animation
        sequence.update(0.5);
        assert!(sequence.is_complete());
        assert!((sequence.current_value() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_sequence_progress() {
        let mut sequence = AnimationSequence::new();
        sequence.add(Animation::new(0.0, 50.0, 1.0, linear));
        sequence.add(Animation::new(50.0, 100.0, 1.0, linear));
        
        sequence.start();
        
        assert_eq!(sequence.progress(), 0.0);
        
        sequence.update(0.5); // Halfway through first animation
        assert!((sequence.progress() - 0.25).abs() < 0.01);
        
        sequence.update(0.5); // Complete first animation
        assert!((sequence.progress() - 0.5).abs() < 0.01);
        
        sequence.update(1.0); // Complete second animation
        assert_eq!(sequence.progress(), 1.0);
    }

    #[test]
    fn test_sequence_cancel() {
        let mut sequence = AnimationSequence::new();
        sequence.add(Animation::new(0.0, 50.0, 1.0, linear));
        sequence.add(Animation::new(50.0, 100.0, 1.0, linear));
        
        sequence.start();
        sequence.update(0.5);
        sequence.cancel();
        
        assert_eq!(sequence.state(), AnimationState::Cancelled);
        assert!(!sequence.is_running());
    }

    #[test]
    fn test_sequence_reset() {
        let mut sequence = AnimationSequence::new();
        sequence.add(Animation::new(0.0, 50.0, 1.0, linear));
        sequence.add(Animation::new(50.0, 100.0, 1.0, linear));
        
        sequence.start();
        sequence.update(1.5);
        sequence.reset();
        
        assert_eq!(sequence.state(), AnimationState::NotStarted);
        assert_eq!(sequence.current_index(), 0);
    }
}
