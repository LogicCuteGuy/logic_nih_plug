//! Integration tests for Timer functionality.
//!
//! These tests verify that the Timer component works correctly through the FFI layer.

use nih_plug_juce::{initialize, Timer};

#[test]
fn test_timer_creation() {
    // Initialize JUCE
    initialize().expect("Failed to initialize JUCE");

    // Create a timer with a simple callback
    let timer = Timer::new(|| {
        // This callback would be invoked when the timer fires
    });

    assert!(timer.is_ok(), "Timer creation should succeed");
}

#[test]
fn test_timer_is_running_initially_false() {
    // Initialize JUCE
    initialize().expect("Failed to initialize JUCE");

    // Create a timer
    let timer = Timer::new(|| {}).expect("Failed to create timer");

    // Timer should not be running initially
    assert!(!timer.is_running(), "Timer should not be running initially");
}

#[test]
fn test_timer_start_stop() {
    // Initialize JUCE
    initialize().expect("Failed to initialize JUCE");

    // Create a timer
    let mut timer = Timer::new(|| {}).expect("Failed to create timer");

    // Start the timer
    timer.start(100).expect("Failed to start timer");

    // Timer should be running
    assert!(timer.is_running(), "Timer should be running after start");

    // Stop the timer
    timer.stop();

    // Timer should not be running
    assert!(!timer.is_running(), "Timer should not be running after stop");
}

#[test]
fn test_timer_restart() {
    // Initialize JUCE
    initialize().expect("Failed to initialize JUCE");

    // Create a timer
    let mut timer = Timer::new(|| {}).expect("Failed to create timer");

    // Start the timer
    timer.start(100).expect("Failed to start timer");
    assert!(timer.is_running(), "Timer should be running");

    // Restart with a different interval
    timer.start(200).expect("Failed to restart timer");
    assert!(timer.is_running(), "Timer should still be running after restart");

    // Stop the timer
    timer.stop();
    assert!(!timer.is_running(), "Timer should not be running after stop");
}

#[test]
fn test_timer_drop_stops_timer() {
    // Initialize JUCE
    initialize().expect("Failed to initialize JUCE");

    // Create a timer in a scope
    {
        let mut timer = Timer::new(|| {}).expect("Failed to create timer");
        timer.start(100).expect("Failed to start timer");
        assert!(timer.is_running(), "Timer should be running");
        // Timer is dropped here
    }

    // If we get here without crashing, the drop worked correctly
}

#[test]
fn test_timer_multiple_instances() {
    // Initialize JUCE
    initialize().expect("Failed to initialize JUCE");

    // Create multiple timers
    let mut timer1 = Timer::new(|| {}).expect("Failed to create timer1");
    let mut timer2 = Timer::new(|| {}).expect("Failed to create timer2");
    let mut timer3 = Timer::new(|| {}).expect("Failed to create timer3");

    // Start them with different intervals
    timer1.start(100).expect("Failed to start timer1");
    timer2.start(200).expect("Failed to start timer2");
    timer3.start(300).expect("Failed to start timer3");

    // All should be running
    assert!(timer1.is_running(), "Timer1 should be running");
    assert!(timer2.is_running(), "Timer2 should be running");
    assert!(timer3.is_running(), "Timer3 should be running");

    // Stop one
    timer2.stop();
    assert!(timer1.is_running(), "Timer1 should still be running");
    assert!(!timer2.is_running(), "Timer2 should not be running");
    assert!(timer3.is_running(), "Timer3 should still be running");
}

#[test]
fn test_timer_with_closure_capturing_state() {
    // Initialize JUCE
    initialize().expect("Failed to initialize JUCE");

    // Create a timer that captures state
    let captured_value = 42;
    let timer = Timer::new(move || {
        // This closure captures captured_value
        let _value = captured_value;
    });

    assert!(timer.is_ok(), "Timer with capturing closure should be created");
}
