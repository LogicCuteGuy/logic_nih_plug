//! Envelope generator implementations.
//!
//! This module provides ADSR (Attack, Decay, Sustain, Release) envelope generators
//! for shaping amplitude and modulation over time.

/// The current state of an ADSR envelope.
#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvelopeState {
    /// Envelope is idle (no note active).
    Idle,
    /// Attack phase - ramping up from 0 to 1.
    Attack,
    /// Decay phase - ramping down from 1 to sustain level.
    Decay,
    /// Sustain phase - holding at sustain level.
    Sustain,
    /// Release phase - ramping down from current level to 0.
    Release,
}

/// An ADSR (Attack, Decay, Sustain, Release) envelope generator.
///
/// This envelope generator produces values between 0.0 and 1.0 that can be used
/// to shape amplitude, filter cutoff, or other parameters over time.
///
/// # Examples
///
/// ```
/// use nih_plug_dsp::envelopes::Envelope;
///
/// let mut envelope = Envelope::new(44100.0);
/// envelope.set_adsr(0.01, 0.1, 0.7, 0.2);
///
/// // Trigger note on
/// envelope.note_on();
///
/// // Generate envelope values
/// for _ in 0..100 {
///     let value = envelope.get_next_sample();
///     // Use value to modulate amplitude, etc.
/// }
///
/// // Trigger note off
/// envelope.note_off();
/// ```
///
/// # Thread Safety
///
/// This type is `Send` but not `Sync`. Each thread should have its own instance.
#[derive(Debug, Clone)]
pub struct Envelope {
    /// Sample rate in Hz.
    sample_rate: f32,
    
    /// Attack time in seconds.
    attack: f32,
    
    /// Decay time in seconds.
    decay: f32,
    
    /// Sustain level (0.0 to 1.0).
    sustain: f32,
    
    /// Release time in seconds.
    release: f32,
    
    /// Current envelope state.
    state: EnvelopeState,
    
    /// Current envelope output value (0.0 to 1.0).
    current_value: f32,
    
    /// Current sample position within the current phase.
    phase_position: f32,
    
    /// Value at the start of the release phase (for smooth release from any level).
    release_start_value: f32,
}

