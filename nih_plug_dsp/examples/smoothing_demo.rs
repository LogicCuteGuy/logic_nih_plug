//! Example demonstrating the SmoothedValue parameter smoother.
//!
//! This example shows how to use SmoothedValue to smoothly interpolate
//! parameter changes over time, avoiding clicks and discontinuities.

use nih_plug_dsp::smoothing::SmoothedValue;

fn main() {
    println!("SmoothedValue Demo\n");

    // Create a smoother for a frequency parameter
    let mut frequency = SmoothedValue::<f32>::new(440.0);
    
    // Configure for 44.1kHz sample rate with 50ms smoothing time
    frequency.reset(44100.0, 0.05);
    
    println!("Initial frequency: {} Hz", frequency.current());
    
    // Change to a new frequency
    frequency.set_target(880.0);
    println!("\nSmoothing from 440 Hz to 880 Hz over 50ms...");
    
    // Generate some smoothed values
    println!("\nFirst 10 samples:");
    for i in 0..10 {
        let value = frequency.next();
        println!("  Sample {}: {:.2} Hz", i, value);
    }
    
    // Skip ahead to see more of the transition
    for _ in 0..2000 {
        frequency.next();
    }
    
    println!("\nAfter 2000 samples:");
    for i in 0..5 {
        let value = frequency.next();
        println!("  Sample {}: {:.2} Hz", 2000 + i, value);
    }
    
    // Complete the smoothing
    while frequency.is_smoothing() {
        frequency.next();
    }
    
    println!("\nFinal frequency: {} Hz", frequency.current());
    
    // Demonstrate immediate value change
    println!("\n--- Immediate Change Demo ---");
    frequency.skip(220.0);
    println!("Immediately jumped to: {} Hz", frequency.current());
    
    // Demonstrate sample rate change
    println!("\n--- Sample Rate Change Demo ---");
    frequency.set_target(440.0);
    println!("Smoothing to 440 Hz at 44.1kHz...");
    
    // Advance a bit
    for _ in 0..100 {
        frequency.next();
    }
    println!("After 100 samples: {:.2} Hz", frequency.current());
    
    // Change sample rate mid-smoothing
    frequency.reset(48000.0, 0.05);
    println!("Changed sample rate to 48kHz, recalculating smoothing...");
    println!("Still smoothing: {}", frequency.is_smoothing());
    
    // Complete smoothing
    while frequency.is_smoothing() {
        frequency.next();
    }
    println!("Final frequency: {} Hz", frequency.current());
}