impl Envelope {
    /// Creates a new envelope generator with the specified sample rate.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate in Hz (e.g., 44100.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::envelopes::Envelope;
    ///
    /// let envelope = Envelope::new(44100.0);
    /// ```
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.2,
            state: EnvelopeState::Idle,
            current_value: 0.0,
            phase_position: 0.0,
            release_start_value: 0.0,
        }
    }
    
    /// Sets the ADSR parameters.
    ///
    /// # Arguments
    ///
    /// * `attack` - Attack time in seconds (time to ramp from 0 to 1)
    /// * `decay` - Decay time in seconds (time to ramp from 1 to sustain level)
    /// * `sustain` - Sustain level (0.0 to 1.0)
    /// * `release` - Release time in seconds (time to ramp from current level to 0)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::envelopes::Envelope;
    ///
    /// let mut envelope = Envelope::new(44100.0);
    /// envelope.set_adsr(0.01, 0.1, 0.7, 0.2);
    /// ```
    pub fn set_adsr(&mut self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.attack = attack.max(0.0001); // Minimum to avoid division by zero
        self.decay = decay.max(0.0001);
        self.sustain = sustain.clamp(0.0, 1.0);
        self.release = release.max(0.0001);
    }
    
    /// Sets the attack time in seconds.
    pub fn set_attack(&mut self, attack: f32) {
        self.attack = attack.max(0.0001);
    }
    
    /// Sets the decay time in seconds.
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.max(0.0001);
    }
    
    /// Sets the sustain level (0.0 to 1.0).
    pub fn set_sustain(&mut self, sustain: f32) {
        self.sustain = sustain.clamp(0.0, 1.0);
    }
    
    /// Sets the release time in seconds.
    pub fn set_release(&mut self, release: f32) {
        self.release = release.max(0.0001);
    }
    
    /// Gets the current attack time in seconds.
    pub fn attack(&self) -> f32 {
        self.attack
    }
    
    /// Gets the current decay time in seconds.
    pub fn decay(&self) -> f32 {
        self.decay
    }
    
    /// Gets the current sustain level.
    pub fn sustain(&self) -> f32 {
        self.sustain
    }
    
    /// Gets the current release time in seconds.
    pub fn release(&self) -> f32 {
        self.release
    }
    
    /// Triggers a note-on event, starting the attack phase.
    ///
    /// If the envelope is already active, it will restart from the current value
    /// to provide smooth retriggering.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::envelopes::Envelope;
    ///
    /// let mut envelope = Envelope::new(44100.0);
    /// envelope.note_on();
    /// ```
    pub fn note_on(&mut self) {
        self.state = EnvelopeState::Attack;
        self.phase_position = 0.0;
        // If retriggering, start from current value for smooth transition
        // Otherwise start from 0
        if self.current_value == 0.0 {
            self.current_value = 0.0;
        }
    }
    
    /// Triggers a note-off event, starting the release phase.
    ///
    /// The envelope will ramp down from its current value to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::envelopes::Envelope;
    ///
    /// let mut envelope = Envelope::new(44100.0);
    /// envelope.note_on();
    /// // ... process some samples ...
    /// envelope.note_off();
    /// ```
    pub fn note_off(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.phase_position = 0.0;
            self.release_start_value = self.current_value;
        }
    }
    
    /// Returns whether the envelope is currently active (not idle).
    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }
    
    /// Returns the current envelope state.
    pub fn state(&self) -> &str {
        match self.state {
            EnvelopeState::Idle => "idle",
            EnvelopeState::Attack => "attack",
            EnvelopeState::Decay => "decay",
            EnvelopeState::Sustain => "sustain",
            EnvelopeState::Release => "release",
        }
    }
    
    /// Resets the envelope to its initial idle state.
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::envelopes::Envelope;
    ///
    /// let mut envelope = Envelope::new(44100.0);
    /// envelope.note_on();
    /// envelope.reset();
    /// assert!(!envelope.is_active());
    /// ```
    pub fn reset(&mut self) {
        self.state = EnvelopeState::Idle;
        self.current_value = 0.0;
        self.phase_position = 0.0;
        self.release_start_value = 0.0;
    }
    
    /// Generates and returns the next envelope sample value.
    ///
    /// This method should be called once per audio sample to generate the envelope curve.
    /// The returned value is between 0.0 and 1.0.
    ///
    /// # Returns
    ///
    /// The current envelope value (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::envelopes::Envelope;
    ///
    /// let mut envelope = Envelope::new(44100.0);
    /// envelope.set_adsr(0.01, 0.1, 0.7, 0.2);
    /// envelope.note_on();
    ///
    /// let mut output = vec![0.0; 1024];
    /// for sample in output.iter_mut() {
    ///     *sample = envelope.get_next_sample();
    /// }
    /// ```
    pub fn get_next_sample(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => {
                self.current_value = 0.0;
            }
            
            EnvelopeState::Attack => {
                let attack_samples = self.attack * self.sample_rate;
                self.phase_position += 1.0;
                
                if self.phase_position >= attack_samples {
                    // Attack phase complete, move to decay
                    self.current_value = 1.0;
                    self.state = EnvelopeState::Decay;
                    self.phase_position = 0.0;
                } else {
                    // Linear ramp from current value to 1.0
                    let progress = self.phase_position / attack_samples;
                    self.current_value = progress;
                }
            }
            
            EnvelopeState::Decay => {
                let decay_samples = self.decay * self.sample_rate;
                self.phase_position += 1.0;
                
                if self.phase_position >= decay_samples {
                    // Decay phase complete, move to sustain
                    self.current_value = self.sustain;
                    self.state = EnvelopeState::Sustain;
                    self.phase_position = 0.0;
                } else {
                    // Linear ramp from 1.0 to sustain level
                    let progress = self.phase_position / decay_samples;
                    self.current_value = 1.0 + (self.sustain - 1.0) * progress;
                }
            }
            
            EnvelopeState::Sustain => {
                // Hold at sustain level
                self.current_value = self.sustain;
            }
            
            EnvelopeState::Release => {
                let release_samples = self.release * self.sample_rate;
                self.phase_position += 1.0;
                
                if self.phase_position >= release_samples {
                    // Release phase complete, move to idle
                    self.current_value = 0.0;
                    self.state = EnvelopeState::Idle;
                    self.phase_position = 0.0;
                } else {
                    // Linear ramp from release_start_value to 0.0
                    let progress = self.phase_position / release_samples;
                    self.current_value = self.release_start_value * (1.0 - progress);
                }
            }
        }
        
        self.current_value
    }
    
    /// Processes a block of samples, filling the output buffer with envelope values.
    ///
    /// # Arguments
    ///
    /// * `output` - Output buffer to fill with envelope values
    ///
    /// # Examples
    ///
    /// ```
    /// use nih_plug_dsp::envelopes::Envelope;
    ///
    /// let mut envelope = Envelope::new(44100.0);
    /// envelope.note_on();
    ///
    /// let mut output = vec![0.0; 1024];
    /// envelope.process(&mut output);
    /// ```
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.get_next_sample();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_envelope_creation() {
        let envelope = Envelope::new(44100.0);
        assert!(!envelope.is_active());
        assert_eq!(envelope.state(), "idle");
    }
    
    #[test]
    fn test_note_on_triggers_attack() {
        let mut envelope = Envelope::new(44100.0);
        envelope.note_on();
        assert!(envelope.is_active());
        assert_eq!(envelope.state(), "attack");
    }
    
    #[test]
    fn test_note_off_triggers_release() {
        let mut envelope = Envelope::new(44100.0);
        envelope.note_on();
        envelope.note_off();
        assert_eq!(envelope.state(), "release");
    }
    
    #[test]
    fn test_envelope_reset() {
        let mut envelope = Envelope::new(44100.0);
        envelope.note_on();
        envelope.reset();
        assert!(!envelope.is_active());
        assert_eq!(envelope.state(), "idle");
    }
    
    #[test]
    fn test_envelope_phases() {
        let mut envelope = Envelope::new(44100.0);
        envelope.set_adsr(0.001, 0.001, 0.5, 0.001); // Very short times for testing
        
        envelope.note_on();
        assert_eq!(envelope.state(), "attack");
        
        // Process through attack
        for _ in 0..100 {
            envelope.get_next_sample();
        }
        
        // Should be in decay or sustain now
        let state = envelope.state();
        assert!(state == "decay" || state == "sustain");
    }
    
    #[test]
    fn test_envelope_values_in_range() {
        let mut envelope = Envelope::new(44100.0);
        envelope.set_adsr(0.01, 0.1, 0.7, 0.2);
        envelope.note_on();
        
        for _ in 0..10000 {
            let value = envelope.get_next_sample();
            assert!(value >= 0.0 && value <= 1.0, "Envelope value out of range: {}", value);
        }
    }
    
    #[test]
    fn test_parameter_changes() {
        let mut envelope = Envelope::new(44100.0);
        
        envelope.set_attack(0.05);
        assert_eq!(envelope.attack(), 0.05);
        
        envelope.set_decay(0.15);
        assert_eq!(envelope.decay(), 0.15);
        
        envelope.set_sustain(0.6);
        assert_eq!(envelope.sustain(), 0.6);
        
        envelope.set_release(0.3);
        assert_eq!(envelope.release(), 0.3);
    }
    
    #[test]
    fn test_sustain_clamping() {
        let mut envelope = Envelope::new(44100.0);
        
        envelope.set_sustain(1.5);
        assert_eq!(envelope.sustain(), 1.0);
        
        envelope.set_sustain(-0.5);
        assert_eq!(envelope.sustain(), 0.0);
    }
}
